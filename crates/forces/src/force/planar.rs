use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3,
    prelude::{Component, Transform, World},
    reflect::Reflect,
};
use ecs_ex::ForeignComponent;
use struct_derive::WithFields;
use transform_ex::TransformFieldMask;

use crate::TransformDerivative;

use super::{Force, ReflectForce};

/// Adapter for projecting a force onto a plane defined by the provided normal
#[derive(Debug, Default, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct Planar {
    pub normal: Vec3,
    pub reference: Option<ForeignComponent<Transform>>,
    pub fields: TransformFieldMask,
}

impl Force for Planar {
    fn force(
        &self,
        world: &World,
        displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let normal = if let Some(reference) = self.reference {
            let reference = reference.get(world)?;
            reference.rotation * self.normal
        } else {
            self.normal
        };
        Some(displacement.apply(planar(normal), self.fields))
    }
}

fn planar(normal: Vec3) -> impl Fn(Vec3) -> Vec3 {
    move |v| {
        if v != Vec3::ZERO {
            let normal = normal.normalize();
            (normal * v.dot(normal)) - v
        } else {
            Vec3::ZERO
        }
    }
}
