use bevy::{
    asset::{self as bevy_asset, load_internal_asset},
    hierarchy::HierarchySystem,
    prelude::{
        shape::Cube, AddAsset, Assets, Commands, CoreStage, Handle, Mesh,
        ParallelSystemDescriptorCoercion, Plugin, ResMut, SystemSet,
    },
};

use crate::prelude::{
    board_added, cell_mesh_added, clear_failed_moves, clear_succesful_moves, delayed_auto_repeat,
    model_instance_added, move_model_instances, propagate_global_positions, update_voxels,
    voxel_removed, Board, BoardComputePlugin, BoardMaterial, BoardRotation, BoardTransform, Cell,
    CellMesh, GlobalBoardTransform, 
};

use bevy_instancing::prelude::InstancedMaterialPlugin;

use result_system::ResultSystem;

use super::{board_flush_moves, board_flush_removes};

pub struct BoardPlugin;

use bevy::{
    prelude::{HandleUntyped, Shader},
    reflect::TypeUuid,
};

pub const BOARD_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2832496304849745969);

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            BOARD_SHADER_HANDLE,
            "material/board.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(BoardComputePlugin);

        app.register_type::<Board>()
            .register_type::<BoardTransform>()
            .register_type::<GlobalBoardTransform>()
            .register_type::<BoardRotation>()
            .register_type::<Cell>();

        app.add_startup_system(setup_board);

        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(board_added)
                .with_system(cell_mesh_added.after(board_added))
                .with_system(clear_succesful_moves)
                .with_system(clear_failed_moves),
        );

        app.add_system(delayed_auto_repeat)
            .add_system(move_model_instances.after(delayed_auto_repeat))
            .add_system(model_instance_added);

        app.add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(propagate_global_positions.after(HierarchySystem::ParentUpdate))
                .with_system(
                    voxel_removed
                        .after(propagate_global_positions)
                        .before(board_flush_removes),
                )
                .with_system(
                    update_voxels
                        .result_system()
                        .after(propagate_global_positions)
                        .before(board_flush_moves),
                )
                .with_system(board_flush_moves)
                .with_system(board_flush_removes),
        );

        // Material
        app.add_asset::<BoardMaterial>()
            .add_plugin(InstancedMaterialPlugin::<BoardMaterial>::default());

        app.world
            .resource_mut::<Assets<BoardMaterial>>()
            .set_untracked(Handle::<BoardMaterial>::default(), BoardMaterial::default());
    }
}

pub fn setup_board(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let mesh = meshes.add(Cube::default().into());
    commands.insert_resource(CellMesh(mesh));
}
