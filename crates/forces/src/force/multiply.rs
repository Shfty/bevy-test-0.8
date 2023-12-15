use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3,
    prelude::{Component, World},
    reflect::Reflect,
};
use struct_derive::WithFields;
use transform_ex::TransformFieldMask;

use crate::TransformDerivative;

use super::{Force, ReflectForce};

/// Multiplication adapter, i.e. spring tension or damping factor
#[derive(Debug, Default, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct Multiply {
    pub multiplier: f32,
    pub fields: TransformFieldMask,
}

impl Force for Multiply {
    fn force(&self, _: &World, displacement: TransformDerivative) -> Option<TransformDerivative> {
        Some(displacement.apply(mul(self.multiplier), self.fields))
    }
}

fn mul(multiplier: f32) -> impl Fn(Vec3) -> Vec3 {
    move |v| v * multiplier
}
