use crate::prelude::{
    Board, BoardTransformBundle, CollisionLayer, GlobalBoardTransform, 
};
use bevy::prelude::{default, Bundle, Changed, Component, Entity, Query, RemovedComponents};

use anyhow::Result;
use bevy_instancing::prelude::InstanceColor;
use ecs_ex::entity_default;

#[derive(Debug, Copy, Clone, Component)]
pub struct Voxel {
    pub board: Entity,
    pub collision_layer: CollisionLayer,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            board: entity_default(),
            collision_layer: default(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Bundle)]
pub struct VoxelBundle {
    pub voxel: Voxel,
    pub mesh_instance_color: InstanceColor,
    #[bundle]
    pub position_bundle: BoardTransformBundle,
}

pub fn update_voxels(
    mut query_board: Query<&mut Board>,
    mut query_voxel: Query<
        (Entity, &GlobalBoardTransform, &mut Voxel),
        Changed<GlobalBoardTransform>,
    >,
) -> Result<()> {
    for (entity, board_transform, voxel) in query_voxel.iter_mut() {
        let mut board = query_board.get_mut(voxel.board)?;
        board.move_voxel(entity, board_transform.translation);
    }

    Ok(())
}

pub fn voxel_removed(removed: RemovedComponents<Voxel>, mut query_board: Query<&mut Board>) {
    for entity in removed.iter() {
        for mut board in query_board.iter_mut() {
            board.remove_voxel(entity);
        }
    }
}
