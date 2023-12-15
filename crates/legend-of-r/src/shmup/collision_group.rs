use bevy::{
    prelude::{
        Commands, Component, CoreStage, Entity, ParallelSystemDescriptorCoercion, Plugin, Query,
        ReflectComponent,
    },
    reflect::Reflect,
};

use crate::prelude::{insert_ball_collider, insert_capsule_collider, ReflectBitflags};

pub struct CollisionGroupsPlugin;

impl Plugin for CollisionGroupsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<CollisionGroup>()
            .register_type::<CollisionGroupFlags>();

        app.add_startup_system(collision_groups)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                collision_groups
                    .after(insert_ball_collider)
                    .after(insert_capsule_collider),
            );
    }
}

// Nomenclature:
//
// Body - Physics body used for collision resolution
// Hurtbox - Receives damage from hitboxes
// Hitbox - Deals damage to hurtboxes
// Sensor - Used for intersection tests not covered by the above
bitflags::bitflags! {
    #[derive(Default, Reflect)]
    #[reflect(Bitflags)]
    pub struct CollisionGroup: u32 {
        const STATIC        = 0b00000000001;

        const SHIP_BODY     = 0b00000000010;
        const SHIP_HURTBOX  = 0b00000000100;
        const SHIP_HITBOX   = 0b00000001000;
        const SHIP_SENSOR   = 0b00000010000;

        const FORCE_BODY    = 0b00000100000;
        const FORCE_HITBOX  = 0b00001000000;
        const FORCE_SENSOR  = 0b00010000000;

        const ENEMY_BODY    = 0b00100000000;
        const ENEMY_HURTBOX = 0b01000000000;
        const ENEMY_HITBOX  = 0b10000000000;
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
#[reflect(Component)]
pub struct CollisionGroupFlags {
    pub memberships: CollisionGroup,
    pub filters: CollisionGroup,
}

impl From<CollisionGroupFlags> for bevy_rapier2d::prelude::CollisionGroups {
    fn from(groups: CollisionGroupFlags) -> Self {
        bevy_rapier2d::prelude::CollisionGroups {
            memberships: groups.memberships.bits(),
            filters: groups.filters.bits(),
        }
    }
}

pub fn collision_groups(query: Query<(Entity, &CollisionGroupFlags)>, mut commands: Commands) {
    for (entity, collision_groups) in query.iter() {
        commands
            .entity(entity)
            .remove::<CollisionGroupFlags>()
            .insert(bevy_rapier2d::prelude::CollisionGroups::from(
                *collision_groups,
            ));
    }
}
