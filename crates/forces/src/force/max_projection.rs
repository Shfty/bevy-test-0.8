use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3,
    prelude::{default, Component, Transform, World},
    reflect::Reflect,
};
use ecs_ex::ForeignComponent;
use struct_derive::WithFields;
use transform_ex::{TransformField, TransformFieldMask};

use crate::TransformDerivative;

use super::{Force, ReflectForce};

/// Force adapter, converts a displacement into a max-length force along a given axis
#[derive(Debug, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct MaxProjection {
    pub axis: Vec3,
    pub reference: Option<ForeignComponent<Transform>>,
    pub min: f32,
    pub max: f32,
    pub fields: TransformFieldMask,
}

impl Default for MaxProjection {
    fn default() -> Self {
        Self {
            axis: -Vec3::Z,
            reference: None,
            min: -1.0,
            max: 1.0,
            fields: default(),
        }
    }
}

impl Force for MaxProjection {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let axis = if let Some(reference) = self.reference {
            let reference = reference.get(world)?;
            reference.rotation * self.axis
        } else {
            self.axis
        };

        if self.fields.contains(TransformField::Translation) {
            displacement.translation +=
                proj_disp(displacement.translation, axis, self.min, self.max);
        }

        if self.fields.contains(TransformField::Rotation) {
            displacement.rotation += proj_disp(displacement.rotation, axis, self.min, self.max);
        }

        if self.fields.contains(TransformField::Scale) {
            displacement.scale += proj_disp(displacement.scale, axis, self.min, self.max);
        }

        Some(displacement)
    }
}

fn proj_disp(displacement: Vec3, axis: Vec3, min: f32, max: f32) -> Vec3 {
    let axis = axis.normalize();

    let delta_proj = axis.dot(displacement);
    let delta_proj = if delta_proj > 0.0 {
        -(delta_proj - max).max(0.0)
    } else if delta_proj < 0.0 {
        (min - delta_proj).max(0.0)
    } else {
        0.0
    };

    axis * delta_proj
}
