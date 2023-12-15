use bevy::prelude::{
    default, Component, EventReader, Gamepad, GamepadButtonType, GamepadEvent, GamepadEventType,
    Query,
};

#[derive(Debug, Copy, Clone, Component)]
pub struct InputButton {
    pub gamepad: Gamepad,
    pub button: GamepadButtonType,
    pub value: bool,
}

impl Default for InputButton {
    fn default() -> Self {
        Self {
            gamepad: Gamepad(0),
            button: GamepadButtonType::South,
            value: default(),
        }
    }
}

pub fn input_button(mut events: EventReader<GamepadEvent>, mut query: Query<&mut InputButton>) {
    for event in events.iter() {
        for mut input in query.iter_mut() {
            let event_type = match event {
                GamepadEvent(gamepad, event_type) if *gamepad == input.gamepad => event_type,
                _ => continue,
            };

            let value = match event_type {
                GamepadEventType::ButtonChanged(button_type, value)
                    if *button_type == input.button =>
                {
                    *value > 0.0
                }
                _ => continue,
            };

            input.value = value;
        }
    }
}
