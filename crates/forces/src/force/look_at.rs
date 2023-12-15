use bevy::{
    ecs::reflect::ReflectComponent,
    math::{Quat, Vec3},
    prelude::{default, Component, Transform, World},
    reflect::Reflect,
};
use ecs_ex::ForeignComponent;
use struct_derive::WithFields;
use transform_ex::quat_ex::QuatEx;

use crate::TransformDerivative;

use super::{Force, ReflectForce};

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub enum LookAlong {
    X,
    Y,
    Z,
}

impl Default for LookAlong {
    fn default() -> Self {
        LookAlong::Z
    }
}

/// A look-at force that converts a positional displacement into a rotational one
/// Note: Requires a displacement force before it in the chain
#[derive(Debug, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct LookAt {
    pub target: ForeignComponent<Transform>,

    pub reference: ForeignComponent<Transform>,
    pub reference_axis: Vec3,

    pub along_axis: LookAlong,
}

impl Default for LookAt {
    fn default() -> Self {
        Self {
            target: default(),
            reference: default(),
            reference_axis: Vec3::Y,
            along_axis: LookAlong::Z,
        }
    }
}

impl Force for LookAt {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let reference_trx = *self.reference.get(world)?;
        let reference_axis = reference_trx.rotation * self.reference_axis;

        let delta = displacement.translation;

        let look_at_rotation = if delta != Vec3::ZERO {
            let delta = delta.normalize();
            match self.along_axis {
                LookAlong::X => Quat::look_to_right(delta, reference_axis.normalize()),
                LookAlong::Y => Quat::look_to_up(delta, reference_axis.normalize()),
                LookAlong::Z => Quat::look_to_forward(delta, reference_axis.normalize()),
            }
        } else {
            Quat::IDENTITY
        };

        let target_rotation = self.target.get(world)?.rotation;
        let rotation = target_rotation.inverse() * look_at_rotation;
        displacement.translation = default();
        displacement.rotation = rotation.to_scaled_axis();

        Some(displacement)
    }
}
