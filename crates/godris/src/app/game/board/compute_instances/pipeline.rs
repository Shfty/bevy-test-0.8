use std::borrow::Cow;

use bevy::{
    prelude::{debug, FromWorld, Shader, World},
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
    },
};

use crate::prelude::COMPUTE_INSTANCES_HANDLE;

pub struct BoardComputePipeline {
    pub pipeline: CachedComputePipelineId,
    pub bind_group_layout_uniform: BindGroupLayout,
    pub bind_group_layout_storage: BindGroupLayout,
}

impl FromWorld for BoardComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout_uniform =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: bevy::render::render_resource::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group_layout_storage =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: bevy::render::render_resource::BufferBindingType::Storage {
                                read_only: true,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: bevy::render::render_resource::BufferBindingType::Storage {
                                read_only: false,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: bevy::render::render_resource::BufferBindingType::Storage {
                                read_only: false,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![
                bind_group_layout_uniform.clone(),
                bind_group_layout_storage.clone(),
            ]),
            shader: COMPUTE_INSTANCES_HANDLE.typed::<Shader>(),
            shader_defs: vec![],
            entry_point: Cow::from("instances"),
        });

        debug!("Created BoardComputePipeline");

        BoardComputePipeline {
            pipeline,
            bind_group_layout_uniform,
            bind_group_layout_storage,
        }
    }
}
