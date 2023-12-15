use bevy::prelude::{Commands, Entity, World};

pub trait Spawn {
    fn spawn(&mut self) -> Entity;
}

impl Spawn for World {
    fn spawn(&mut self) -> Entity {
        self.spawn().id()
    }
}

impl Spawn for Commands<'_, '_> {
    fn spawn(&mut self) -> Entity {
        self.spawn().id()
    }
}

pub trait FromSpawn {
    fn from_spawn(spawn: &mut impl Spawn) -> Self;
}
