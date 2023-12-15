use std::marker::PhantomData;

use bevy::{
    ecs::entity::MapEntities,
    ecs::reflect::{ReflectComponent, ReflectMapEntities},
    prelude::{default, Component, Entity},
    reflect::Reflect,
};
use ecs_ex::entity_default;

use crate::prelude::EdgeType;

/// Reference to the Arc connected to a given Edge
#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct EdgeArc<T>
where
    T: 'static + Send + Sync + EdgeType,
{
    pub arc: Entity,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for EdgeArc<T>
where
    T: 'static + Send + Sync + EdgeType,
{
    fn default() -> Self {
        Self {
            arc: entity_default(),
            _phantom: default(),
        }
    }
}

impl<T> MapEntities for EdgeArc<T>
where
    T: 'static + Send + Sync + EdgeType,
{
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.arc = entity_map.get(self.arc)?;
        Ok(())
    }
}
