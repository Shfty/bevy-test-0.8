pub mod systems;

use std::ops::RangeInclusive;

use bevy::{
    ecs::system::AsSystemLabel,
    math::{Vec2, Vec3},
    pbr::{
        wireframe::{Wireframe, WireframePlugin},
        AlphaMode, PbrBundle, StandardMaterial,
    },
    prelude::{
        default, shape, Assets, Bundle, Color, Commands, Component, CoreStage, Entity, Handle,
        IntoChainSystem, Mesh, ParallelSystemDescriptorCoercion, Plugin, ResMut, SystemSet,
        Transform,
    },
};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::{ActiveCollisionTypes, Collider, RigidBody};

use anyhow::Result;

use crate::{
    plugins::rapier_ex::{
        kinematic::depenetration::{depenetrate_kinematic, KinematicDepenetration},
        kinematic::forces::{
            forces::{add, after_length, damper, from_displacement, to_point, to_sphere},
            ForceType, KinematicAcceleration, KinematicForce, KinematicForces, KinematicVelocity,
        },
        kinematic::hierarchy::{BuildKinematicChildren, KinematicTransform},
        kinematic::look_at::{Axis, KinematicLookAt},
        kinematic::shared_transform::KinematicSharedTransform,
        transform_interpolation::{RapierStage, TransformInterpolation},
    },
    util::{handle_error, TransformField, TransformFieldMask},
};

use systems::{mech_feet, mech_input};

use self::systems::{
    euler_look_at_input, mech_aim_rotation, mech_aim_target, mech_arms_debug_lines,
    mech_head_debug_lines, mech_torso_debug_lines, mech_wish_look_debug_lines,
};

use super::rapier_ex::kinematic::look_at::kinematic_look_at;

pub struct MechPlugin;

impl Plugin for MechPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(WireframePlugin)
            .add_plugin(DebugLinesPlugin::with_depth_test(false))
            .add_system_set_to_stage(
                RapierStage::PrePhysics,
                SystemSet::new()
                    .with_system(
                        mech_feet
                            .chain(handle_error)
                            .before(depenetrate_kinematic)
                            .label(mech_feet.as_system_label()),
                    )
                    .with_system(
                        mech_aim_rotation
                            .chain(handle_error)
                            .after(kinematic_look_at)
                            .label(mech_aim_rotation.as_system_label()),
                    )
                    .with_system(
                        mech_aim_target
                            .chain(handle_error)
                            .after(mech_aim_rotation)
                            .label(mech_aim_target.as_system_label()),
                    )
                    .with_system(
                        euler_look_at_input
                            .chain(handle_error)
                            .after(mech_aim_target)
                            .label(euler_look_at_input.as_system_label()),
                    ), /*
                       .with_system(
                           euler_look_at
                               .chain(handle_error)
                               .after(euler_look_at_input)
                               .label(euler_look_at.as_system_label()),
                       ),
                       */
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::default().with_system(
                    mech_input
                        .chain(handle_error)
                        .after(bevy::input::gamepad::gamepad_event_system)
                        .label(mech_input.as_system_label()),
                ),
            )
            .add_system(mech_torso_debug_lines)
            .add_system(mech_head_debug_lines)
            .add_system(mech_arms_debug_lines)
            .add_system(
                mech_wish_look_debug_lines
                    .chain(handle_error)
                    .label(mech_wish_look_debug_lines.as_system_label()),
            );
    }
}

const TORSO_RATE: f32 = 4.0;
const OTHER_RATE: f32 = 32.0;

#[derive(Debug, Copy, Clone, Component)]
enum ActiveFoot {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Mech {
    pub parts: MechParts,
    pub handles: MechHandles,
    pub state: MechState,
    pub wish_look: Vec2,
    pub wish_walk: Vec2,
    pub wish_turn: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct MechParts {
    pub base: Entity,
    pub left_hip: Entity,
    pub right_hip: Entity,
    pub left_foot: Entity,
    pub right_foot: Entity,
    pub spine: Entity,
    pub torso: Entity,
    pub neck: Entity,
    pub head: Entity,
    pub left_shoulder: Entity,
    pub left_arm: Entity,
    pub right_shoulder: Entity,
    pub right_arm: Entity,
}

#[derive(Debug, Copy, Clone)]
pub struct MechHandles {
    pub step_from: Entity,
    pub step_to: Entity,
    pub aim_rotation: Entity,
    pub aim_target: Entity,
    pub base_target: Entity,
}

#[derive(Debug, Copy, Clone)]
pub enum MechState {
    Idle,
    Walking {
        active_foot: ActiveFoot,
        step_duration: f32,
        step_progress: f32,
        step_translate: Vec2,
        step_rotate: f32,
        step_height: f32,
    },
}

#[derive(Default, Component)]
pub struct Foot;

#[derive(Default, Component)]
pub struct Base;

#[derive(Default, Component)]
pub struct Hip;

#[derive(Component)]
pub struct EulerLookAt {
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub rate_yaw: f32,
    pub rate_pitch: f32,
    pub range_yaw: RangeInclusive<f32>,
    pub range_pitch: RangeInclusive<f32>,
    yaw: f32,
    pitch: f32,
}

impl EulerLookAt {
    fn new(
        rate_yaw: f32,
        rate_pitch: f32,
        range_yaw: RangeInclusive<f32>,
        range_pitch: RangeInclusive<f32>,
    ) -> Self {
        EulerLookAt {
            rate_yaw,
            rate_pitch,
            range_yaw,
            range_pitch,
            ..default()
        }
    }
}

impl Default for EulerLookAt {
    fn default() -> Self {
        Self {
            target_yaw: 0.0,
            target_pitch: 0.0,
            rate_yaw: 10.0,
            rate_pitch: 5.0,
            range_yaw: -45.0_f32.to_radians()..=45.0_f32.to_radians(),
            range_pitch: -45.0_f32.to_radians()..=45.0_f32.to_radians(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Torso;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct TorsoMesh;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Head;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct HeadMesh;

#[derive(Debug, Copy, Clone, Component)]
pub enum Arm {
    Left,
    Right,
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct AimRotation;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct AimTarget;

#[derive(Default, Component)]
pub struct StepFrom;

#[derive(Default, Component)]
pub struct StepTo;

/*
pub fn build_mech(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    transform: Transform,
) -> Result<()> {
    let mut wire_material: StandardMaterial = Color::WHITE.into();
    wire_material.alpha_mode = AlphaMode::Mask(1.1);
    let wire_material_handle = materials.add(wire_material);

    let aim_rotation = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: 0.25,
                    subdivisions: 1,
                }
                .into(),
            ),
            material: wire_material_handle.clone(),
            ..default()
        })
        .insert(AimRotation)
        .insert(TransformInterpolation::default())
        .insert(KinematicTransform {
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            ..default()
        })
        .id();

    let aim_target = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: 0.25,
                    subdivisions: 1,
                }
                .into(),
            ),
            material: wire_material_handle.clone(),
            ..default()
        })
        .insert(Wireframe)
        .insert(KinematicTransform::default())
        .insert(TransformInterpolation::default())
        .insert(AimTarget)
        .id();

    commands
        .entity(aim_rotation)
        .add_kinematic_child(aim_target);

    let left_foot = commands
        .spawn_bundle(KinematicMeshBundle::new(
            transform * Transform::from_translation(Vec3::new(-0.6, -1.0, 0.0)),
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 1.0))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.25, 0.25, 0.5),
        ))
        .insert(KinematicDepenetration::default())
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC)
        .insert(Foot)
        .id();

    let right_foot = commands
        .spawn_bundle(KinematicMeshBundle::new(
            transform * Transform::from_translation(Vec3::new(0.6, -1.0, 0.0)),
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 1.0))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.25, 0.25, 0.5),
        ))
        .insert(KinematicDepenetration::default())
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC)
        .insert(Foot)
        .id();

    let foot_center = commands
        .spawn_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.125,
                subdivisions: 1,
            })),
            wire_material_handle.clone(),
        ))
        .insert(KinematicSharedTransform {
            targets: vec![left_foot, right_foot],
            field: TransformField::Translation | TransformField::Rotation,
        })
        .id();

    let base_target = commands
        .spawn_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.125,
                subdivisions: 1,
            })),
            wire_material_handle.clone(),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            global: TransformFieldMask::all(),
        })
        .id();

    commands
        .entity(foot_center)
        .add_kinematic_child(base_target);

    let left_hip_target = commands
        .spawn_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                subdivisions: 1,
            })),
            wire_material_handle.clone(),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_xyz(-0.35, 0.0, 0.0),
            ..default()
        })
        .id();

    commands
        .entity(base_target)
        .add_kinematic_child(left_hip_target);

    let right_hip_target = commands
        .spawn_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                subdivisions: 1,
            })),
            wire_material_handle.clone(),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_xyz(0.35, 0.0, 0.0),
            ..default()
        })
        .id();

    commands
        .entity(base_target)
        .add_kinematic_child(right_hip_target);

    let base = commands.spawn().id();
    let left_hip = commands.spawn().id();
    let right_hip = commands.spawn().id();

    commands
        .entity(base)
        .insert_bundle(KinematicMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.25, 0.5))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.25, 0.125, 0.25),
        ))
        .insert(KinematicSharedTransform {
            targets: vec![left_hip, right_hip],
            field: TransformField::Translation.into(),
        })
        .insert(KinematicLookAt {
            source: right_hip,
            target: left_hip,
            up: base_target,
            along: Axis::X,
        })
        .insert(Base);

    commands
        .entity(left_hip)
        .insert_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(0.25, 0.25, 0.25))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.125, 0.125, 0.125),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_xyz(-0.3, 0.0, 0.0),
            ..default()
        })
        .insert(KinematicVelocity::default())
        .insert(KinematicAcceleration::default())
        .insert(
            KinematicForces::default()
                // Hip -> Hip Target spring-damper, constraint
                .with_force(KinematicForce::new(
                    left_hip,
                    left_hip_target,
                    ForceType::Acceleration,
                    add(to_point(), damper(0.1)),
                    TransformField::Translation,
                ))
                .with_force(KinematicForce::new(
                    left_hip,
                    left_hip_target,
                    ForceType::Transform,
                    after_length(0.1, from_displacement(to_sphere(0.1))),
                    TransformField::Translation,
                ))
                // Foot -> Hip spring
                .with_force(KinematicForce::new(
                    left_hip,
                    left_foot,
                    ForceType::Acceleration,
                    add(to_sphere(1.0), damper(0.1)),
                    TransformField::Translation,
                )),
        )
        .insert(KinematicSharedTransform {
            targets: vec![base],
            field: TransformField::Rotation.into(),
        })
        .insert(Hip);

    commands
        .entity(right_hip)
        .insert_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(0.25, 0.25, 0.25))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.125, 0.125, 0.125),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_xyz(0.3, 0.0, 0.0),
            ..default()
        })
        .insert(KinematicVelocity::default())
        .insert(KinematicAcceleration::default())
        .insert(
            KinematicForces::default()
                // Hip -> Hip Target spring-damper, constraint
                .with_force(KinematicForce::new(
                    right_hip,
                    right_hip_target,
                    ForceType::Acceleration,
                    add(to_point(), damper(0.1)),
                    TransformField::Translation,
                ))
                .with_force(KinematicForce::new(
                    right_hip,
                    right_hip_target,
                    ForceType::Transform,
                    after_length(0.1, from_displacement(to_sphere(0.1))),
                    TransformField::Translation,
                ))
                // Foot -> Hip spring
                .with_force(KinematicForce::new(
                    right_hip,
                    right_foot,
                    ForceType::Acceleration,
                    add(to_sphere(1.0), damper(0.1)),
                    TransformField::Translation,
                ))
        )
        .insert(KinematicSharedTransform {
            targets: vec![base],
            field: TransformField::Rotation.into(),
        })
        .insert(Hip);

    let torso = commands
        .spawn_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 0.75))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.5, 0.5, 0.375),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_translation(Vec3::new(0.0, 0.7, 0.0)),
            ..default()
        })
        .insert(TorsoMesh)
        .id();

    let spine = commands.spawn().id();
    commands
        .entity(spine)
        .insert_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                subdivisions: 1,
            })),
            wire_material_handle.clone(),
        ))
        .insert(Torso)
        .insert(EulerLookAt::new(
            TORSO_RATE,
            TORSO_RATE,
            -45.0_f32.to_radians()..=45_f32.to_radians(),
            -45.0_f32.to_radians()..=45_f32.to_radians(),
        ))
        .insert(KinematicLookAt {
            source: torso,
            target: aim_target,
            up: base,
            along: Axis::Z,
        })
        /*
            .insert(KinematicForces(vec![
                // Pitch constraint
                KinematicForce::new(
                    spine,
                    base,
                    ForceType::Transform,
                    after_length(
                        std::f32::consts::PI * 0.25,
                        from_displacement(to_sphere(std::f32::consts::PI * 0.25)),
                    ),
                    TransformField::Rotation,
                ),
            ]))
        */
        .insert(KinematicAcceleration::default())
        .insert(KinematicVelocity::default())
        .insert(KinematicTransform {
            transform: Transform::from_xyz(0.0, 0.1, 0.0),
            global: TransformField::Translation.into(),
        });

    commands.entity(spine).add_kinematic_child(torso);

    let head = commands
        .spawn_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 0.5))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.25, 0.25, 0.25),
        ))
        .insert(HeadMesh)
        .insert(KinematicTransform {
            transform: Transform::from_translation(Vec3::new(0.0, 0.35, 0.0)),
            ..default()
        })
        .id();

    let neck = commands.spawn().id();
    commands
        .entity(neck)
        .insert_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                subdivisions: 1,
            })),
            wire_material_handle.clone(),
        ))
        .insert(EulerLookAt::new(
            OTHER_RATE,
            OTHER_RATE,
            -70.0_f32.to_radians()..=70_f32.to_radians(),
            -70.0_f32.to_radians()..=70_f32.to_radians(),
        ))
        .insert(KinematicLookAt {
            source: head,
            target: aim_target,
            up: torso,
            along: Axis::Z,
        })
        .insert(KinematicForces(vec![
            // Pitch constraint
            KinematicForce::new(
                neck,
                torso,
                ForceType::Transform,
                after_length(
                    std::f32::consts::PI * 0.25,
                    from_displacement(to_sphere(std::f32::consts::PI * 0.25)),
                ),
                TransformField::Rotation,
            ),
        ]))
        .insert(KinematicAcceleration::default())
        .insert(KinematicVelocity::default())
        .insert(KinematicTransform {
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            global: TransformField::Translation.into(),
        })
        .insert(Head);

    commands.entity(neck).add_kinematic_child(head);

    let left_shoulder = commands.spawn().id();
    commands
        .entity(left_shoulder)
        .insert_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 0.5))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.25, 0.25, 0.25),
        ))
        .insert(Arm::Left)
        .insert(EulerLookAt::new(
            OTHER_RATE,
            OTHER_RATE,
            -45.0_f32.to_radians()..=90_f32.to_radians(),
            -45.0_f32.to_radians()..=45_f32.to_radians(),
        ))
        .insert(KinematicLookAt {
            source: left_shoulder,
            target: aim_target,
            up: torso,
            along: Axis::Z,
        })
        .insert(KinematicForces(vec![
            // Pitch constraint
            KinematicForce::new(
                left_shoulder,
                torso,
                ForceType::Transform,
                after_length(
                    std::f32::consts::PI * 0.25,
                    from_displacement(to_sphere(std::f32::consts::PI * 0.25)),
                ),
                TransformField::Rotation,
            ),
        ]))
        .insert(KinematicAcceleration::default())
        .insert(KinematicVelocity::default())
        .insert(KinematicTransform {
            transform: Transform::from_xyz(-0.75, 0.35, 0.0),
            global: TransformField::Translation.into(),
        });

    let left_arm = commands
        .spawn_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(0.4, 0.4, 0.4))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.2, 0.2, 0.2),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.25)),
            ..default()
        })
        .id();

    commands.entity(left_shoulder).add_kinematic_child(left_arm);

    let right_shoulder = commands.spawn().id();
    commands
        .entity(right_shoulder)
        .insert_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 0.5))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.25, 0.25, 0.25),
        ))
        .insert(Arm::Right)
        .insert(EulerLookAt::new(
            OTHER_RATE,
            OTHER_RATE,
            -90.0_f32.to_radians()..=45_f32.to_radians(),
            -45.0_f32.to_radians()..=45_f32.to_radians(),
        ))
        .insert(KinematicLookAt {
            source: right_shoulder,
            target: aim_target,
            up: torso,
            along: Axis::Z,
        })
        .insert(KinematicForces(vec![
            // Pitch constraint
            KinematicForce::new(
                right_shoulder,
                torso,
                ForceType::Transform,
                after_length(
                    std::f32::consts::PI * 0.25,
                    from_displacement(to_sphere(std::f32::consts::PI * 0.25)),
                ),
                TransformField::Rotation,
            ),
        ]))
        .insert(KinematicAcceleration::default())
        .insert(KinematicVelocity::default())
        .insert(KinematicTransform {
            transform: Transform::from_xyz(0.75, 0.35, 0.0),
            global: TransformField::Translation.into(),
        });

    let right_arm = commands
        .spawn_bundle(KinematicMeshBundle::new(
            default(),
            meshes.add(Mesh::from(shape::Box::new(0.4, 0.4, 0.4))),
            materials.add(Color::rgb(0.8, 0.25, 0.25).into()),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.2, 0.2, 0.2),
        ))
        .insert(KinematicTransform {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.25)),
            ..default()
        })
        .id();

    commands
        .entity(right_shoulder)
        .add_kinematic_child(right_arm);

    let step_from = commands
        .spawn_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 1.0))),
            wire_material_handle.clone(),
        ))
        .insert(StepFrom)
        .id();

    let step_to = commands
        .spawn_bundle(WireframeMeshBundle::new(
            transform,
            meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 1.0))),
            wire_material_handle,
        ))
        .insert(StepTo)
        .id();

    commands.entity(base).add_kinematic_child(spine);
    commands.entity(torso).add_kinematic_child(neck);
    commands.entity(torso).add_kinematic_child(left_shoulder);
    commands.entity(torso).add_kinematic_child(right_shoulder);

    commands.spawn().insert(Mech {
        parts: MechParts {
            base,
            left_hip,
            right_hip,
            left_foot,
            right_foot,
            spine,
            torso,
            neck,
            head,
            left_shoulder,
            left_arm,
            right_shoulder,
            right_arm,
        },
        handles: MechHandles {
            step_from,
            step_to,
            aim_rotation,
            aim_target,
            base_target,
        },
        wish_look: Vec2::new(0.0, 0.0),
        wish_walk: Vec2::new(0.0, 0.0),
        wish_turn: 0.0,
        state: MechState::Walking {
            active_foot: ActiveFoot::Right,
            step_duration: 0.5,
            step_progress: 0.5,
            step_translate: Vec2::new(2.0, 6.0),
            step_rotate: 12.0,
            step_height: 4.0,
        },
    });

    Ok(())
}
*/
