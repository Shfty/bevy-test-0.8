use crate::{
    scene::InsertSceneArchive,
    prelude::{default_entity, InsertTimelineAlive, InsertTimelineDamage},
};
use bevy::{
    ecs::system::Command,
    prelude::{default, BuildWorldChildren, Entity},
};

pub const SCENE_ENEMY: &str = "meshes/Enemy.gltf#Scene0";

pub struct InsertEnemy {
    pub playfield: Entity,
    pub timeline: Entity,
    pub entity: Entity,
    pub scene: InsertSceneArchive,
    pub timeline_damage: InsertTimelineDamage,
    pub timeline_alive: InsertTimelineAlive,
}

impl Default for InsertEnemy {
    fn default() -> Self {
        Self {
            playfield: default_entity(),
            timeline: default_entity(),
            entity: default_entity(),
            scene: InsertSceneArchive {
                path: SCENE_ENEMY.into(),
                ..default()
            },
            timeline_damage: default(),
            timeline_alive: default(),
        }
    }
}

impl Command for InsertEnemy {
    fn write(self, world: &mut bevy::prelude::World) {
        let mut scene = self.scene;
        scene.entity = self.entity;
        scene.write(world);

        world
            .entity_mut(self.playfield)
            .push_children(&[self.entity]);

        let mut timeline_damage = self.timeline_damage;
        timeline_damage.timeline = self.timeline;
        timeline_damage.entity = self.entity;
        timeline_damage.write(world);

        let mut timeline_alive = self.timeline_alive;
        timeline_alive.timeline = self.timeline;
        timeline_alive.entity = self.entity;
        timeline_alive.write(world);
    }
}
