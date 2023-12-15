//! Plugin for adding lives-based respawn capability to an entity

use bevy::{
    ecs::{reflect::ReflectComponent, system::Command},
    prelude::{
        default, info, BuildWorldChildren, Component, CoreStage, Entity, EventReader, EventWriter,
        Name, ParallelSystemDescriptorCoercion, Plugin, Query,
    },
    reflect::Reflect,
};

use crate::{
    animation::{
        adapters::{
            evaluate::EvaluateTrait, time_source::TimeSourceTrait,
            try_animate_components::TryAnimateComponentsTrait,
        },
        animations::discrete::{DeterminismId, Discrete, DiscreteStop, DiscreteStopsTrait},
        timeline::{Timeline, TimelineTime},
        AnimationTagTrait, BuildAnimation, Last,
    },
    prelude::{timeline_death, Alive, DeathEvent, HitPoints, TimelineAlive, TimelineDamage},
    util::default_entity,
};

pub struct LivesPlugin;

impl Plugin for LivesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Lives>();

        app.add_event::<RespawnEvent>();

        app.add_system_to_stage(CoreStage::PostUpdate, lives_death.after(timeline_death))
            .add_system_to_stage(CoreStage::PostUpdate, lives_respawn.after(lives_death));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
#[reflect(Component)]
pub struct Lives {
    pub lives: usize,
}

impl Default for Lives {
    fn default() -> Self {
        Self { lives: 2 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
#[reflect(Component)]
pub struct TimelineLives {
    pub timeline: Entity,
    pub lives_stops: Entity,
}

impl Default for TimelineLives {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            lives_stops: default_entity(),
        }
    }
}

pub struct InsertTimelineLives {
    pub timeline: Entity,
    pub entity: Entity,
    pub lives: usize,
}

impl Default for InsertTimelineLives {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            entity: default_entity(),
            lives: 2,
        }
    }
}

impl Command for InsertTimelineLives {
    fn write(self, world: &mut bevy::prelude::World) {
        // Lives
        let animation_lives = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: Lives { lives: self.lives },
            ..default()
        }]);

        let animation_lives_stops = animation_lives.id();

        let animation_lives = animation_lives
            .try_animating_component(self.entity)
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Last>()
            .insert(Name::new(format!("Entity {:?} Lives", self.entity)))
            .id();

        world
            .entity_mut(self.entity)
            .insert(TimelineLives {
                timeline: self.timeline,
                lives_stops: animation_lives_stops,
            })
            .push_children(&[animation_lives]);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct RespawnEvent {
    pub entity: Entity,
    pub t: f64,
    pub determinism: DeterminismId,
}

pub fn lives_death(
    mut death_events: EventReader<DeathEvent>,
    query_lives: Query<(&TimelineDamage, &TimelineLives)>,
    query_timeline: Query<&Timeline>,
    mut query_lives_stops: Query<&mut Discrete<Lives>>,
    mut respawn_events: EventWriter<RespawnEvent>,
) {
    for event in death_events.iter() {
        let (timeline_damage, timeline_lives) =
            if let Ok(components) = query_lives.get(event.entity) {
                components
            } else {
                continue;
            };

        info!("Death event for {:?}", event.entity);

        let timeline = query_timeline.get(timeline_damage.timeline).unwrap();
        let t = timeline.t;

        // If lives are available, queue a respawn
        let mut lives_stops = query_lives_stops
            .get_mut(timeline_lives.lives_stops)
            .unwrap();

        let lives = lives_stops.stops().last().unwrap().value.lives;

        if lives == 0 {
            continue;
        }

        respawn_events.send(RespawnEvent {
            entity: event.entity,
            t: t + 1.0,
            determinism: event.determinism,
        });

        // Record the current lives count so the respawn event can have a determism anchor
        lives_stops.insert_stop(DiscreteStop {
            t,
            value: Lives { lives },
            determinism: event.determinism,
            ..default()
        });
    }
}

pub fn lives_respawn(
    mut respawn_events: EventReader<RespawnEvent>,
    query_lives: Query<(Entity, &TimelineDamage, &TimelineAlive, &TimelineLives)>,
    mut query_hit_points_stops: Query<&mut Discrete<HitPoints>>,
    mut query_alive_stops: Query<&mut Discrete<Alive>>,
    mut query_lives_stops: Query<&mut Discrete<Lives>>,
) {
    for (ship_entity, timeline_damage, timeline_death, timeline_lives) in query_lives.iter() {
        for event in respawn_events.iter() {
            if event.entity != ship_entity {
                continue;
            }

            let t = event.t;

            // Decrement lives
            let mut lives_stops = query_lives_stops
                .get_mut(timeline_lives.lives_stops)
                .unwrap();

            let lives = lives_stops.stops().last().unwrap().value.lives;

            lives_stops.insert_stop(DiscreteStop {
                t,
                value: Lives { lives: lives - 1 },
                determinism: event.determinism,
                ..default()
            });

            // Reset hit points
            let mut hit_points_stops = query_hit_points_stops
                .get_mut(timeline_damage.hit_points_stops)
                .unwrap();

            hit_points_stops.insert_stop(DiscreteStop {
                t: event.t,
                value: HitPoints(1),
                determinism: event.determinism,
                ..default()
            });

            // Reset alive
            let mut alive_stops = query_alive_stops
                .get_mut(timeline_death.alive_stops)
                .unwrap();
            alive_stops.insert_stop(DiscreteStop {
                t,
                value: Alive(true),
                determinism: event.determinism,
                ..default()
            });
        }
    }
}
