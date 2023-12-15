use std::{marker::PhantomData, ops::Mul};

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilder, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles,
    AnimationTime, TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
};

/// Animation adapter for multiplying the results of two other animations
#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Multiply<T>
where
    T: 'static + Send + Sync + Default,
{
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for Multiply<T>
where
    T: 'static + Send + Sync + Default + Mul<T, Output = T>,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, t: AnimationTime) -> Self::Type {
        info_span!("Multiply").in_scope(|| {
            let handles = world
                .entity(animation)
                .get::<AnimationHandles<T>>()
                .unwrap();

            let lhs = handles[0];
            let rhs = handles[1];

            let lhs_component = lhs.animate(world, t);
            let rhs_component = rhs.animate(world, t);

            lhs_component.mul(rhs_component)
        })
    }
}

impl<T> TimelineAnimation for Multiply<T>
where
    T: 'static + Send + Sync + Default + Mul<T, Output = T>,
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

pub trait MultiplyTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn multiplied_with<F, U>(self, rhs: F) -> AnimationEntityBuilder<'a, I, Multiply<T::Type>>
    where
        F: FnOnce(AnimationBuilder<'a, I>) -> AnimationEntityBuilder<'a, I, U>,
        U: TimelineAnimation<Type = T::Type>;
}

impl<'w, 's, 'a, I, T> MultiplyTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn multiplied_with<F, U>(self, rhs: F) -> AnimationEntityBuilder<'a, I, Multiply<T::Type>>
    where
        F: FnOnce(AnimationBuilder<'a, I>) -> AnimationEntityBuilder<'a, I, U>,
        U: TimelineAnimation<Type = T::Type>,
    {
        let handle_lhs = self.handle();

        let commands = self.inner;
        let rhs_commands = rhs(commands);
        let handle_rhs = rhs_commands.handle();
        let commands = rhs_commands.inner;

        let mut commands = commands.spawn();
        commands
            .insert(Name::new("Multiply"))
            .insert(Multiply::<T::Type>::default())
            .insert(AnimationHandles {
                animations: vec![handle_lhs, handle_rhs],
            })
            .add_child(handle_lhs.animation)
            .add_child(handle_rhs.animation);

        commands
    }
}
