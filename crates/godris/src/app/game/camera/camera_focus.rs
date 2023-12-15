use bevy::{prelude::{Component, Entity, Query, Transform}, ecs::system::Command};
use ecs_ex::entity_default;

use crate::prelude::{board_size, BoardTransform};

#[derive(Debug, Copy, Clone, Component)]
pub struct CameraFocus {
    pub target: Entity,
    pub use_target: bool,
}

impl Default for CameraFocus {
    fn default() -> Self {
        Self {
            target: entity_default(),
            use_target: true,
        }
    }
}

pub fn camera_focus(
    query_board_transform: Query<&BoardTransform>,
    mut query_camera: Query<(&CameraFocus, &mut Transform)>,
) {
    let board_size = board_size();
    for (focus, mut transform) in query_camera.iter_mut() {
        if let Ok(board_transform) = query_board_transform.get(focus.target) {
            if focus.use_target {
                transform.translation.x = board_transform.translation.x as f32 - board_size.x / 2.0;
                transform.translation.z = board_transform.translation.z as f32 - board_size.z / 2.0;
                return;
            }
        }

        transform.translation.x = 0.0;
        transform.translation.z = 0.0;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ToggleFocus {
    pub camera: Entity,
}

impl Default for ToggleFocus {
    fn default() -> Self {
        ToggleFocus {
            camera: entity_default(),
        }
    }
}

impl Command for ToggleFocus {
    fn write(self, world: &mut bevy::prelude::World) {
        let mut focus = world.entity_mut(self.camera);
        let mut focus = focus.get_mut::<CameraFocus>().unwrap();
        focus.use_target = !focus.use_target;
    }
}
