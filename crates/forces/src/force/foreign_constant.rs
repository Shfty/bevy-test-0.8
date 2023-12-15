use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    prelude::{Component, Transform, World},
    reflect::Reflect,
};
use struct_derive::WithFields;

use crate::TransformDerivative;
use ecs_ex::ForeignComponent;

use super::{Force, ReflectForce};

/// A force that pulls data from a TransformDerivative component
#[derive(Debug, Default, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, MapEntities, Force)]
pub struct ForeignConstant {
    pub target: ForeignComponent<TransformDerivative>,
    pub reference: Option<ForeignComponent<Transform>>,
}

impl MapEntities for ForeignConstant {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.target.map_entities(entity_map)?;
        Ok(())
    }
}

impl Force for ForeignConstant {
    fn force(&self, world: &World, derivative: TransformDerivative) -> Option<TransformDerivative> {
        let foreign = self.target.get(world).copied().unwrap_or_default();
        let foreign = if let Some(reference) = self.reference {
            let reference = reference.get(world).copied().unwrap_or_default();
            TransformDerivative {
                translation: reference.rotation * foreign.translation,
                rotation: reference.rotation * foreign.rotation,
                scale: reference.rotation * foreign.scale,
            }
        } else {
            foreign
        };
        Some(derivative + foreign)
    }
}
