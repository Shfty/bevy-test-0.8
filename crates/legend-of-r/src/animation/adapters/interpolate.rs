use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, Quat, Transform, Vec3, Vec4, World},
    reflect::Reflect,
};
use bevy_egui::egui::Vec2;

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    AnimationWidget, Discrete, DiscreteStop, Discretize, TimelineAnimation,
    TimelineAnimationContext,
};

/// Animation adapter that interpolates between discrete stops
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Interpolate<T>
where
    T: 'static + Send + Sync + Default,
{
    pub clamp: bool,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Animate for Interpolate<Option<T>>
where
    T: 'static + Send + Sync + Clone + Default + Lerp,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Interpolate").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let clamp = data.clamp;

            let animation = Self::handle::<Option<T>>(world, animation).animation;

            if !time.paused && time.t > time.prev_t {
                if let Some(mut discrete) = world.entity_mut(animation).get_mut::<Discrete<T>>() {
                    discrete.prune_non_deterministic(time.t);
                } else if let Some(mut discretize) =
                    world.entity_mut(animation).get_mut::<Discretize<T>>()
                {
                    discretize.discrete.prune_non_deterministic(time.t);
                } else {
                    panic!("Interpolate target has no Discrete or Discretize animation");
                };
            }

            let stops = if let Some(discrete) = world.entity(animation).get::<Discrete<T>>() {
                discrete
                    .stops()
                    .enumerate()
                    .map(|(i, stop)| (i, stop.t))
                    .collect::<Vec<_>>()
            } else if let Some(discretize) = world.entity(animation).get::<Discretize<T>>() {
                discretize
                    .discrete
                    .stops()
                    .enumerate()
                    .map(|(i, stop)| (i, stop.value))
                    .collect::<Vec<_>>()
            } else {
                panic!("Interpolate target has no Discrete or Discretize animation");
            };

            if stops.len() == 1 {
                if let Some(discrete) = world.entity(animation).get::<Discrete<T>>() {
                    let stop = discrete.stops().next().unwrap();
                    return stop.value.clone();
                } else if let Some(discretize) = world.entity(animation).get::<Discretize<T>>() {
                    let entity = world.entity(animation);
                    let handle = entity.get::<AnimationHandles<T>>().unwrap()[0];

                    let stop = discretize.discrete.stops().next().unwrap();
                    return handle.animate(
                        world,
                        AnimationTime {
                            t: stop.value,
                            ..time
                        },
                    );
                } else {
                    unreachable!()
                }
            }

            let mut stops_sorted = stops.clone();
            stops_sorted.sort_by(|(_, lhs), (_, rhs)| {
                (lhs - time.t)
                    .abs()
                    .partial_cmp(&(rhs - time.t).abs())
                    .unwrap()
            });

            let (closest_i, closest_t) = stops_sorted[0];

            let (from_i, to_i) = if closest_t < time.t {
                if closest_i == stops.len() - 1 {
                    (closest_i - 1, closest_i)
                } else {
                    (closest_i, closest_i + 1)
                }
            } else {
                if closest_i == 0 {
                    (closest_i, closest_i + 1)
                } else {
                    (closest_i - 1, closest_i)
                }
            };

            let (from, from_value, to, to_value) =
                if let Some(discrete) = world.entity(animation).get::<Discrete<T>>() {
                    let DiscreteStop {
                        t: from,
                        value: from_value,
                        ..
                    } = discrete.stops().skip(from_i).next().unwrap();

                    let DiscreteStop {
                        t: to,
                        value: to_value,
                        ..
                    } = discrete.stops().skip(to_i).next().unwrap();

                    (*from, from_value.clone(), *to, to_value.clone())
                } else if let Some(discretize) = world.entity(animation).get::<Discretize<T>>() {
                    let from = discretize
                        .discrete
                        .stops()
                        .skip(from_i)
                        .next()
                        .unwrap()
                        .value;

                    let to = discretize.discrete.stops().skip(to_i).next().unwrap().value;

                    let entity = world.entity(animation);
                    let handle = entity.get::<AnimationHandles<T>>().unwrap()[0];

                    let from_value = handle.animate(world, AnimationTime { t: from, ..time });
                    let to_value = handle.animate(world, AnimationTime { t: to, ..time });

                    (from, from_value, to, to_value)
                } else {
                    unreachable!()
                };

            if time.t == from {
                return from_value;
            } else if time.t == to {
                return to_value;
            }

            let delta = time.t - from;
            let len = (to - from).abs();
            let mut t = if len > 0.0 { delta / len } else { 0.0 };

            if clamp {
                t = t.clamp(0.0, 1.0)
            }

            from_value.lerp(&to_value, t as f32)
        })
    }
}

impl<T> TimelineAnimation for Interpolate<Option<T>>
where
    T: 'static + Send + Sync + Clone + Default + Lerp,
{
    fn visit(
        world: &World,
        animation: Entity,
        TimelineAnimationContext {
            timeline_ui,
            animation_ui,
        }: TimelineAnimationContext,
    ) {
        Self::handle::<Option<T>>(world, animation).visit(
            world,
            TimelineAnimationContext {
                timeline_ui,
                animation_ui,
            },
        );

        let timeline_animation = if let Some(AnimationWidget::TimelineAnimation(animation)) =
            animation_ui
                .plot_widgets
                .iter_mut()
                .find(|animation| matches!(animation, AnimationWidget::TimelineAnimation(_)))
        {
            animation
        } else {
            return;
        };

        timeline_animation.min_t = None;
        timeline_animation.max_t = None;
    }
}

pub trait Lerp {
    fn lerp(&self, rhs: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        lerp::Lerp::lerp(*self, *rhs, t)
    }
}

impl Lerp for Vec2 {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        lerp::Lerp::lerp(*self, *rhs, t)
    }
}

impl Lerp for Vec3 {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        lerp::Lerp::lerp(*self, *rhs, t)
    }
}

impl Lerp for Vec4 {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        lerp::Lerp::lerp(*self, *rhs, t)
    }
}

impl Lerp for Quat {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        self.slerp(*rhs, t)
    }
}

impl Lerp for Transform {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        Transform {
            translation: self.translation.lerp(rhs.translation, t),
            rotation: self.rotation.slerp(rhs.rotation, t),
            scale: self.scale.lerp(rhs.scale, t),
        }
    }
}

pub trait InterpolateTrait<'w, 's, 'a, I, T, V>: Sized
where
    T: Animate<Type = Option<V>>,
    V: 'static + Send + Sync + Default,
{
    fn interpolate(self) -> AnimationEntityBuilder<'a, I, Interpolate<T::Type>> {
        self.interpolate_impl(false)
    }

    fn interpolate_clamped(self) -> AnimationEntityBuilder<'a, I, Interpolate<T::Type>> {
        self.interpolate_impl(true)
    }

    fn interpolate_impl(self, clamp: bool) -> AnimationEntityBuilder<'a, I, Interpolate<T::Type>>;
}

impl<'w, 's, 'a, I, T, V> InterpolateTrait<'w, 's, 'a, I, T, V> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation<Type = Option<V>>,
    V: 'static + Send + Sync + Default,
{
    fn interpolate_impl(self, clamp: bool) -> AnimationEntityBuilder<'a, I, Interpolate<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();
        commands
            .insert(Name::new("Interpolate"))
            .insert(Interpolate::<T::Type> { clamp, ..default() })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
