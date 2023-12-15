use bevy::{
    input::{gamepad::gamepad_event_system, keyboard::keyboard_input_system},
    prelude::{
        CoreStage, GamepadButton, GamepadButtonType, Input, KeyCode,
        ParallelSystemDescriptorCoercion, Plugin, Res, ResMut, SystemLabel, SystemSet,
    },
    reflect::Reflect,
    utils::HashMap,
};

#[derive(SystemLabel)]
pub struct PlayerInputLabel;

pub struct PlayerInputPlugin;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub enum PlayerInput {
    Up,
    Down,
    Left,
    Right,
    Fire,
    Force,
    SpeedUp,
    SlowDown,
}

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Input<PlayerInput>>()
            .init_resource::<PlayerInputMap>();

        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::default()
                .with_system(
                    player_input_clear
                        .after(keyboard_input_system)
                        .after(gamepad_event_system)
                        .before(player_input_press)
                        .before(player_input_release),
                )
                .with_system(player_input_press)
                .with_system(player_input_release)
                .label(PlayerInputLabel),
        );
    }
}

#[derive(Debug, Clone)]
pub struct PlayerInputMap {
    pub gamepad: HashMap<GamepadButtonType, PlayerInput>,
    pub keyboard: HashMap<KeyCode, PlayerInput>,
}

impl Default for PlayerInputMap {
    fn default() -> Self {
        Self {
            gamepad: [
                (GamepadButtonType::DPadLeft, PlayerInput::Left),
                (GamepadButtonType::DPadDown, PlayerInput::Down),
                (GamepadButtonType::DPadUp, PlayerInput::Up),
                (GamepadButtonType::DPadRight, PlayerInput::Right),
                (GamepadButtonType::East, PlayerInput::Fire),
                (GamepadButtonType::North, PlayerInput::Force),
                (GamepadButtonType::LeftTrigger, PlayerInput::SpeedUp),
                (GamepadButtonType::RightTrigger, PlayerInput::SlowDown),
            ]
            .into_iter()
            .collect(),
            keyboard: [
                (KeyCode::H, PlayerInput::Left),
                (KeyCode::J, PlayerInput::Down),
                (KeyCode::K, PlayerInput::Up),
                (KeyCode::L, PlayerInput::Right),
                (KeyCode::F, PlayerInput::Fire),
                (KeyCode::D, PlayerInput::Force),
                (KeyCode::S, PlayerInput::SpeedUp),
                (KeyCode::A, PlayerInput::SlowDown),
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl PlayerInputMap {
    pub fn gamepad(&self, gamepad_button: &GamepadButtonType) -> Option<PlayerInput> {
        self.gamepad.get(gamepad_button).copied()
    }

    pub fn keyboard(&self, key_code: &KeyCode) -> Option<PlayerInput> {
        self.keyboard.get(key_code).copied()
    }
}

fn player_input_clear(mut player_input: ResMut<Input<PlayerInput>>) {
    player_input.clear();
}

pub fn player_input_press(
    key_input: Res<Input<KeyCode>>,
    gamepad_input: Res<Input<GamepadButton>>,
    player_input_map: Res<PlayerInputMap>,
    mut player_input: ResMut<Input<PlayerInput>>,
) {
    for press in key_input.get_just_pressed() {
        if let Some(input) = player_input_map.keyboard(press) {
            player_input.press(input);
        }
    }

    for press in gamepad_input.get_just_pressed() {
        if let Some(input) = player_input_map.gamepad(&press.button_type) {
            player_input.press(input);
        }
    }
}

pub fn player_input_release(
    key_input: Res<Input<KeyCode>>,
    gamepad_input: Res<Input<GamepadButton>>,
    player_input_map: Res<PlayerInputMap>,
    mut player_input: ResMut<Input<PlayerInput>>,
) {
    for press in key_input.get_just_released() {
        if let Some(input) = player_input_map.keyboard(press) {
            player_input.release(input);
        }
    }

    for press in gamepad_input.get_just_released() {
        if let Some(input) = player_input_map.gamepad(&press.button_type) {
            player_input.release(input);
        }
    }
}
