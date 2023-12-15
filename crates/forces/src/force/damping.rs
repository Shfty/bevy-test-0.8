use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    prelude::{Component, World},
    reflect::Reflect,
};
use struct_derive::WithFields;
use transform_ex::{TransformField, TransformFieldMask};

use crate::TransformDerivative;
use ecs_ex::ForeignComponent;

use super::{Force, ReflectForce, Velocity};

/// Damping force, applies inverse of velocity
#[derive(Debug, Default, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, MapEntities, Force)]
pub struct Damping {
    pub target: ForeignComponent<Velocity>,
    pub fields: TransformFieldMask,
}

impl MapEntities for Damping {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.target.map_entities(entity_map)?;
        Ok(())
    }
}

impl Force for Damping {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let velocity = self.target.get(world)?;
        if self.fields.contains(TransformField::Translation) {
            displacement.translation -= velocity.translation;
        }
        if self.fields.contains(TransformField::Rotation) {
            displacement.rotation -= velocity.rotation;
        }
        if self.fields.contains(TransformField::Scale) {
            displacement.scale -= velocity.scale;
        }
        Some(displacement)
    }
}
