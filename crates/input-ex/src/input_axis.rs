use bevy::{
    math::{Vec2, Vec3},
    prelude::{
        default, Component, EventReader, Gamepad, GamepadAxisType, GamepadEvent, GamepadEventType,
        Query,
    },
};

#[derive(Debug, Copy, Clone, Component)]
pub struct InputAxis {
    pub gamepad: Gamepad,
    pub axis: GamepadAxisType,
    pub value: f32,
    pub multiplier: f32,
}

impl Default for InputAxis {
    fn default() -> Self {
        Self {
            gamepad: Gamepad(0),
            axis: GamepadAxisType::LeftStickX,
            value: default(),
            multiplier: 1.0,
        }
    }
}

fn input_axis_impl(event: &GamepadEvent, input: &mut InputAxis) {
    let event_type = match event {
        GamepadEvent(gamepad, event_type) if *gamepad == input.gamepad => event_type,
        _ => return,
    };

    let value = match event_type {
        GamepadEventType::AxisChanged(axis_type, value) if *axis_type == input.axis => value,
        _ => return,
    };

    input.value = *value * input.multiplier;
}

pub fn input_axis(mut events: EventReader<GamepadEvent>, mut query: Query<&mut InputAxis>) {
    for event in events.iter() {
        for mut input in query.iter_mut() {
            input_axis_impl(&event, &mut input);
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct InputAxis2 {
    pub x: Option<InputAxis>,
    pub y: Option<InputAxis>,
}

impl InputAxis2 {
    pub fn value(&self) -> Vec2 {
        Vec2::new(
            self.x.map(|axis| axis.value).unwrap_or_default(),
            self.y.map(|axis| axis.value).unwrap_or_default(),
        )
    }
}

pub fn input_axis_2(mut events: EventReader<GamepadEvent>, mut query: Query<&mut InputAxis2>) {
    for event in events.iter() {
        for mut input in query.iter_mut() {
            if let Some(ref mut x) = input.x {
                input_axis_impl(&event, x);
            }
            if let Some(ref mut y) = input.y {
                input_axis_impl(&event, y);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct InputAxis3 {
    pub x: Option<InputAxis>,
    pub y: Option<InputAxis>,
    pub z: Option<InputAxis>,
}

impl InputAxis3 {
    pub fn value(&self) -> Vec3 {
        Vec3::new(
            self.x.map(|axis| axis.value).unwrap_or_default(),
            self.y.map(|axis| axis.value).unwrap_or_default(),
            self.z.map(|axis| axis.value).unwrap_or_default(),
        )
    }
}

pub fn input_axis_3(mut events: EventReader<GamepadEvent>, mut query: Query<&mut InputAxis3>) {
    for event in events.iter() {
        for mut input in query.iter_mut() {
            if let Some(ref mut x) = input.x {
                input_axis_impl(&event, x);
            }
            if let Some(ref mut y) = input.y {
                input_axis_impl(&event, y);
            }
            if let Some(ref mut z) = input.z {
                input_axis_impl(&event, z);
            }
        }
    }
}
