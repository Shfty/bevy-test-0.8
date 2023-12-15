use bevy::{
    ecs::system::Command,
    prelude::{default, AssetServer, Entity, Handle, Name},
    scene::{Scene, SceneBundle},
};

use crate::{
    hierarchy::HierarchyBundle,
    prelude::{ArchiveHandles, ArchiveScene},
    util::default_entity,
};

#[derive(Debug, Clone)]
pub struct InsertScene {
    pub entity: Entity,
    pub path: String,
}

impl Default for InsertScene {
    fn default() -> Self {
        Self {
            entity: default_entity(),
            path: default(),
        }
    }
}

impl Command for InsertScene {
    fn write(self, world: &mut bevy::prelude::World) {
        let asset_server = world.resource::<AssetServer>();
        let scene: Handle<Scene> = asset_server.load(&self.path);
        world
            .entity_mut(self.entity)
            .insert_bundle(SceneBundle { scene, ..default() })
            .insert(Name::new("Scene Instance"));
    }
}

#[derive(Debug, Clone)]
pub struct InsertSceneArchive {
    pub entity: Entity,
    pub path: String,
}

impl Default for InsertSceneArchive {
    fn default() -> Self {
        Self {
            entity: default_entity(),
            path: default(),
        }
    }
}

impl Command for InsertSceneArchive {
    fn write(self, world: &mut bevy::prelude::World) {
        let asset_server = world.resource::<AssetServer>();
        let scene: Handle<Scene> = asset_server.load(&self.path);
        world
            .entity_mut(self.entity)
            .insert(ArchiveScene(scene))
            .insert_bundle(HierarchyBundle::default())
            .insert(ArchiveHandles::default().with_scene())
            .insert(Name::new("Scene Instance"));
    }
}
