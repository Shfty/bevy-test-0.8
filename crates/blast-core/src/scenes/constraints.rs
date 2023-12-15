// TODO: Fix look-at rotation intermittently jumping around
//
// TODO: Better spring generalization
//       * Technically, all forces and constraints that modify a transform are a form of spring
//       * Can't generalize by embedding closures in components since they don't serialize
//       * Most existing forces could be replaced with an operation that modifies a spring's offset parameters
//       * However, such operations would need to run as part of the constraint solver
//       * Need some way to generalize a spring modifier
//
// TODO: Constraint-friendly 'cast ray and set transform' implementation for look target
//
// TODO: Two-bone IK constraint for arms and legs
//
// TODO: Reimplement walking
//       * Mad Cat style rig achieves yaw via free-rotating ankle joints
//         * Legs only ever walk forward in local space - no strafing or twisting
//         * Effectively rotates the whole mech around the planted ankle to turn
//         * Should be able to model turning by engaging a fixed distance constraint between the feet
//           * Remainder of constraints will resolve themselves to the appropriate transforms
//
// TODO: Implement torso gyro simulation
//       * Functions to keep the base level with world Y
//       * Introduces counter-rotation in hip joints to compensate for external forces
//       * Not rotating the base is equivalent to a perfect gyro that doesn't lag or desync
//       * Need to figure out how to model an imperfect gyro
//         * What flaws exist in real-world implementations?
//

use std::f32::consts::FRAC_PI_2;

use anyhow::Result;
use bevy::{
    core::Name,
    pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial},
    prelude::{
        default, shape, Assets, Color, Commands, Component, Entity, Gamepad, GamepadAxisType, Mesh,
        Query, ResMut, Transform, Vec3,
    },
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{ActiveCollisionTypes, Collider, RigidBody};

use flycam::build_flycam;

use ecs_ex::WithName;
use forces::Displacement;
use glam::{EulerRot, Quat};
use input_ex::input_axis::{InputAxis, InputAxis3};
use rapier_ex::interpolation::TransformInterpolation;
use transform_ex::TransformField;

#[allow(dead_code)]
pub fn setup(
    mut force_delta_time: ResMut<forces::ForceDeltaTime>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    **force_delta_time = 1.0 / 5.0;

    let mesh_base = meshes.add(Mesh::from(shape::Box::new(1.0, 0.5, 1.0)));
    let mesh_cube = meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0)));
    let mesh_sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.5,
        subdivisions: 1,
    }));

    let material_red = materials.add(Color::rgb(0.8, 0.25, 0.25).into());
    let material_green = materials.add(Color::rgb(0.25, 0.8, 0.25).into());
    let material_blue = materials.add(Color::rgb(0.25, 0.25, 0.8).into());
    let material_white = materials.add(Color::rgb(0.8, 0.8, 0.8).into());

    // Light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            range: 40.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 8.0, -6.0),
        ..default()
    });

    // Camera
    build_flycam(
        &mut commands,
        Transform::from_xyz(10.0, 0.0, -6.0).looking_at(Vec3::new(0.0, 0.0, -6.0), Vec3::Y),
    );

    // Input
    let look_input = commands
        .spawn()
        .insert(InputAxis3 {
            x: Some(InputAxis {
                gamepad: Gamepad(0),
                axis: GamepadAxisType::LeftStickX,
                multiplier: -1.0,
                ..default()
            }),
            y: Some(InputAxis {
                gamepad: Gamepad(0),
                axis: GamepadAxisType::LeftStickY,
                multiplier: -1.0,
                ..default()
            }),
            z: None,
        })
        .id();

    // Floor
    for i in 0..4 {
        commands
            .spawn_bundle(PbrBundle {
                transform: Transform::from_xyz(0.0, -10.0 + i as f32, 16.0 + -4.0 * i as f32),
                mesh: meshes.add(Mesh::from(shape::Box::new(10.0, 10.0, 4.0))),
                material: material_white.clone(),
                ..default()
            })
            .insert(RigidBody::Fixed)
            .insert(Collider::cuboid(5.0, 5.0, 2.0))
            .with_name("Floor");
    }

    for i in 0..5 {
        commands
            .spawn_bundle(PbrBundle {
                transform: Transform::from_xyz(0.0, -6.0 - i as f32, -4.0 * i as f32),
                mesh: meshes.add(Mesh::from(shape::Box::new(10.0, 10.0, 4.0))),
                material: material_white.clone(),
                ..default()
            })
            .insert(RigidBody::Fixed)
            .insert(Collider::cuboid(5.0, 5.0, 2.0))
            .with_name("Floor");
    }

    // Wall
    let _wall = commands
        .spawn_bundle(PbrBundle {
            transform: Transform::from_xyz(0.0, 0.0, -17.5),
            mesh: meshes.add(Mesh::from(shape::Box::new(10.0, 10.0, 1.0))),
            material: material_white.clone(),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(5.0, 5.0, 0.5))
        .with_name("Left Foot")
        .id();

    // Mech
    let look_raycast = commands.spawn().id();
    let base = commands.spawn().id();
    let torso = commands.spawn().id();
    let head = commands.spawn().id();
    let pom_pom = commands.spawn().id();
    let left_rocket_pod = commands.spawn().id();
    let left_shoulder = commands.spawn().id();
    let left_elbow = commands.spawn().id();
    let left_hand = commands.spawn().id();
    let right_rocket_pod = commands.spawn().id();
    let right_shoulder = commands.spawn().id();
    let right_elbow = commands.spawn().id();
    let right_hand = commands.spawn().id();

    // Left Foot
    let left_foot = commands
        .spawn()
        .insert_bundle(PbrBundle {
            transform: Transform::from_xyz(-1.0, 0.0, 0.0),
            mesh: mesh_cube.clone(),
            material: material_red.clone(),
            ..default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC)
        .insert(forces::Velocity::default())
        .insert(forces::Acceleration::default())
        .insert(forces::integrators::verlet::VelocityVerlet::default())
        .id();

    let _left_foot_force = forces::KinematicForceBuilder::new(&mut commands)
        .with_target(left_foot)
        .with_force(forces::force::rapier::Gravity::default())
        .spawn();

    let _left_foot_impulse = forces::KinematicImpulseBuilder::new(&mut commands)
        .with_target(left_foot)
        .with_force(
            forces::force::Constant::default().with_constant(forces::TransformDerivative {
                translation: Vec3::Z * -1.0,
                ..default()
            }),
        )
        .with_force(forces::force::Damping::default().with_target(left_foot))
        .spawn();

    let left_foot_constraint = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_foot)
        .with_force(forces::force::rapier::Depenetration::default().with_target(left_foot))
        .spawn();

    // Right Foot
    let right_foot = commands
        .spawn()
        .insert_bundle(PbrBundle {
            transform: Transform::from_xyz(1.0, 0.0, 0.0),
            mesh: mesh_cube.clone(),
            material: material_blue.clone(),
            ..default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC)
        .insert(forces::Velocity::default())
        .insert(forces::Acceleration::default())
        .insert(forces::integrators::verlet::VelocityVerlet::default())
        .id();

    let _right_foot_force = forces::KinematicForceBuilder::new(&mut commands)
        .with_target(right_foot)
        .with_force(forces::force::rapier::Gravity::default())
        .spawn();

    let _right_foot_impulse = forces::KinematicImpulseBuilder::new(&mut commands)
        .with_target(right_foot)
        .with_force(
            forces::force::Constant::default().with_constant(forces::TransformDerivative {
                translation: Vec3::Z * -0.5,
                ..default()
            }),
        )
        .with_force(forces::force::Damping::default().with_target(right_foot))
        .spawn();

    let right_foot_constraint = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_foot)
        .with_force(forces::force::rapier::Depenetration::default().with_target(right_foot))
        .spawn();

    let foot_center = commands
        .spawn()
        .insert_bundle(PbrBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mesh: mesh_cube.clone(),
            material: material_green.clone(),
            ..default()
        })
        .id();

    let foot_center_constraint = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(foot_center)
        .with_force(
            forces::force::Displacement::default()
                .with_from(foot_center)
                .with_to(left_foot),
        )
        .with_force(
            forces::force::Displacement::default()
                .with_from(foot_center)
                .with_to(right_foot),
        )
        .with_force(forces::force::Multiply::default().with_multiplier(0.5))
        .with_dependency(left_foot_constraint)
        .with_dependency(right_foot_constraint)
        .spawn();

    // Look Target
    let look_anchor = commands
        .spawn()
        .insert_bundle(TransformBundle::default())
        .insert(TransformInput {
            euler: EulerRot::YXZ,
            entities: TransformInputEntities {
                rotation: Some(look_input),
                ..default()
            },
            factor: forces::TransformDerivative {
                rotation: Vec3::new(FRAC_PI_2 * 1.5, FRAC_PI_2, 0.0),
                ..default()
            },
        })
        .id();

    let look_anchor_to_foot_center = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(look_anchor)
        .with_force(
            Displacement::default()
                .with_from(look_anchor)
                .with_to(foot_center)
                .with_fields(TransformField::Translation),
        )
        .with_dependency(foot_center_constraint)
        .spawn();

    let look_anchor_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(look_anchor)
        .with_force(
            forces::Constant::default().with_constant(forces::TransformDerivative {
                translation: Vec3::new(0.0, 2.75, 0.0),
                ..default()
            }),
        )
        .with_dependency(look_anchor_to_foot_center)
        .spawn();

    let look_target = commands
        .spawn_bundle(TransformBundle::default())
        .with_name("Look Target")
        .id();

    let look_target_to_look_anchor = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(look_target)
        .with_force(
            Displacement::default()
                .with_from(look_target)
                .with_to(look_anchor),
        )
        .with_dependency(look_anchor_offset)
        .spawn();

    let look_target_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(look_target)
        .with_force(
            forces::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(0.0, 0.0, -100.0),
                    ..default()
                })
                .with_reference(Some(look_anchor.into())),
        )
        .with_dependency(look_target_to_look_anchor)
        .spawn();

    commands
        .entity(look_raycast)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_red.clone(),
            ..default()
        })
        .with_name("Look Raycast");

    let look_raycast_constraint = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(look_raycast)
        .with_force(
            forces::force::rapier::Raycast::default()
                .with_target(look_raycast)
                .with_from(look_anchor)
                .with_to(look_target)
                .with_filter(forces::force::rapier::RaycastFilter::Blacklist(vec![
                    pom_pom,
                    head,
                    torso,
                    base,
                    left_rocket_pod,
                    left_shoulder,
                    left_elbow,
                    left_hand,
                    right_rocket_pod,
                    right_shoulder,
                    right_elbow,
                    right_hand,
                ])),
        )
        .with_dependency(look_target_offset)
        .spawn();

    // Base
    commands
        .entity(base)
        .insert_bundle(PbrBundle {
            mesh: mesh_base,
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.25, 0.5))
        .insert(Name::new("Base"));

    let base_to_foot_center = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(base)
        .with_force(Displacement::default().with_from(base).with_to(foot_center))
        .with_dependency(look_raycast_constraint)
        .spawn();

    let base_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(base)
        .with_force(
            forces::force::Constant::default().with_constant(forces::TransformDerivative {
                translation: Vec3::new(0.0, 2.0, 0.0),
                ..default()
            }),
        )
        .with_dependency(base_to_foot_center)
        .spawn();

    let base_look_at = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(base)
        .with_force(
            forces::force::Displacement::default()
                .with_from(base)
                .with_to(look_raycast)
                .with_fields(TransformField::Translation),
        )
        .with_force(
            forces::force::Constant::default().with_constant(forces::TransformDerivative {
                translation: -Vec3::Y,
                ..default()
            }),
        )
        .with_force(
            forces::force::Planar::default()
                .with_normal(Vec3::X)
                .with_reference(Some(foot_center.into()))
                .with_fields(TransformField::Translation),
        )
        .with_force(
            forces::force::LookAt::default()
                .with_target(base)
                .with_reference(foot_center),
        )
        .with_force(
            forces::force::MaxProjection::default()
                .with_axis(Vec3::X)
                .with_min(-std::f32::consts::FRAC_PI_8)
                .with_max(std::f32::consts::FRAC_PI_8)
                .with_fields(TransformField::Rotation),
        )
        .with_dependency(base_offset)
        .spawn();

    // Torso
    commands
        .entity(torso)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Torso"));

    let torso_to_base = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(torso)
        .with_force(Displacement::default().with_from(torso).with_to(base))
        .with_dependency(base_look_at)
        .spawn();

    let torso_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(torso)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(0.0, 1.0, 0.0),
                    ..default()
                })
                .with_reference(Some(base.into())),
        )
        .with_dependency(torso_to_base)
        .spawn();

    let torso_look_at = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(torso)
        .with_force(
            forces::force::Displacement::default()
                .with_from(torso)
                .with_to(look_raycast)
                .with_fields(TransformField::Translation),
        )
        .with_force(
            forces::force::Planar::default()
                .with_normal(Vec3::Y)
                .with_reference(Some(base.into()))
                .with_fields(TransformField::Translation),
        )
        .with_force(
            forces::force::LookAt::default()
                .with_target(torso)
                .with_reference(base),
        )
        .with_force(
            forces::force::MaxProjection::default()
                .with_axis(Vec3::Y)
                .with_min(-std::f32::consts::FRAC_PI_4 * 1.5)
                .with_max(std::f32::consts::FRAC_PI_4 * 1.5)
                .with_fields(TransformField::Rotation),
        )
        .with_dependency(torso_offset)
        .spawn();

    // Head
    commands
        .entity(head)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Head"));

    let head_to_torso = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(head)
        .with_force(
            forces::force::Displacement::default()
                .with_from(head)
                .with_to(torso),
        )
        .with_dependency(torso_look_at)
        .spawn();

    let _head_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(head)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: -Vec3::Z,
                    ..default()
                })
                .with_reference(Some(torso.into())),
        )
        .with_dependency(head_to_torso)
        .spawn();

    // Left Rocket Pod
    commands
        .entity(left_rocket_pod)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Left Rocket Pod"));

    let left_rocket_pod_to_torso = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_rocket_pod)
        .with_force(
            forces::force::Displacement::default()
                .with_from(left_rocket_pod)
                .with_to(torso),
        )
        .with_dependency(torso_look_at)
        .spawn();

    let _left_rocket_pod_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_rocket_pod)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(-0.75, 1.0, 0.0),
                    ..default()
                })
                .with_reference(Some(torso.into())),
        )
        .with_dependency(left_rocket_pod_to_torso)
        .spawn();

    // Right Rocket Pod
    commands
        .entity(right_rocket_pod)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Right Rocket Pod"));

    let right_rocket_pod_to_torso = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_rocket_pod)
        .with_force(
            forces::force::Displacement::default()
                .with_from(right_rocket_pod)
                .with_to(torso),
        )
        .with_dependency(torso_look_at)
        .spawn();

    let _right_rocket_pod_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_rocket_pod)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(0.75, 1.0, 0.0),
                    ..default()
                })
                .with_reference(Some(torso.into())),
        )
        .with_dependency(right_rocket_pod_to_torso)
        .spawn();

    // Pom-pom
    commands
        .entity(pom_pom)
        .insert_bundle(PbrBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 5.5, 0.0),
                ..default()
            },
            mesh: mesh_sphere.clone(),
            material: material_red.clone(),
            ..default()
        })
        .insert(forces::Velocity::default())
        .insert(forces::Acceleration::default())
        .insert(forces::integrators::verlet::Verlet::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(0.5))
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC)
        .insert(TransformInterpolation::default())
        .insert(Name::new("Pom-Pom"));

    let _pom_pom_to_head = forces::KinematicForceBuilder::new(&mut commands)
        .with_target(pom_pom)
        .with_force(
            forces::force::Displacement::default()
                .with_from(pom_pom)
                .with_to(head),
        )
        .with_force(
            forces::force::Constant::default().with_constant(forces::TransformDerivative {
                translation: Vec3::new(0.0, 1.0, 0.0),
                ..default()
            }),
        )
        .with_force(forces::force::Damping::default().with_target(pom_pom))
        .spawn();

    let _pom_pom_constraint = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(pom_pom)
        .with_force(forces::force::rapier::Depenetration::default().with_target(pom_pom))
        .spawn();

    // Left Shoulder
    commands
        .entity(left_shoulder)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Left Shoulder"));

    let left_shoulder_to_torso = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_shoulder)
        .with_force(
            forces::force::Displacement::default()
                .with_from(left_shoulder)
                .with_to(torso),
        )
        .with_dependency(torso_look_at)
        .spawn();

    let left_shoulder_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_shoulder)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(-1.0, 0.0, 0.0),
                    ..default()
                })
                .with_reference(Some(torso.into())),
        )
        .with_dependency(left_shoulder_to_torso)
        .spawn();

    let left_shoulder_look_at = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_shoulder)
        .with_force(
            Displacement::default()
                .with_from(left_shoulder)
                .with_to(look_raycast),
        )
        .with_force(
            forces::force::Constant::default().with_constant(forces::TransformDerivative {
                translation: Vec3::Y,
                ..default()
            }),
        )
        .with_force(
            forces::force::Planar::default()
                .with_normal(Vec3::X)
                .with_reference(Some(torso.into()))
                .with_fields(TransformField::Translation),
        )
        .with_force(
            forces::force::LookAt::default()
                .with_target(left_shoulder)
                .with_reference(torso),
        )
        .with_dependency(left_shoulder_offset)
        .spawn();

    // Left Elbow
    commands
        .entity(left_elbow)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Left Elbow"));

    let left_elbow_to_left_shoulder = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_elbow)
        .with_force(
            Displacement::default()
                .with_from(left_elbow)
                .with_to(left_shoulder),
        )
        .with_dependency(left_shoulder_look_at)
        .spawn();

    let left_elbow_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_elbow)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(0.0, -1.0, 0.0),
                    ..default()
                })
                .with_reference(Some(left_shoulder.into())),
        )
        .with_dependency(left_elbow_to_left_shoulder)
        .spawn();

    let left_elbow_look_at = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_elbow)
        .with_force(
            forces::force::Displacement::default()
                .with_from(left_elbow)
                .with_to(look_raycast),
        )
        .with_force(
            forces::force::Planar::default()
                .with_normal(Vec3::Y)
                .with_reference(Some(left_shoulder.into())),
        )
        .with_force(
            forces::force::LookAt::default()
                .with_target(left_elbow)
                .with_reference(left_shoulder),
        )
        .with_force(
            forces::force::MaxProjection::default()
                .with_axis(Vec3::Y)
                .with_reference(Some(left_shoulder.into()))
                .with_min(-std::f32::consts::FRAC_PI_4)
                .with_max(std::f32::consts::FRAC_PI_2)
                .with_fields(TransformField::Rotation),
        )
        .with_dependency(left_elbow_offset)
        .spawn();

    // Left Hand
    commands
        .entity(left_hand)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Left Hand"));

    let left_hand_to_left_elbow = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_hand)
        .with_force(
            forces::force::Displacement::default()
                .with_from(left_hand)
                .with_to(left_elbow),
        )
        .with_dependency(left_elbow_look_at)
        .spawn();

    let _left_hand_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(left_hand)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: -Vec3::Z,
                    ..default()
                })
                .with_reference(Some(left_elbow.into())),
        )
        .with_dependency(left_hand_to_left_elbow)
        .spawn();

    // Right Shoulder
    commands
        .entity(right_shoulder)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Right Shoulder"));

    let right_shoulder_to_torso = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_shoulder)
        .with_force(
            forces::force::Displacement::default()
                .with_from(right_shoulder)
                .with_to(torso),
        )
        .with_dependency(torso_look_at)
        .spawn();

    let right_shoulder_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_shoulder)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(1.0, 0.0, 0.0),
                    ..default()
                })
                .with_reference(Some(torso.into())),
        )
        .with_dependency(right_shoulder_to_torso)
        .spawn();

    let right_shoulder_look_at = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_shoulder)
        .with_force(
            Displacement::default()
                .with_from(right_shoulder)
                .with_to(look_raycast),
        )
        .with_force(
            forces::force::Constant::default().with_constant(forces::TransformDerivative {
                translation: Vec3::Y,
                ..default()
            }),
        )
        .with_force(
            forces::force::Planar::default()
                .with_normal(Vec3::X)
                .with_reference(Some(torso.into()))
                .with_fields(TransformField::Translation),
        )
        .with_force(
            forces::force::LookAt::default()
                .with_target(right_shoulder)
                .with_reference(torso),
        )
        .with_dependency(right_shoulder_offset)
        .spawn();

    // Right Elbow
    commands
        .entity(right_elbow)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Right Elbow"));

    let right_elbow_to_right_shoulder = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_elbow)
        .with_force(
            Displacement::default()
                .with_from(right_elbow)
                .with_to(right_shoulder),
        )
        .with_dependency(right_shoulder_look_at)
        .spawn();

    let right_elbow_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_elbow)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: Vec3::new(0.0, -1.0, 0.0),
                    ..default()
                })
                .with_reference(Some(left_shoulder.into())),
        )
        .with_dependency(right_elbow_to_right_shoulder)
        .spawn();

    let right_elbow_look_at = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_elbow)
        .with_force(
            forces::force::Displacement::default()
                .with_from(right_elbow)
                .with_to(look_raycast),
        )
        .with_force(
            forces::force::Planar::default()
                .with_normal(Vec3::Y)
                .with_reference(Some(right_shoulder.into())),
        )
        .with_force(
            forces::force::LookAt::default()
                .with_target(right_elbow)
                .with_reference(right_shoulder),
        )
        .with_force(
            forces::force::MaxProjection::default()
                .with_axis(Vec3::Y)
                .with_reference(Some(right_shoulder.into()))
                .with_min(-std::f32::consts::FRAC_PI_2)
                .with_max(std::f32::consts::FRAC_PI_4)
                .with_fields(TransformField::Rotation),
        )
        .with_dependency(right_elbow_offset)
        .spawn();

    // Right Hand
    commands
        .entity(right_hand)
        .insert_bundle(PbrBundle {
            mesh: mesh_cube.clone(),
            material: material_white.clone(),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Name::new("Right Hand"));

    let right_hand_to_right_elbow = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_hand)
        .with_force(
            forces::force::Displacement::default()
                .with_from(right_hand)
                .with_to(right_elbow),
        )
        .with_dependency(right_elbow_look_at)
        .spawn();

    let _right_hand_offset = forces::KinematicConstraintBuilder::new(&mut commands)
        .with_target(right_hand)
        .with_force(
            forces::force::Constant::default()
                .with_constant(forces::TransformDerivative {
                    translation: -Vec3::Z,
                    ..default()
                })
                .with_reference(Some(right_elbow.into())),
        )
        .with_dependency(right_hand_to_right_elbow)
        .spawn();
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TransformInputEntities {
    pub translation: Option<Entity>,
    pub rotation: Option<Entity>,
    pub scale: Option<Entity>,
    pub factor: forces::TransformDerivative,
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct TransformInput {
    pub euler: EulerRot,
    pub entities: TransformInputEntities,
    pub factor: forces::TransformDerivative,
}

pub fn transform_input(
    query_input: Query<&InputAxis3>,
    mut query_motor: Query<(&TransformInput, &mut Transform)>,
) -> Result<()> {
    for (transform_input, mut transform) in query_motor.iter_mut() {
        if let Some(translation) = transform_input.entities.translation {
            let input = query_input.get(translation)?;
            transform.translation = input.value() * transform_input.factor.translation;
        }

        if let Some(rotation) = transform_input.entities.rotation {
            let input_axis = query_input.get(rotation)?;
            let value = input_axis.value() * transform_input.factor.rotation;
            transform.rotation = Quat::from_euler(transform_input.euler, value.x, value.y, value.z);
        }

        if let Some(scale) = transform_input.entities.scale {
            let input = query_input.get(scale)?;
            transform.scale = input.value() * transform_input.factor.scale;
        }
    }

    Ok(())
}

/*
#[derive(Debug, Default, Copy, Clone, Deref, DerefMut, Component)]
pub struct MotorInput(pub TransformInputEntities);

pub fn motor_input(
    query_input: Query<&InputAxis3>,
    mut query_motor: Query<(&MotorInput, &mut Motor)>,
) -> Result<()> {
    for (transform_input, mut motor) in query_motor.iter_mut() {
        if let Some(translation) = transform_input.translation {
            let input = query_input.get(translation)?;
            motor.force.translation = input.value() * transform_input.factor.translation;
        }

        if let Some(rotation) = transform_input.rotation {
            let input = query_input.get(rotation)?;
            motor.force.rotation = input.value() * transform_input.factor.rotation;
        }

        if let Some(scale) = transform_input.scale {
            let input = query_input.get(scale)?;
            motor.force.scale = input.value() * transform_input.factor.scale;
        }
    }

    Ok(())
}
*/
