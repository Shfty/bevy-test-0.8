pub mod systems;

use systems::{
    cache_transform_interpolation, extract_transform_interpolation, prepare_transform_interpolation,
};

use bevy::{
    prelude::{
        Component, ExclusiveSystemDescriptorCoercion, GlobalTransform, IntoExclusiveSystem,
        ParallelSystemDescriptorCoercion, Plugin,
    },
    render::{RenderApp, RenderStage},
    transform::transform_propagate_system,
    utils::Instant,
};

use crate::fixed_timestep::RapierStage;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct TransformInterpolation {
    pub from: Option<GlobalTransform>,
    pub to: Option<GlobalTransform>,
    pub ts: Option<Instant>,
}

impl TransformInterpolation {
    pub fn lerp(&self, dt: f32) -> Option<GlobalTransform> {
        if let (Some(mut from), Some(to), Some(ts)) = (self.from, self.to, self.ts) {
            let t = Instant::now().duration_since(ts).as_secs_f32() / dt;
            from.translation = from.translation.lerp(to.translation, t);
            from.rotation = from.rotation.lerp(to.rotation, t);
            from.scale = from.scale.lerp(to.scale, t);
            Some(from)
        } else {
            None
        }
    }
}

pub struct TransformInterpolationPlugin;

impl Plugin for TransformInterpolationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(
            RapierStage::CacheInterpolationTransforms,
            cache_transform_interpolation.after(transform_propagate_system),
        );

        app.sub_app_mut(RenderApp)
            .add_system_to_stage(RenderStage::Extract, extract_transform_interpolation)
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_transform_interpolation
                    .exclusive_system()
                    .at_start(),
            );
    }
}
