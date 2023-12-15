use bevy::prelude::{
    default, Bundle, Commands, Component, Entity, Query, Reflect, ReflectComponent, ReflectDefault,
    With,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, RigidBody};

use crate::prelude::{CollisionGroupFlags, ReflectBundle};

#[derive(Debug, Clone, Bundle, Reflect)]
#[reflect(Default, Bundle)]
pub struct KinematicBodyBundle {
    pub rigid_body: RigidBody,
    pub collision_groups: CollisionGroupFlags,
    pub active_collision_types: ActiveCollisionTypes,
}

impl Default for KinematicBodyBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::KinematicPositionBased,
            collision_groups: default(),
            active_collision_types: ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct InsertKinematicRigidBody;

pub fn insert_kinematic_rigid_body(
    query: Query<Entity, With<InsertKinematicRigidBody>>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        let mut entity = commands.entity(entity);
        entity.remove::<InsertKinematicRigidBody>();
        entity.insert(RigidBody::KinematicPositionBased);
    }
}
