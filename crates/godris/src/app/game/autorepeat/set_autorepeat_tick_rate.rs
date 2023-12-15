use bevy::{
    ecs::system::Command,
    prelude::{default, Entity},
};

use crate::prelude::DelayedAutoRepeat;

use ecs_ex::entity_default;

#[derive(Debug, Copy, Clone)]
pub struct AutorepeatSetTickRate {
    pub autorepeat: Entity,
    pub tick_rate: f32,
}

impl Default for AutorepeatSetTickRate {
    fn default() -> Self {
        Self {
            autorepeat: entity_default(),
            tick_rate: default(),
        }
    }
}

impl Command for AutorepeatSetTickRate {
    fn write(self, world: &mut bevy::prelude::World) {
        if let Some(mut autorepeat) = world
            .entity_mut(self.autorepeat)
            .get_mut::<DelayedAutoRepeat>()
        {
            autorepeat.set_tick_rate(self.tick_rate)
        }
    }
}
