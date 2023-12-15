use std::num::NonZeroU64;

use bevy::{
    math::UVec3,
    prelude::{debug, default, Commands, Handle, Query, Res, With},
    render::{
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferBinding,
            BufferInitDescriptor, BufferUsages,
        },
        renderer::RenderDevice,
    },
};
use bytemuck::{Pod, Zeroable};

use crate::prelude::Board;

use bevy_instancing::prelude::{
    GpuColorMeshInstance, InstanceBlockBuffer, InstanceBlockRange, SpecializedInstancedMaterial,
};

use super::{board_buffer::BoardBuffer, BoardComputePipeline};

/// The collection of bind groups and other data necessary to compute one set of instance data
pub struct BoardComputeJob {
    pub bind_group_uniform: BindGroup,
    pub bind_group_storage: BindGroup,
    pub cell_count: u32,
}

/// Resource containing pending [IndirectComputeJob]s
pub struct BoardComputeQueue(pub Vec<BoardComputeJob>);

#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Uniforms {
    board_size: UVec3,
    _pad0: u32,
    board_visible_min: UVec3,
    _pad1: u32,
    board_visible_max: UVec3,
    _pad2: u32,
}

/// Creates [IndirectComputeJob]s from bind groups and pushes them into the [IndirectComputeQueue]
pub fn queue_board_compute_jobs<M: SpecializedInstancedMaterial>(
    mut commands: Commands,
    pipeline: Res<BoardComputePipeline>,
    render_device: Res<RenderDevice>,
    query_board_buffer: Query<(&Board, &BoardBuffer)>,
    query_instance_block: Query<(&InstanceBlockRange, &InstanceBlockBuffer), With<Handle<M>>>,
) {
    let mut bind_groups_queue = vec![];

    for (board, board_buffer) in query_board_buffer.iter() {
        if board_buffer.length == 0 {
            continue;
        }

        debug!(
            "Board opaque meshes: {:?}, transparent meshes: {:?}",
            board.instance_block_opaque, board.instance_block_transparent
        );

        let (instance_block_range_opaque, instance_block_buffer_opaque) =
            if let Ok(components) = query_instance_block.get(board.instance_block_opaque) {
                components
            } else {
                continue;
            };

        let (instance_block_range_transparent, instance_block_buffer_transparent) =
            if let Ok(components) = query_instance_block.get(board.instance_block_transparent) {
                components
            } else {
                continue;
            };

        let uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::bytes_of(&Uniforms {
                board_size: board.size,
                board_visible_min: board.visible_min.unwrap_or_default(),
                board_visible_max: board.visible_max.unwrap_or(board.size - UVec3::ONE),
                ..default()
            }),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group_uniform = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.bind_group_layout_uniform,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        debug!(
            "Opaque offset: {:?}, size: {:?}",
            instance_block_range_opaque.offset, instance_block_range_opaque.instance_count
        );
        debug!(
            "Transparent offset: {:?}, size: {:?}",
            instance_block_range_transparent.offset,
            instance_block_range_transparent.instance_count
        );

        let bind_group_storage = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.bind_group_layout_storage,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: board_buffer.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &instance_block_buffer_opaque.buffer,
                        offset: std::mem::size_of::<GpuColorMeshInstance>() as u64
                            * instance_block_range_opaque.offset,
                        size: NonZeroU64::new(
                            std::mem::size_of::<GpuColorMeshInstance>() as u64
                                * instance_block_range_opaque.instance_count,
                        ),
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &instance_block_buffer_transparent.buffer,
                        offset: std::mem::size_of::<GpuColorMeshInstance>() as u64
                            * instance_block_range_transparent.offset,
                        size: NonZeroU64::new(
                            (std::mem::size_of::<GpuColorMeshInstance>() as u64
                                * instance_block_range_transparent.instance_count)
                                as u64,
                        ),
                    }),
                },
            ],
        });

        debug!(
            "Queueing board compute job for {} cells",
            board_buffer.length
        );
        bind_groups_queue.push(BoardComputeJob {
            bind_group_uniform,
            bind_group_storage,
            cell_count: board_buffer.length as u32,
        });
    }

    commands.insert_resource(BoardComputeQueue(bind_groups_queue));
}
