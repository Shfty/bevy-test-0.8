use bevy::{
    asset::AssetPath,
    hierarchy::BuildWorldChildren,
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        ElementState, Input,
    },
    prelude::{
        default, Assets, EventReader, MouseButton, PerspectiveCameraBundle, PerspectiveProjection,
        Query, Res, ResMut, Scene, Transform, Vec3, Without, World,
    },
    window::Windows,
};

use transform_ex::{euler_rotation::EulerRotation, projected_translation::ProjectedTranslation};

use crate::FlycamBundle;

use super::Flycam;

pub fn scene_flycam(mut scenes: ResMut<Assets<Scene>>) {
    let mut world = World::new();

    // camera
    let camera = world.spawn().id();
    let zoom_target = world.spawn().id();

    world
        .entity_mut(camera)
        .insert_bundle(FlycamBundle {
            euler_rotation: default(),
            flycam: Flycam {
                zoom_target_entity: Some(zoom_target),
                ..default()
            },
            perspective_camera: PerspectiveCameraBundle {
                perspective_projection: PerspectiveProjection {
                    fov: 105.0_f32.to_radians(),
                    ..default()
                },
                ..default()
            },
        })
        .push_children(&[zoom_target]);

    world
        .entity_mut(zoom_target)
        .insert(Transform::identity())
        .insert(ProjectedTranslation {
            projection_entity: camera,
            depth: 1.0,
        });

    let scene = Scene::new(world);
    let path = AssetPath::new("flycam".into(), None);
    scenes.set_untracked(path, scene);
}

pub fn flycam_mouse_capture(
    mut windows: ResMut<Windows>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
) -> Result<(), &'static str> {
    let window = windows
        .get_primary_mut()
        .ok_or("Failed to get primary window")?;

    for event in mouse_button_events.iter() {
        match (event.button, event.state) {
            (MouseButton::Right | MouseButton::Middle, state) => {
                window.set_cursor_lock_mode(state == ElementState::Pressed);
            }
            _ => (),
        }
    }

    Ok(())
}

pub fn flycam_translate(
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query_flycam: Query<(&Flycam, &mut Transform)>,
    query_transform: Query<&Transform, Without<Flycam>>,
) -> Result<(), &'static str> {
    for (flycam, mut transform) in query_flycam.iter_mut() {
        let local_x = transform.local_x();
        let local_y = transform.local_y();
        let local_z = if let Some(target_entity) = flycam.zoom_target_entity {
            let target_pos = -query_transform
                .get(target_entity)
                .or(Err("Failed to get zoom target entity"))?
                .translation;

            if target_pos != Vec3::ZERO {
                target_pos.normalize()
            } else {
                transform.local_z()
            }
        } else {
            transform.local_z()
        };

        if mouse_button_input.pressed(MouseButton::Middle) {
            for event in mouse_motion_events.iter() {
                transform.translation += local_x * event.delta.x * flycam.translation_sensitivity.x;
                transform.translation += local_y * event.delta.y * flycam.translation_sensitivity.y;
            }
        }

        for event in mouse_wheel_events.iter() {
            transform.translation += local_z * event.y * flycam.translation_sensitivity.z;
        }
    }

    Ok(())
}

pub fn flycam_rotate(
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut transforms: Query<(&Flycam, &mut EulerRotation)>,
) {
    if mouse_button_input.pressed(MouseButton::Right) {
        for (flycam, mut euler_rotation) in transforms.iter_mut() {
            for event in mouse_motion_events.iter() {
                let sensitivity = flycam.rotation_sensitivity;
                let delta = event.delta * sensitivity;
                euler_rotation.rotation.x += delta.x;
                euler_rotation.rotation.y += delta.y;
            }
        }
    }
}
