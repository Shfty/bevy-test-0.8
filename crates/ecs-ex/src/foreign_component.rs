use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use bevy::{
    ecs::entity::MapEntities,
    prelude::{default, Component, Entity, Mut, World},
    reflect::Reflect,
};
use struct_derive::WithFields;

use crate::entity_default;

/// Wrapper struct for strongly typing a component reference
#[derive(Debug, Copy, Clone, Serialize, Deserialize, WithFields, Reflect)]
pub struct ForeignComponent<T: Component> {
    entity: Entity,
    #[reflect(ignore)]
    #[with_fields(ignore)]
    _phantom: PhantomData<T>,
}

impl<T: Component> Default for ForeignComponent<T> {
    fn default() -> Self {
        Self {
            entity: entity_default(),
            _phantom: default(),
        }
    }
}

impl<T: Component> From<Entity> for ForeignComponent<T> {
    fn from(entity: Entity) -> Self {
        ForeignComponent::new(entity)
    }
}

impl<T: Component> Into<Entity> for ForeignComponent<T> {
    fn into(self) -> Entity {
        self.entity
    }
}

impl<T: Component> Deref for ForeignComponent<T> {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T: Component> DerefMut for ForeignComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<T: Component> ForeignComponent<T> {
    pub fn new(entity: Entity) -> Self {
        ForeignComponent {
            entity,
            _phantom: default(),
        }
    }

    pub fn get<'w>(&self, world: &'w World) -> Option<&'w T> {
        world.get(self.entity)
    }

    pub fn get_mut<'w>(&self, world: &'w mut World) -> Option<Mut<'w, T>> {
        world.get_mut(self.entity)
    }
}

impl<T: Component> MapEntities for ForeignComponent<T> {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.entity = entity_map.get(self.entity)?;
        Ok(())
    }
}
