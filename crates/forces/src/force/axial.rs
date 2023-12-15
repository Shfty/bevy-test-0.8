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
/// Adapter for projecting a force along the provided axis
#[derive(Debug, Default, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct Axial {
    pub axis: Vec3,
    pub fields: TransformFieldMask,
}

impl Force for Axial {
    fn force(&self, _: &World, displacement: TransformDerivative) -> Option<TransformDerivative> {
        Some(displacement.apply(axial(self.axis), self.fields))
    }
}

fn axial(axis: Vec3) -> impl Fn(Vec3) -> Vec3 {
    move |v| {
        if v != Vec3::ZERO {
            axis.normalize() * v.dot(axis)
        } else {
            Vec3::ZERO
        }
    }
}
