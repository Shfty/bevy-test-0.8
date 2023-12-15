use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, World},
    reflect::Reflect,
};
use bevy_rapier3d::plugin::RapierConfiguration;
use struct_derive::WithFields;

use crate::TransformDerivative;

use crate::{Force, ReflectForce};

/// The force necessary to move an body out of collision
#[derive(Debug, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct Gravity {
    pub factor: f32,
}

impl Default for Gravity {
    fn default() -> Self {
        Gravity { factor: 1.0 }
    }
}

impl Force for Gravity {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let rapier_config = world.get_resource::<RapierConfiguration>()?;
        let deriv = rapier_config.gravity * self.factor;
        displacement.translation += deriv;
        Some(displacement)
    }
}
