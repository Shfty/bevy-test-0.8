use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, Plugin, Query, Res, Transform, Vec3},
    reflect::Reflect,
    time::Time,
};

pub struct LinearMoveToPlugin;

impl Plugin for LinearMoveToPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<LinearMoveTo>();

        app.add_system(linear_move_to);
    }
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct LinearMoveTo {
    pub active: bool,
    pub target: Vec3,
    pub speed: f32,
    pub factor: Vec3,
}

pub fn linear_move_to(time: Res<Time>, mut query: Query<(&LinearMoveTo, &mut Transform)>) {
    for (mover, mut transform) in query.iter_mut() {
        if !mover.active {
            continue;
        }

        let dt = time.delta_seconds();
        let delta = (mover.target - transform.translation) * mover.factor;
        let sign = delta.normalize_or_zero();

        let delta_move = sign * mover.speed * dt;
        transform.translation += delta_move.abs().min(delta.abs()) * sign;
    }
}

