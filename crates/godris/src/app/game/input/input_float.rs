use bevy::{
    ecs::reflect::ReflectComponent,
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        ElementState,
    },
    math::Vec2,
    prelude::{
        default, Component, EventReader, Gamepad, GamepadAxisType, GamepadButtonType, GamepadEvent,
        KeyCode, MouseButton, Query,
    }, reflect::{Reflect, FromReflect},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Reflect, FromReflect)]
pub enum InputType {
    Key(KeyCode),
    MouseX,
    MouseY,
    MouseButton(MouseButton),
    MouseWheelX,
    MouseWheelY,
    Axis(Gamepad, GamepadAxisType),
    Button(Gamepad, GamepadButtonType),
}

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct InputFloat {
    pub input: InputType,
    pub value: f32,
}

impl Default for InputFloat {
    fn default() -> Self {
        Self {
            input: InputType::Axis(Gamepad(1), GamepadAxisType::LeftStickX),
            value: default(),
        }
    }
}

impl InputFloat {
    pub fn float(&self) -> f32 {
        self.value
    }
}

pub fn input_float_start() {}

pub fn input_float_gamepad(
    mut events: EventReader<GamepadEvent>,
    mut query: Query<&mut InputFloat>,
) {
    for GamepadEvent(gamepad, event) in events.iter() {
        for mut input in query.iter_mut() {
            match input.input {
                InputType::Axis(input_gamepad, input_axis_type) => {
                    if *gamepad != input_gamepad {
                        continue;
                    }

                    if let bevy::prelude::GamepadEventType::AxisChanged(axis_type, value) = event {
                        if *axis_type != input_axis_type {
                            continue;
                        }

                        input.value = *value;
                    }
                }
                InputType::Button(input_gamepad, input_button_type) => {
                    if *gamepad != input_gamepad {
                        continue;
                    }

                    if let bevy::prelude::GamepadEventType::ButtonChanged(button_type, value) =
                        event
                    {
                        if *button_type != input_button_type {
                            continue;
                        }

                        input.value = *value;
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn input_float_keyboard(
    mut events: EventReader<KeyboardInput>,
    mut query: Query<&mut InputFloat>,
) {
    for KeyboardInput {
        key_code, state, ..
    } in events.iter()
    {
        let key_code = if let Some(key_code) = key_code {
            key_code
        } else {
            continue;
        };

        for mut input in query.iter_mut() {
            match input.input {
                InputType::Key(input_key_code) => {
                    if *key_code != input_key_code {
                        continue;
                    }

                    input.value = if *state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    };
                }
                _ => (),
            }
        }
    }
}

pub fn input_float_mouse_motion(
    mut events: EventReader<MouseMotion>,
    mut query: Query<&mut InputFloat>,
) {
    for mouse_button_input in events.iter() {
        for mut input in query.iter_mut() {
            match input.input {
                InputType::MouseX => {
                    input.value = mouse_button_input.delta.x;
                }
                InputType::MouseY => {
                    input.value = mouse_button_input.delta.y;
                }
                _ => (),
            }
        }
    }
}

pub fn input_float_mouse_button(
    mut events: EventReader<MouseButtonInput>,
    mut query: Query<&mut InputFloat>,
) {
    for mouse_button_input in events.iter() {
        for mut input in query.iter_mut() {
            match input.input {
                InputType::MouseButton(input_button) => {
                    if mouse_button_input.button != input_button {
                        continue;
                    }

                    input.value = if mouse_button_input.state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn input_float_mouse_wheel(
    mut events: EventReader<MouseWheel>,
    mut query: Query<&mut InputFloat>,
) {
    let mut delta = Vec2::ZERO;

    for MouseWheel { x, y, .. } in events.iter() {
        delta.x += *x;
        delta.y += *y;
    }

    for mut input in query.iter_mut() {
        match input.input {
            InputType::MouseWheelX => {
                input.value = delta.x;
            }
            InputType::MouseWheelY => {
                input.value = delta.y;
            }
            _ => (),
        }
    }
}

pub fn input_float_end() {}
