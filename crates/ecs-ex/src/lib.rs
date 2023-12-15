mod foreign_component;
mod foreign_query;

pub use foreign_component::ForeignComponent;
pub use foreign_query::ForeignQuery;

use std::borrow::Cow;

use bevy::{
    core::Name,
    ecs::system::EntityCommands,
    prelude::{Commands, Component, Entity},
};

/// Helper function for creating a default invalid entity
/// Mimics how bevy handles this under the hood - likely to change at some point
pub fn entity_default() -> Entity {
    Entity::from_raw(u32::MAX)
}

/// Helper trait for spawning an entity with a single component
pub trait SpawnComponent<'w, 's, 'a, T: Component> {
    fn spawn_component(&'a mut self, component: T) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a, T: Component> SpawnComponent<'w, 's, 'a, T> for Commands<'w, 's> {
    fn spawn_component(&'a mut self, component: T) -> EntityCommands<'w, 's, 'a> {
        let mut entity = self.spawn();
        entity.insert(component);
        entity
    }
}

/// Helper trait for adding names to entities
pub trait WithName<'w, 's, 'a> {
    fn with_name(&mut self, name: impl Into<Cow<'static, str>>) -> &mut Self;
}

impl<'w, 's, 'a> WithName<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn with_name(&mut self, name: impl Into<Cow<'static, str>>) -> &mut Self {
        self.insert(Name::new(name))
    }
}
