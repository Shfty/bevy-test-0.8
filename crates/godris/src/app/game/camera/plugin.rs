use bevy::{prelude::{CoreStage, ParallelSystemDescriptorCoercion, Plugin, SystemSet}, render::camera::camera_system};

use crate::prelude::{
    camera_focus::camera_focus, camera_zoom::camera_zoom, input_float_end,
    orbit_camera::orbit_camera, LerpCameraProjectionPlugin, LerpCameraProjection,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(LerpCameraProjectionPlugin)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::default()
                    .with_system(orbit_camera.after(input_float_end))
                    .with_system(camera_zoom.after(input_float_end))
                    .with_system(camera_focus),
            )
            .add_system_to_stage(CoreStage::PostUpdate, camera_focus.before(camera_system::<LerpCameraProjection>));
    }
}
