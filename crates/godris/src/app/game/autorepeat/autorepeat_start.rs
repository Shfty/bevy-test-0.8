use bevy::{ecs::system::Command, prelude::Entity};

use crate::prelude::DelayedAutoRepeat;

use ecs_ex::entity_default;

#[derive(Debug, Copy, Clone)]
pub struct AutorepeatStart {
    pub autorepeat: Entity,
}

impl Default for AutorepeatStart {
    fn default() -> Self {
        Self {
            autorepeat: entity_default(),
        }
    }
}

impl Command for AutorepeatStart {
    fn write(self, world: &mut bevy::prelude::World) {
        if let Some(mut autorepeat) = world
            .entity_mut(self.autorepeat)
            .get_mut::<DelayedAutoRepeat>()
        {
            autorepeat.start()
        }
    }
}

