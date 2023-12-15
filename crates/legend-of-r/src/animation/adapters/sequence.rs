use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilder, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles,
    AnimationTime, TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
};

/// Animation adapter for playing one animation after another
#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Sequence<T>
where
    T: 'static + Send + Sync + Default,
{
    pub stop: f64,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Animate for Sequence<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Sequence").in_scope(|| {
            let handles = world
                .entity(animation)
                .get::<AnimationHandles<T>>()
                .unwrap();
            let lhs = handles[0];
            let rhs = handles[1];

            let data = Self::data(world, animation).unwrap();
            let stop = data.stop;

            if time.t < stop {
                lhs.animate(world, time)
            } else {
                rhs.animate(
                    world,
                    AnimationTime {
                        t: time.t - stop,
                        prev_t: time.prev_t - stop,
                        ..time
                    },
                )
            }
        })
    }
}

impl<T> TimelineAnimation for Sequence<T>
where
    T: 'static + Send + Sync + Default,
{
    fn visit(
        world: &World,
        animation: Entity,
        TimelineAnimationContext {
            timeline_ui,
            animation_ui,
        }: TimelineAnimationContext,
    ) {
        for handle in world
            .entity(animation)
            .get::<AnimationHandles<T>>()
            .unwrap()
            .iter()
        {
            handle.visit(
                world,
                TimelineAnimationContext {
                    timeline_ui,
                    animation_ui,
                },
            );
        }
    }
}

pub trait SequenceTrait<'w, 's, 'a, 'b, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn sequenced_with<F, U>(
        self,
        rhs: F,
        stop: f64,
    ) -> AnimationEntityBuilder<'b, I, Sequence<T::Type>>
    where
        F: FnOnce(AnimationBuilder<'b, I>) -> AnimationEntityBuilder<'b, I, U>,
        U: TimelineAnimation<Type = T::Type>;
}

impl<'w, 's, 'a: 'b, 'b, I, T> SequenceTrait<'w, 's, 'a, 'b, I, T>
    for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn sequenced_with<F, U>(
        self,
        rhs: F,
        stop: f64,
    ) -> AnimationEntityBuilder<'b, I, Sequence<T::Type>>
    where
        F: FnOnce(AnimationBuilder<'b, I>) -> AnimationEntityBuilder<'b, I, U>,
        U: TimelineAnimation<Type = T::Type>,
    {
        let handle_lhs = self.handle();

        let commands = self.inner;
        let rhs_commands = rhs(commands);
        let handle_rhs = rhs_commands.handle();
        let commands = rhs_commands.inner;

        let mut commands = commands.spawn();
        commands
            .insert(Name::new("Sequence"))
            .insert(Sequence::<T::Type> { stop, ..default() })
            .insert(AnimationHandles {
                animations: vec![handle_lhs, handle_rhs],
            })
            .add_child(handle_lhs.animation)
            .add_child(handle_rhs.animation);

        commands
    }
}
