use std::fmt::Debug;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, Component, Entity, Name, Transform, Vec3, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilder, AnimationBuilderTrait, AnimationEntityBuilder, AnimationTime,
    ReflectIntegrationBlacklist, TimelineAnimation, TimelineAnimationContext,
};

/// Animation that produces a transform that oscillates along the Y axis
#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct AxialFunction {
    axis: Vec3,
    #[reflect(ignore)]
    f: fn(f32) -> f32,
}

impl Default for AxialFunction {
    fn default() -> Self {
        Self {
            axis: Vec3::X,
            f: |_| default(),
        }
    }
}

impl Animate for AxialFunction {
    type Type = Transform;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        let data = Self::data(world, animation).unwrap();
        let v = (data.f)(time.t as f32);
        Transform::from_translation(data.axis * v)
    }
}

impl TimelineAnimation for AxialFunction {
    fn visit(_world: &World, _animation: Entity, _timeline_ui: TimelineAnimationContext) {}
}

pub trait AxialFunctionTrait<'w, 's, 'a, I> {
    fn from_axial_function(
        self,
        axis: Vec3,
        f: fn(f32) -> f32,
    ) -> AnimationEntityBuilder<'a, I, AxialFunction>;
}

impl<'w, 's, 'a, I> AxialFunctionTrait<'w, 's, 'a, I> for AnimationBuilder<'a, I>
where
    I: AnimationBuilderTrait,
{
    fn from_axial_function(
        self,
        axis: Vec3,
        f: fn(f32) -> f32,
    ) -> AnimationEntityBuilder<'a, I, AxialFunction> {
        let mut commands = self.spawn();
        commands
            .insert(Name::new("Axial Function"))
            .insert(AxialFunction { axis, f });

        commands
    }
}
