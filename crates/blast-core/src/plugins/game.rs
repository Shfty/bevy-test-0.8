use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::{DebugRenderMode, RapierDebugRenderPlugin};
use forces::{
    integrators::{integrate_begin, prepare_accelerations},
    kinematic_force::apply_kinematic_impulses,
    ForcesPlugin,
};
use result_system::ResultSystem;

use anyhow::{Context, Result};
use bevy::{
    ecs::system::AsSystemLabel,
    input::gamepad::gamepad_event_system,
    prelude::{default, CoreStage, Msaa, ParallelSystemDescriptorCoercion, Plugin, ResMut},
    window::Windows,
};
use transform_ex::projected_translation::ProjectedTranslationPlugin;

use flycam::FlycamPlugin;

use rapier_ex::{fixed_timestep::RapierStage, RapierExPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Msaa { samples: 4 });

        app.add_plugin(DebugLinesPlugin::with_depth_test(false))
            .add_plugin(RapierExPlugin {
                fixed_dt: Some(1.0 / 5.0),
            })
            .add_plugin(ForcesPlugin {
                register_systems: false,
            })
            .add_plugin(RapierDebugRenderPlugin {
                depth_test: false,
                mode: DebugRenderMode::COLLIDER_SHAPES
                    | DebugRenderMode::SOLVER_CONTACTS
                    | DebugRenderMode::CONTACTS,
                ..default()
            })
            .add_plugin(ProjectedTranslationPlugin)
            .add_plugin(FlycamPlugin);
        //.add_plugin(MechPlugin);

        // Hook forces systems into fixed timestep
        app.add_system_set_to_stage(
            RapierStage::PrePhysics,
            ForcesPlugin::systems_prepare_integration(),
        );
        app.add_system_set_to_stage(
            RapierStage::PrePhysics,
            ForcesPlugin::systems_solve_forces()
                .after(prepare_accelerations.as_system_label())
                .before(integrate_begin.as_system_label()),
        );
        app.add_system_set_to_stage(
            RapierStage::PrePhysics,
            ForcesPlugin::systems_integrate().after(apply_kinematic_impulses.as_system_label()),
        );
        app.add_system_set_to_stage(
            RapierStage::PostPhysics,
            ForcesPlugin::systems_solve_constraints(),
        );

        // Game systems
        app.add_startup_system(setup_window_title.result_system())
            .add_startup_system(crate::scenes::constraints::setup)
            .add_startup_system(input_ex::configure_gamepads)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                input_ex::input_axis::input_axis_3.after(gamepad_event_system),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                crate::scenes::constraints::transform_input
                    .result_system()
                    .after(input_ex::input_axis::input_axis_3),
            );
        /*
            .add_startup_system(crate::scenes::look_at::setup)
            .add_system(crate::scenes::look_at::sine_mover)
            .add_system(crate::scenes::look_at::cosine_mover);
        */
    }
}

fn setup_window_title(mut windows: ResMut<Windows>) -> Result<()> {
    // Set window title
    let window = windows
        .get_primary_mut()
        .context("Failed to fetch primary window")?;
    window.set_title("Blast Core".into());
    Ok(())
}
