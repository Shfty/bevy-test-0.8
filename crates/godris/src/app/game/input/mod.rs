pub mod input_controller;
pub mod input_float;
pub mod input_reader;

use bevy::{
    input::{gamepad::gamepad_event_system, keyboard::keyboard_input_system},
    prelude::{
        CoreStage, ExclusiveSystemDescriptorCoercion, IntoExclusiveSystem,
        ParallelSystemDescriptorCoercion, Plugin, SystemSet,
    },
};
use bevy_fnord::prelude::{evaluate_edge, evaluate_edge_mut};

use crate::prelude::{
    input_float_end, input_float_gamepad, input_float_keyboard, input_float_start,
};

use self::{input_float::{
    input_float_mouse_button, input_float_mouse_motion, input_float_mouse_wheel, InputFloat,
    InputType,
}, input_reader::InputReader};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<InputFloat>()
            .register_type::<InputType>()
            .register_type::<InputReader>();

        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::default()
                .with_system(input_float_start)
                .with_system(
                    input_float_gamepad
                        .after(input_float_start)
                        .after(gamepad_event_system)
                        .before(input_float_end),
                )
                .with_system(
                    input_float_keyboard
                        .after(input_float_start)
                        .after(keyboard_input_system)
                        .before(input_float_end),
                )
                .with_system(
                    input_float_mouse_motion
                        .after(input_float_start)
                        // What system governs mouse motion input? Should have an after constraint
                        .before(input_float_end),
                )
                .with_system(
                    input_float_mouse_button
                        .after(input_float_start)
                        // What system governs mouse button input? Should have an after constraint
                        .before(input_float_end),
                )
                .with_system(
                    input_float_mouse_wheel
                        .after(input_float_start)
                        // What system governs mouse wheel input? Should have an after constraint
                        .before(input_float_end),
                )
                .with_system(input_float_end)
                .with_system(evaluate_edge)
                .with_system(evaluate_edge_mut.exclusive_system().at_end()),
        );
    }
}
