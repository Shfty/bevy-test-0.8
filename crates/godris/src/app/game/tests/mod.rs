pub mod indirect_rendering;
pub mod remove_model;

use bevy::{
    core::{Name, Time, Timer},
    hierarchy::DespawnRecursiveExt,
    math::{IVec3, Quat, UVec3},
    pbr::{DirectionalLight, DirectionalLightBundle},
    prelude::{
        default, info, App, AssetServer, Assets, Color, Commands, Component, Deref, DerefMut,
        Entity, Handle, Mesh, Query, Res, ResMut, StartupStage, Transform,
    },
    DefaultPlugins,
};

use bevy_inspector_egui::WorldInspectorPlugin;

use crate::prelude::{spawn_camera, GamePlugin};
use bevy_instancing::prelude::SpecializedInstancedMaterial;

// Removal timer
#[derive(Debug, Default, Clone, Deref, DerefMut, Component)]
pub struct RemoveTimer(pub Timer);

pub fn remove_timer(
    time: Res<Time>,
    mut query: Query<(Entity, &mut RemoveTimer)>,
    mut commands: Commands,
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            info!("Remove entity {entity:?}");
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Set mesh timer
#[derive(Debug, Default, Clone, Component)]
pub struct SetMeshTimer {
    pub timer: Timer,
    pub mesh: Handle<Mesh>,
}

pub fn set_mesh_timer(
    time: Res<Time>,
    mut query: Query<(Entity, &mut SetMeshTimer)>,
    mut commands: Commands,
) {
    for (entity, mut set_mesh_timer) in query.iter_mut() {
        if set_mesh_timer.timer.tick(time.delta()).just_finished() {
            info!("Set entity {entity:?} mesh to {:?}", set_mesh_timer.mesh);
            commands.entity(entity).insert(set_mesh_timer.mesh.clone());
        }
    }
}

// Set alpha mode timer
#[derive(Debug, Default, Clone, Component)]
pub struct SetMaterialTimer<M: SpecializedInstancedMaterial> {
    pub timer: Timer,
    pub material: Handle<M>,
}

pub fn set_material_timer<M: SpecializedInstancedMaterial>(
    time: Res<Time>,
    mut query: Query<(Entity, &mut SetMaterialTimer<M>)>,
    mut commands: Commands,
) {
    for (entity, mut set_mesh_timer) in query.iter_mut() {
        if set_mesh_timer.timer.tick(time.delta()).just_finished() {
            info!(
                "Set entity {entity:?} material to {}",
                std::any::type_name::<M>()
            );
            commands
                .entity(entity)
                .insert(set_mesh_timer.material.clone());
        }
    }
}

// Test app setup boilerplate
fn test_base() -> App {
    let mut app = App::default();

    app.add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup_base);

    app
}

fn setup_base(mut commands: Commands) {
    // Camera
    let (_camera_orbit, _camera_strafe) = spawn_camera(default(), &mut commands);

    // Directional Light
    commands.spawn().insert_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.,
            ..default()
        },
        transform: Transform {
            // Workaround: Pointing straight up or down prevents directional shadow from rendering
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 * 0.6),
            ..default()
        },
        ..default()
    });
}
