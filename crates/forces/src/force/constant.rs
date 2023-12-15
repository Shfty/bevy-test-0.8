use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, Transform, World},
    reflect::Reflect,
};
use ecs_ex::ForeignComponent;
use struct_derive::WithFields;
use transform_ex::{TransformField, TransformFieldMask};

use crate::TransformDerivative;

use super::{Force, ReflectForce};

/// A constant force, i.e. a motor or offset
#[derive(Debug, Default, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct Constant {
    pub constant: TransformDerivative,
    pub reference: Option<ForeignComponent<Transform>>,
    pub fields: TransformFieldMask,
}

impl Force for Constant {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        if self.fields.contains(TransformField::Translation) {
            displacement.translation += self.constant.translation;
        }

        if self.fields.contains(TransformField::Rotation) {
            displacement.rotation += self.constant.rotation;
        }

        if self.fields.contains(TransformField::Scale) {
            displacement.scale += self.constant.scale;
        }

        if let Some(reference) = self.reference {
            let reference = reference.get(world).copied().unwrap_or_default();
            displacement = TransformDerivative {
                translation: reference.rotation * displacement.translation,
                rotation: reference.rotation * displacement.rotation,
                scale: displacement.scale,
            }
        }

        Some(displacement)
    }
}
