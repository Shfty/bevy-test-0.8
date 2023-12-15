use super::*;

use bevy::{
    core_pipeline::{AlphaMask3d, Opaque3d, Transparent3d},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec3,
    pbr::AlphaMode,
    prelude::{
        default,
        shape::{Cube, Icosphere},
        Assets, Camera, Commands, EventWriter, Mesh, PerspectiveCameraBundle, Plugin, ResMut,
        World,
    },
    render::{
        camera::{ActiveCamera, CameraTypePlugin, RenderTarget},
        render_graph::{NodeRunError, RenderGraph, RenderGraphContext, SlotValue},
        render_phase::RenderPhase,
        render_resource::Face,
        renderer::RenderContext,
        RenderApp, RenderStage,
    },
    window::{CreateWindow, PresentMode, WindowDescriptor, WindowId},
};
use ecs_ex::WithName;

use crate::prelude::BoardMaterial;

use bevy_instancing::prelude::{BasicMaterial, ColorInstanceBundle, MeshInstanceBundle};

// Test indirect rendering
#[test]
fn test_indirect_rendering() {
    let mut app = test_base();

    app.add_plugin(SecondWindowCameraPlugin);

    app.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin);

    app.add_startup_system(setup_indirect_rendering);

    app.add_system(remove_timer)
        .add_system(set_mesh_timer)
        .add_system(set_material_timer::<BoardMaterial>);

    app.run()
}

struct SecondWindowCameraPlugin;
impl Plugin for SecondWindowCameraPlugin {
    fn build(&self, app: &mut App) {
        // adds the `ActiveCamera<SecondWindowCamera3d>` resource and extracts the camera into the render world
        app.add_plugin(CameraTypePlugin::<SecondWindowCamera3d>::default());

        let render_app = app.sub_app_mut(RenderApp);

        // add `RenderPhase<Opaque3d>`, `RenderPhase<AlphaMask3d>` and `RenderPhase<Transparent3d>` camera phases
        render_app.add_system_to_stage(RenderStage::Extract, extract_second_camera_phases);

        // add a render graph node that executes the 3d subgraph
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        let second_window_node = render_graph.add_node("second_window_cam", SecondWindowDriverNode);
        render_graph
            .add_node_edge(
                bevy::core_pipeline::node::MAIN_PASS_DEPENDENCIES,
                second_window_node,
            )
            .unwrap();
        render_graph
            .add_node_edge(
                bevy::core_pipeline::node::CLEAR_PASS_DRIVER,
                second_window_node,
            )
            .unwrap();
    }
}

struct SecondWindowDriverNode;
impl bevy::render::render_graph::Node for SecondWindowDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(camera) = world.resource::<ActiveCamera<SecondWindowCamera3d>>().get() {
            graph.run_sub_graph(
                bevy::core_pipeline::draw_3d_graph::NAME,
                vec![SlotValue::Entity(camera)],
            )?;
        }

        Ok(())
    }
}

fn extract_second_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<SecondWindowCamera3d>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Opaque3d>::default(),
            RenderPhase::<AlphaMask3d>::default(),
            RenderPhase::<Transparent3d>::default(),
        ));
    }
}

#[derive(Component, Default)]
struct SecondWindowCamera3d;

fn setup_indirect_rendering(
    mut create_window_events: EventWriter<CreateWindow>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut board_materials: ResMut<Assets<BoardMaterial>>,
    mut commands: Commands,
) {
    let window_id = WindowId::new();

    // sends out a "CreateWindow" event, which will be received by the windowing backend
    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            present_mode: PresentMode::Immediate,
            title: "Second window".to_string(),
            ..default()
        },
    });

    // second window camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        camera: Camera {
            target: RenderTarget::Window(window_id),
            ..default()
        },
        transform: Transform::from_xyz(50.0, 0.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        marker: SecondWindowCamera3d,
        ..PerspectiveCameraBundle::new()
    });

    // Populate scene
    let mesh_cube = meshes.add(Cube::default().into());
    let mesh_sphere = meshes.add(
        Icosphere {
            radius: 0.75,
            ..default()
        }
        .into(),
    );

    let meshes = [mesh_cube, mesh_sphere];

    let material_basic = Handle::<BasicMaterial>::default();

    let material_opaque_no_cull = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None,
    });

    let material_mask_no_cull = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
    });

    let material_blend_no_cull = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
    });

    let material_opaque_cull_front = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Opaque,
        cull_mode: Some(Face::Front),
    });

    let material_mask_cull_front = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: Some(Face::Front),
    });

    let material_blend_cull_front = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Blend,
        cull_mode: Some(Face::Front),
    });

    let material_opaque_cull_back = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Opaque,
        cull_mode: Some(Face::Back),
    });

    let material_mask_cull_back = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: Some(Face::Back),
    });

    let material_blend_cull_back = board_materials.add(BoardMaterial {
        alpha_mode: AlphaMode::Blend,
        cull_mode: Some(Face::Back),
    });

    let materials = [
        material_opaque_no_cull,
        material_mask_no_cull,
        material_blend_no_cull.clone(),
        material_opaque_cull_front,
        material_mask_cull_front,
        material_blend_cull_front.clone(),
        material_opaque_cull_back,
        material_mask_cull_back,
        material_blend_cull_back.clone(),
    ];

    let colors = [
        Color::WHITE,
        Color::RED,
        Color::GREEN,
        Color::BLUE,
        Color::BLACK,
    ];

    let color_count = colors.len();

    let mesh_count = meshes.len();
    for (x, mesh) in meshes.into_iter().enumerate() {
        commands
            .spawn()
            .insert(Name::new("Cube Instance"))
            .insert_bundle(MeshInstanceBundle::<BasicMaterial> {
                mesh: mesh.clone(),
                material: material_basic.clone(),
                transform: Transform::from_xyz(x as f32 * 1.5, 0.0, 0.0).into(),
                ..default()
            })
            .insert(RemoveTimer(Timer::from_seconds(1.0 + x as f32, false)));

        for (y, material) in materials.iter().enumerate() {
            for (z, mut color) in colors.into_iter().enumerate() {
                if *material == material_blend_no_cull
                    || *material == material_blend_cull_front
                    || *material == material_blend_cull_back
                {
                    color.set_a(0.5);
                }
                commands
                    .spawn()
                    .with_name(format!("Cube Instance ({x:}, {y:}, {z:})"))
                    .insert_bundle(ColorInstanceBundle {
                        instance_bundle: MeshInstanceBundle {
                            mesh: mesh.clone(),
                            material: material.clone(),
                            transform: Transform::from_xyz(
                                x as f32 * 1.5,
                                1.5 + y as f32 * 1.5,
                                z as f32 * -1.5,
                            )
                            .into(),
                            ..default()
                        },
                        mesh_instance_color: color.into(),
                    })
                    .insert(RemoveTimer(Timer::from_seconds(
                        1.0 + (x + ((1 + y) * mesh_count)) as f32 + (z as f32 / color_count as f32),
                        false,
                    )));
            }
        }
    }
}
