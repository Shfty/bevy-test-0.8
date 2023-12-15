use bevy::{
    ecs::system::Command,
    prelude::{
        default, info, warn, BuildWorldChildren, Component, Entity, Parent, Transform, World,
    },
};

use crate::{
    animation::animations::discrete::{Discrete, DiscreteStop},
    prelude::{Alive, TimelineAlive},
    util::default_entity,
};

#[derive(Debug, Clone, Component)]
pub struct EntityPool {
    entities: Vec<Entity>,
}

impl Default for EntityPool {
    fn default() -> Self {
        Self {
            entities: default(),
        }
    }
}

impl EntityPool {
    pub fn new(entities: impl IntoIterator<Item = Entity>) -> Self {
        EntityPool {
            entities: entities.into_iter().collect(),
        }
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }

    pub fn inactive(&self, world: &mut World, t: f64) -> impl Iterator<Item = Entity> + '_ {
        self.entities()
            .filter(|entity| {
                let timeline_alive = world.get::<TimelineAlive>(*entity).unwrap();
                let alive_stops = world
                    .get::<Discrete<Alive>>(timeline_alive.alive_stops)
                    .unwrap();

                match alive_stops.stops().last() {
                    Some(DiscreteStop {
                        value: Alive(false),
                        t: stop_t,
                        ..
                    }) if *stop_t < t => true,
                    _ => false,
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn push(&mut self, entity: Entity) {
        self.entities.push(entity)
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities.retain(|candidate| *candidate != entity);
    }
}

pub struct InsertEntityPool<C>
where
    C: Clone + FnOnce(&mut World, Entity, Entity),
{
    pub entity: Entity,
    pub timeline: Entity,
    pub construct_instances: Option<(usize, C)>,
}

impl Default for InsertEntityPool<fn(&mut World, Entity, Entity)> {
    fn default() -> Self {
        Self {
            entity: default_entity(),
            timeline: default_entity(),
            construct_instances: None,
        }
    }
}

impl<C> Command for InsertEntityPool<C>
where
    C: 'static + Send + Sync + Clone + FnOnce(&mut World, Entity, Entity),
{
    fn write(self, world: &mut bevy::prelude::World) {
        let inactive_entities = match self.construct_instances.clone() {
            Some((count, instance_constructor)) => {
                let parent = world.get::<Parent>(self.entity).map(|parent| parent.get());

                (0..count)
                    .map(|_| {
                        let instance = world.spawn().id();

                        instance_constructor.clone()(world, instance, self.timeline);

                        if let Some(parent) = parent {
                            world.entity_mut(parent).push_children(&[instance]);
                        }

                        instance
                    })
                    .collect::<Vec<_>>()
            }
            _ => vec![],
        };

        world
            .entity_mut(self.entity)
            .insert(EntityPool::new(inactive_entities));
    }
}

pub struct UnpoolEntity<P, C> {
    pub entity_pool: Entity,
    pub t: f64,
    pub source: Entity,
    pub emitters: Vec<Transform>,
    pub unpool: P,
    pub instance_constructor: Option<(Entity, C)>,
}

impl<P, C> UnpoolEntity<P, C> {
    pub fn entity_pool(&self) -> Entity {
        self.entity_pool
    }
}

impl Default for UnpoolEntity<fn(&mut World, Entity, Transform), fn(&mut World, Entity, Entity)> {
    fn default() -> Self {
        Self {
            entity_pool: default_entity(),
            t: default(),
            source: default_entity(),
            emitters: vec![Transform::identity()],
            unpool: |_, _, _| (),
            instance_constructor: default(),
        }
    }
}

impl<P, C> Command for UnpoolEntity<P, C>
where
    P: 'static + Send + Sync + Clone + FnOnce(&mut World, Entity, Transform, f64),
    C: 'static + Send + Sync + Clone + FnOnce(&mut World, Entity, Entity),
{
    fn write(self, world: &mut World) {
        info!("Firing vulcan for {:?}", self.source);

        let transform = *world.get::<Transform>(self.source).unwrap();

        let entity_pool = world.get::<EntityPool>(self.entity_pool).unwrap().clone();
        let inactive = entity_pool.inactive(world, self.t).collect::<Vec<_>>();

        if inactive.len() < self.emitters.len() {
            if let Some((timeline, ref instance_constructor)) = self.instance_constructor {
                let count = self.emitters.len() - inactive.len();

                info!(
                    "Insuffient pooled instances. Requested: {}, Available: {}. Spawning {count:}",
                    self.emitters.len(),
                    inactive.len(),
                );

                let parent = world.get::<Parent>(self.source).map(|parent| parent.get());

                for _ in 0..count {
                    let instance = world.spawn().id();
                    info!("Spawned {instance:?}");

                    (instance_constructor.clone())(world, instance, timeline);

                    if let Some(parent) = parent {
                        world.entity_mut(parent).push_children(&[instance]);
                    }

                    world
                        .get_mut::<EntityPool>(self.entity_pool)
                        .unwrap()
                        .push(instance);
                }
            } else {
                warn!(
                    "Insuffient pooled instances. Requested: {}, Available: {}",
                    self.emitters.len(),
                    inactive.len()
                );
                return;
            }
        }

        let entities = world
            .get::<EntityPool>(self.entity_pool)
            .unwrap()
            .clone()
            .inactive(world, self.t)
            .take(self.emitters.len())
            .collect::<Vec<_>>();

        info!("Entities: {entities:?}");

        for (emitter, instance) in self
            .emitters
            .iter()
            .copied()
            .zip(entities)
            .collect::<Vec<_>>()
        {
            self.unpool.clone()(world, instance, transform * emitter, self.t);
        }
    }
}
