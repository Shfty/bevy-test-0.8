use bevy::{
    prelude::{AlphaMode, Color, Material, MaterialPlugin, Plugin},
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct DepthMaterialPlugin;

impl Plugin for DepthMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(MaterialPlugin::<DepthMaterial>::default());
    }
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for DepthMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/depth.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "b3e2ed03-f7f8-41f5-8621-392c92183599"]
pub struct DepthMaterial {}

impl From<Color> for DepthMaterial {
    fn from(_: Color) -> Self {
        DepthMaterial {}
    }
}

