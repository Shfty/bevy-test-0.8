use std::time::Duration;

use bevy::{
    ecs::system::Command,
    prelude::{
        default, BuildWorldChildren, Bundle, Commands, Component, CoreStage, Deref, DerefMut,
        Entity, Input, Name, ParallelSystemDescriptorCoercion, Plugin, Query, ReflectComponent,
        Res, ResMut, Transform, Vec3, With, World,
    },
    reflect::Reflect,
    scene::SceneSpawner,
    time::{Time, Timer},
};
use bevy_rapier2d::prelude::RapierContext;

use crate::{
    animation::animations::discrete::Determinisms,
    scene::InsertSceneArchive,
    prelude::{
        default_entity, evaluate_tagged, hitbox_collision, Alive, AnimateComponentsTrait,
        AnimationTagTrait, BuildAnimation, Discrete, DiscreteStop, DiscreteStopsTrait, EntityPool,
        EvaluateTrait, InsertTimelineAlive, InsertTimelineDamage, InterpolateTrait, PlayerInput,
        TimeSourceTrait, Timeline, TimelineAlive, TimelineTime, TryAnimateComponentFieldsTrait,
        UnpoolEntity, Update,
    },
};

use super::archive::ArchiveSceneInstance;

pub const SCENE_PLAYER_BULLET: &str = "meshes/PlayerBullet.gltf#Scene0";
pub const SCENE_ENEMY_BULLET: &str = "meshes/EnemyBullet.gltf#Scene0";

pub struct VulcanPlugin;

impl Plugin for VulcanPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(vulcan_fire.before(evaluate_tagged::<Update>))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                bullet_collision.after(hitbox_collision),
            );
    }
}

#[derive(Debug, Clone, Component)]
pub struct Vulcan {
    pub timeline: Entity,
    pub emitters: Vec<Transform>,
}

impl Default for Vulcan {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            emitters: vec![default()],
        }
    }
}

#[derive(Debug, Clone, Component, Deref, DerefMut)]
pub struct VulcanTimer {
    pub timer: Timer,
}

impl Default for VulcanTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(100), false),
        }
    }
}

#[derive(Debug, Clone, Default, Bundle)]
pub struct VulcanBundle {
    pub vulcan: Vulcan,
    pub vulcan_timer: VulcanTimer,
}

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct BulletAnimation {
    pub from_stops: Entity,
    pub to_stops: Entity,
}

impl Default for BulletAnimation {
    fn default() -> Self {
        Self {
            from_stops: default_entity(),
            to_stops: default_entity(),
        }
    }
}

pub struct SpawnBullet {
    pub timeline: Entity,
    pub entity: Entity,
    pub scene: InsertSceneArchive,
    pub bullet_animation: InsertBulletAnimation,
    pub timeline_damage: InsertTimelineDamage,
    pub timeline_alive: InsertTimelineAlive,
}

impl Default for SpawnBullet {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            entity: default_entity(),
            scene: default(),
            bullet_animation: default(),
            timeline_damage: default(),
            timeline_alive: default(),
        }
    }
}

impl Command for SpawnBullet {
    fn write(self, world: &mut bevy::prelude::World) {
        let mut timeline_alive = self.timeline_alive;
        timeline_alive.timeline = self.timeline;
        timeline_alive.entity = self.entity;
        timeline_alive.write(world);

        let mut timeline_damage = self.timeline_damage;
        timeline_damage.timeline = self.timeline;
        timeline_damage.entity = self.entity;
        timeline_damage.write(world);

        // Mesh
        let mut scene = self.scene;
        scene.entity = self.entity;
        scene.write(world);

        let mut bullet_animation = self.bullet_animation;
        bullet_animation.entity = self.entity;
        bullet_animation.timeline = self.timeline;
        bullet_animation.write(world);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InsertBulletAnimation {
    pub entity: Entity,
    pub timeline: Entity,
    pub bullet_animation: BulletAnimation,
}

impl Default for InsertBulletAnimation {
    fn default() -> Self {
        Self {
            entity: default_entity(),
            timeline: default_entity(),
            bullet_animation: default(),
        }
    }
}

impl Command for InsertBulletAnimation {
    fn write(self, world: &mut World) {
        let bullet_animation = world.build_animation().from_discrete_stops([
            DiscreteStop {
                t: 0.0,
                value: Transform::default(),
                ..default()
            },
            DiscreteStop {
                t: 1.0,
                value: Transform::default(),
                ..default()
            },
        ]);

        let transform_stops = bullet_animation.id();

        let bullet_animation = bullet_animation
            .interpolate()
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .animating_component(self.entity)
            .tagged::<Update>()
            .evaluate()
            .insert(Name::new("Bullet Animation"))
            .id();

        let from_animation = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: DiscreteStop {
                t: 0.0,
                value: Transform::default(),
                ..default()
            },
            ..default()
        }]);

        let from_stops = from_animation.id();

        let from_animation = from_animation
            .try_animating_component_field::<Discrete<Transform>, _, _>(
                transform_stops,
                |discrete| discrete.stops().next().unwrap(),
                |discrete| discrete.stops_mut().next().unwrap(),
            )
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .tagged::<Update>()
            .evaluate()
            .insert(Name::new("Bullet Translation From"))
            .id();

        world
            .entity_mut(transform_stops)
            .push_children(&[from_animation]);

        let to_animation = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: DiscreteStop {
                t: 0.0,
                value: Transform::default(),
                ..default()
            },
            ..default()
        }]);

        let to_stops = to_animation.id();

        let to_animation = to_animation
            .try_animating_component_field::<Discrete<Transform>, _, _>(
                transform_stops,
                |discrete| discrete.stops().last().unwrap(),
                |discrete| discrete.stops_mut().last().unwrap(),
            )
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .tagged::<Update>()
            .evaluate()
            .insert(Name::new("Bullet Translation To"))
            .id();

        world
            .entity_mut(transform_stops)
            .push_children(&[to_animation]);

        let mut bullet_animation_component = self.bullet_animation;
        bullet_animation_component.from_stops = from_stops;
        bullet_animation_component.to_stops = to_stops;
        world
            .entity_mut(self.entity)
            .insert(bullet_animation_component)
            .push_children(&[bullet_animation]);
    }
}

pub fn vulcan_fire(
    time: Res<Time>,
    input: Res<Input<PlayerInput>>,
    mut query_vulcan: Query<
        (Entity, &Vulcan, &mut VulcanTimer),
        (With<Transform>, With<EntityPool>),
    >,
    query_timeline: Query<&Timeline>,
    mut commands: Commands,
) {
    for (vulcan_entity, vulcan, mut vulcan_timer) in query_vulcan.iter_mut() {
        vulcan_timer.tick(time.delta());

        let timeline = query_timeline.get(vulcan.timeline).unwrap();

        if vulcan_timer.finished() {
            if input.pressed(PlayerInput::Fire) {
                commands.add(UnpoolEntity {
                    source: vulcan_entity,
                    entity_pool: vulcan_entity,
                    t: timeline.t,
                    emitters: vulcan.emitters.clone(),
                    unpool: |world: &mut World,
                             bullet_entity: Entity,
                             transform: Transform,
                             t: f64| {
                        let bullet = world.get::<BulletAnimation>(bullet_entity).unwrap();
                        let from_stops = bullet.from_stops;
                        let to_stops = bullet.to_stops;

                        let mut determinisms = world.resource_mut::<Determinisms>();
                        let determinism = determinisms.next();

                        let mut from_stops = world
                            .get_mut::<Discrete<DiscreteStop<Transform>>>(from_stops)
                            .unwrap();

                        from_stops.insert_stop(DiscreteStop {
                            t,
                            value: DiscreteStop {
                                t,
                                value: transform,
                                ..default()
                            },
                            determinism,
                            ..default()
                        });

                        let mut to_stops = world
                            .get_mut::<Discrete<DiscreteStop<Transform>>>(to_stops)
                            .unwrap();

                        to_stops.insert_stop(DiscreteStop {
                            t,
                            value: DiscreteStop {
                                t: t + 0.02,
                                value: transform * Transform::from_translation(Vec3::X),
                                ..default()
                            },
                            determinism,
                            ..default()
                        });

                        let timeline_alive = world.get::<TimelineAlive>(bullet_entity).unwrap();

                        let mut alive_stops = world
                            .get_mut::<Discrete<Alive>>(timeline_alive.alive_stops)
                            .unwrap();

                        alive_stops.insert_stop(DiscreteStop {
                            t,
                            value: Alive(true),
                            determinism,
                            ..default()
                        });
                    },
                    instance_constructor: None as Option<(_, fn(&mut _, _, _))>,
                });

                vulcan_timer.reset();
            }
        }
    }
}

pub fn bullet_collision(
    rapier_context: Res<RapierContext>,
    mut determinisms: ResMut<Determinisms>,
    query_vulcan: Query<&EntityPool>,
    query_bullet: Query<&ArchiveSceneInstance>,
    query_timeline_alive: Query<&TimelineAlive>,
    query_timeline: Query<&Timeline>,
    mut query_alive_stops: Query<&mut Discrete<Alive>>,
    scene_spawner: Res<SceneSpawner>,
) {
    let entity2collider = rapier_context.entity2collider();
    let entity2body = rapier_context.entity2body();

    for entity_pool in query_vulcan.iter() {
        for bullet in entity_pool.entities() {
            let instance = if let Ok(instance) = query_bullet.get(bullet) {
                instance
            } else {
                continue;
            };

            let (body, handle) = if let Some(instance) = scene_spawner
                .iter_instance_entities(**instance)
                .unwrap()
                .find_map(|entity| {
                    if entity2body.get(&entity).is_some() {
                        if let Some(foo) = entity2collider.get(&entity).copied() {
                            Some((entity, foo))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }) {
                instance
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
                if lhs != body && rhs != body {
                    continue;
                }

                let timeline_alive = query_timeline_alive.get(bullet).unwrap();
                let timeline = query_timeline.get(timeline_alive.timeline).unwrap();

                let mut stops_alive = query_alive_stops
                    .get_mut(timeline_alive.alive_stops)
                    .unwrap();

                stops_alive.insert_stop(DiscreteStop {
                    t: timeline.t,
                    value: Alive(false),
                    determinism: determinisms.next(),
                    ..default()
                });

                break;
            }
        }
    }
}
