use bevy::{
    ecs::system::Command,
    prelude::{
        default, BuildWorldChildren, Bundle, Camera, Camera3dBundle, Component, Entity, Name,
        PerspectiveProjection, Plugin, Query, Res, Transform, With,
    },
    render::{camera::Projection, view::RenderLayers},
    transform::TransformBundle,
};

use crate::{
    hierarchy::HierarchyBundle,
    prelude::{
        camera::{
            CameraPivotSource, CameraPivotTarget, CameraViewportTarget,
            CustomPerspectiveProjection, PerspectiveCameraArmTarget, PlaneTransformTarget,
        },
        playfield::Playfield,
        AspectRatio,
    },
};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(background_projection);
    }
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct BackgroundCamera;

#[derive(Bundle)]
pub struct BackgroundBundle {
    pub name: Name,
    #[bundle]
    pub hierarchy: HierarchyBundle,
}

impl Default for BackgroundBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Background"),
            hierarchy: HierarchyBundle { ..default() },
        }
    }
}

pub struct AssembleBackground {
    pub entity: Entity,
}

impl Command for AssembleBackground {
    fn write(self, world: &mut bevy::prelude::World) {
        let mut camera_3d = world.spawn();

        camera_3d
            .insert(Name::new("Camera"))
            .insert(BackgroundCamera)
            .insert(PerspectiveCameraArmTarget)
            .insert(CameraViewportTarget)
            .insert_bundle(Camera3dBundle {
                camera: Camera {
                    priority: 0,
                    ..default()
                },
                ..default()
            })
            .insert(RenderLayers::layer(1))
            .insert(CustomPerspectiveProjection {
                perspective: PerspectiveProjection {
                    fov: 60.0_f32.to_radians(),
                    ..default()
                },
                ..default()
            })
            .remove::<Projection>();

        let camera_3d = camera_3d.id();

        let camera_transform = world
            .spawn()
            .insert(Name::new("Camera Transform"))
            .insert(CameraPivotTarget)
            .insert_bundle(TransformBundle::default())
            .push_children(&[camera_3d])
            .id();

        let plane_transform = world
            .spawn()
            .insert(Name::new("Plane Transform"))
            .insert(PlaneTransformTarget)
            .insert_bundle(TransformBundle::default())
            .push_children(&[camera_transform])
            .id();

        world
            .entity_mut(self.entity)
            .insert_bundle(BackgroundBundle::default())
            .push_children(&[plane_transform]);
    }
}

pub fn background_projection(
    aspect: Res<AspectRatio>,
    query_playfield: Query<&Playfield>,
    query_camera_pivot: Query<&Transform, With<CameraPivotSource>>,
    mut query_camera_3d: Query<&mut CustomPerspectiveProjection, With<BackgroundCamera>>,
) {
    let playfield = if let Some(playfield) = query_playfield.iter().next() {
        playfield
    } else {
        return;
    };

    let camera_pivot = query_camera_pivot.iter().next().unwrap();
    let playfield_size = playfield.size(**aspect, camera_pivot.translation.z);

    let mut camera_3d = query_camera_3d.iter_mut().next().unwrap();
    camera_3d.target_aspect = playfield_size.x / playfield_size.y;
}
