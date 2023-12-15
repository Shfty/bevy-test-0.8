use bevy::{
    prelude::{Bundle, Entity, FromWorld, World},
    reflect::{FromType, Reflect},
};

#[derive(Copy, Clone)]
pub struct ReflectBundle {
    pub insert: fn(&mut World, Entity, &dyn Reflect),
    pub remove: fn(&mut World, Entity),
}

impl ReflectBundle {
    pub fn insert(&self, world: &mut World, entity: Entity, bundle: &dyn Reflect) {
        (self.insert)(world, entity, bundle)
    }

    pub fn remove(&self, world: &mut World, entity: Entity) {
        (self.remove)(world, entity)
    }
}

impl<T> FromType<T> for ReflectBundle
where
    T: Bundle + Reflect + FromWorld,
{
    fn from_type() -> Self {
        ReflectBundle {
            insert: |world, entity, reflected_bundle| {
                let mut bundle = T::from_world(world);
                bundle.apply(reflected_bundle);
                world.entity_mut(entity).insert_bundle(bundle);
            },
            remove: |world, entity| {
                world.entity_mut(entity).remove_bundle::<T>();
            },
        }
    }
}

