use std::{hash::Hash, marker::PhantomData};

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{
        default, Component, CoreStage, Input, ParallelSystemDescriptorCoercion, Plugin, Query, Res,
        Transform, Vec3, With,
    },
    reflect::Reflect,
    time::Time,
};

use crate::prelude::{player_input_press, player_input_release};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinearMoveInputMap<T> {
    pub left: Option<T>,
    pub right: Option<T>,
    pub up: Option<T>,
    pub down: Option<T>,
}

pub struct LinearMovePlugin<T> {
    pub inputs: LinearMoveInputMap<T>,
    pub _phantom: PhantomData<T>,
}

impl<T> LinearMovePlugin<T> {
    pub fn new(inputs: LinearMoveInputMap<T>) -> Self {
        LinearMovePlugin {
            inputs,
            _phantom: default(),
        }
    }
}

impl<T> Plugin for LinearMovePlugin<T>
where
    T: 'static + Send + Sync + Copy + Eq + Hash + Reflect,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<LinearMove>();
        app.register_type::<LinearMoveInput<T>>();

        app.insert_resource(self.inputs.clone());

        app.add_system_to_stage(
            CoreStage::PreUpdate,
            linear_move_input::<T>
                .after(player_input_press)
                .after(player_input_release),
        )
        .add_system(linear_move_integrate);
    }
}

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct LinearMoveInput<T>
where
    T: 'static + Send + Sync + Clone + Reflect,
{
    pub factor: f32,
    pub normalize: bool,

    pub left: Option<T>,
    pub right: Option<T>,
    pub up: Option<T>,
    pub down: Option<T>,
}

impl<T> Default for LinearMoveInput<T>
where
    T: 'static + Send + Sync + Clone + Reflect,
{
    fn default() -> Self {
        Self {
            factor: 1.0,
            normalize: true,
            left: default(),
            right: default(),
            up: default(),
            down: default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub struct LinearMove {
    pub active: bool,
    pub delta: Vec3,
}

impl Default for LinearMove {
    fn default() -> Self {
        Self {
            active: true,
            delta: default(),
        }
    }
}

pub fn linear_move_input<T>(
    input: Res<Input<T>>,
    mut query: Query<(&LinearMoveInput<T>, &mut LinearMove), With<LinearMoveInput<T>>>,
) where
    T: 'static + Send + Sync + Copy + Eq + Hash + Reflect,
{
    for (linear_move_input, mut linear_move) in query.iter_mut() {
        if linear_move_input.factor == 0.0 {
            linear_move.delta = Vec3::ZERO;
            continue;
        }

        let left = linear_move_input
            .left
            .map(|left| input.pressed(left) as usize as f32)
            .unwrap_or_default();

        let right = linear_move_input
            .right
            .map(|right| input.pressed(right) as usize as f32)
            .unwrap_or_default();

        let up = linear_move_input
            .up
            .map(|up| input.pressed(up) as usize as f32)
            .unwrap_or_default();

        let down = linear_move_input
            .down
            .map(|down| input.pressed(down) as usize as f32)
            .unwrap_or_default();

        let delta = Vec3::new(right - left, up - down, 0.0);
        let delta = if linear_move_input.normalize && delta.length() > 0.0 {
            delta.normalize()
        } else {
            delta
        };

        let new_delta = delta * linear_move_input.factor;
        if linear_move.delta != new_delta {
            linear_move.delta = new_delta;
        }
    }
}

pub fn linear_move_integrate(time: Res<Time>, mut query: Query<(&LinearMove, &mut Transform)>) {
    let dt = time.delta_seconds();
    for (linear_move, mut transform) in query.iter_mut() {
        if !linear_move.active {
            continue;
        }

        let rotation = transform.rotation;

        transform.translation += rotation * linear_move.delta * dt;
    }
}
