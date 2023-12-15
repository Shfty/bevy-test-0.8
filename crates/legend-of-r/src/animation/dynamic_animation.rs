//! Utility module for spawning two-stop animations designed to be
//! updated by other animations

use std::marker::PhantomData;

use bevy::{
    ecs::system::Command,
    prelude::{default, BuildWorldChildren, Component, Entity, Name},
};

use crate::util::default_entity;

use super::{
    adapters::{
        animate_component_fields::AnimateComponentFieldsTrait,
        evaluate::EvaluateTrait,
        interpolate::{InterpolateTrait, Lerp},
        time_source::TimeSourceTrait,
        try_animate_component_fields::TryAnimateComponentFieldsTrait,
    },
    animations::discrete::{Discrete, DiscreteStop, DiscreteStopsTrait},
    timeline::TimelineTime,
    AnimationTagTrait, BuildAnimation, Update,
};

#[derive(Debug, Copy, Clone, Component)]
pub struct DynamicAnimation<C, T> {
    pub timeline: Entity,
    pub from_stops: Entity,
    pub to_stops: Entity,
    pub _phantom: PhantomData<(C, T)>,
}

impl<C, T> Default for DynamicAnimation<C, T> {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            from_stops: default_entity(),
            to_stops: default_entity(),
            _phantom: default(),
        }
    }
}

pub struct InsertDynamicAnimation<C, T> {
    pub timeline: Entity,
    pub entity: Entity,
    pub target: Entity,
    pub accessor: fn(&C) -> &T,
    pub mutator: fn(&mut C) -> &mut T,
}

impl<C, T> Command for InsertDynamicAnimation<C, T>
where
    C: Component,
    T: 'static + Send + Sync + Default + Clone + PartialEq + Lerp,
{
    fn write(self, world: &mut bevy::prelude::World) {
        let dynamic = world.build_animation().from_discrete_stops([
            DiscreteStop {
                t: 0.0,
                value: T::default(),
                ..default()
            },
            DiscreteStop {
                t: 1.0,
                value: T::default(),
                ..default()
            },
        ]);

        let dynamic_stops = dynamic.id();

        let dynamic = dynamic
            .interpolate_clamped()
            .animating_component_field(self.target, self.accessor, self.mutator)
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new(format!("Dynamic Animation")))
            .id();

        world.entity_mut(self.target).push_children(&[dynamic]);

        let dynamic_from = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: DiscreteStop {
                t: 0.0,
                value: T::default(),
                ..default()
            },
            ..default()
        }]);

        let dynamic_from_stops = dynamic_from.id();

        let dynamic_from = dynamic_from
            .try_animating_component_field::<Discrete<T>, _, _>(
                dynamic_stops,
                |discrete| discrete.stops().next().unwrap(),
                |discrete| discrete.stops_mut().next().unwrap(),
            )
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new("Dynamic Animation From"))
            .id();

        let dynamic_to = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: DiscreteStop {
                t: 1.0,
                value: T::default(),
                ..default()
            },
            ..default()
        }]);

        let dynamic_to_stops = dynamic_to.id();

        let rotation_to = dynamic_to
            .try_animating_component_field::<Discrete<T>, _, _>(
                dynamic_stops,
                |discrete| discrete.stops().last().unwrap(),
                |discrete| discrete.stops_mut().last().unwrap(),
            )
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new("Dynamic Animation To"))
            .id();

        world
            .entity_mut(dynamic_stops)
            .push_children(&[dynamic_from, rotation_to]);

        world
            .entity_mut(self.entity)
            .insert(DynamicAnimation::<C, T> {
                timeline: self.timeline,
                from_stops: dynamic_from_stops,
                to_stops: dynamic_to_stops,
                ..default()
            });
    }
}
