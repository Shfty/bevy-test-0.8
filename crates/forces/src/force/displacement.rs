use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    prelude::{Component, Transform, World},
    reflect::Reflect,
};
use struct_derive::WithFields;
use transform_ex::{TransformField, TransformFieldMask};

use crate::TransformDerivative;
use ecs_ex::ForeignComponent;

use super::{Force, ReflectForce};

/// Displacement force, i.e. a spring
#[derive(Debug, Default, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, MapEntities, Force)]
pub struct Displacement {
    pub from: ForeignComponent<Transform>,
    pub to: ForeignComponent<Transform>,
    pub fields: TransformFieldMask,
}

impl MapEntities for Displacement {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.from.map_entities(entity_map)?;
        self.to.map_entities(entity_map)?;
        Ok(())
    }
}

impl Force for Displacement {
    fn force(
        &self,
        world: &World,
        mut derivative: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let from = self.from.get(world)?;
        let to = self.to.get(world)?;

        if self.fields.contains(TransformField::Translation) {
            derivative.translation += to.translation - from.translation;
        }
        if self.fields.contains(TransformField::Rotation) {
            derivative.rotation += (from.rotation.inverse() * to.rotation).to_scaled_axis();
        }
        if self.fields.contains(TransformField::Scale) {
            derivative.scale += to.scale - from.scale;
        }

        Some(derivative)
    }
}
