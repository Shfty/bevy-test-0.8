pub mod systems;

use bevy::{
    ecs::reflect::ReflectComponent,
    hierarchy::BuildChildren,
    math::{Vec2, Vec3},
    prelude::{
        default, Bundle, Commands, Component, Entity, PerspectiveCameraBundle,
        PerspectiveProjection, Plugin, Transform,
    },
    reflect::Reflect,
};

use result_system::ResultSystem;

use transform_ex::{
    euler_rotation::{EulerRotation, EulerRotationPlugin},
    projected_translation::ProjectedTranslation,
};

use systems::{flycam_mouse_capture, flycam_rotate, flycam_translate};

pub struct FlycamPlugin;

impl Plugin for FlycamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Flycam>()
            .add_plugin(EulerRotationPlugin)
            .add_system(flycam_mouse_capture.result_system())
            .add_system(flycam_rotate)
            .add_system(flycam_translate.result_system());
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Flycam {
    pub translation_sensitivity: Vec3,
    pub rotation_sensitivity: Vec2,
    pub zoom_target_entity: Option<Entity>,
}

impl Default for Flycam {
    fn default() -> Self {
        Self {
            translation_sensitivity: Vec3::new(-0.005, 0.005, -0.5),
            rotation_sensitivity: Vec2::new(-0.0025, -0.0025),
            zoom_target_entity: None,
        }
    }
}

#[derive(Bundle)]
pub struct FlycamBundle<M: Component> {
    pub flycam: Flycam,
    pub euler_rotation: EulerRotation,
    #[bundle]
    pub perspective_camera: PerspectiveCameraBundle<M>,
}

pub fn build_flycam(commands: &mut Commands, transform: Transform) {
    // camera
    let camera = commands.spawn().id();
    let zoom_target = commands.spawn().id();

    commands
        .entity(camera)
        .insert_bundle(FlycamBundle {
            euler_rotation: default(),
            flycam: Flycam {
                zoom_target_entity: Some(zoom_target),
                ..default()
            },
            perspective_camera: PerspectiveCameraBundle {
                transform,
                perspective_projection: PerspectiveProjection {
                    fov: 105.0_f32.to_radians(),
                    ..default()
                },
                ..default()
            },
        })
        .add_child(zoom_target);

    commands
        .entity(zoom_target)
        .insert(Transform::identity())
        .insert(ProjectedTranslation {
            projection_entity: camera,
            depth: 1.0,
        });
}
