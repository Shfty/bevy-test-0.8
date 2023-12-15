pub mod component_bundle;
pub mod delayed_input;
pub mod depth_material;
pub mod reciprocal;
pub mod sign;

use std::sync::Arc;

use bevy::prelude::{Entity, StandardMaterial};
use parking_lot::Mutex;

pub type SharedMut<T> = Arc<Mutex<T>>;

/// Utility function for starting combinator chains
pub const fn unit<T>(t: T) -> T {
    t
}

/// Free construtor for SharedMut<T>
pub fn shared_mut<T>(t: T) -> SharedMut<T> {
    Arc::new(Mutex::new(t))
}

/// Utility function for creating an entity inside [`Default`] impls
pub fn default_entity() -> Entity {
    Entity::from_raw(u32::MAX)
}

pub type GameMaterial = StandardMaterial;


/*
#[derive(Clone)]
pub struct CloneComponent<T> {
    pub from: Entity,
    pub to: Entity,
    pub prev_value: T,
    pub is_forward: bool,
}

impl<T> CommandMut for CloneComponent<T>
where
    T: Clone + Component,
{
    fn write_mut(&mut self, world: &mut World) {
        let value = if self.is_forward {
            let value = world.entity(self.from).get::<T>().unwrap().clone();
            self.prev_value = value.clone();
            value
        } else {
            self.prev_value.clone()
        };

        world.entity_mut(self.to).insert(value);
    }
}

impl<T> Reciprocal for CloneComponent<T>
where
    T: Clone,
{
    type Reciprocal = Self;

    fn reciprocal(&self) -> Self::Reciprocal {
        Self {
            is_forward: !self.is_forward,
            ..self.clone()
        }
    }
}
*/
