use anyhow::{Context, Result};
use bevy::prelude::{
    EventReader, GamepadAxisType, GamepadEvent, GamepadEventType, GlobalTransform, Query, Res,
    ResMut, Transform, Vec3, With, Without,
};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::{
    plugin::{RapierConfiguration, RapierContext},
    prelude::InteractionGroups,
};
use glam::{EulerRot, Quat};

use crate::plugins::{
    rapier_ex::kinematic::hierarchy::{KinematicTransform, KinematicParents},
    rapier_ex::transform_interpolation::{FixedDeltaTime, TransformInterpolation},
};

use super::{
    ActiveFoot, Arm, Base, EulerLookAt, Foot, Head, Mech, MechState, StepFrom, StepTo, Torso,
};

pub fn mech_input(
    mut gamepad_event: EventReader<GamepadEvent>,
    mut query_mech: Query<&mut Mech>,
) -> Result<()> {
    for event in gamepad_event.iter() {
        for mut mech in query_mech.iter_mut() {
            match &event {
                GamepadEvent(gamepad, GamepadEventType::AxisChanged(axis_type, value)) => {
                    let value = *value;
                    match (gamepad.0, axis_type) {
                        (0, GamepadAxisType::LeftStickX) => {
                            mech.wish_look.x = -value;
                        }
                        (0, GamepadAxisType::LeftStickY) => {
                            mech.wish_look.y = -value;
                        }
                        (0, GamepadAxisType::RightZ) => {
                            //mech.wish_turn = -value;
                        }
                        (0, GamepadAxisType::DPadX) => {}
                        (0, GamepadAxisType::DPadY) => {}
                        (1, GamepadAxisType::LeftZ) => {
                            mech.wish_walk.y = value;
                        }
                        (1, GamepadAxisType::RightZ) => {
                            //mech.wish_walk.x = value;
                            mech.wish_turn = -value;
                        }
                        (1, GamepadAxisType::DPadX) => {}
                        (1, GamepadAxisType::DPadY) => {}
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }

    Ok(())
}

pub fn mech_feet(
    rapier_config: Res<RapierConfiguration>,
    mut query_mech: Query<&mut Mech>,
    query_base: Query<&Transform, (Without<Foot>, Without<StepTo>, Without<StepFrom>)>,
    mut query_foot: Query<&mut Transform, (With<Foot>, Without<StepTo>, Without<StepFrom>)>,
    mut query_step_from: Query<
        &mut Transform,
        (
            With<StepFrom>,
            Without<Base>,
            Without<Foot>,
            Without<StepTo>,
        ),
    >,
    mut query_step_to: Query<
        &mut Transform,
        (
            With<StepTo>,
            Without<Base>,
            Without<Foot>,
            Without<StepFrom>,
        ),
    >,
) -> Result<()> {
    for mut mech in query_mech.iter_mut() {
        mech.state = match mech.state {
            MechState::Idle => todo!(),
            MechState::Walking {
                mut active_foot,
                step_duration,
                mut step_progress,
                step_translate,
                step_rotate,
                step_height,
            } => {
                let base_transform = query_base
                    .get(mech.handles.base_target)
                    .context("Failed to get base")?;

                let mut step_from_transform = query_step_from
                    .get_mut(mech.handles.step_from)
                    .context("Failed to get step-from")?;
                let mut step_to_transform = query_step_to
                    .get_mut(mech.handles.step_to)
                    .context("Failed to get step-to")?;

                let switching = step_progress >= step_duration;

                if switching {
                    step_progress = 0.0;
                    active_foot = match active_foot {
                        ActiveFoot::Left => ActiveFoot::Right,
                        ActiveFoot::Right => ActiveFoot::Left,
                    };
                }

                let mut foot_transform = match active_foot {
                    ActiveFoot::Left => query_foot
                        .get_mut(mech.parts.left_foot)
                        .context("Failed to get left foot"),
                    ActiveFoot::Right => query_foot
                        .get_mut(mech.parts.right_foot)
                        .context("Failed to get right foot"),
                }?;

                let dt = rapier_config.fixed_dt()?;

                *step_from_transform = *foot_transform;
                *step_to_transform = *base_transform;

                let mut wish_direction = mech.wish_walk;
                if wish_direction.length() > 1.0 {
                    wish_direction = wish_direction.normalize();
                }

                step_to_transform.translation += -base_transform.local_y()
                    + (match active_foot {
                        ActiveFoot::Left => -base_transform.local_x(),
                        ActiveFoot::Right => base_transform.local_x(),
                    } * 0.6)
                    + base_transform.local_x() * wish_direction.x * step_translate.x
                    + base_transform.local_z() * wish_direction.y * step_translate.y;
                step_to_transform.translation.y = step_to_transform.translation.y;
                step_to_transform.rotate_around(
                    base_transform.translation,
                    Quat::from_axis_angle(
                        base_transform.local_y(),
                        (mech.wish_turn * 30.0).to_radians(),
                    ),
                );

                step_progress += dt;
                let progress = step_progress / step_duration;
                let mut linvel = Vec3::ZERO;

                if *step_to_transform != *step_from_transform {
                    let vel_xz = step_to_transform.translation - step_from_transform.translation;
                    linvel.x = vel_xz.x;
                    linvel.z = vel_xz.z;
                }

                let vert = (progress * std::f32::consts::PI).cos()
                    * wish_direction.length().max(mech.wish_turn.abs())
                    * step_height;
                linvel.y = vert;

                let mut angvel = Vec3::new(0.0, 0.0, 0.0);
                let cx = step_from_transform
                    .local_z()
                    .cross(step_to_transform.local_z())
                    .dot(step_from_transform.local_y())
                    * step_rotate;
                angvel.y = cx;

                foot_transform.translation += linvel * dt;
                foot_transform.rotation *=
                    Quat::from_euler(EulerRot::XYZ, angvel.x * dt, angvel.y * dt, angvel.z * dt)
                        .normalize();

                MechState::Walking {
                    active_foot,
                    step_duration,
                    step_progress,
                    step_translate,
                    step_rotate,
                    step_height,
                }
            }
        }
    }

    Ok(())
}

pub fn euler_look_at(
    rapier_config: Res<RapierConfiguration>,
    mut query: Query<(&mut EulerLookAt, &mut KinematicTransform)>,
) -> Result<()> {
    for (mut euler_look_at, mut local_transform) in query.iter_mut() {
        let dt = rapier_config.fixed_dt()?;

        let diff_pitch = euler_look_at.target_pitch - euler_look_at.pitch;
        let diff_yaw = euler_look_at.target_yaw - euler_look_at.yaw;

        euler_look_at.pitch = euler_look_at.pitch + euler_look_at.rate_pitch * diff_pitch * dt;
        euler_look_at.yaw = euler_look_at.yaw + euler_look_at.rate_yaw * diff_yaw * dt;
        local_transform.transform.rotation = Quat::from_axis_angle(Vec3::X, euler_look_at.pitch)
            * Quat::from_axis_angle(Vec3::Y, euler_look_at.yaw).normalize();
    }

    Ok(())
}

pub fn euler_look_at_input(
    query_mech: Query<&Mech>,
    query_parent: Query<&KinematicParents>,
    query_transform: Query<&Transform>,
    mut query_euler: Query<&mut EulerLookAt>,
) -> Result<()> {
    for mech in query_mech.iter() {
        let aim_target_transform = *query_transform.get(mech.handles.aim_target)?;

        for (part, offset) in [
            (mech.parts.spine, Vec3::new(0.0, 0.65, 0.0)),
            (mech.parts.neck, Vec3::new(0.0, 0.35, 0.0)),
            (mech.parts.left_shoulder, Vec3::ZERO),
            (mech.parts.right_shoulder, Vec3::ZERO),
        ] {
            let parent_entity = query_parent.get(part).context("Failed to get parent")?;
            let parent_transform = query_transform
                .get(parent_entity[0])
                .context("Failed to get parent transform")?;
            let parent_transform_inv =
                Transform::from_matrix(parent_transform.compute_matrix().inverse());

            let mut local_transform = parent_transform_inv
                * *query_transform
                    .get(part)
                    .context("Failed to get Transform")?;
            local_transform.translation += local_transform.rotation * offset;

            let local_target = parent_transform_inv * aim_target_transform.translation;

            let (y, x, _) = if local_target != local_transform.translation {
                let dir = (local_target - local_transform.translation).normalize();
                let cross = dir.cross(local_transform.local_y());
                if cross.length() == 0.0 {
                    (0.0, 0.0, 0.0)
                } else {
                    local_transform
                        .looking_at(local_target, local_transform.local_y())
                        .rotation
                        .to_euler(EulerRot::YXZ)
                }
            } else {
                (0.0, 0.0, 0.0)
            };

            let mut euler = query_euler
                .get_mut(part)
                .context("Failed to get EulerLookAt")?;

            euler.target_yaw = y.clamp(*euler.range_yaw.start(), *euler.range_yaw.end());
            euler.target_pitch = x.clamp(*euler.range_pitch.start(), *euler.range_pitch.end());
        }
    }

    Ok(())
}

pub fn mech_head_debug_lines(
    mut lines: ResMut<DebugLines>,
    query: Query<&GlobalTransform, With<Head>>,
) {
    for transform in query.iter() {
        let start = transform.translation + transform.local_y() * 0.35;
        lines.line(start, start + transform.local_z() * -10.0, 0.0);
    }
}

pub fn mech_arms_debug_lines(
    mut lines: ResMut<DebugLines>,
    query: Query<&GlobalTransform, With<Arm>>,
) {
    for transform in query.iter() {
        let start = transform.translation;
        lines.line(start, start + transform.local_z() * -10.0, 0.0);
    }
}

pub fn mech_torso_debug_lines(
    mut lines: ResMut<DebugLines>,
    query: Query<&GlobalTransform, With<Torso>>,
) {
    for transform in query.iter() {
        let start = transform.translation + transform.local_y() * 0.65;
        lines.line(start, start + transform.local_z() * -10.0, 0.0);
    }
}

pub fn mech_wish_look_debug_lines(
    rapier_config: Res<RapierConfiguration>,
    mut lines: ResMut<DebugLines>,
    query_mech: Query<&Mech>,
    query_transform: Query<&GlobalTransform>,
    query_lerp: Query<&TransformInterpolation>,
) -> Result<()> {
    for mech in query_mech.iter() {
        let head_transform = query_transform
            .get(mech.parts.head)
            .context("Failed to get head transform")?;

        let aim_target_lerp = query_lerp
            .get(mech.handles.aim_target)
            .context("Failed to get aim target transform")?;

        if let Some(lerp) = aim_target_lerp.lerp(rapier_config.fixed_dt()?) {
            let start = head_transform.translation;
            lines.line(start, lerp.translation, 0.0);
        }
    }

    Ok(())
}

pub fn mech_aim_rotation(
    query_mech: Query<&Mech>,
    query_euler: Query<&EulerLookAt>,
    query_global: Query<&GlobalTransform>,
    mut query_transform: Query<&mut Transform>,
) -> Result<()> {
    for mech in query_mech.iter() {
        let base_transform = *query_transform
            .get(mech.handles.base_target)
            .context("Failed to get base transform")?;

        let spine_transform = *query_global
            .get(mech.parts.spine)
            .context("Failed to get spine transform")?;

        let spine_euler = query_euler
            .get(mech.parts.spine)
            .context("Failed to get torso euler")?;

        let left_arm_euler = query_euler
            .get(mech.parts.left_shoulder)
            .context("Failed to get left arm euler")?;

        let right_arm_euler = query_euler
            .get(mech.parts.right_shoulder)
            .context("Failed to get right arm euler")?;

        let min_yaw = spine_euler.range_yaw.start() + right_arm_euler.range_yaw.start();
        let max_yaw = spine_euler.range_yaw.end() + left_arm_euler.range_yaw.end();

        let min_pitch = spine_euler.range_pitch.start() + right_arm_euler.range_pitch.start();
        let max_pitch = spine_euler.range_pitch.end() + left_arm_euler.range_pitch.end();

        let rotation = Quat::from_axis_angle(
            Vec3::Y,
            mech.wish_look.x
                * if mech.wish_look.x > 0.0 {
                    max_yaw
                } else if mech.wish_look.x < 0.0 {
                    -min_yaw
                } else {
                    0.0
                },
        ) * Quat::from_axis_angle(
            Vec3::X,
            mech.wish_look.y
                * if mech.wish_look.y > 0.0 {
                    max_pitch
                } else if mech.wish_look.y < 0.0 {
                    -min_pitch
                } else {
                    0.0
                },
        )
        .normalize();

        let mut aim_rotation_transform = query_transform
            .get_mut(mech.handles.aim_rotation)
            .context("Failed to get aim rotation transform")?;

        aim_rotation_transform.translation =
            spine_transform.translation + spine_transform.local_y() * 1.45;
        aim_rotation_transform.rotation = base_transform.rotation * rotation;
    }

    Ok(())
}

pub fn mech_aim_target(
    rapier: Res<RapierContext>,
    query_mech: Query<&Mech>,
    query_global: Query<&mut GlobalTransform>,
    mut query_kinematic_parent: Query<&mut KinematicTransform, With<KinematicParents>>,
) -> Result<()> {
    for mech in query_mech.iter() {
        let aim_rotation_transform = query_global
            .get(mech.handles.aim_rotation)
            .context("Failed to get aim rotation")?;

        let mut local_transform = query_kinematic_parent
            .get_mut(mech.handles.aim_target)
            .context("Failed to get aim target")?;

        if let Some((_, toi)) = rapier.cast_ray(
            aim_rotation_transform.translation,
            -aim_rotation_transform.local_z(),
            100.0,
            true,
            InteractionGroups::all(),
            Some(&|entity| {
                ![
                    mech.parts.left_shoulder,
                    mech.parts.left_arm,
                    mech.parts.right_shoulder,
                    mech.parts.right_arm,
                    mech.parts.head,
                    mech.parts.spine,
                    mech.parts.torso,
                    mech.parts.base,
                    mech.parts.left_foot,
                    mech.parts.right_foot,
                ]
                .contains(&entity)
            }),
        ) {
            if toi != 0.0 {
                local_transform.transform.translation.z = -toi;
            }
        } else {
            local_transform.transform.translation.z = -100.0;
        }
    }

    Ok(())
}
