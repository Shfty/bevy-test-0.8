use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
        system::EntityCommands,
    },
    hierarchy::BuildChildren,
    prelude::{default, Component, Entity, Gamepad, GamepadAxisType, GamepadButtonType},
    reflect::Reflect,
};

use crate::prelude::{InputFloat, InputType};

use ecs_ex::{entity_default, WithName};

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct InputReader {
    #[reflect(ignore)]
    pub gamepad: Gamepad,
    pub digital_x: Entity,
    pub digital_y: Entity,
    pub analog_x: Entity,
    pub analog_y: Entity,
    pub rotate_cw: Entity,
    pub rotate_ccw: Entity,
    pub soft_drop: Entity,
    pub hard_drop: Entity,
    pub camera_x: Entity,
    pub camera_y: Entity,
    pub camera_zoom_in: Entity,
    pub camera_zoom_out: Entity,
    pub switch_focus: Entity,
    pub switch_projection: Entity,
}

impl MapEntities for InputReader {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.digital_x = entity_map.get(self.digital_x)?;
        self.digital_y = entity_map.get(self.digital_y)?;
        self.analog_x = entity_map.get(self.analog_x)?;
        self.analog_y = entity_map.get(self.analog_y)?;
        self.rotate_cw = entity_map.get(self.rotate_cw)?;
        self.rotate_ccw = entity_map.get(self.rotate_ccw)?;
        self.soft_drop = entity_map.get(self.soft_drop)?;
        self.hard_drop = entity_map.get(self.hard_drop)?;
        self.camera_x = entity_map.get(self.camera_x)?;
        self.camera_y = entity_map.get(self.camera_y)?;
        self.camera_zoom_in = entity_map.get(self.camera_zoom_in)?;
        self.camera_zoom_out = entity_map.get(self.camera_zoom_out)?;
        self.switch_focus = entity_map.get(self.switch_focus)?;
        self.switch_projection = entity_map.get(self.switch_projection)?;
        Ok(())
    }
}

impl Default for InputReader {
    fn default() -> Self {
        Self {
            gamepad: Gamepad(0),
            digital_x: entity_default(),
            digital_y: entity_default(),
            analog_x: entity_default(),
            analog_y: entity_default(),
            soft_drop: entity_default(),
            hard_drop: entity_default(),
            camera_x: entity_default(),
            camera_y: entity_default(),
            camera_zoom_in: entity_default(),
            camera_zoom_out: entity_default(),
            rotate_cw: entity_default(),
            rotate_ccw: entity_default(),
            switch_focus: entity_default(),
            switch_projection: entity_default(),
        }
    }
}

impl InputReader {
    pub fn spawn<'w, 's, 'a>(
        &mut self,
        mut commands: EntityCommands<'w, 's, 'a>,
    ) -> EntityCommands<'w, 's, 'a> {
        commands
            .with_name("Input Reader")
            .with_children(|commands| {
                self.digital_x = commands
                    .spawn()
                    .with_name("Digital X")
                    .insert(InputFloat {
                        input: InputType::Axis(self.gamepad, GamepadAxisType::DPadX),
                        ..default()
                    })
                    .id();

                self.digital_y = commands
                    .spawn()
                    .with_name("Digital Y")
                    .insert(InputFloat {
                        input: InputType::Axis(self.gamepad, GamepadAxisType::DPadY),
                        ..default()
                    })
                    .id();

                self.analog_x = commands
                    .spawn()
                    .with_name("Analog X")
                    .insert(InputFloat {
                        input: InputType::Axis(self.gamepad, GamepadAxisType::LeftStickX),
                        ..default()
                    })
                    .id();

                self.analog_y = commands
                    .spawn()
                    .with_name("Analog Y")
                    .insert(InputFloat {
                        input: InputType::Axis(self.gamepad, GamepadAxisType::LeftStickY),
                        ..default()
                    })
                    .id();

                self.rotate_cw = commands
                    .spawn()
                    .with_name("Rotate CW")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::RightTrigger),
                        ..default()
                    })
                    .id();

                self.rotate_ccw = commands
                    .spawn()
                    .with_name("Rotate CCW")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::LeftTrigger),
                        ..default()
                    })
                    .id();

                self.soft_drop = commands
                    .spawn()
                    .with_name("Soft Drop")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::RightTrigger2),
                        ..default()
                    })
                    .id();

                self.hard_drop = commands
                    .spawn()
                    .with_name("Hard Drop")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::LeftTrigger2),
                        ..default()
                    })
                    .id();

                self.camera_x = commands
                    .spawn()
                    .with_name("Camera X")
                    .insert(InputFloat {
                        input: InputType::Axis(self.gamepad, GamepadAxisType::RightStickX),
                        ..default()
                    })
                    .id();

                self.camera_y = commands
                    .spawn()
                    .with_name("Camera Y")
                    .insert(InputFloat {
                        input: InputType::Axis(self.gamepad, GamepadAxisType::RightStickY),
                        ..default()
                    })
                    .id();

                self.camera_zoom_in = commands
                    .spawn()
                    .with_name("Camera Zoom In")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::West),
                        ..default()
                    })
                    .id();

                self.camera_zoom_out = commands
                    .spawn()
                    .with_name("Camera Zoom Out")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::South),
                        ..default()
                    })
                    .id();

                self.switch_focus = commands
                    .spawn()
                    .with_name("Camera Switch Focus")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::LeftThumb),
                        ..default()
                    })
                    .id();

                self.switch_projection = commands
                    .spawn()
                    .with_name("Camera Switch Projection")
                    .insert(InputFloat {
                        input: InputType::Button(self.gamepad, GamepadButtonType::RightThumb),
                        ..default()
                    })
                    .id();
            })
            .insert(*self);

        commands
    }
}
