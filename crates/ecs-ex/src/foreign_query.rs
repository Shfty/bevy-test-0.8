use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy::{
    ecs::{
        entity::MapEntities,
        query::{Fetch, QueryEntityError, QueryItem, WorldQuery},
    },
    prelude::{default, Entity, Query},
    reflect::Reflect,
};
use struct_derive::WithFields;

use crate::entity_default;

/// Wrapper struct for strongly typing a query against another entity
#[derive(Debug, Copy, Clone, WithFields, Reflect)]
pub struct ForeignQuery<T: 'static + WorldQuery + Send + Sync> {
    entity: Entity,
    #[reflect(ignore)]
    #[with_fields(ignore)]
    _phantom: PhantomData<T>,
}

impl<T: 'static + WorldQuery + Send + Sync> Default for ForeignQuery<T> {
    fn default() -> Self {
        Self {
            entity: entity_default(),
            _phantom: default(),
        }
    }
}

impl<T: 'static + WorldQuery + Send + Sync> From<Entity> for ForeignQuery<T> {
    fn from(entity: Entity) -> Self {
        ForeignQuery::new(entity)
    }
}

impl<T: 'static + WorldQuery + Send + Sync> Into<Entity> for ForeignQuery<T> {
    fn into(self) -> Entity {
        self.entity
    }
}

impl<T: 'static + WorldQuery + Send + Sync> Deref for ForeignQuery<T> {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T: 'static + WorldQuery + Send + Sync> DerefMut for ForeignQuery<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

pub type QueryReadOnlyItem<'w, 's, Q> =
    <<Q as WorldQuery>::ReadOnlyFetch as Fetch<'w, 's>>::Item;

impl<T: 'static + WorldQuery + Send + Sync> ForeignQuery<T> {
    pub fn new(entity: Entity) -> Self {
        ForeignQuery {
            entity,
            _phantom: default(),
        }
    }

    pub fn query<'s>(
        &'s self,
        query: &'s Query<T>,
    ) -> Result<QueryReadOnlyItem<'_, 's, T>, QueryEntityError> {
        query.get(self.entity)
    }

    pub fn query_mut<'s>(
        &'s self,
        query: &'s mut Query<T>,
    ) -> Result<QueryItem<T>, QueryEntityError> {
        query.get_mut(self.entity)
    }
}

impl<T: 'static + WorldQuery + Send + Sync> MapEntities for ForeignQuery<T> {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.entity = entity_map.get(self.entity)?;
        Ok(())
    }
}

