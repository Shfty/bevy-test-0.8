use bevy::{
    core::Time,
    ecs::system::Command,
    math::{Mat4, Vec2},
    prelude::{
        default, Bundle, Camera, Component, CoreStage, Entity, GlobalTransform,
        OrthographicProjection, ParallelSystemDescriptorCoercion, PerspectiveProjection, Plugin,
        Query, Res, SystemLabel, Transform,
    },
    render::{
        camera::{camera_system, Camera3d, CameraProjection},
        primitives::Frustum,
        view::{check_visibility, update_frusta, VisibleEntities},
    },
    transform::TransformSystem,
};
use ecs_ex::entity_default;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct LerpCameraProjection<LHS = PerspectiveProjection, RHS = OrthographicProjection> {
    pub lhs: LHS,
    pub rhs: RHS,
    pub factor: f32,
    pub target_factor: f32,
    pub speed_in: f32,
    pub speed_out: f32,
}

impl<LHS, RHS> CameraProjection for LerpCameraProjection<LHS, RHS>
where
    LHS: CameraProjection,
    RHS: CameraProjection,
{
    fn get_projection_matrix(&self) -> bevy::math::Mat4 {
        let lhs = self.lhs.get_projection_matrix();
        let rhs = self.rhs.get_projection_matrix();
        Mat4::from_cols(
            lhs.x_axis.lerp(rhs.x_axis, self.factor),
            lhs.y_axis.lerp(rhs.y_axis, self.factor),
            lhs.z_axis.lerp(rhs.z_axis, self.factor),
            lhs.w_axis.lerp(rhs.w_axis, self.factor),
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        self.lhs.update(width, height);
        self.rhs.update(width, height);
    }

    fn depth_calculation(&self) -> bevy::render::camera::DepthCalculation {
        let lhs = self.lhs.depth_calculation();
        let _rhs = self.lhs.depth_calculation();

        if !matches!(lhs, _rhs) {
            panic!("Incompatible depth calculations for LerpCameraProjection");
        }

        lhs
    }

    fn far(&self) -> f32 {
        Vec2::new(self.lhs.far(), 0.0)
            .lerp(Vec2::new(self.rhs.far(), 0.0), self.factor)
            .x
    }
}

#[derive(Bundle)]
pub struct LerpCameraBundle<LHS = PerspectiveProjection, RHS = OrthographicProjection, M = Camera3d>
where
    LHS: Component + CameraProjection,
    RHS: Component + CameraProjection,
    M: Component,
{
    pub camera: Camera,
    pub projection: LerpCameraProjection<LHS, RHS>,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub marker: M,
}

impl<LHS, RHS, M> LerpCameraBundle<LHS, RHS, M>
where
    LHS: Component + CameraProjection + Default,
    RHS: Component + CameraProjection + Default,
    M: Component + Default,
{
    pub fn new() -> Self {
        let transform = Transform::default();
        let projection = LerpCameraProjection::default();

        let view_projection =
            projection.get_projection_matrix() * transform.compute_matrix().inverse();

        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            projection.far(),
        );

        LerpCameraBundle {
            camera: default(),
            projection,
            visible_entities: default(),
            frustum,
            transform,
            global_transform: transform.into(),
            marker: default(),
        }
    }
}

impl<LHS, RHS, M> Default for LerpCameraBundle<LHS, RHS, M>
where
    LHS: Component + CameraProjection + Default,
    RHS: Component + CameraProjection + Default,
    M: Component + Default,
{
    fn default() -> Self {
        LerpCameraBundle::new()
    }
}

pub fn lerp_camera_projection<LHS, RHS>(
    time: Res<Time>,
    mut query: Query<&mut LerpCameraProjection<LHS, RHS>>,
) where
    LHS: Component,
    RHS: Component,
{
    for mut projection in query.iter_mut() {
        use lerp::Lerp;
        projection.factor = projection.factor.lerp(
            projection.target_factor,
            if projection.factor < projection.target_factor {
                projection.speed_in
            } else {
                projection.speed_out
            } * time.delta_seconds(),
        );
    }
}

pub struct LerpCameraProjectionPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum LerpCameraProjectionSystems {
    UpdateLerpFrusta,
}

impl Plugin for LerpCameraProjectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use LerpCameraProjectionSystems::*;

        app.add_system(lerp_camera_projection::<PerspectiveProjection, OrthographicProjection>)
            .add_system_to_stage(CoreStage::PostUpdate, camera_system::<LerpCameraProjection>)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update_frusta::<LerpCameraProjection>
                    .label(UpdateLerpFrusta)
                    .after(TransformSystem::TransformPropagate)
                    .before(check_visibility),
            );
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ToggleProjection {
    pub camera: Entity,
}

impl Default for ToggleProjection {
    fn default() -> Self {
        ToggleProjection {
            camera: entity_default(),
        }
    }
}

impl Command for ToggleProjection {
    fn write(self, world: &mut bevy::prelude::World) {
        let mut projection = world.entity_mut(self.camera);
        let mut projection = projection.get_mut::<LerpCameraProjection>().unwrap();
        projection.target_factor = 1.0 - projection.target_factor;
    }
}
