use bevy::{
    asset::load_internal_asset,
    core_pipeline::node::MAIN_PASS_DEPENDENCIES,
    prelude::{App, HandleUntyped, ParallelSystemDescriptorCoercion, Plugin, Shader},
    reflect::TypeUuid,
    render::{
        render_component::ExtractComponentPlugin, render_graph::RenderGraph, RenderApp, RenderStage,
    },
};

use crate::prelude::{Board, BoardMaterial};

use bevy_instancing::prelude::queue_instanced_materials;

use super::{
    board_buffer::prepare_board_buffer, queue_board_compute_jobs, BoardComputeNode,
    BoardComputePipeline,
};

pub const COMPUTE_INSTANCES_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10731426847536679408);

pub struct BoardComputePlugin;

use bevy::asset as bevy_asset;

impl Plugin for BoardComputePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            COMPUTE_INSTANCES_HANDLE,
            "compute_instances.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractComponentPlugin::<Board>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<BoardComputePipeline>()
            .add_system_to_stage(RenderStage::Prepare, prepare_board_buffer)
            .add_system_to_stage(
                RenderStage::Queue,
                queue_board_compute_jobs::<BoardMaterial>
                    .after(queue_instanced_materials::<BoardMaterial>),
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("board_compute", BoardComputeNode::default());
        render_graph
            .add_node_edge("board_compute", MAIN_PASS_DEPENDENCIES)
            .unwrap();
    }
}
