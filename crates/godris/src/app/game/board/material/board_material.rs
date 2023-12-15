use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::AlphaMode,
    prelude::{default, AssetServer, Handle, Shader},
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, Face,
            RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
        renderer::RenderDevice,
    },
};

use crate::prelude::BOARD_SHADER_HANDLE;

use bevy_instancing::prelude::{InstancedMaterialPipeline, SpecializedInstancedMaterial, ColorMeshInstance};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "6dc3b9fc-fcfd-4149-8f20-5d3a1573e5da"]
pub struct BoardMaterial {
    pub alpha_mode: AlphaMode,
    pub cull_mode: Option<Face>,
}

impl Default for BoardMaterial {
    fn default() -> Self {
        Self {
            alpha_mode: default(),
            cull_mode: Some(Face::Back),
        }
    }
}

#[derive(Clone)]
pub struct GpuBoardMaterial {
    pub bind_group: BindGroup,
    pub alpha_mode: AlphaMode,
    pub cull_mode: Option<Face>,
}

impl RenderAsset for BoardMaterial {
    type ExtractedAsset = BoardMaterial;
    type PreparedAsset = GpuBoardMaterial;
    type Param = (SRes<RenderDevice>, SRes<InstancedMaterialPipeline<Self>>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuBoardMaterial {
            bind_group,
            alpha_mode: extracted_asset.alpha_mode,
            cull_mode: extracted_asset.cull_mode,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct BoardMaterialKey {
    pub cull_mode: Option<Face>,
}

impl PartialOrd for BoardMaterialKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cull_mode
            .map(|cull_mode| cull_mode as usize)
            .partial_cmp(&other.cull_mode.map(|cull_mode| cull_mode as usize))
    }
}

impl Ord for BoardMaterialKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cull_mode
            .map(|cull_mode| cull_mode as usize)
            .cmp(&other.cull_mode.map(|cull_mode| cull_mode as usize))
    }
}

impl SpecializedInstancedMaterial for BoardMaterial {
    type PipelineKey = BoardMaterialKey;
    type BatchKey = BoardMaterialKey;
    type Instance = ColorMeshInstance;

    fn pipeline_key(
        render_asset: &<BoardMaterial as RenderAsset>::PreparedAsset,
    ) -> Self::BatchKey {
        BoardMaterialKey {
            cull_mode: render_asset.cull_mode,
        }
    }

    fn batch_key(render_asset: &<BoardMaterial as RenderAsset>::PreparedAsset) -> Self::BatchKey {
        BoardMaterialKey {
            cull_mode: render_asset.cull_mode,
        }
    }

    fn vertex_shader(_: &AssetServer) -> Option<Handle<Shader>> {
        Some(BOARD_SHADER_HANDLE.typed::<Shader>())
    }

    fn fragment_shader(_: &AssetServer) -> Option<Handle<Shader>> {
        Some(BOARD_SHADER_HANDLE.typed::<Shader>())
    }

    fn specialize(
        _pipeline: &InstancedMaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        key: Self::BatchKey,
        _layout: &MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.cull_mode;
        if let Some(label) = &mut descriptor.label {
            *label = format!("custom_{}", *label).into();
        }
        Ok(())
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[],
            label: Some("material layout"),
        })
    }

    fn alpha_mode(material: &<Self as RenderAsset>::PreparedAsset) -> AlphaMode {
        material.alpha_mode
    }
}
