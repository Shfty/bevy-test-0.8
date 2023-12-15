pub mod push_pending_grid_moves;

use bevy::prelude::{Bundle, Component, Deref, DerefMut};

use crate::prelude::BoardTransform;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MoveType {
    Hit,
    Land,
    Lock,
    Delete,
    Displace,
}

impl Default for MoveType {
    fn default() -> Self {
        MoveType::Hit
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct GridMove {
    pub delta: BoardTransform,
    pub move_type: MoveType,
}

#[derive(Debug, Default, Clone, Deref, DerefMut, Component)]
pub struct PendingGridMoves(Vec<GridMove>);

#[derive(Debug, Default, Clone, Deref, DerefMut, Component)]
pub struct FailedGridMoves(Vec<GridMove>);

#[derive(Debug, Default, Clone, Deref, DerefMut, Component)]
pub struct SuccessfulGridMoves(Vec<GridMove>);

#[derive(Debug, Default, Clone, Bundle)]
pub struct GridMovesBundle {
    moves: PendingGridMoves,
    succesful_moves: SuccessfulGridMoves,
    failed_moves: FailedGridMoves,
}
