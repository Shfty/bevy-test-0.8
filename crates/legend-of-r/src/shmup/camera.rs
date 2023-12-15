use bevy::{
    ecs::{reflect::ReflectComponent, system::Command},
    prelude::{
        default, BuildWorldChildren, Bundle, Camera, Component, CoreStage, Entity, Mat4, Name,
        OrthographicProjection, ParallelSystemDescriptorCoercion, PerspectiveProjection, Plugin,
        Query, Res, SystemLabel, Transform, Vec3, Vec4, With, Without,
    },
    reflect::Reflect,
    render::{
        camera::{CameraProjection, CameraProjectionPlugin, CameraUpdateSystem, Viewport},
        view::update_frusta,
    },
    transform::{TransformBundle, TransformSystem},
};

use crate::{animation::timeline::TimelineUiState, prelude::PlayfieldRotationSourceBundle};

pub struct CameraPlugin;

#[derive(SystemLabel)]
pub struct UpdateCustomPerspectiveFrusta;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(CameraProjectionPlugin::<CustomPerspectiveProjection>::default());

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_frusta::<CustomPerspectiveProjection>
                .label(UpdateCustomPerspectiveFrusta)
                .after(TransformSystem::TransformPropagate),
        );

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            camera_viewport.after(CameraUpdateSystem),
        );

        app.add_system(plane_transform)
            .add_system(camera_pivot)
            .add_system(perspective_camera_arm)
            .add_system(orthographic_camera_arm);
    }
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct PlaneTransformSource;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct PlaneTransformTarget;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct CameraPivotSource;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct CameraPivotTarget;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct PerspectiveCameraArmTarget;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct OrthographicCameraTarget;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct CameraViewportTarget;

#[derive(Bundle)]
pub struct CameraPivotBundle {
    name: Name,
    source: CameraPivotSource,
    #[bundle]
    transform: TransformBundle,
}

impl Default for CameraPivotBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Camera Pivot"),
            source: default(),
            transform: Transform::from_xyz(0.0, 0.0, 16.0).into(),
        }
    }
}

#[derive(Bundle)]
pub struct PlaneTransformBundle {
    name: Name,
    transform_source: PlaneTransformSource,
    #[bundle]
    transform: TransformBundle,
}

impl Default for PlaneTransformBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Plane Transform"),
            transform_source: default(),
            transform: default(),
        }
    }
}

pub struct AssembleCameraRig {
    pub entity: Entity,
}

impl Command for AssembleCameraRig {
    fn write(self, world: &mut bevy::prelude::World) {
        let camera_transform = world
            .spawn()
            .insert_bundle(CameraPivotBundle::default())
            .id();

        let plane_transform = world
            .spawn()
            .insert_bundle(PlaneTransformBundle::default())
            .id();

        let playfield_rotation = world
            .spawn()
            .insert_bundle(PlayfieldRotationSourceBundle::default())
            .id();

        world
            .entity_mut(self.entity)
            .insert(Name::new("Camera Rig"))
            .push_children(&[camera_transform, plane_transform, playfield_rotation]);
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct CustomPerspectiveProjection {
    pub perspective: PerspectiveProjection,
    pub target_aspect: f32,
}

impl Default for CustomPerspectiveProjection {
    fn default() -> Self {
        Self {
            perspective: Default::default(),
            target_aspect: 1.0,
        }
    }
}

impl CameraProjection for CustomPerspectiveProjection {
    fn get_projection_matrix(&self) -> bevy::prelude::Mat4 {
        let target_aspect_inv = 1.0 / self.target_aspect;

        let f = 1.0 / (0.5 * self.perspective.fov).tan();

        if self.perspective.aspect_ratio < self.target_aspect {
            Mat4::from_cols(
                Vec4::new(f * target_aspect_inv, 0.0, 0.0, 0.0),
                Vec4::new(
                    0.0,
                    f * self.perspective.aspect_ratio * target_aspect_inv,
                    0.0,
                    0.0,
                ),
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(0.0, 0.0, self.perspective.near, 0.0),
            )
        } else {
            Mat4::from_cols(
                Vec4::new(f / self.perspective.aspect_ratio, 0.0, 0.0, 0.0),
                Vec4::new(0.0, f, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(0.0, 0.0, self.perspective.near, 0.0),
            )
        }
    }

    fn update(&mut self, width: f32, height: f32) {
        self.perspective.update(width, height)
    }

    fn depth_calculation(&self) -> bevy::render::camera::DepthCalculation {
        self.perspective.depth_calculation()
    }

    fn far(&self) -> f32 {
        self.perspective.far()
    }
}

pub fn plane_transform(
    query_transform_source: Query<
        &Transform,
        (With<PlaneTransformSource>, Without<PlaneTransformTarget>),
    >,
    mut query_transform_target: Query<
        &mut Transform,
        (With<PlaneTransformTarget>, Without<PlaneTransformSource>),
    >,
) {
    let source_transform = if let Some(components) = query_transform_source.iter().next() {
        components
    } else {
        return;
    };

    for mut target_transform in query_transform_target.iter_mut() {
        *target_transform = Transform {
            translation: source_transform.translation,
            rotation: source_transform.rotation,
            scale: Vec3::ONE,
        }
    }
}

pub fn camera_pivot(
    query_transform_source: Query<
        &Transform,
        (With<CameraPivotSource>, Without<CameraPivotTarget>),
    >,
    mut query_transform_target: Query<
        &mut Transform,
        (With<CameraPivotTarget>, Without<CameraPivotSource>),
    >,
) {
    let source_transform = if let Some(components) = query_transform_source.iter().next() {
        components
    } else {
        return;
    };

    for mut target_transform in query_transform_target.iter_mut() {
        *target_transform = Transform {
            translation: Vec3::new(
                source_transform.translation.x,
                source_transform.translation.y,
                0.0,
            ),
            rotation: source_transform.rotation,
            scale: Vec3::ONE,
        }
    }
}

pub fn perspective_camera_arm(
    query_offset_source: Query<
        &Transform,
        (With<CameraPivotSource>, Without<PerspectiveCameraArmTarget>),
    >,
    mut query_offset_target: Query<
        &mut Transform,
        (With<PerspectiveCameraArmTarget>, Without<CameraPivotSource>),
    >,
) {
    let playfield = if let Some(playfield) = query_offset_source.iter().next() {
        playfield
    } else {
        return;
    };

    for mut target_transform in query_offset_target.iter_mut() {
        *target_transform = Transform::from_xyz(0.0, 0.0, playfield.translation.z);
    }
}

pub fn orthographic_camera_arm(
    query_offset_source: Query<
        &Transform,
        (With<CameraPivotSource>, Without<OrthographicCameraTarget>),
    >,
    mut query_offset_target: Query<
        (&OrthographicProjection, &mut Transform),
        (With<OrthographicCameraTarget>, Without<CameraPivotSource>),
    >,
) {
    let camera_arm = if let Some(playfield) = query_offset_source.iter().next() {
        playfield
    } else {
        return;
    };

    for (projection, mut target_transform) in query_offset_target.iter_mut() {
        *target_transform =
            Transform::from_xyz(0.0, 0.0, projection.far - 100.0 / camera_arm.translation.z);
    }
}

pub fn camera_viewport(
    timeline_ui_state: Res<TimelineUiState>,
    mut query: Query<&mut Camera, With<CameraViewportTarget>>,
) {
    for mut camera in query.iter_mut() {
        if let Some(mut size) = camera.physical_target_size() {
            size.y = (timeline_ui_state.rect.top() as u32).max(1);
            camera.viewport = Some(Viewport {
                physical_size: size,
                ..default()
            });
        } else {
            camera.viewport = None;
        }
    }
}
