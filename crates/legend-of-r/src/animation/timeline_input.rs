use bevy::{
    input::{gamepad::gamepad_event_system, keyboard::keyboard_input_system},
    prelude::{
        CoreStage, GamepadButton, GamepadButtonType, Input, KeyCode,
        ParallelSystemDescriptorCoercion, Plugin, Query, Res, ResMut, SystemLabel, SystemSet,
    },
    reflect::Reflect,
};

use crate::prelude::Timeline;

#[derive(SystemLabel)]
pub struct TimelineInputLabel;

pub struct TimelineInputPlugin;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub enum TimelineInput {
    TogglePause,
    Rewind,
}

impl TryFrom<GamepadButton> for TimelineInput {
    type Error = ();

    fn try_from(value: GamepadButton) -> Result<Self, Self::Error> {
        Ok(match value.button_type {
            GamepadButtonType::Start => TimelineInput::TogglePause,
            GamepadButtonType::Select => TimelineInput::Rewind,
            _ => return Err(()),
        })
    }
}

impl TryFrom<KeyCode> for TimelineInput {
    type Error = ();

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        Ok(match value {
            KeyCode::Space => TimelineInput::TogglePause,
            KeyCode::Back => TimelineInput::Rewind,
            _ => return Err(()),
        })
    }
}

impl Plugin for TimelineInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Input::<TimelineInput>::default());

        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::default()
                .with_system(
                    timeline_input_clear
                        .after(keyboard_input_system)
                        .after(gamepad_event_system)
                        .before(timeline_input_press)
                        .before(timeline_input_release),
                )
                .with_system(timeline_input_press)
                .with_system(timeline_input_release)
                .with_system(
                    timeline_input
                        .after(timeline_input_press)
                        .after(timeline_input_release),
                )
                .label(TimelineInputLabel),
        );
    }
}

fn timeline_input_clear(mut timeline_input: ResMut<Input<TimelineInput>>) {
    timeline_input.clear();
}

pub fn timeline_input_press(
    key_input: Res<Input<KeyCode>>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut timeline_input: ResMut<Input<TimelineInput>>,
) {
    for press in key_input.get_just_pressed() {
        if let Ok(input) = TimelineInput::try_from(*press) {
            timeline_input.press(input);
        }
    }

    for press in gamepad_input.get_just_pressed() {
        if let Ok(input) = TimelineInput::try_from(*press) {
            timeline_input.press(input);
        }
    }
}

pub fn timeline_input_release(
    key_input: Res<Input<KeyCode>>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut timeline_input: ResMut<Input<TimelineInput>>,
) {
    for press in key_input.get_just_released() {
        if let Ok(input) = TimelineInput::try_from(*press) {
            timeline_input.release(input);
        }
    }

    for press in gamepad_input.get_just_released() {
        if let Ok(input) = TimelineInput::try_from(*press) {
            timeline_input.release(input);
        }
    }
}

pub fn timeline_input(timeline_input: Res<Input<TimelineInput>>, mut query: Query<&mut Timeline>) {
    if timeline_input.just_pressed(TimelineInput::TogglePause) {
        for mut timeline in query.iter_mut() {
            timeline.paused = !timeline.paused;
        }
    }

    if timeline_input.just_pressed(TimelineInput::Rewind) {
        for mut timeline in query.iter_mut() {
            timeline.t = 0.0;
        }
    }
}
