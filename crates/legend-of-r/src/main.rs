pub mod animation;
pub mod body;
pub mod collider;
pub mod collision;
pub mod debug_dump;
pub mod egui_diagnostics;
pub mod games;
pub mod gltf_entity;
pub mod gltf_json;
pub mod hierarchy;
pub mod integration;
pub mod iterator;
pub mod scene;
pub mod mesh_2d;
pub mod movement;
pub mod prelude;
pub mod shmup;
pub mod user_interface;
pub mod util;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    gltf::GltfExtras,
    pbr::CubemapVisibleEntities,
    prelude::{
        info, App, Button, ClearColor, Color, ComputedVisibility, CoreStage, Msaa,
        ParallelSystemDescriptorCoercion, Parent,
    },
    reflect::{ReflectDeserialize, ReflectSerialize, TypeRegistry},
    render::{
        camera::CameraRenderGraph,
        primitives::{CubemapFrusta, Frustum},
        view::VisibleEntities,
    },
    ui::{widget::ImageMode, CalculatedSize, FocusPolicy, Interaction, Node, UiColor},
    DefaultPlugins,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, AdditionalMassProperties};
use gltf_entity::GltfEntityPlugin;
use gltf_json::GltfJsonPlugin;
use integration::register_test_types;
use prelude::{
    convert_mesh_2d, insert_ball_collider, insert_capsule_collider, insert_kinematic_rigid_body,
    Bitflags, CollisionGroupsPlugin, ConvertMesh2d, DebugDumpPlugin, ExportIntegrationPlugin,
    HitboxBundle, InsertBallCollider, InsertCapsuleCollider, InsertKinematicRigidBody,
    KinematicBodyBundle, ReflectBitflags, ReflectIntegrationBlacklist, SensorBundle, ShmupPlugin,
    Timeline,
};

use crate::prelude::{HurtboxBundle, TimelineDamage};

impl Bitflags for ActiveCollisionTypes {
    fn flags() -> std::collections::BTreeMap<String, usize> {
        [
            ("DYNAMIC_DYNAMIC", ActiveCollisionTypes::DYNAMIC_DYNAMIC),
            ("DYNAMIC_KINEMATIC", ActiveCollisionTypes::DYNAMIC_KINEMATIC),
            ("DYNAMIC_STATIC", ActiveCollisionTypes::DYNAMIC_STATIC),
            (
                "KINEMATIC_KINEMATIC",
                ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            ),
            ("KINEMATIC_STATIC", ActiveCollisionTypes::KINEMATIC_STATIC),
            ("STATIC_STATIC", ActiveCollisionTypes::STATIC_STATIC),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.bits() as usize))
        .collect()
    }
}

fn main() {
    let mut app = App::default();

    info!("Main constructed app");

    app.add_plugins(DefaultPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin)
        .add_plugin(ShmupPlugin)
        .add_plugin(games::legend_of_r::LegendOfRPlugin)
        .add_plugin(user_interface::UserInterfacePlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(CollisionGroupsPlugin)
        .add_plugin(GltfJsonPlugin)
        .add_plugin(GltfEntityPlugin);

    info!("Main added first plugin set");

    register_test_types(&mut app);

    app.register_type::<ActiveCollisionTypes>()
        .register_type::<InsertKinematicRigidBody>()
        .register_type::<InsertBallCollider>()
        .register_type::<InsertCapsuleCollider>()
        .register_type::<ConvertMesh2d>()
        .register_type::<KinematicBodyBundle>()
        .register_type::<SensorBundle>()
        .register_type::<HitboxBundle>()
        .register_type::<HurtboxBundle>();

    info!("Main registered types");

    let mut type_registry = app.world.resource::<TypeRegistry>().write();
    type_registry.register_type_data::<ActiveCollisionTypes, ReflectSerialize>();
    type_registry.register_type_data::<ActiveCollisionTypes, ReflectDeserialize>();
    type_registry.register_type_data::<ActiveCollisionTypes, ReflectBitflags>();

    type_registry.register_type_data::<AdditionalMassProperties, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<Button, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<CalculatedSize, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<CameraRenderGraph, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<ComputedVisibility, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<CubemapFrusta, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<CubemapVisibleEntities, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<FocusPolicy, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<Frustum, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<GltfExtras, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<ImageMode, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<Interaction, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<Node, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<Parent, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<Timeline, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<TimelineDamage, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<UiColor, ReflectIntegrationBlacklist>();
    type_registry.register_type_data::<VisibleEntities, ReflectIntegrationBlacklist>();

    info!("Main registered type data");

    /*
    let value = [1234, 2345, 3456, 4567];
    let serializer = bevy::reflect::serde::ReflectSerializer::new(&value, &type_registry);
    panic!("{}", serde_json::to_string_pretty(&serializer).unwrap());
    */

    drop(type_registry);

    info!("Main dropped type registry");

    app.add_system_to_stage(CoreStage::PreUpdate, insert_kinematic_rigid_body)
        .add_system_to_stage(
            CoreStage::PreUpdate,
            insert_ball_collider.after(insert_kinematic_rigid_body),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            insert_capsule_collider.after(insert_kinematic_rigid_body),
        );

    app.add_system_to_stage(CoreStage::PreUpdate, convert_mesh_2d);

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK));

    info!("Main adding export plugins");

    app.add_plugin(ExportIntegrationPlugin {
        path: "assets/blender/types_shmup.json",
    });

    app.add_plugin(DebugDumpPlugin);

    app.run();
}
