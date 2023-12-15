use bevy::{
    core::FixedTimestep,
    prelude::{
        App, CoreStage, Deref, DerefMut, ParallelSystemDescriptorCoercion, Plugin, ResMut,
        StageLabel, StartupStage, SystemStage,
    },
    reflect::Reflect,
    transform::transform_propagate_system,
};
use bevy_rapier3d::plugin::{
    NoUserData, PhysicsStages, RapierConfiguration, RapierPhysicsPlugin, TimestepMode,
};

pub const SETUP_FIXED_TIMESTEP: &'static str = "setup_fixed_timestep";

#[derive(Debug, Copy, Clone, Deref, DerefMut, Reflect)]
pub struct FixedDeltaTime {
    dt: f32,
}

impl Default for FixedDeltaTime {
    fn default() -> Self {
        Self { dt: 1.0 / 60.0 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StageLabel)]
pub enum RapierStage {
    PrePhysics,
    SyncBackend,
    StepSimulation,
    Writeback,
    PostPhysics,
    CacheInterpolationTransforms,
    DetectDespawn,
}

pub struct RapierFixedTimestepPlugin {
    pub dt: f64,
}

impl Plugin for RapierFixedTimestepPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FixedDeltaTime { dt: self.dt as f32 });

        app.add_stage_before(
            CoreStage::Update,
            RapierStage::PrePhysics,
            SystemStage::parallel().with_run_criteria(FixedTimestep::step(self.dt)),
        )
        .add_stage_after(
            RapierStage::PrePhysics,
            RapierStage::SyncBackend,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(self.dt))
                .with_system_set(RapierPhysicsPlugin::<NoUserData>::get_systems(
                    PhysicsStages::SyncBackend,
                )),
        )
        .add_stage_after(
            RapierStage::SyncBackend,
            RapierStage::StepSimulation,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(self.dt))
                .with_system_set(RapierPhysicsPlugin::<NoUserData>::get_systems(
                    PhysicsStages::StepSimulation,
                )),
        )
        .add_stage_after(
            RapierStage::StepSimulation,
            RapierStage::Writeback,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(self.dt))
                .with_system_set(RapierPhysicsPlugin::<NoUserData>::get_systems(
                    PhysicsStages::Writeback,
                )),
        )
        .add_stage_after(
            RapierStage::Writeback,
            RapierStage::PostPhysics,
            SystemStage::parallel().with_run_criteria(FixedTimestep::step(self.dt)),
        )
        .add_stage_after(
            RapierStage::PostPhysics,
            RapierStage::CacheInterpolationTransforms,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(self.dt))
                .with_system(transform_propagate_system),
        )
        .add_stage_before(
            CoreStage::Last,
            RapierStage::DetectDespawn,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(self.dt))
                .with_system_set(RapierPhysicsPlugin::<NoUserData>::get_systems(
                    PhysicsStages::DetectDespawn,
                )),
        )
        .add_startup_system_to_stage(
            StartupStage::PreStartup,
            setup_fixed_timestep(self.dt as f32, 1).label(SETUP_FIXED_TIMESTEP),
        );

        app.add_plugin(
            RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false),
        );
    }
}

/// Configure rapier to use a fixed timestep with the given dt
pub fn setup_fixed_timestep(dt: f32, substeps: usize) -> impl FnMut(ResMut<RapierConfiguration>) {
    move |mut rapier_config: ResMut<RapierConfiguration>| {
        rapier_config.timestep_mode = TimestepMode::Fixed { dt, substeps }
    }
}
