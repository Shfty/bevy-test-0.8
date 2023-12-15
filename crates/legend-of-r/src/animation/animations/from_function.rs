use std::fmt::Debug;

use bevy::prelude::{Component, Entity, Name, World};

use crate::prelude::{
    Animate, AnimationBuilder, AnimationBuilderTrait, AnimationEntityBuilder, AnimationTime,
    TimelineAnimation, TimelineAnimationContext,
};

/// Animation that generates a value from a function
#[derive(Debug, Default, Copy, Clone, Component)]
pub struct FromFunction<F, T>
where
    F: Fn(&mut World, Entity, AnimationTime) -> T,
{
    pub f: F,
}

impl<F, T> Animate for FromFunction<F, T>
where
    T: 'static + Send + Sync,
    F: 'static + Send + Sync + Clone + Fn(&mut World, Entity, AnimationTime) -> T,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        let data = Self::data(world, animation).unwrap();
        let f = data.f.clone();

        f(world, animation, time)
    }
}

impl<F, T> TimelineAnimation for FromFunction<F, T>
where
    T: 'static + Send + Sync + Default,
    F: 'static + Send + Sync + Clone + Fn(&mut World, Entity, AnimationTime) -> T,
{
    fn visit(_world: &World, _animation: Entity, _timeline_ui: TimelineAnimationContext) {}
}

pub trait FromFunctionTrait<'w, 's, 'a, I, F, T>
where
    F: Fn(&mut World, Entity, AnimationTime) -> T,
{
    fn from_function(self, f: F) -> AnimationEntityBuilder<'a, I, FromFunction<F, T>>;
}

impl<'w, 's, 'a, I, F, T> FromFunctionTrait<'w, 's, 'a, I, F, T> for AnimationBuilder<'a, I>
where
    I: AnimationBuilderTrait,
    F: 'static + Send + Sync + Fn(&mut World, Entity, AnimationTime) -> T,
    T: 'static + Send + Sync,
{
    fn from_function(self, f: F) -> AnimationEntityBuilder<'a, I, FromFunction<F, T>> {
        let mut commands = self.spawn();

        commands
            .insert(Name::new("FromFunction"))
            .insert(FromFunction { f });

        commands
    }
}

