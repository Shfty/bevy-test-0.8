use std::f32::consts::FRAC_PI_2;

use bevy::{
    ecs::{reflect::ReflectComponent, system::Command},
    prelude::{
        default, info, BuildWorldChildren, Bundle, Component, CoreStage, Entity, EventReader,
        EventWriter, Input, Name, ParallelSystemDescriptorCoercion, Parent, Plugin, Quat, Query,
        Res, ResMut, SystemSet, Transform, Vec3, With, Without, World,
    },
    reflect::Reflect,
};
use bevy_rapier2d::{
    na::Vector2,
    prelude::{ActiveCollisionTypes, Collider, PhysicsStages, RapierContext},
};

use crate::{
    animation::animations::discrete::{DeterminismId, Determinisms, DiscreteStop},
    body::KinematicBodyBundle,
    collision::{
        contact_depenetration::InsertContactDepenetration,
        shapecast_depenetration::InsertShapecastDepenetration,
    },
    hierarchy::HierarchyBundle,
    scene::{InsertScene, InsertSceneArchive},
    prelude::{
        contact_depenetration, default_entity, linear_move_integrate, linear_move_to, lives_death,
        player_input_press, player_input_release, shapecast_depenetration, ship_alive,
        timeline_damage, Alive, AnimateComponentFieldsTrait, AnimationTagTrait, ArchiveBundle,
        AspectRatio, BuildAnimation, CameraPivotSource, CollisionGroup, CollisionGroupFlags,
        ComponentBundle, ContactDepenetration, DeathEvent, Discrete, DiscreteStopsTrait,
        EvaluateTrait, HitboxBundle, InsertEntityPool, InterpolateTrait, LinearMoveTo,
        LinearMoveToPlugin, PlayerInput, Playfield, PlayfieldRotationTargetBundle, RespawnEvent,
        SensorBundle, ShapecastDepenetration, SpawnBullet, TimeSourceTrait, Timeline,
        TimelineDamage, TimelineTime, TryAnimateComponentsTrait, TryReplaceComponentsTrait, Update,
        Vulcan, VulcanTimer, SCENE_PLAYER_BULLET,
    },
};
pub const SCENE_FORCE: &str = "meshes/Force.gltf#Scene0";

pub struct ForcePlugin;

impl Plugin for ForcePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Force>().register_type::<ForceState>();

        app.add_plugin(LinearMoveToPlugin);

        app.add_event::<ForceStateEvent>()
            .add_event::<ForceInputEvent>()
            .add_event::<ForceVulcanEvent>();

        app.add_system_to_stage(
            CoreStage::PreUpdate,
            force_input
                .after(player_input_press)
                .after(player_input_release),
        )
        .add_system(
            force_dock_transform
                .after(linear_move_integrate)
                .after(ship_alive),
        )
        .add_system(force_target.before(linear_move_to))
        .add_system_set_to_stage(
            PhysicsStages::StepSimulation,
            SystemSet::default()
                .with_system(force_contact_depenetration.before(contact_depenetration))
                .with_system(force_ship_contact_depenetration.before(contact_depenetration))
                .with_system(force_shapecast_depenetration.before(shapecast_depenetration))
                .with_system(force_ship_shapecast_depenetration.before(shapecast_depenetration)),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            force_ship_death.after(timeline_damage),
        )
        .add_system_to_stage(CoreStage::PostUpdate, force_ship_respawn.after(lives_death))
        .add_system_to_stage(CoreStage::PostUpdate, force_body_collision)
        .add_system_to_stage(CoreStage::PostUpdate, force_sensor_collision)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            force_state_event
                .after(force_ship_death)
                .after(force_ship_respawn)
                .after(force_body_collision),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            force_input_event.after(force_state_event),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            force_vulcan_event.after(force_state_event),
        );
    }
}

#[derive(Bundle)]
pub struct ForceBundle {
    pub name: Name,
    pub force: Force,
    #[bundle]
    pub hierarchy: HierarchyBundle,
    #[bundle]
    pub body: KinematicBodyBundle,
    pub collider: Collider,
    pub linear_move_to: LinearMoveTo,
    pub vulcan: Vulcan,
}

impl Default for ForceBundle {
    fn default() -> Self {
        let edge = Transform::from_translation(Vec3::X * 0.5);

        Self {
            name: Name::new("Force"),
            force: default(),
            hierarchy: HierarchyBundle {
                transform: Transform::from_xyz(5.0, 0.0, 0.0).into(),
                ..default()
            },
            body: KinematicBodyBundle {
                active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
                collision_groups: CollisionGroupFlags {
                    memberships: CollisionGroup::FORCE_BODY,
                    filters: CollisionGroup::STATIC | CollisionGroup::SHIP_BODY,
                },
                ..default()
            },
            collider: Collider::ball(0.75),
            linear_move_to: default(),
            vulcan: Vulcan {
                timeline: default_entity(),
                emitters: vec![
                    Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, -FRAC_PI_2)) * edge,
                    Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, -0.3)) * edge,
                    Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, 0.3)) * edge,
                    Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2)) * edge,
                ],
            },
        }
    }
}

#[derive(Bundle)]
pub struct ForceHitboxBundle {
    name: Name,
    #[bundle]
    pub hierarchy_bundle: HierarchyBundle,
    #[bundle]
    pub hitbox_bundle: HitboxBundle,
    #[bundle]
    pub collider_bundle: ArchiveBundle<ComponentBundle<Collider>>,
}

impl Default for ForceHitboxBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Force Hitbox"),
            hierarchy_bundle: default(),
            hitbox_bundle: HitboxBundle {
                sensor_bundle: SensorBundle {
                    collision_groups: CollisionGroupFlags {
                        memberships: CollisionGroup::FORCE_HITBOX,
                        filters: CollisionGroup::ENEMY_HURTBOX,
                    },
                    ..default()
                },
                ..default()
            },
            collider_bundle: ArchiveBundle {
                bundle: ComponentBundle {
                    component: Collider::ball(0.75),
                },
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct ForceSensorBundle {
    name: Name,
    #[bundle]
    pub hierarchy_bundle: HierarchyBundle,
    #[bundle]
    pub sensor_bundle: SensorBundle,
    #[bundle]
    pub collider_bundle: ArchiveBundle<ComponentBundle<Collider>>,
}

impl Default for ForceSensorBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Force Sensor"),
            hierarchy_bundle: default(),
            sensor_bundle: SensorBundle {
                collision_groups: CollisionGroupFlags {
                    memberships: CollisionGroup::FORCE_SENSOR,
                    filters: CollisionGroup::SHIP_SENSOR,
                },
                ..default()
            },
            collider_bundle: ArchiveBundle {
                bundle: ComponentBundle {
                    component: Collider::ball(0.75),
                },
                ..default()
            },
        }
    }
}

pub fn force_vulcan(timeline: Entity) -> Vulcan {
    let edge = Transform::from_xyz(0.5, 0.0, 0.0);
    Vulcan {
        timeline,
        emitters: vec![
            Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, -FRAC_PI_2)) * edge,
            Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, -0.3)) * edge,
            Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, 0.3)) * edge,
            Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2)) * edge,
        ],
        ..default()
    }
}

pub struct AssembleForce<C>
where
    C: Clone + FnOnce(&mut World, Entity, Entity),
{
    pub force: Entity,
    pub ship: Entity,
    pub playfield: Entity,
    pub timeline: Entity,
    pub force_bundle: ForceBundle,
    pub scene: InsertScene,
    pub force_visuals_bundle: HierarchyBundle,
    pub force_visuals_rotation_bundle: PlayfieldRotationTargetBundle,
    pub force_hitbox_bundle: ForceHitboxBundle,
    pub force_sensor_bundle: ForceSensorBundle,
    pub insert_bullet_pool: InsertEntityPool<C>,
}

impl Default for AssembleForce<fn(&mut World, Entity, Entity)> {
    fn default() -> Self {
        Self {
            force: default_entity(),
            ship: default_entity(),
            playfield: default_entity(),
            timeline: default_entity(),
            force_visuals_bundle: default(),
            force_visuals_rotation_bundle: default(),
            force_hitbox_bundle: default(),
            force_sensor_bundle: default(),
            force_bundle: default(),
            scene: InsertScene {
                path: SCENE_FORCE.into(),
                ..default()
            },
            insert_bullet_pool: InsertEntityPool {
                construct_instances: Some((16, |world, entity, timeline| {
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

impl<C> Command for AssembleForce<C>
where
    C: 'static + Send + Sync + Clone + FnOnce(&mut World, Entity, Entity),
{
    fn write(self, world: &mut World) {
        // Mesh
        let force_mesh = world.spawn().id();

        let mut scene = self.scene;
        scene.entity = force_mesh;
        scene.write(world);

        // Force visuals
        let force_visuals = world.spawn().id();

        // Rotation animation
        let rotation_animation = world
            .build_animation()
            .from_discrete_stops([
                DiscreteStop {
                    t: 0.0,
                    value: Quat::default(),
                    ..default()
                },
                DiscreteStop {
                    t: 2.0,
                    value: Quat::from_scaled_axis(Vec3::Z * -std::f32::consts::PI),
                    ..default()
                },
            ])
            .interpolate()
            .animating_component_field::<Transform>(
                force_visuals,
                |transform| &transform.rotation,
                |transform| &mut transform.rotation,
            )
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new(format!("Force Rotation")))
            .id();

        world
            .entity_mut(force_visuals)
            .insert_bundle(self.force_visuals_bundle)
            .insert(Name::new("Force Visuals"))
            .push_children(&[force_mesh, rotation_animation]);

        // Playfield rotation
        let force_visuals_rotation = world
            .spawn()
            .insert_bundle(self.force_visuals_rotation_bundle)
            .push_children(&[force_visuals])
            .id();

        // Hitbox
        let force_hitbox = world.spawn().insert_bundle(self.force_hitbox_bundle).id();

        // Dockbox
        let force_sensor = world.spawn().insert_bundle(self.force_sensor_bundle).id();

        // State animation
        let state_animation = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: ForceState::default(),
            ..default()
        }]);

        let state_stops = state_animation.id();

        let state_animation = state_animation
            .try_animating_component(self.force)
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new(format!("Force State")))
            .id();

        // Input animation
        let input_animation = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: Some(ForceInput),
            ..default()
        }]);

        let input_stops = input_animation.id();

        let input_animation = input_animation
            .try_replacing_component(self.force)
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new(format!("Force Input")))
            .id();

        // Refire animation
        let refire_animation = world.build_animation().from_discrete_stops([DiscreteStop {
            t: 0.0,
            value: Some(VulcanTimer::default()),
            ..default()
        }]);

        let refire_stops = refire_animation.id();

        let refire_animation = refire_animation
            .try_replacing_component(self.force)
            .with_time_source(TimelineTime {
                timeline: self.timeline,
            })
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new(format!("Force Refire")))
            .id();

        // Main force entity
        let mut force_bundle = self.force_bundle;
        force_bundle.force.timeline = self.timeline;
        force_bundle.force.ship = self.ship;
        force_bundle.force.sensor = force_sensor;
        force_bundle.force.state_stops = state_stops;
        force_bundle.force.input_stops = input_stops;
        force_bundle.force.refire_stops = refire_stops;
        force_bundle.vulcan.timeline = self.timeline;

        world
            .entity_mut(self.force)
            .insert_bundle(force_bundle)
            .push_children(&[
                force_visuals_rotation,
                force_hitbox,
                force_sensor,
                state_animation,
                input_animation,
                refire_animation,
            ]);

        InsertContactDepenetration { entity: self.force }.write(world);

        InsertShapecastDepenetration { entity: self.force }.write(world);

        // Insert ShipForce adapter on ship entity
        world
            .entity_mut(self.ship)
            .insert(ShipForce { force: self.force });

        // Attach force to playfield
        world
            .entity_mut(self.playfield)
            .push_children(&[self.force]);

        // Spawn vulcan bullets for force entity
        let mut insert_bullet_pool = self.insert_bullet_pool;
        insert_bullet_pool.entity = self.force;
        insert_bullet_pool.timeline = self.timeline;
        insert_bullet_pool.write(world);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
#[reflect(Component)]
pub enum ForceState {
    Free,
    Recall,
    DockedFront,
    DockedRear,
    LaunchForward,
    LaunchBackward,
}

impl Default for ForceState {
    fn default() -> Self {
        ForceState::Free
    }
}

/// Core force entity, holds fundamental properties and entity references
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Force {
    pub timeline: Entity,
    pub ship: Entity,
    pub sensor: Entity,
    pub state_stops: Entity,
    pub input_stops: Entity,
    pub refire_stops: Entity,
    pub docking_distance: f32,
    pub speed_free: f32,
    pub speed_launch: f32,
    pub speed_docking: f32,
    pub speed_recall: f32,
}

impl Default for Force {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            ship: default_entity(),
            sensor: default_entity(),
            state_stops: default_entity(),
            input_stops: default_entity(),
            refire_stops: default_entity(),
            docking_distance: 3.0,
            speed_free: 4.0,
            speed_launch: 32.0,
            speed_docking: 4.0,
            speed_recall: 8.0,
        }
    }
}

/// Component for adding force compatibility to a ship
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct ShipForce {
    pub force: Entity,
}

impl Default for ShipForce {
    fn default() -> Self {
        Self {
            force: default_entity(),
        }
    }
}

/// Marker trait indicating a force should receive input,
#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ForceInput;

pub fn force_target(
    aspect: Res<AspectRatio>,
    query_playfield: Query<&Playfield>,
    query_camera_pivot: Query<&Transform, With<CameraPivotSource>>,
    query_ship: Query<&Transform, With<ShipForce>>,
    mut query_force: Query<(&Transform, &Force, &ForceState, &mut LinearMoveTo)>,
) {
    let playfield = query_playfield.iter().next().unwrap();
    let camera_pivot = query_camera_pivot.iter().next().unwrap();
    let playfield_half_size = playfield.half_size(**aspect, camera_pivot.translation.z);
    let playfield_quarter_size = playfield.quarter_size(**aspect, camera_pivot.translation.z);

    for (transform_force, force, force_state, mut linear_move_to) in query_force.iter_mut().next() {
        let transform_ship = query_ship.get(force.ship).ok();

        // Configure movement
        linear_move_to.active = !matches!(
            force_state,
            ForceState::DockedFront | ForceState::DockedRear
        );

        linear_move_to.speed = match force_state {
            ForceState::Free => force.speed_free,
            ForceState::Recall => match transform_ship {
                Some(transform_ship)
                    if (transform_ship.translation - transform_force.translation).length()
                        < force.docking_distance =>
                {
                    force.speed_docking
                }
                _ => force.speed_recall,
            },
            ForceState::DockedFront | ForceState::DockedRear => 0.0,
            ForceState::LaunchForward | ForceState::LaunchBackward => force.speed_launch,
        };

        match force_state {
            ForceState::Free => {
                match transform_ship {
                    Some(transform_ship) if transform_ship.translation.x > 0.0 => {
                        linear_move_to.target.x = -playfield_quarter_size.x;
                    }
                    _ => linear_move_to.target.x = playfield_quarter_size.x,
                }

                if transform_force.translation.x == linear_move_to.target.x {
                    linear_move_to.factor = Vec3::Y;
                    if let Some(transform_ship) = transform_ship {
                        linear_move_to.target.y = transform_ship.translation.y;
                    }
                } else {
                    linear_move_to.factor = Vec3::X;
                };
            }
            ForceState::Recall => {
                linear_move_to.factor = Vec3::ONE;
                if let Some(transform_ship) = transform_ship {
                    linear_move_to.target = transform_ship.translation;
                    if transform_force.translation.x < transform_ship.translation.x {
                        linear_move_to.target -= Vec3::X;
                    }
                    if transform_force.translation.x > transform_ship.translation.x {
                        linear_move_to.target += Vec3::X;
                    }
                }
            }
            ForceState::LaunchForward => {
                linear_move_to.factor = Vec3::X;
                linear_move_to.target.x = playfield_half_size.x;
            }
            ForceState::LaunchBackward => {
                linear_move_to.factor = Vec3::X;
                linear_move_to.target.x = -playfield_half_size.x;
            }
            _ => (),
        }
    }
}

pub fn force_dock_transform(
    mut query_force: Query<(&Force, &ForceState, &mut Transform), Without<ShipForce>>,
    query_ship: Query<&Transform, With<ShipForce>>,
) {
    for (force, force_state, mut force_transform) in query_force.iter_mut() {
        let ship_transform = query_ship.get(force.ship).unwrap();

        match force_state {
            ForceState::DockedFront => {
                force_transform.translation =
                    ship_transform.translation + Vec3::new(1.85, 0.0, 0.0);
            }
            ForceState::DockedRear => {
                force_transform.translation =
                    ship_transform.translation + Vec3::new(-1.85, 0.0, 0.0);
            }
            _ => (),
        }
    }
}

/// Change state to free when launched into collision
pub fn force_body_collision(
    rapier_context: Res<RapierContext>,
    query_force: Query<(Entity, &Force, &ForceState)>,
    query_timeline: Query<&Timeline>,
    mut state_events: EventWriter<ForceStateEvent>,
) {
    for (force_entity, force, force_state) in query_force.iter() {
        let collider_handle =
            if let Some(collider_handle) = rapier_context.entity2collider().get(&force_entity) {
                collider_handle
            } else {
                continue;
            };

        for contact in rapier_context.narrow_phase.contacts_with(*collider_handle) {
            let (lhs, rhs) = match (
                rapier_context.collider_entity(contact.collider1),
                rapier_context.collider_entity(contact.collider2),
            ) {
                (Some(lhs), Some(rhs)) => (lhs, rhs),
                _ => continue,
            };

            if !contact.has_any_active_contact {
                continue;
            }

            let timeline = query_timeline.get(force.timeline).unwrap();
            let t = timeline.t;

            if lhs != force_entity && rhs != force_entity {
                continue;
            }

            let normal = if let Some((manifold, _)) = contact.find_deepest_contact() {
                if lhs == force_entity {
                    manifold.local_n2
                } else if rhs == force_entity {
                    manifold.local_n1
                } else {
                    unreachable!()
                }
            } else {
                continue;
            };

            let force_velocity = match *force_state {
                ForceState::LaunchForward => Vector2::new(1.0, 0.0),
                ForceState::LaunchBackward => Vector2::new(-1.0, 0.0),
                _ => continue,
            };

            if normal.dot(&force_velocity) < -0.95 {
                state_events.send(ForceStateEvent {
                    target: force_entity,
                    state: ForceState::Free,
                    t,
                });
            }
        }
    }
}

trait IsDescendantOf {
    fn is_descendant_of(&self, ancestor: Entity, query_parent: &Query<Option<&Parent>>) -> bool;
}

impl IsDescendantOf for Entity {
    fn is_descendant_of(&self, ancestor: Entity, query_parent: &Query<Option<&Parent>>) -> bool {
        let mut candidate = *self;
        loop {
            let parent = query_parent.get(candidate).unwrap();

            if let Some(parent) = parent {
                if **parent == ancestor {
                    return true;
                } else {
                    candidate = **parent
                }
            } else {
                return false;
            }
        }
    }
}

/// Dock with the ship on collision
pub fn force_sensor_collision(
    rapier_context: Res<RapierContext>,
    query_ship: Query<(&Alive, &Transform), With<ShipForce>>,
    query_force: Query<(Entity, &Force, &ForceState, &Transform)>,
    query_timeline: Query<&Timeline>,
    query_parent: Query<Option<&Parent>>,
    mut state_events: EventWriter<ForceStateEvent>,
) {
    for (force_entity, force, force_state, force_transform) in query_force.iter() {
        let ship_components = query_ship.get(force.ship).ok();

        let collider_handle =
            if let Some(collider_handle) = rapier_context.entity2collider().get(&force.sensor) {
                collider_handle
            } else {
                continue;
            };

        for (lhs, rhs) in rapier_context
            .narrow_phase
            .intersections_with(*collider_handle)
            .filter_map(|(lhs, rhs, inter)| if inter { Some((lhs, rhs)) } else { None })
        {
            let (lhs, rhs) = match (
                rapier_context.collider_entity(lhs),
                rapier_context.collider_entity(rhs),
            ) {
                (Some(lhs), Some(rhs)) => (lhs, rhs),
                _ => continue,
            };

            let timeline = query_timeline.get(force.timeline).unwrap();
            let t = timeline.t;

            match ship_components {
                Some((ship_alive, ship_transform))
                    if ship_alive.0
                        && (lhs.is_descendant_of(force.ship, &query_parent)
                            && rhs.is_descendant_of(force_entity, &query_parent))
                        || (lhs.is_descendant_of(force_entity, &query_parent)
                            && rhs.is_descendant_of(force.ship, &query_parent)) =>
                {
                    match *force_state {
                        ForceState::Free | ForceState::Recall => {
                            state_events.send(ForceStateEvent {
                                target: force_entity,
                                state: if ship_transform.translation.x
                                    < force_transform.translation.x
                                {
                                    info!("Force docking front");
                                    ForceState::DockedFront
                                } else {
                                    info!("Force docking rear");
                                    ForceState::DockedRear
                                },
                                t,
                            });
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn force_contact_depenetration(
    mut query_force: Query<(Entity, &ForceState, &mut ContactDepenetration)>,
) {
    for (force_entity, force_state, mut depenetration) in query_force.iter_mut() {
        depenetration.targets = match force_state {
            ForceState::DockedFront | ForceState::DockedRear => vec![],
            _ => vec![force_entity],
        }
    }
}

pub fn force_shapecast_depenetration(
    mut query_force: Query<(Entity, &ForceState, &mut ShapecastDepenetration)>,
) {
    for (force_entity, force_state, mut depenetration) in query_force.iter_mut() {
        depenetration.targets = match force_state {
            ForceState::DockedFront | ForceState::DockedRear => vec![],
            _ => vec![force_entity],
        }
    }
}

pub fn force_ship_contact_depenetration(
    mut query_ship: Query<(Entity, &ShipForce, &mut ContactDepenetration)>,
    query_force_state: Query<&ForceState>,
) {
    for (ship_entity, ship_force, mut contact_depenetration) in query_ship.iter_mut() {
        let force_state = query_force_state.get(ship_force.force);
        let targets = match force_state {
            Ok(ForceState::DockedRear | ForceState::DockedFront) => {
                vec![ship_entity, ship_force.force]
            }
            _ => vec![ship_entity],
        };

        contact_depenetration.targets = targets;
    }
}

pub fn force_ship_shapecast_depenetration(
    mut query_ship: Query<(Entity, &ShipForce, &mut ShapecastDepenetration)>,
    query_force_state: Query<&ForceState>,
) {
    for (ship_entity, ship_force, mut contact_depenetration) in query_ship.iter_mut() {
        let force_state = query_force_state.get(ship_force.force);
        let targets = match force_state {
            Ok(ForceState::DockedRear | ForceState::DockedFront) => {
                vec![ship_entity, ship_force.force]
            }
            _ => vec![ship_entity],
        };

        contact_depenetration.targets = targets;
    }
}

pub fn force_input(
    input_ship: Res<Input<PlayerInput>>,
    query_force: Query<(Entity, &Force, &ForceState), With<ForceInput>>,
    query_timeline: Query<&Timeline>,
    mut state_events: EventWriter<ForceStateEvent>,
) {
    for (force_entity, force, force_state) in query_force.iter() {
        let timeline = query_timeline.get(force.timeline).unwrap();
        let t = timeline.t;

        match *force_state {
            ForceState::Free => {
                if input_ship.just_pressed(PlayerInput::Force) {
                    state_events.send(ForceStateEvent {
                        target: force_entity,
                        state: ForceState::Recall,
                        t,
                    });
                }
            }
            ForceState::DockedFront => {
                if input_ship.just_pressed(PlayerInput::Force) {
                    state_events.send(ForceStateEvent {
                        target: force_entity,
                        state: ForceState::LaunchForward,
                        t,
                    });
                }
            }
            ForceState::DockedRear => {
                if input_ship.just_pressed(PlayerInput::Force) {
                    state_events.send(ForceStateEvent {
                        target: force_entity,
                        state: ForceState::LaunchBackward,
                        t,
                    });
                }
            }
            _ => (),
        }
    }
}

pub fn force_ship_death(
    mut death_events: EventReader<DeathEvent>,
    query_timeline: Query<&Timeline>,
    query_ship_force: Query<(&ShipForce, &TimelineDamage)>,
    mut state_events: EventWriter<ForceStateEvent>,
    mut input_events: EventWriter<ForceInputEvent>,
    mut vulcan_events: EventWriter<ForceVulcanEvent>,
) {
    for event in death_events.iter() {
        let (ship_force, timeline_damage) =
            if let Ok(components) = query_ship_force.get(event.entity) {
                components
            } else {
                continue;
            };

        let timeline = query_timeline.get(timeline_damage.timeline).unwrap();
        let t = timeline.t;

        // Detach the force and remove its refire component
        state_events.send(ForceStateEvent {
            target: ship_force.force,
            state: ForceState::Free,
            t,
        });

        input_events.send(ForceInputEvent {
            target: ship_force.force,
            enabled: false,
            t,
            determinism: event.determinism,
        });

        vulcan_events.send(ForceVulcanEvent {
            target: ship_force.force,
            enabled: false,
            t,
            determinism: event.determinism,
        });
    }
}

pub fn force_ship_respawn(
    mut respawn_events: EventReader<RespawnEvent>,
    query_ship: Query<(Entity, &ShipForce)>,
    mut input_events: EventWriter<ForceInputEvent>,
    mut vulcan_events: EventWriter<ForceVulcanEvent>,
) {
    for (ship_entity, ship_force) in query_ship.iter() {
        for event in respawn_events.iter() {
            if event.entity != ship_entity {
                continue;
            }

            let t = event.t;

            // Restore force's input and refire capability
            input_events.send(ForceInputEvent {
                target: ship_force.force,
                enabled: true,
                t,
                determinism: event.determinism,
            });

            vulcan_events.send(ForceVulcanEvent {
                target: ship_force.force,
                enabled: true,
                t,
                determinism: event.determinism,
            });
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ForceStateEvent {
    pub target: Entity,
    pub state: ForceState,
    pub t: f64,
}

pub fn force_state_event(
    mut state_events: EventReader<ForceStateEvent>,
    mut determinisms: ResMut<Determinisms>,
    query_force: Query<(&Force, &ForceState)>,
    query_timeline: Query<&Timeline>,
    mut query_state_stops: Query<&mut Discrete<ForceState>>,
    mut vulcan_events: EventWriter<ForceVulcanEvent>,
) {
    for event in state_events.iter() {
        let (force, force_state) = query_force.get(event.target).unwrap();

        let timeline = query_timeline.get(force.timeline).unwrap();

        let determinism = determinisms.next();

        let mut state_stops = query_state_stops.get_mut(force.state_stops).unwrap();
        state_stops.insert_stop(DiscreteStop {
            t: event.t,
            value: event.state,
            determinism,
            ..default()
        });

        match (force_state, event.state) {
            (ForceState::DockedRear | ForceState::DockedFront, _) => {
                vulcan_events.send(ForceVulcanEvent {
                    target: event.target,
                    enabled: true,
                    t: timeline.t,
                    determinism,
                })
            }
            (_, ForceState::DockedRear | ForceState::DockedFront) => {
                vulcan_events.send(ForceVulcanEvent {
                    target: event.target,
                    enabled: false,
                    t: timeline.t,
                    determinism,
                })
            }
            _ => (),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ForceInputEvent {
    pub target: Entity,
    pub enabled: bool,
    pub t: f64,
    pub determinism: DeterminismId,
}

pub fn force_input_event(
    mut input_events: EventReader<ForceInputEvent>,
    query_force: Query<&Force, With<ForceInput>>,
    mut query_input_stops: Query<&mut Discrete<Option<ForceInput>>>,
) {
    for event in input_events.iter() {
        let force = query_force.get(event.target).unwrap();
        let mut refire_stops = query_input_stops.get_mut(force.input_stops).unwrap();

        refire_stops.insert_stop(DiscreteStop {
            t: event.t,
            value: if event.enabled {
                Some(ForceInput::default())
            } else {
                None
            },
            determinism: event.determinism,
            ..default()
        });
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ForceVulcanEvent {
    pub target: Entity,
    pub enabled: bool,
    pub t: f64,
    pub determinism: DeterminismId,
}

pub fn force_vulcan_event(
    mut vulcan_events: EventReader<ForceVulcanEvent>,
    query_force: Query<&Force, With<ForceState>>,
    mut query_refire_stops: Query<&mut Discrete<Option<VulcanTimer>>>,
) {
    for event in vulcan_events.iter() {
        let force = query_force.get(event.target).unwrap();
        let mut refire_stops = query_refire_stops.get_mut(force.refire_stops).unwrap();
        refire_stops.insert_stop(DiscreteStop {
            t: event.t,
            value: if event.enabled {
                Some(VulcanTimer::default())
            } else {
                None
            },
            determinism: event.determinism,
            ..default()
        });
    }
}
