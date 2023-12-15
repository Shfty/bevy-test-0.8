use bevy::{
    core::Time,
    ecs::system::Command,
    prelude::{default, Component, Entity, Query},
};
use ecs_ex::entity_default;

use crate::prelude::LerpCameraProjection;

#[derive(Debug, Copy, Clone, Component)]
pub struct CameraZoom {
    pub zoom_out: Entity,
    pub zoom_in: Entity,
    pub fov: f32,
    pub min: f32,
    pub max: f32,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self {
            zoom_out: entity_default(),
            zoom_in: entity_default(),
            fov: default(),
            min: default(),
            max: default(),
        }
    }
}

pub fn camera_zoom(mut query: Query<(&mut CameraZoom, &mut LerpCameraProjection)>) {
    for (camera_zoom, mut projection) in query.iter_mut() {
        projection.lhs.fov = camera_zoom.fov;
        projection.rhs.scale = camera_zoom.fov * 0.05;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AdjustZoom {
    pub camera: Entity,
    pub delta: f32,
}

impl Default for AdjustZoom {
    fn default() -> Self {
        Self {
            camera: entity_default(),
            delta: default(),
        }
    }
}

impl Command for AdjustZoom {
    fn write(self, world: &mut bevy::prelude::World) {
        let time = world.resource::<Time>();
        let dt = time.delta_seconds();

        let mut zoom = world.entity_mut(self.camera);
        let mut zoom = zoom.get_mut::<CameraZoom>().unwrap();

        zoom.fov += self.delta * dt;
        zoom.fov = zoom.fov.clamp(zoom.min, zoom.max);
    }
}
