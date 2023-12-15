use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

use bevy::{
    core::Time,
    ecs::system::Command,
    math::{Quat, Vec3},
    prelude::{default, Component, Entity, Query, Transform},
};
use ecs_ex::entity_default;

use crate::prelude::board_max;

#[derive(Debug, Copy, Clone, Component)]
pub struct OrbitCamera {
    pub input_yaw: Entity,
    pub input_pitch: Entity,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            input_yaw: entity_default(),
            input_pitch: entity_default(),
            yaw: default(),
            pitch: default(),
        }
    }
}

pub fn orbit_camera(mut query: Query<(&OrbitCamera, &mut Transform)>) {
    for (orbit_camera, mut transform) in query.iter_mut() {
        let yaw = Quat::from_axis_angle(Vec3::Y, orbit_camera.yaw);
        let pitch = Quat::from_axis_angle(yaw * Vec3::X, orbit_camera.pitch);
        transform.rotation = pitch * yaw;
        transform.translation = transform.local_z() * board_max().max_element() * 2.0;
    }
}

pub struct PitchCamera {
    pub camera: Entity,
    pub delta: f32,
}

impl Command for PitchCamera {
    fn write(self, world: &mut bevy::prelude::World) {
        let time = world.resource::<Time>();
        let dt = time.delta_seconds();

        let mut orbit_camera = world.entity_mut(self.camera);
        let mut orbit_camera = orbit_camera.get_mut::<OrbitCamera>().unwrap();

        orbit_camera.pitch += self.delta * dt;
        orbit_camera.pitch = orbit_camera.pitch.max(-FRAC_PI_2).min(FRAC_PI_4);
    }
}

pub struct YawCamera {
    pub camera: Entity,
    pub delta: f32,
}

impl Command for YawCamera {
    fn write(self, world: &mut bevy::prelude::World) {
        let time = world.resource::<Time>();
        let dt = time.delta_seconds();

        let mut orbit_camera = world.entity_mut(self.camera);
        let mut orbit_camera = orbit_camera.get_mut::<OrbitCamera>().unwrap();

        orbit_camera.yaw += self.delta * dt;
        orbit_camera.yaw = orbit_camera.yaw % TAU;
    }
}
