use bevy::prelude::{
    default, Bundle, Commands, Component, Entity, Query, Reflect, ReflectComponent, ReflectDefault,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, Collider, Sensor};

use crate::prelude::{ArchiveBundle, CollisionGroupFlags, ReflectBundle};

#[derive(Clone, Bundle, Reflect)]
#[reflect(Default, Bundle)]
pub struct SensorBundle {
    pub active_collision_types: ActiveCollisionTypes,
    pub collision_groups: CollisionGroupFlags,
    pub sensor: Sensor,
}

impl Default for SensorBundle {
    fn default() -> Self {
        Self {
            active_collision_types: ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            collision_groups: default(),
            sensor: default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Default, Component)]
pub struct InsertBallCollider {
    radius: f32,
    archivable: bool,
}

impl Default for InsertBallCollider {
    fn default() -> Self {
        Self {
            radius: 1.0,
            archivable: default(),
        }
    }
}

pub fn insert_ball_collider(query: Query<(Entity, &InsertBallCollider)>, mut commands: Commands) {
    for (entity, insert_ball_collider) in query.iter() {
        let mut entity = commands.entity(entity);
        entity.remove::<InsertBallCollider>();

        if insert_ball_collider.archivable {
            entity.insert_bundle(ArchiveBundle {
                bundle: (Collider::ball(insert_ball_collider.radius),),
                ..default()
            });
        } else {
            entity.insert(Collider::ball(insert_ball_collider.radius));
        }
    }
}

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Default, Component)]
pub struct InsertCapsuleCollider {
    pub half_height: f32,
    pub radius: f32,
    pub archivable: bool,
}

impl Default for InsertCapsuleCollider {
    fn default() -> Self {
        Self {
            half_height: 1.0,
            radius: 1.0,
            archivable: default(),
        }
    }
}

pub fn insert_capsule_collider(
    query: Query<(Entity, &InsertCapsuleCollider)>,
    mut commands: Commands,
) {
    for (entity, insert_capsule_collider) in query.iter() {
        let mut entity = commands.entity(entity);
        entity.remove::<InsertCapsuleCollider>();

        let collider = Collider::capsule_x(
            insert_capsule_collider.half_height,
            insert_capsule_collider.radius,
        );
        if insert_capsule_collider.archivable {
            entity.insert_bundle(ArchiveBundle {
                bundle: (collider,),
                ..default()
            });
        } else {
            entity.insert(collider);
        }
    }
}
