//! Plugin for adding gearing-like speed control to a ship

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Changed, Component, Input, Plugin, Query, Res},
    reflect::Reflect,
};

use crate::{movement::linear_move::LinearMoveInput, prelude::PlayerInput};

pub struct ShiftSpeedPlugin;

impl Plugin for ShiftSpeedPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(shift_speed_input);
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ShiftSpeed {
    gear: usize,
    speeds: Vec<f32>,
}

impl Default for ShiftSpeed {
    fn default() -> Self {
        Self {
            gear: 1,
            speeds: vec![4.0, 8.0, 12.0, 16.0],
        }
    }
}

impl ShiftSpeed {
    pub fn shift_up(&mut self) {
        self.gear = self.gear.saturating_add(1).min(self.speeds.len() - 1);
    }

    pub fn shift_down(&mut self) {
        self.gear = self.gear.saturating_sub(1);
    }

    pub fn speed(&self) -> f32 {
        self.speeds[self.gear]
    }
}

pub fn shift_speed_input(
    input: Res<Input<PlayerInput>>,
    mut query_shift_speed: Query<&mut ShiftSpeed>,
) {
    let mut shift_speed = if let Some(components) = query_shift_speed.iter_mut().next() {
        components
    } else {
        return;
    };

    if input.just_pressed(PlayerInput::SlowDown) {
        shift_speed.shift_down();
    }

    if input.just_pressed(PlayerInput::SpeedUp) {
        shift_speed.shift_up();
    }
}

/// Update a LinearMoveInput's factor when its attached ShiftSpeed changes
pub fn shift_speed_linear_factor<T>(
    mut query_ship: Query<(&ShiftSpeed, &mut LinearMoveInput<T>), Changed<ShiftSpeed>>,
) where
    T: Clone + Reflect,
{
    for (shift_speed, mut linear_move_input) in query_ship.iter_mut() {
        linear_move_input.factor = shift_speed.speed();
    }
}
