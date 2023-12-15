//! Abstraction for removing components from the ECS and storing them for restoration

use std::fmt::Debug;

use bevy::{
    ecs::{system::Command, world::EntityMut},
    prelude::{
        default, BuildWorldChildren, Bundle, Children, Component, Deref, DerefMut, Entity, Handle,
    },
    scene::{InstanceId, Scene, SceneSpawner},
};

use crate::prelude::default_entity;

/// Type-erased handle for archiving or unarchiving components and entities
#[derive(Default, Clone, Component)]
pub struct ArchiveHandles {
    pub archive_handles: Vec<fn(&mut EntityMut)>,
    pub unarchive_handles: Vec<fn(&mut EntityMut)>,
}

#[derive(Debug, Copy, Clone, Deref, DerefMut, Component)]
pub struct ArchiveSceneInstance(pub InstanceId);

#[derive(Debug, Clone, Deref, DerefMut, Component)]
pub struct ArchiveScene(pub Handle<Scene>);

impl ArchiveHandles {
    /// Create a set of handles to move components in and out of an ArchiveStorage
    pub fn with_bundle<T>(mut self) -> Self
    where
        T: Bundle,
    {
        self.archive_handles.push(|entity| {
            ArchiveStorage::<T>::try_archive(entity);
        });

        self.unarchive_handles.push(|entity| {
            ArchiveStorage::<T>::try_unarchive(entity);
        });

        self
    }

    /// Create a set of handles to spawn and despawn a Scene
    pub fn with_scene(mut self) -> Self {
        self.archive_handles.push(|entity| {
            if let Some(instance) = entity.remove::<ArchiveSceneInstance>() {
                let children = (*entity.get::<Children>().unwrap())
                    .into_iter()
                    .copied()
                    .collect::<Vec<_>>();

                let world = unsafe { entity.world_mut() };
                let scene_spawner = world.resource_mut::<SceneSpawner>();

                for instance_entity in scene_spawner
                    .iter_instance_entities(*instance)
                    .unwrap()
                    .collect::<Vec<_>>()
                {
                    if children.contains(&&instance_entity) {
                        entity.remove_children(&[instance_entity]);
                    }
                }

                let world = unsafe { entity.world_mut() };
                let mut scene_spawner = world.resource_mut::<SceneSpawner>();
                scene_spawner.despawn_instance(*instance);
            }
        });

        self.unarchive_handles.push(|entity| {
            if entity.get::<ArchiveSceneInstance>().is_some() {
                return;
            }

            let handle = entity.get::<ArchiveScene>().unwrap().clone();
            let entity_id = entity.id();
            let world = unsafe { entity.world_mut() };
            let mut scene_spawner = world.resource_mut::<SceneSpawner>();
            let id = scene_spawner.spawn_as_child((*handle).clone(), entity_id);

            entity.insert(ArchiveSceneInstance(id));
        });

        self
    }

    pub fn archive(&self, entity: &mut EntityMut) {
        for handle in self.archive_handles.iter() {
            handle(entity)
        }
    }

    pub fn unarchive(&self, entity: &mut EntityMut) {
        for handle in self.unarchive_handles.iter() {
            handle(entity)
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct ArchiveStorage<T> {
    pub archive: Option<T>,
}

impl<T> Default for ArchiveStorage<T> {
    fn default() -> Self {
        Self { archive: default() }
    }
}

impl<T> ArchiveStorage<T>
where
    T: Bundle,
{
    pub fn try_archive(entity: &mut EntityMut) -> Option<()> {
        if let Some(bundle) = entity.remove_bundle() {
            let mut archive = entity.get_mut::<ArchiveStorage<T>>().unwrap();
            archive.archive.replace(bundle);
            Some(())
        } else {
            None
        }
    }

    pub fn archive(entity: &mut EntityMut) {
        let id = entity.id();
        Self::try_archive(entity).unwrap_or_else(|| panic!("Failed to archive entity {id:?}"))
    }

    pub fn try_unarchive(entity: &mut EntityMut) -> Option<()> {
        let mut archive = entity.get_mut::<ArchiveStorage<T>>().unwrap();
        if let Some(bundle) = archive.archive.take() {
            entity.insert_bundle(bundle);
            Some(())
        } else {
            None
        }
    }

    pub fn unarchive(entity: &mut EntityMut) {
        let id = entity.id();
        Self::try_unarchive(entity).unwrap_or_else(|| panic!("Failed to unarchive entity {id:?}"))
    }
}

/// Utility bundle for including one archivable type and a corresponding ArchiveHandles
///
/// Not suitable for entities with multiple archive storages,
/// as ArchiveHandles must hold references to all of them
#[derive(Clone, Bundle)]
pub struct ArchiveBundle<T>
where
    T: Bundle,
{
    #[bundle]
    pub bundle: T,
    pub archive_storage: ArchiveStorage<T>,
    pub archive_handles: ArchiveHandles,
}

impl<T> Default for ArchiveBundle<T>
where
    T: Default + Bundle,
{
    fn default() -> Self {
        Self {
            bundle: default(),
            archive_storage: default(),
            archive_handles: ArchiveHandles::default().with_bundle::<T>(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ArchiveCommand {
    pub target: Entity,
    pub recursive: bool,
}

impl Default for ArchiveCommand {
    fn default() -> Self {
        Self {
            target: default_entity(),
            recursive: default(),
        }
    }
}

impl Command for ArchiveCommand {
    fn write(self, world: &mut bevy::prelude::World) {
        if let Some(handles) = world.entity(self.target).get::<ArchiveHandles>().cloned() {
            handles.archive(&mut world.entity_mut(self.target));
        }

        if self.recursive {
            if let Some(children) = world.entity(self.target).get::<Children>() {
                for child in children.into_iter().copied().collect::<Vec<_>>() {
                    Self {
                        target: child,
                        recursive: true,
                    }
                    .write(world)
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Unarchive {
    pub target: Entity,
    pub recursive: bool,
}

impl Default for Unarchive {
    fn default() -> Self {
        Self {
            target: default_entity(),
            recursive: default(),
        }
    }
}

impl Command for Unarchive {
    fn write(self, world: &mut bevy::prelude::World) {
        if let Some(handles) = world.entity(self.target).get::<ArchiveHandles>().cloned() {
            handles.unarchive(&mut world.entity_mut(self.target));
        }

        if self.recursive {
            if let Some(children) = world.entity(self.target).get::<Children>() {
                for child in children.into_iter().copied().collect::<Vec<_>>() {
                    Self {
                        target: child,
                        recursive: true,
                    }
                    .write(world)
                }
            }
        }
    }
}
