use bevy::{input::gamepad::GamepadSettings, prelude::ResMut};

pub mod input_axis;
pub mod input_button;

pub fn configure_gamepads(mut settings: ResMut<GamepadSettings>) {
    settings.default_axis_settings.positive_high = 1.0;
    settings.default_axis_settings.positive_low = 0.0;
    settings.default_axis_settings.negative_low = 0.0;
    settings.default_axis_settings.negative_high = -1.0;
    settings.default_axis_settings.threshold = 0.0;
}
