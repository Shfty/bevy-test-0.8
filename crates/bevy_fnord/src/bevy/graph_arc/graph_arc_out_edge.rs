use bevy::{
    ecs::entity::MapEntities,
    ecs::reflect::ReflectComponent,
    prelude::{Component, Entity},
    reflect::Reflect,
};
use ecs_ex::entity_default;

/// A connection between two edges
#[derive(Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct GraphArcOutEdge {
    pub edge_out: Entity,
}

impl Default for GraphArcOutEdge {
    fn default() -> Self {
        Self {
            edge_out: entity_default(),
        }
    }
}

impl MapEntities for GraphArcOutEdge {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.edge_out = entity_map.get(self.edge_out)?;
        Ok(())
    }
}

