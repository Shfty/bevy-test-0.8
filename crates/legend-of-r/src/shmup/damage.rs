use bevy::{
    ecs::{reflect::ReflectComponent, system::Command},
    prelude::{
        default, info, Assets, BuildWorldChildren, Bundle, Changed, Children, Commands, Component,
        CoreStage, Deref, DerefMut, Entity, EventReader, EventWriter, Handle, Mesh, Name,
        ParallelSystemDescriptorCoercion, Parent, Plugin, Query, Res, ResMut, StandardMaterial,
        With, World, ReflectDefault,
    },
    reflect::Reflect,
    sprite::{ColorMaterial, Mesh2dHandle},
};
use bevy_rapier2d::prelude::{Collider, RapierContext};

use crate::{
    animation::{
        adapters::{
            evaluate::EvaluateTrait, time_source::TimeSourceTrait,
            try_animate_components::TryAnimateComponentsTrait,
        },
        animations::discrete::{
            DeterminismId, Determinisms, Discrete, DiscreteStop, DiscreteStopsTrait,
        },
        timeline::TimelineTime,
        AnimationTagTrait, AnimationTime, BuildAnimation, Last,
    },
    prelude::{
        default_entity, evaluate_tagged, ArchiveCommand, PostUpdate, SensorBundle, Timeline,
        Unarchive, ReflectBundle,
    },
};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<HitPoints>()
            .register_type::<Alive>()
            .register_type::<Hitbox>()
            .register_type::<Hurtbox>()
            .register_type::<TimelineDamage>();

        app.add_event::<DamageEvent>().add_event::<DeathEvent>();

        app.add_system_to_stage(CoreStage::PostUpdate, hitbox_collision)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                timeline_damage
                    .after(hitbox_collision)
                    .before(timeline_death),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                timeline_death
                    .after(hitbox_collision)
                    .before(evaluate_tagged::<PostUpdate>),
            )
            .add_system_to_stage(CoreStage::Last, alive.after(evaluate_tagged::<Last>));
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut, Component, Reflect,
)]
#[reflect(Component)]
pub struct HitPoints(pub usize);

impl Default for HitPoints {
    fn default() -> Self {
        Self(1)
    }
}

impl HitPoints {
    pub fn damaged(self, damage: isize) -> Self {
        HitPoints((self.0 as isize).saturating_sub(damage).max(0) as usize)
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut, Component, Reflect,
)]
#[reflect(Component)]
pub struct Alive(pub bool);

impl Default for Alive {
    fn default() -> Self {
        Self(true)
    }
}

pub trait Condition {
    fn condition(&self, world: &World) -> bool;
}

impl<F> Condition for F
where
    F: Fn(&World) -> bool,
{
    fn condition(&self, world: &World) -> bool {
        self(world)
    }
}

impl Condition for bool {
    fn condition(&self, _: &World) -> bool {
        *self
    }
}

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Hitbox {
    pub damage: isize,
}

impl Default for Hitbox {
    fn default() -> Self {
        Self { damage: 1 }
    }
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Hurtbox;

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct TimelineDamage {
    pub timeline: Entity,
    pub hit_points_stops: Entity,
}

impl Default for TimelineDamage {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            hit_points_stops: default_entity(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InsertTimelineDamage {
    pub timeline: Entity,
    pub entity: Entity,
    pub hit_points: usize,
}

impl Default for InsertTimelineDamage {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            entity: default_entity(),
            hit_points: 1,
        }
    }
}

impl Command for InsertTimelineDamage {
    fn write(self, world: &mut World) {
        let animation_hit_points = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: HitPoints(self.hit_points),
            ..default()
        }]);

        let animation_hit_points_stops = animation_hit_points.id();

        let animation_hit_points = animation_hit_points
            .try_animating_component(self.entity)
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<PostUpdate>()
            .insert(Name::new(format!("Entity {:?} Hit Points", self.entity)))
            .id();

        world
            .entity_mut(self.entity)
            .insert(TimelineDamage {
                timeline: self.timeline,
                hit_points_stops: animation_hit_points_stops,
            })
            .push_children(&[animation_hit_points]);
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct TimelineAlive {
    pub timeline: Entity,
    pub alive_stops: Entity,
    pub descendants_enabled_stops: Vec<(f64, Entity, Entity)>,
}

impl Default for TimelineAlive {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            alive_stops: default_entity(),
            descendants_enabled_stops: default(),
        }
    }
}

#[derive(Clone)]
pub struct InsertTimelineAlive {
    pub timeline: Entity,
    pub entity: Entity,
    pub alive_at: Option<f64>,
    pub dead_at: Option<f64>,
}

pub type AliveFilter = fn(&World, Entity, AnimationTime) -> bool;

impl Default for InsertTimelineAlive {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            entity: default_entity(),
            alive_at: default(),
            dead_at: default(),
        }
    }
}

impl Command for InsertTimelineAlive {
    fn write(self, world: &mut World) {
        let mut alive_stops = vec![];

        if let Some(alive_at) = self.alive_at {
            if alive_at > 0.0 {
                alive_stops.push(DiscreteStop {
                    t: 0.0,
                    value: Alive(false),
                    ..default()
                });

                ArchiveCommand {
                    target: self.entity,
                    recursive: true,
                }
                .write(world);
            }

            alive_stops.push(DiscreteStop {
                t: alive_at,
                value: Alive(true),
                ..default()
            });
        } else {
            alive_stops.push(DiscreteStop {
                t: 0.0,
                value: Alive(false),
                ..default()
            });

            ArchiveCommand {
                target: self.entity,
                recursive: true,
            }
            .write(world);
        }

        if let Some(dead) = self.dead_at {
            alive_stops.push(DiscreteStop {
                t: dead,
                value: Alive(false),
                ..default()
            });
        }

        let animation_alive = world.build_animation().from_discrete_stops(alive_stops);

        let animation_alive_stops = animation_alive.id();

        let animation_alive = animation_alive
            .try_animating_component(self.entity)
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<PostUpdate>()
            .insert(Name::new(format!("Entity {:?} Alive", self.entity)))
            .id();

        world
            .entity_mut(self.entity)
            .insert(TimelineAlive {
                timeline: self.timeline,
                alive_stops: animation_alive_stops,
                ..default()
            })
            .push_children(&[animation_alive]);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DamageEvent {
    pub source: Entity,
    pub target: Entity,
    pub damage: isize,
}

#[derive(Debug, Copy, Clone)]
pub struct DeathEvent {
    pub entity: Entity,
    pub determinism: DeterminismId,
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ConvertMesh2d;

pub fn convert_mesh_2d(
    query_convert: Query<(Entity, &Children), With<ConvertMesh2d>>,
    mut commands: Commands,
) {
    for (entity, children) in query_convert.iter() {
        commands.entity(entity).remove::<ConvertMesh2d>();
        for child in children {
            let child = *child;
            commands.add(move |world: &mut World| {
                let mut entity = world.entity_mut(child);
                if let Some(handle) = entity.remove::<Handle<Mesh>>() {
                    entity.insert(Mesh2dHandle(handle));
                }

                if let Some(handle) = entity.remove::<Handle<StandardMaterial>>() {
                    let standard_materials = world.resource_mut::<Assets<StandardMaterial>>();
                    let standard_material = standard_materials.get(&handle).unwrap();

                    let color_material = ColorMaterial {
                        color: standard_material.emissive,
                        texture: standard_material.emissive_texture.clone(),
                    };

                    let mut color_materials = world.resource_mut::<Assets<ColorMaterial>>();
                    let material = color_materials.add(color_material);

                    world.entity_mut(child).insert(material);
                }
            })
        }
    }
}

#[derive(Clone, Bundle, Reflect)]
#[reflect(Default, Bundle)]
pub struct HitboxBundle {
    pub hitbox: Hitbox,
    #[bundle]
    pub sensor_bundle: SensorBundle,
}

impl Default for HitboxBundle {
    fn default() -> Self {
        Self {
            hitbox: default(),
            sensor_bundle: default(),
        }
    }
}

#[derive(Clone, Bundle, Reflect)]
#[reflect(Default, Bundle)]
pub struct HurtboxBundle {
    pub hurtbox: Hurtbox,
    #[bundle]
    pub sensor_bundle: SensorBundle,
}

impl Default for HurtboxBundle {
    fn default() -> Self {
        Self {
            hurtbox: Hurtbox,
            sensor_bundle: default(),
        }
    }
}

pub struct SpawnHurtbox {
    pub entity: Entity,
    pub hurtbox: HurtboxBundle,
}

impl Default for SpawnHurtbox {
    fn default() -> Self {
        Self {
            entity: default_entity(),
            hurtbox: default(),
        }
    }
}

impl Command for SpawnHurtbox {
    fn write(self, world: &mut World) {
        world.entity_mut(self.entity).insert_bundle(self.hurtbox);
    }
}

pub fn hitbox_collision(
    rapier_context: Res<RapierContext>,
    mut damage_events: EventWriter<DamageEvent>,
    query_hitbox: Query<(Entity, &Hitbox), With<Collider>>,
    query_hurtbox: Query<(), With<Hurtbox>>,
    query_parent: Query<&Parent>,
    query_timeline_damage: Query<&TimelineDamage>,
) {
    let entity2collider = rapier_context.entity2collider();

    // Iterate over hitbox entities
    for (hitbox_entity, hitbox) in query_hitbox.iter() {
        let handle = if let Some(handle) = entity2collider.get(&hitbox_entity) {
            *handle
        } else {
            continue;
        };

        for (lhs, rhs) in rapier_context
            .narrow_phase
            .intersections_with(handle)
            .filter_map(|(lhs, rhs, inter)| if inter { Some((lhs, rhs)) } else { None })
        {
            let (lhs, rhs) = match (
                rapier_context.collider_entity(lhs),
                rapier_context.collider_entity(rhs),
            ) {
                (Some(lhs), Some(rhs)) => (lhs, rhs),
                _ => continue,
            };

            // Identify colliders, early out if hitbox is not present
            let other = if lhs == hitbox_entity {
                rhs
            } else if rhs == hitbox_entity {
                lhs
            } else {
                continue;
            };

            // Early out if the other collider has no hurtbox
            if query_hurtbox.get(other).is_err() {
                continue;
            }

            // Walk up the hierarchy to find a TimelineDamage component
            let mut candidate = other;
            loop {
                if query_timeline_damage.get(candidate).is_ok() {
                    break;
                }

                if let Ok(parent) = query_parent.get(candidate) {
                    candidate = **parent;
                } else {
                    return;
                }
            }

            damage_events.send(DamageEvent {
                source: hitbox_entity,
                target: candidate,
                damage: hitbox.damage,
            });
        }
    }
}

pub fn timeline_damage(
    mut damage_events: EventReader<DamageEvent>,
    mut determinisms: ResMut<Determinisms>,
    query_timeline: Query<&Timeline>,
    query_timeline_damage: Query<&TimelineDamage>,
    mut query_hit_points_stops: Query<&mut Discrete<HitPoints>>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for event in damage_events.iter() {
        let timeline_damage = if let Ok(components) = query_timeline_damage.get(event.target) {
            components
        } else {
            continue;
        };

        let timeline = query_timeline.get(timeline_damage.timeline).unwrap();
        if timeline.paused {
            continue;
        }

        let timeline = query_timeline.get(timeline_damage.timeline).unwrap();
        let t = timeline.t;

        // Add a new alive stop to the timeline with a distict determinism group
        let mut hit_points_stops = query_hit_points_stops
            .get_mut(timeline_damage.hit_points_stops)
            .unwrap();

        // Fetch active hit points stop
        let hit_points = hit_points_stops.stop_at(t).unwrap();

        // Early out if hit points are already empty
        if hit_points.value.0 == 0 {
            continue;
        }

        info!(
            "Entity {:?} damaged for {:?} hit points by {:?}",
            event.target, event.damage, event.source
        );

        // Calculate new hit points
        let new_hit_points = hit_points.value.damaged(event.damage);

        let determinism = determinisms.next();

        // Add new hit point stop to the timeline
        hit_points_stops.insert_stop(DiscreteStop {
            t,
            value: new_hit_points,
            determinism,
            ..default()
        });

        // Finish if the entity is still alive
        if *new_hit_points > 0 {
            continue;
        }

        death_events.send(DeathEvent {
            entity: event.target,
            determinism,
        });
    }
}

pub fn timeline_death(
    mut death_events: EventReader<DeathEvent>,
    query_timeline: Query<&Timeline>,
    query_timeline_alive: Query<&TimelineAlive>,
    mut query_alive_stops: Query<&mut Discrete<Alive>>,
    mut query_disabled_stops: Query<&mut Discrete<bool>>,
) {
    for event in death_events.iter() {
        let timeline_alive = if let Ok(components) = query_timeline_alive.get(event.entity) {
            components
        } else {
            continue;
        };

        info!("Death event for {:?}", event.entity);

        let timeline = query_timeline.get(timeline_alive.timeline).unwrap();
        let t = timeline.t;

        // Add a new alive stop to the timeline
        let mut alive_stops = query_alive_stops
            .get_mut(timeline_alive.alive_stops)
            .unwrap();

        alive_stops.insert_stop(DiscreteStop {
            t,
            value: Alive(false),
            determinism: event.determinism,
            ..default()
        });

        // Insert descendant disabled stops
        for (alive_t, from, to) in timeline_alive.descendants_enabled_stops.iter() {
            if *alive_t <= t {
                continue;
            }

            info!("Disabling alive stops for from {from:?} and to {to:?}");
            let mut from_stops = query_disabled_stops.get_mut(*from).unwrap();
            from_stops.insert_stop(DiscreteStop {
                t,
                value: true,
                determinism: event.determinism,
                ..default()
            });

            let mut to_stops = query_disabled_stops.get_mut(*to).unwrap();
            to_stops.insert_stop(DiscreteStop {
                t,
                value: true,
                determinism: event.determinism,
                ..default()
            });
        }
    }
}

pub fn alive(query_ship: Query<(Entity, &Alive), Changed<Alive>>, mut commands: Commands) {
    for (entity, alive) in query_ship.iter() {
        if **alive {
            //info!("Unarchiving {entity:?}");
            commands.add(Unarchive {
                target: entity,
                recursive: true,
            });
        } else {
            //info!("Archiving {entity:?}");
            commands.add(ArchiveCommand {
                target: entity,
                recursive: true,
            });
        }
    }
}
