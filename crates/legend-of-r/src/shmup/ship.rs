use std::f32::consts::FRAC_PI_8;

use bevy::{
    ecs::system::Command,
    prelude::{
        default, BuildWorldChildren, Bundle, Changed, Commands, Component, Entity, Name,
        ParallelSystemDescriptorCoercion, Plugin, Quat, Query, ResMut, Transform, Vec3, With,
        World,
    },
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, Collider};

use crate::{
    animation::{
        animations::discrete::{Determinisms, DiscreteStop},
        dynamic_animation::{DynamicAnimation, InsertDynamicAnimation},
    },
    body::KinematicBodyBundle,
    collision::{
        contact_depenetration::InsertContactDepenetration,
        shapecast_depenetration::InsertShapecastDepenetration,
    },
    hierarchy::HierarchyBundle,
    scene::InsertSceneArchive,
    prelude::{
        default_entity, evaluate_tagged, shift_speed_input, shift_speed_linear_factor, Alive,
        ArchiveBundle, CollisionGroup, CollisionGroupFlags, Discrete, InsertEntityPool,
        InsertTimelineAlive, InsertTimelineDamage, InsertTimelineLives, LinearMove,
        LinearMoveInput, LinearMoveInputMap, LinearMovePlugin, PlayerInput, ShiftSpeed,
        SpawnBullet, Timeline, Update, Vulcan, VulcanBundle, SCENE_PLAYER_BULLET,
    },
};

pub const SCENE_SHIP: &str = "meshes/Ship.gltf#Scene0";

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(LinearMovePlugin::new({
            LinearMoveInputMap {
                left: PlayerInput::Left.into(),
                right: PlayerInput::Right.into(),
                up: PlayerInput::Up.into(),
                down: PlayerInput::Down.into(),
            }
        }));

        app.add_system(shift_speed_linear_factor::<PlayerInput>.after(shift_speed_input))
            .add_system(ship_rotation.before(evaluate_tagged::<Update>))
            .add_system(ship_alive);
    }
}

/// Ship core component, holds references to related entities
#[derive(Debug, Clone, Component)]
pub struct Ship {
    pub timeline: Entity,
    pub rotation_from_stops: Entity,
    pub rotation_to_stops: Entity,
}

impl Default for Ship {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            rotation_from_stops: default_entity(),
            rotation_to_stops: default_entity(),
        }
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    name: Name,
    #[bundle]
    hierarchy: HierarchyBundle,
    ship: Ship,
    speed: ShiftSpeed,
    #[bundle]
    body: KinematicBodyBundle,
    #[bundle]
    archive: ArchiveBundle<ShipArchiveBundle>,
}

impl Default for ShipBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Ship"),
            ship: default(),
            speed: default(),
            hierarchy: HierarchyBundle {
                transform: Transform::from_xyz(-5.0, 0.0, 0.0).into(),
                ..default()
            },
            body: KinematicBodyBundle {
                collision_groups: CollisionGroupFlags {
                    memberships: CollisionGroup::SHIP_BODY,
                    filters: CollisionGroup::STATIC,
                },
                active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC
                    | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                ..default()
            },
            archive: default(),
        }
    }
}

#[derive(Bundle)]
pub struct ShipArchiveBundle {
    linear_move: LinearMove,
    linear_move_input: LinearMoveInput<PlayerInput>,
    #[bundle]
    vulcan: VulcanBundle,
    collider: Collider,
}

impl Default for ShipArchiveBundle {
    fn default() -> Self {
        Self {
            linear_move: default(),
            linear_move_input: LinearMoveInput {
                factor: 4.0,
                left: Some(PlayerInput::Left),
                right: Some(PlayerInput::Right),
                up: Some(PlayerInput::Up),
                down: Some(PlayerInput::Down),
                ..default()
            },
            vulcan: VulcanBundle {
                vulcan: Vulcan {
                    emitters: vec![Transform::from_xyz(0.75, 0.0, 0.0)],
                    ..default()
                },
                ..default()
            },
            collider: Collider::capsule_x(0.5, 0.6),
        }
    }
}

pub struct AssembleShip<C>
where
    C: Clone + FnOnce(&mut World, Entity, Entity),
{
    pub timeline: Entity,
    pub playfield: Entity,
    pub entity: Entity,
    pub ship_bundle: ShipBundle,
    pub scene: InsertSceneArchive,
    pub insert_timeline_damage: InsertTimelineDamage,
    pub insert_timeline_alive: InsertTimelineAlive,
    pub insert_timeline_lives: InsertTimelineLives,
    pub insert_entity_pool: InsertEntityPool<C>,
}

impl Default for AssembleShip<fn(&mut World, Entity, Entity)> {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            playfield: default_entity(),
            entity: default_entity(),
            ship_bundle: default(),
            scene: InsertSceneArchive {
                path: SCENE_SHIP.into(),
                ..default()
            },
            insert_timeline_damage: default(),
            insert_timeline_alive: InsertTimelineAlive {
                alive_at: Some(0.0),
                ..default()
            },
            insert_timeline_lives: default(),
            insert_entity_pool: InsertEntityPool {
                construct_instances: Some((3, |world, entity, timeline| {
                    SpawnBullet {
                        entity,
                        timeline,
                        scene: InsertSceneArchive {
                            path: SCENE_PLAYER_BULLET.into(),
                            ..default()
                        },
                        ..default()
                    }
                    .write(world)
                })),
                ..default()
            },
        }
    }
}

impl<C> Command for AssembleShip<C>
where
    C: 'static + Send + Sync + Clone + FnOnce(&mut World, Entity, Entity),
{
    fn write(self, world: &mut bevy::prelude::World) {
        // Scene
        let scene_entity = world.spawn().id();
        let mut scene = self.scene;
        scene.entity = scene_entity;
        scene.write(world);

        // Main ship entity
        let mut ship_bundle = self.ship_bundle;
        ship_bundle.archive.bundle.vulcan.vulcan.timeline = self.timeline;

        world
            .entity_mut(self.entity)
            .insert_bundle(ship_bundle)
            .push_children(&[scene_entity]);

        let mut timeline_damage = self.insert_timeline_damage;
        timeline_damage.timeline = self.timeline;
        timeline_damage.entity = self.entity;
        timeline_damage.write(world);

        let mut timeline_alive = self.insert_timeline_alive;
        timeline_alive.timeline = self.timeline;
        timeline_alive.entity = self.entity;
        timeline_alive.write(world);

        let mut timeline_lives = self.insert_timeline_lives;
        timeline_lives.timeline = self.timeline;
        timeline_lives.entity = self.entity;
        timeline_lives.write(world);

        InsertDynamicAnimation::<Transform, Quat> {
            timeline: self.timeline,
            entity: self.entity,
            target: scene_entity,
            accessor: |transform| &transform.rotation,
            mutator: |transform| &mut transform.rotation,
        }
        .write(world);

        InsertContactDepenetration {
            entity: self.entity,
        }
        .write(world);

        InsertShapecastDepenetration {
            entity: self.entity,
        }
        .write(world);

        // Add as child of playfield
        world
            .entity_mut(self.playfield)
            .push_children(&[self.entity]);

        // Spawn vulcan bullets
        let mut insert_bullet_pool = self.insert_entity_pool;
        insert_bullet_pool.entity = self.entity;
        insert_bullet_pool.timeline = self.timeline;
        insert_bullet_pool.write(world);
    }
}

pub fn ship_rotation(
    mut determinisms: ResMut<Determinisms>,
    query_ship: Query<
        (&DynamicAnimation<Transform, Quat>, &LinearMove),
        (With<Ship>, Changed<LinearMove>),
    >,
    query_timeline: Query<&Timeline>,
    mut query_rotation_stops: Query<&mut Discrete<DiscreteStop<Quat>>>,
) {
    for (rotation_animation, linear_move) in query_ship.iter() {
        let timeline = query_timeline.get(rotation_animation.timeline).unwrap();

        let mesh_rotation = query_rotation_stops
            .get_mut(rotation_animation.to_stops)
            .unwrap()
            .stops()
            .last()
            .unwrap()
            .value
            .value;

        let mut rotation_from_stops = if let Ok(rotation_stops) =
            query_rotation_stops.get_mut(rotation_animation.from_stops)
        {
            rotation_stops
        } else {
            continue;
        };

        let t = timeline.t;

        let determinism = determinisms.next();

        rotation_from_stops.insert_stop(DiscreteStop {
            t,
            value: DiscreteStop {
                t,
                value: mesh_rotation,
                ..default()
            },
            determinism,
            ..default()
        });

        let mut rotation_to_stops =
            if let Ok(rotation_stops) = query_rotation_stops.get_mut(rotation_animation.to_stops) {
                rotation_stops
            } else {
                continue;
            };

        rotation_to_stops.insert_stop(DiscreteStop {
            t,
            value: DiscreteStop {
                t: t + 0.1,
                value: Quat::from_axis_angle(Vec3::Z, linear_move.delta.y * 0.1 as f32 * FRAC_PI_8),
                ..default()
            },
            determinism,
            ..default()
        });
    }
}

// TODO: Remove this in favour of transform animation timeline updates
pub fn ship_alive(
    query_ship: Query<(Entity, &Alive), (With<Ship>, Changed<Alive>)>,
    mut commands: Commands,
) {
    for (ship_entity, alive) in query_ship.iter() {
        if !**alive {
            commands.entity(ship_entity).insert(Transform::default());
        }
    }
}
