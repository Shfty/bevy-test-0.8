use bevy::{prelude::{Entity, default}, ecs::system::Command};
use ecs_ex::entity_default;

use super::{GridMove, PendingGridMoves};

#[derive(Debug, Clone)]
pub struct PushPendingGridMoves {
    pub pending_grid_moves: Entity,
    pub moves: Vec<GridMove>,
}

impl Default for PushPendingGridMoves {
    fn default() -> Self {
        Self {
            pending_grid_moves: entity_default(),
            moves: default(),
        }
    }
}

impl Command for PushPendingGridMoves {
    fn write(self, world: &mut bevy::prelude::World) {
        if let Some(mut moves) = world
            .entity_mut(self.pending_grid_moves)
            .get_mut::<PendingGridMoves>()
        {
            moves.extend(self.moves.into_iter())
        }
    }
}

