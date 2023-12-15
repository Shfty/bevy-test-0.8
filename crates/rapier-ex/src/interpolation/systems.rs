use crate::fixed_timestep::FixedDeltaTime;

use super::TransformInterpolation;
use bevy::{
    pbr::MeshUniform,
    prelude::{Commands, Entity, GlobalTransform, Query, Res},
    utils::Instant,
};

/// Store the latest physics transform for interpolation
pub fn cache_transform_interpolation(
    mut query_interp: Query<(&GlobalTransform, &mut TransformInterpolation)>,
) {
    for (transform, mut interp) in query_interp.iter_mut() {
        interp.from = interp.to.take();
        interp.to = Some(*transform);
        interp.ts = Some(Instant::now());
    }
}

/// Copy transform interpolation data to the render world
pub fn extract_transform_interpolation(
    dt: Res<FixedDeltaTime>,
    query: Query<(Entity, &TransformInterpolation)>,
    mut commands: Commands,
) {
    commands.insert_resource(*dt);
    for (entity, interp) in query.iter() {
        commands.insert_or_spawn_batch(vec![(entity, (*interp,))].into_iter());
    }
}

/// Interpolate mesh uniform transforms between the associated entity's last two physics steps
pub fn prepare_transform_interpolation(
    dt: Res<FixedDeltaTime>,
    mut query_interp: Query<(&TransformInterpolation, &mut MeshUniform)>,
) {
    for (interp, mut mesh_uniform) in query_interp.iter_mut() {
        if let Some(lerp) = interp.lerp(**dt) {
            mesh_uniform.transform = lerp.compute_matrix();
            mesh_uniform.inverse_transpose_model = mesh_uniform.transform.inverse().transpose()
        }
    }
}
