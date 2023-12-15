use bevy::{ecs::system::Command, prelude::Entity};

use crate::prelude::DelayedAutoRepeat;

use ecs_ex::entity_default;

#[derive(Debug, Copy, Clone)]
pub struct AutorepeatStop {
    pub autorepeat: Entity,
}

impl Default for AutorepeatStop {
    fn default() -> Self {
        Self {
            autorepeat: entity_default(),
        }
    }
}

impl Command for AutorepeatStop {
    fn write(self, world: &mut bevy::prelude::World) {
        if let Some(mut autorepeat) = world
            .entity_mut(self.autorepeat)
            .get_mut::<DelayedAutoRepeat>()
        {
            autorepeat.stop()
        }
    }
}

