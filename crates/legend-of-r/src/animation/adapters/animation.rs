use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimatePointer, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles,
    AnimationTime, TimelineAnimation, TimelineAnimationContext,
};

/// Unit animation adapter, passes through and does nothing else
///
/// Useful for breaking a handle out of an animation chain
/// for later manual evaluation
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Animation<T>
where
    T: 'static + Send + Sync + Default,
    AnimatePointer<T>: Copy,
{
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for Animation<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Offset")
            .in_scope(|| Self::handle::<T>(world, animation).animate(world, time))
    }
}

impl<T> TimelineAnimation for Animation<T>
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
        Self::handle::<T>(world, animation).visit(
            world,
            TimelineAnimationContext {
                timeline_ui,
                animation_ui,
            },
        );
    }
}

pub trait AnimationTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn as_animation(self) -> AnimationEntityBuilder<'a, I, Animation<T::Type>>;
}

impl<'w, 's, 'a, I, T> AnimationTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn as_animation(self) -> AnimationEntityBuilder<'a, I, Animation<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();
        commands
            .insert(Name::new("Offset"))
            .insert(Animation::<T::Type>::default())
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}

