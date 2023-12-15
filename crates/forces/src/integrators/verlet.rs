use bevy::{
    ecs::reflect::ReflectComponent,
    math::Quat,
    prelude::{Component, Query, Res, Transform, With},
    reflect::Reflect,
};

use crate::{Acceleration, ForceDeltaTime, Velocity};

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Verlet {
    pub prev_transform: Transform,
    prev_transform_valid: bool,
}

fn verlet_position(
    dt: f32,
    prev_transform: &Transform,
    transform: &mut Transform,
    acceleration: &Acceleration,
) {
    transform.translation = transform.translation * 2.0 - prev_transform.translation
        + acceleration.translation * dt * dt;
    transform.rotation = Quat::from_scaled_axis(
        transform.rotation.to_scaled_axis() * 2.0 - prev_transform.rotation.to_scaled_axis()
            + acceleration.rotation * dt * dt,
    );
    transform.scale = transform.scale * 2.0 - prev_transform.scale + acceleration.scale * dt * dt;
}

fn integrate_velocity(dt: f32, acceleration: &Acceleration, velocity: &mut Velocity) {
    velocity.translation += acceleration.translation * dt;
    velocity.rotation += acceleration.rotation * dt;
    velocity.scale += acceleration.scale * dt;
}

fn infer_velocity(
    dt: f32,
    prev_transform: &Transform,
    transform: &Transform,
    velocity: &mut Velocity,
) {
    velocity.translation = transform.translation - prev_transform.translation;
    velocity.rotation =
        (prev_transform.rotation.inverse() * transform.rotation).to_scaled_axis() / dt;
    velocity.scale = transform.scale - prev_transform.scale;
}

pub fn verlet(
    dt: Res<ForceDeltaTime>,
    mut query: Query<(&Acceleration, &mut Velocity, &mut Transform, &mut Verlet)>,
) {
    let dt = **dt;
    for (acceleration, mut velocity, mut transform, mut verlet) in query.iter_mut() {
        let prev_transform = if verlet.prev_transform_valid {
            verlet.prev_transform
        } else {
            *transform
        };

        let temp = *transform;

        verlet_position(dt, &prev_transform, &mut transform, acceleration);
        infer_velocity(dt, &prev_transform, &mut transform, &mut velocity);

        verlet.prev_transform = temp;
        verlet.prev_transform_valid = true;
    }
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct StormerVerlet {
    prev_transform: Transform,
    prev_transform_valid: bool,
}

pub fn stormer_verlet(
    dt: Res<ForceDeltaTime>,
    mut query: Query<(
        &Acceleration,
        &mut Velocity,
        &mut Transform,
        &mut StormerVerlet,
    )>,
) {
    let dt = **dt;
    for (acceleration, mut velocity, mut transform, mut stormer_verlet) in query.iter_mut() {
        let prev_transform = if stormer_verlet.prev_transform_valid {
            stormer_verlet.prev_transform
        } else {
            *transform
        };

        let temp = *transform;

        verlet_position(dt, &prev_transform, &mut transform, acceleration);

        stormer_verlet.prev_transform = temp;
        stormer_verlet.prev_transform_valid = true;

        integrate_velocity(dt, acceleration, &mut velocity);
    }
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct VelocityVerlet;

pub fn velocity_verlet(
    dt: Res<ForceDeltaTime>,
    mut query: Query<(&Acceleration, &mut Velocity, &mut Transform), With<VelocityVerlet>>,
) {
    let dt = **dt;
    for (acceleration, mut velocity, mut transform) in query.iter_mut() {
        transform.translation +=
            velocity.translation * dt + 0.5 * acceleration.translation * dt * dt;
        transform.rotation *=
            Quat::from_scaled_axis(velocity.rotation * dt + acceleration.rotation * dt * dt);
        transform.scale += velocity.scale * dt + 0.5 * acceleration.scale * dt * dt;

        integrate_velocity(dt, acceleration, &mut velocity);
    }
}
