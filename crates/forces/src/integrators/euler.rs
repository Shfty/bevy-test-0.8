use crate::{Acceleration, Velocity};

use crate::ForceDeltaTime;
use bevy::math::Quat;
use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, Query, Res, Transform, With},
    reflect::Reflect,
};
use transform_ex::quat_ex::QuatEx;

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ExplicitEuler;

fn euler_position(dt: f32, velocity: &Velocity, transform: &mut Transform) {
    transform.translation += velocity.translation * dt;
    transform.rotation *= Quat::delta_rotation(velocity.rotation, dt);
    transform.scale += velocity.scale * dt;
}

fn euler_velocity(dt: f32, acceleration: &Acceleration, velocity: &mut Velocity) {
    velocity.translation += acceleration.translation * dt;
    velocity.rotation += acceleration.rotation * dt;
    velocity.scale += acceleration.scale * dt;
}

pub fn explicit_euler(
    dt: Res<ForceDeltaTime>,
    mut query: Query<(&Acceleration, &mut Velocity, &mut Transform), With<ExplicitEuler>>,
) {
    let dt = **dt;
    for (acceleration, mut velocity, mut transform) in query.iter_mut() {
        euler_position(dt, &velocity, &mut transform);
        euler_velocity(dt, &acceleration, &mut velocity);
    }
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ImplicitEuler;

pub fn implicit_euler(
    dt: Res<ForceDeltaTime>,
    mut query: Query<(&Acceleration, &mut Velocity, &mut Transform), With<ImplicitEuler>>,
) {
    let dt = **dt;
    for (acceleration, mut velocity, mut transform) in query.iter_mut() {
        euler_velocity(dt, &acceleration, &mut velocity);
        euler_position(dt, &velocity, &mut transform);
    }
}
