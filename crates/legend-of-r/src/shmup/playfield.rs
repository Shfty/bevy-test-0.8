use std::marker::PhantomData;

use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, core_3d::Camera3dDepthLoadOp},
    ecs::system::Command,
    prelude::{
        default, BuildWorldChildren, Bundle, Camera, Camera2d, Camera2dBundle, Camera3d,
        Camera3dBundle, Component, Entity, Name, OrthographicProjection, Plugin, Quat, Query,
        ReflectComponent, ReflectDefault, ReflectDeserialize, ReflectSerialize, Res, Transform,
        Vec2, Vec3, With, Without,
    },
    reflect::Reflect,
    render::{
        camera::{Projection, ScalingMode},
        view::RenderLayers,
    },
    transform::TransformBundle,
};
use bevy_rapier2d::prelude::{Collider, CollisionGroups, Vect};
use serde::{Serialize, Deserialize};

use crate::{
    hierarchy::HierarchyBundle,
    prelude::{
        AspectRatio, CameraPivotSource, CameraPivotTarget, CameraViewportTarget, CollisionGroup,
        OrthographicCameraTarget,
    },
    util::default_entity,
};

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<PlayfieldRotationTarget>();

        app.add_system(playfield_projection)
            .add_system(playfield_boundary)
            .add_system(playfield_rotation);
    }
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Playfield;

impl Playfield {
    pub fn size(&self, aspect: f32, camera_z: f32) -> Vec2 {
        let mut size = Vec2::splat(camera_z) * 2.0;

        match aspect.partial_cmp(&1.0).unwrap() {
            std::cmp::Ordering::Less => {
                size.x *= aspect;
            }
            std::cmp::Ordering::Greater => {
                size.y /= aspect;
            }
            _ => (),
        }

        size
    }

    pub fn half_size(&self, aspect: f32, camera_z: f32) -> Vec2 {
        self.size(aspect, camera_z) * 0.5
    }

    pub fn quarter_size(&self, aspect: f32, camera_z: f32) -> Vec2 {
        self.size(aspect, camera_z) * 0.25
    }
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct PlayfieldCamera2d;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct PlayfieldCamera3d;

#[derive(Bundle)]
pub struct PlayfieldBundle {
    pub name: Name,
    pub playfield: Playfield,
    #[bundle]
    pub hierarchy: HierarchyBundle,
}

impl Default for PlayfieldBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Playfield"),
            playfield: default(),
            hierarchy: HierarchyBundle {
                ..default()
            },
        }
    }
}

pub struct AssemblePlayfield {
    pub entity: Entity,
    pub playfield_bundle: PlayfieldBundle,
}

impl Default for AssemblePlayfield {
    fn default() -> Self {
        Self {
            entity: default_entity(),
            playfield_bundle: default(),
        }
    }
}

impl Command for AssemblePlayfield {
    fn write(self, world: &mut bevy::prelude::World) {
        let camera_ortho_3d = world
            .spawn()
            .insert(Name::new("Playfield Camera 3D"))
            .insert(PlayfieldCamera3d)
            .insert(OrthographicCameraTarget)
            .insert(CameraViewportTarget)
            .insert_bundle(Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::None,
                    depth_load_op: Camera3dDepthLoadOp::Load,
                },
                camera: Camera {
                    priority: 1,
                    ..default()
                },
                projection: Projection::Orthographic(default()),
                ..default()
            })
            .insert(OrthographicProjection::default())
            .insert(RenderLayers::layer(0))
            .id();

        let camera_ortho_2d = world
            .spawn()
            .insert(Name::new("Playfield Camera 2D"))
            .insert(PlayfieldCamera2d)
            .insert(OrthographicCameraTarget)
            .insert(CameraViewportTarget)
            .insert_bundle(Camera2dBundle {
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None,
                },
                camera: Camera {
                    priority: 2,
                    ..default()
                },
                ..default()
            })
            .insert(Transform::default())
            .insert(RenderLayers::layer(0))
            .id();

        let camera_rig = world
            .spawn()
            .insert(Name::new("Camera Rig"))
            .insert(CameraPivotTarget)
            .insert_bundle(TransformBundle::default())
            .push_children(&[camera_ortho_2d, camera_ortho_3d])
            .id();

        world
            .entity_mut(self.entity)
            .insert_bundle(self.playfield_bundle)
            .push_children(&[camera_rig]);
    }
}

#[derive(Component)]
pub struct Boundary<T>(PhantomData<T>);

impl<T> Default for Boundary<T> {
    fn default() -> Self {
        Self(default())
    }
}

pub struct Left;
pub struct Right;
pub struct Top;
pub struct Bottom;

pub struct SpawnBoundary {
    pub playfield: Entity,
}

impl Command for SpawnBoundary {
    fn write(self, world: &mut bevy::prelude::World) {
        let floor = world
            .spawn()
            .insert(Name::new("Floor"))
            .insert(Boundary::<Bottom>::default())
            .insert_bundle(TransformBundle::default())
            .insert(Collider::halfspace(Vect::Y).unwrap())
            .insert(CollisionGroups {
                memberships: CollisionGroup::STATIC.bits(),
                filters: CollisionGroup::all().bits(),
            })
            .id();

        let ceiling = world
            .spawn()
            .insert(Name::new("Ceiling"))
            .insert(Boundary::<Top>::default())
            .insert_bundle(TransformBundle::default())
            .insert(Collider::halfspace(Vect::NEG_Y).unwrap())
            .insert(CollisionGroups {
                memberships: CollisionGroup::STATIC.bits(),
                filters: CollisionGroup::all().bits(),
            })
            .id();

        let wall_left = world
            .spawn()
            .insert(Name::new("Left Wall"))
            .insert(Boundary::<Left>::default())
            .insert_bundle(TransformBundle::default())
            .insert(Collider::halfspace(Vect::X).unwrap())
            .insert(CollisionGroups {
                memberships: CollisionGroup::STATIC.bits(),
                filters: CollisionGroup::all().bits(),
            })
            .id();

        let wall_right = world
            .spawn()
            .insert(Name::new("Right Wall"))
            .insert(Boundary::<Right>::default())
            .insert_bundle(TransformBundle::default())
            .insert(Collider::halfspace(Vect::NEG_X).unwrap())
            .insert(CollisionGroups {
                memberships: CollisionGroup::STATIC.bits(),
                filters: CollisionGroup::all().bits(),
            })
            .id();

        world
            .entity_mut(self.playfield)
            .push_children(&[floor, ceiling, wall_left, wall_right]);
    }
}

pub fn playfield_projection(
    aspect: Res<AspectRatio>,
    query_playfield: Query<&Playfield>,
    query_camera_pivot: Query<&Transform, With<CameraPivotSource>>,
    mut query_camera_2d: Query<
        &mut OrthographicProjection,
        (With<PlayfieldCamera2d>, Without<PlayfieldCamera3d>),
    >,
    mut query_camera_3d: Query<
        &mut OrthographicProjection,
        (With<PlayfieldCamera3d>, Without<PlayfieldCamera2d>),
    >,
) {
    let playfield = if let Some(playfield) = query_playfield.iter().next() {
        playfield
    } else {
        return;
    };

    let camera_pivot = query_camera_pivot.iter().next().unwrap();

    let playfield_size = playfield.size(**aspect, camera_pivot.translation.z);
    let scaling_mode = ScalingMode::Auto {
        min_width: playfield_size.x,
        min_height: playfield_size.y,
    };

    let mut camera_2d = query_camera_2d.iter_mut().next().unwrap();
    camera_2d.scaling_mode = scaling_mode.clone();

    let mut camera_3d = query_camera_3d.iter_mut().next().unwrap();
    camera_3d.scaling_mode = scaling_mode;
}

pub fn playfield_boundary(
    aspect: Res<AspectRatio>,
    query_playfield: Query<&Playfield>,
    query_camera_pivot: Query<Entity, With<CameraPivotSource>>,
    query_left: Query<Entity, With<Boundary<Left>>>,
    query_right: Query<Entity, With<Boundary<Right>>>,
    query_top: Query<Entity, With<Boundary<Top>>>,
    query_bottom: Query<Entity, With<Boundary<Bottom>>>,
    mut query_transform: Query<&mut Transform>,
) {
    let playfield = if let Some(playfield) = query_playfield.iter().next() {
        playfield
    } else {
        return;
    };

    let camera_pivot = query_camera_pivot.iter().next().unwrap();
    let camera_pivot = query_transform.get(camera_pivot).unwrap();
    let half_size = playfield.half_size(**aspect, camera_pivot.translation.z);

    let boundary_left = query_left.iter().next().unwrap();
    let mut boundary_left = query_transform.get_mut(boundary_left).unwrap();
    boundary_left.translation.x = -half_size.x;

    let boundary_right = query_right.iter().next().unwrap();
    let mut boundary_right = query_transform.get_mut(boundary_right).unwrap();
    boundary_right.translation.x = half_size.x;

    let boundary_top = query_top.iter().next().unwrap();
    let mut boundary_top = query_transform.get_mut(boundary_top).unwrap();
    boundary_top.translation.y = half_size.y;

    let boundary_bottom = query_bottom.iter().next().unwrap();
    let mut boundary_bottom = query_transform.get_mut(boundary_bottom).unwrap();
    boundary_bottom.translation.y = -half_size.y;
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct PlayfieldRotationSource;

#[derive(Debug, Default, Copy, Clone, Component, Reflect, Serialize, Deserialize)]
#[reflect(Default, Component, Serialize, Deserialize)]
pub struct PlayfieldRotationTarget;

#[derive(Bundle)]
pub struct PlayfieldRotationSourceBundle {
    name: Name,
    transform_source: PlayfieldRotationSource,
    #[bundle]
    transform: TransformBundle,
}

#[derive(Default, Bundle)]
pub struct PlayfieldRotationTargetBundle {
    #[bundle]
    pub hierarchy: HierarchyBundle,
    pub playfield_rotation_target: PlayfieldRotationTarget,
}

pub fn view_right() -> Quat {
    Quat::from_scaled_axis(Vec3::Y * -std::f32::consts::FRAC_PI_2)
}

pub fn view_left() -> Quat {
    Quat::from_scaled_axis(Vec3::Y * std::f32::consts::FRAC_PI_2)
}

pub fn view_top() -> Quat {
    Quat::from_scaled_axis(Vec3::X * std::f32::consts::FRAC_PI_2)
}

pub fn view_bottom() -> Quat {
    Quat::from_scaled_axis(Vec3::X * -std::f32::consts::FRAC_PI_2)
}

pub fn view_back() -> Quat {
    Quat::IDENTITY
}

pub fn view_front() -> Quat {
    Quat::from_scaled_axis(Vec3::Y * std::f32::consts::PI)
}

impl Default for PlayfieldRotationSourceBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Playfield Rotation"),
            transform_source: default(),
            transform: Transform::from_rotation(view_right()).into(),
        }
    }
}

pub fn playfield_rotation(
    query_rotation_source: Query<
        &Transform,
        (
            With<PlayfieldRotationSource>,
            Without<PlayfieldRotationTarget>,
        ),
    >,
    mut query_rotation_target: Query<
        &mut Transform,
        (
            With<PlayfieldRotationTarget>,
            Without<PlayfieldRotationSource>,
        ),
    >,
) {
    if let Some(source_transform) = query_rotation_source.iter().next() {
        for mut target_transform in query_rotation_target.iter_mut() {
            target_transform.rotation = source_transform.rotation;
        }
    }
}
