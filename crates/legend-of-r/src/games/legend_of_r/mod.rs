pub mod force;

use bevy_rapier2d::na::{Point3, Unit, Vector3};

use bevy::{
    ecs::system::Command,
    prelude::{
        default,
        shape::{Capsule, Cube, Icosphere, Plane},
        AmbientLight, AssetServer, Assets, BuildChildren, Color, Commands, DirectionalLightBundle,
        Entity, EulerRot, MaterialMeshBundle, Mesh, Name, Plugin, Quat, Res, ResMut, Transform,
        Vec3, World,
    },
    render::{mesh::Indices, render_resource::PrimitiveTopology, view::RenderLayers},
};
use parry3d::shape::{Ball, ConvexPolyhedron, Cuboid, HalfSpace, Segment, SharedShape, Triangle};

use crate::{
    animation::{
        adapters::{
            animation::{Animation, AnimationTrait},
            discretize::DiscretizeTrait,
            sequence::SequenceTrait,
            try_animate_component_fields::TryAnimateComponentFieldsTrait,
        },
        animations::discrete::{Discrete, DiscreteStop},
        Animate, AnimationTime, PreUpdate,
    },
    scene::InsertSceneArchive,
    prelude::{
        interval, AfterTrait, Alive, AnimateComponentsTrait, AnimationTagTrait, AssembleBackground,
        AssembleCameraRig, AssembleForce, AssemblePlaneCollider, AssemblePlayfield, AssembleShip,
        AxialFunctionTrait, BeforeTrait, BuildAnimation, BulletAnimation, CurveTrait, DilateTrait,
        DiscreteStopsTrait, EntityComponentTrait, EvaluateTrait, FlattenTrait, ForcePlugin,
        HitPoints, InsertEnemy, InsertEntityPool, InsertTimelineAlive, InterpolateTrait,
        MultiplyTrait, OffsetIterator, OffsetTrait, PartitionIterator, SpawnBoundary, SpawnBullet,
        TimeSourceTrait, Timeline, TimelineAlive, TimelineDamage, TimelineTime,
        TryAnimateComponentsTrait, UnpoolEntity, Update, SCENE_ENEMY, SCENE_ENEMY_BULLET,
    },
    util::GameMaterial,
};

pub struct LegendOfRPlugin;

impl Plugin for LegendOfRPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ForcePlugin);

        app.add_startup_system(setup);
    }
}

pub fn setup<'w, 's>(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GameMaterial>>,
    mut commands: Commands<'w, 's>,
) {
    asset_server.watch_for_changes().unwrap();

    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::default()
            .looking_at(Vec3::new(-1.0, -1.0, -1.0).normalize(), Vec3::Y),
        ..default()
    });

    let camera_rig = commands.spawn().id();
    commands.add(AssembleCameraRig { entity: camera_rig });

    // Background
    let background = commands.spawn().id();
    commands.add(AssembleBackground { entity: background });

    let background_mesh = meshes.add(Cube::new(0.5).into());
    let background_material = materials.add(Color::WHITE.into());

    let background_objects = (0..128)
        .into_iter()
        .map(|_| {
            let translate_x = (2.0 * rand::random::<f32>() - 1.0) * 32.0;
            let translate_y = (2.0 * rand::random::<f32>() - 1.0) * 18.0;

            let rotate_x = rand::random::<f32>() * std::f32::consts::TAU;
            let rotate_y = rand::random::<f32>() * std::f32::consts::TAU;
            let rotate_z = rand::random::<f32>() * std::f32::consts::TAU;

            commands
                .spawn_bundle(MaterialMeshBundle::<GameMaterial> {
                    transform: Transform::from_xyz(translate_x, translate_y, -10.0)
                        .with_rotation(Quat::from_euler(
                            EulerRot::XYZ,
                            rotate_x,
                            rotate_y,
                            rotate_z,
                        ))
                        .with_scale(Vec3::ONE * 0.1)
                        .into(),
                    mesh: background_mesh.clone(),
                    material: background_material.clone(),
                    ..default()
                })
                .insert(RenderLayers::layer(1))
                .id()
        })
        .collect::<Vec<_>>();

    commands
        .entity(background)
        .push_children(&background_objects);

    // Playfield
    let timeline_entity = commands
        .spawn()
        .insert(Name::new("Timeline"))
        .insert(Timeline::default())
        .id();

    let playfield = commands.spawn().id();
    let ship = commands.spawn().id();
    let force = commands.spawn().id();

    commands.add(AssemblePlayfield {
        entity: playfield,
        playfield_bundle: default(),
        ..default()
    });
    commands.add(SpawnBoundary { playfield });
    commands.add(AssembleShip {
        playfield,
        timeline: timeline_entity,
        entity: ship,
        ..default()
    });
    commands.add(AssembleForce {
        playfield,
        timeline: timeline_entity,
        ship,
        force,
        ..default()
    });

    // Timeline
    let timeline = TimelineTime::new(timeline_entity);

    let bullet_pool = commands.spawn().id();
    commands.entity(playfield).add_child(bullet_pool);

    let instance_constructor = move |world: &mut World, entity: Entity, timeline: Entity| {
        SpawnBullet {
            timeline,
            entity,
            scene: InsertSceneArchive {
                path: SCENE_ENEMY_BULLET.into(),
                ..default()
            },
            ..default()
        }
        .write(world)
    };

    commands.add(InsertEntityPool {
        entity: bullet_pool,
        timeline: timeline_entity,
        ..default()
    });

    // Spawn enemies
    let iter = interval(0.5).offset(1.0);

    const WAVE_1_COUNT: usize = 4;
    const WAVE_1_SALVO: usize = 10;

    let (wave_1, _iter) = iter.partition_iter(WAVE_1_COUNT);
    wave_1.enumerate().for_each(move |(i, t)| {
        let enemy = commands.spawn().id();

        let animation_transform = commands
            .build_animation()
            .from_discrete_stops([
                DiscreteStop {
                    t: 0.0,
                    value: Transform::from_xyz(17.0, 5.0 - i as f32 * 2.0, 0.0)
                        .with_rotation(Quat::from_scaled_axis(Vec3::Z * std::f32::consts::PI)),
                    ..default()
                },
                DiscreteStop {
                    t: 1.0,
                    value: Transform::from_xyz(-17.0, 5.0 - i as f32 * 2.0, 0.0)
                        .with_rotation(Quat::from_scaled_axis(Vec3::Z * std::f32::consts::PI)),
                    ..default()
                },
            ])
            .interpolate()
            .with_curve(4.0)
            .with_dilation(0.5)
            .with_offset(-t)
            .after(t)
            .before(t + 2.0)
            .flatten()
            .as_animation();

        let enemy_animation = animation_transform.id();

        let animation_transform = animation_transform
            .with_time_source(timeline)
            .try_animating_component(enemy)
            .evaluate()
            .tagged::<Update>()
            .insert(Name::new(format!("Enemy {enemy:?} Transform")))
            .id();

        commands.entity(enemy).add_child(animation_transform);

        commands.add(InsertEnemy {
            playfield,
            timeline: timeline_entity,
            entity: enemy,
            scene: InsertSceneArchive {
                path: SCENE_ENEMY.into(),
                ..default()
            },
            timeline_alive: InsertTimelineAlive {
                alive_at: Some(t),
                dead_at: Some(t + 2.0),
                ..default()
            },
            ..default()
        });

        for t in interval(0.1).offset(t).offset(1.0).take(WAVE_1_SALVO) {
            commands.add(UnpoolEntity {
                source: enemy,
                entity_pool: bullet_pool,
                t,
                unpool: move |world: &mut World, entity: Entity, transform: Transform, t: f64| {
                    let bullet = world.get::<BulletAnimation>(entity).unwrap();

                    let from = t;
                    let to = t + 1.0;
                    let speed = 0.6;

                    // Transform animation
                    let from_stops_entity = bullet.from_stops;
                    let to_stops_entity = bullet.to_stops;

                    let value = Animation::<Option<Transform>>::animate(
                        world,
                        enemy_animation,
                        AnimationTime { t, ..default() },
                    )
                    .unwrap()
                        * transform;

                    let mut from_stops = world
                        .get_mut::<Discrete<DiscreteStop<Transform>>>(from_stops_entity)
                        .unwrap();

                    from_stops.insert_stop(DiscreteStop {
                        t: from,
                        value: DiscreteStop {
                            t: from,
                            value,
                            ..default()
                        },
                        ..default()
                    });

                    let mut to_stops = world
                        .get_mut::<Discrete<DiscreteStop<Transform>>>(to_stops_entity)
                        .unwrap();

                    to_stops.insert_stop(DiscreteStop {
                        t,
                        value: DiscreteStop {
                            t: t + speed,
                            value: default(),
                            ..default()
                        },
                        ..default()
                    });

                    // Lerp target animation
                    world
                        .build_animation()
                        .from_component(Transform::default())
                        .sequenced_with(|animation| animation.from_entity_component(force), 0.01)
                        .discretized_to([0.0, from])
                        .try_animating_component_field::<Discrete<DiscreteStop<Transform>>, _, _>(
                            to_stops_entity,
                            move |stops| &stops.stop_at(to).unwrap().value.value,
                            move |stops| &mut stops.stop_at_mut(to).unwrap().value.value,
                        )
                        .with_time_source(timeline)
                        .evaluate()
                        .tagged::<PreUpdate>()
                        .insert(Name::new("Bullet Lerp Target"));

                    // Hit points animation
                    let timeline_hit_points = world.get::<TimelineDamage>(entity).unwrap();
                    let hit_points_stops_entity = timeline_hit_points.hit_points_stops;

                    let mut hit_points_stops = world
                        .get_mut::<Discrete<HitPoints>>(hit_points_stops_entity)
                        .unwrap();

                    hit_points_stops.insert_stop(DiscreteStop {
                        t: from,
                        value: HitPoints(1),
                        ..default()
                    });

                    // Alive animation
                    let timeline_alive = world.get::<TimelineAlive>(entity).unwrap();
                    let alive_stops_entity = timeline_alive.alive_stops;

                    let mut alive_stops = world
                        .get_mut::<Discrete<Alive>>(alive_stops_entity)
                        .unwrap();

                    alive_stops.insert_stop(DiscreteStop {
                        t: from,
                        value: Alive(true),
                        ..default()
                    });

                    alive_stops.insert_stop(DiscreteStop {
                        t: to,
                        value: Alive(false),
                        ..default()
                    });

                    // Alive disabled animations
                    let alive_from_disabled =
                        world.build_animation().from_discrete_stops([DiscreteStop {
                            t: 0.0,
                            value: false,
                            ..default()
                        }]);

                    let alive_from_enabled_entity = alive_from_disabled.id();

                    alive_from_disabled
                        .try_animating_component_field::<Discrete<Alive>, _, _>(
                            alive_stops_entity,
                            move |stops| &stops.stop_at(from).unwrap().disabled,
                            move |stops| &mut stops.stop_at_mut(from).unwrap().disabled,
                        )
                        .with_time_source(timeline)
                        .evaluate()
                        .tagged::<Update>()
                        .insert(Name::new("Bullet Alive From Enabled"));

                    let alive_to_disabled =
                        world.build_animation().from_discrete_stops([DiscreteStop {
                            t: 0.0,
                            value: false,
                            ..default()
                        }]);

                    let alive_to_enabled_entity = alive_to_disabled.id();

                    alive_to_disabled
                        .try_animating_component_field::<Discrete<Alive>, _, _>(
                            alive_stops_entity,
                            move |stops| &stops.stop_at(to).unwrap().disabled,
                            move |stops| &mut stops.stop_at_mut(to).unwrap().disabled,
                        )
                        .with_time_source(timeline)
                        .evaluate()
                        .tagged::<Update>()
                        .insert(Name::new("Bullet Alive To Enabled"));

                    world
                        .entity_mut(enemy)
                        .get_mut::<TimelineAlive>()
                        .unwrap()
                        .descendants_enabled_stops
                        .push((from, alive_from_enabled_entity, alive_to_enabled_entity))
                },
                instance_constructor: Some((timeline_entity, instance_constructor.clone())),
                emitters: vec![Transform::default()],
            });
        }
    });

    /*
    let (wave_2, _) = iter.partition_iter(4);
    wave_2.offset(2.0).enumerate().for_each(|(i, t)| {
        let enemy = commands.spawn().id();

        commands.timeline_enemy(
            SpawnEnemy {
                playfield,
                entity: enemy,
                mesh: mesh_enemy.clone(),
                material: material_enemy.clone(),
                collider: collider_enemy.clone(),
            },
            unit(
                value(Transform::from_xyz(18.0, -5.0 + i as f32 * 2.0, 0.0))
                    .with_min_t(Some(0.0))
                    .with_max_t(Some(0.5)),
            )
            .sequence(
                value(Transform::from_xyz(-18.0, 3.0, 0.0))
                    .with_min_t(Some(0.0))
                    .with_max_t(Some(0.5)),
            )
            .discretize([0.0, 1.0].into_iter().map(|stop| {
                (
                    stop,
                    DiscretizeStop {
                        t: stop,
                        flags: TimelineEventFlags::empty(),
                    },
                )
            }))
            .interpolate()
            .mul(orbit(Vec3::X, Vec3::Y, 1.0).speed(3.141))
            .duration(4.0)
            .discretize(interval(4.0 / 8.0).take(8).map(|stop| {
                (
                    stop,
                    DiscretizeStop {
                        t: stop,
                        flags: TimelineEventFlags::empty(),
                    },
                )
            }))
            .interpolate()
            .delay(t)
            .cull_time_range(),
            default(),
        );
    });
    */

    /*
    setup_animated_primitives(
        &mut meshes,
        &mut materials,
        &mut commands,
        timeline,
        playfield,
        background,
    );
    */
}

pub fn setup_animated_primitives<'w, 's>(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<GameMaterial>,
    commands: &mut Commands<'w, 's>,
    timeline: TimelineTime,
    playfield: Entity,
    background: Entity,
) {
    // Playfield-intersecting segment
    let segment_3d = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle::<GameMaterial> {
            mesh: meshes.add(
                Capsule {
                    radius: 0.01,
                    depth: 2.0,
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(RenderLayers::layer(1))
        .id();
    let segment_2d = commands.spawn().id();

    commands.add(AssemblePlaneCollider {
        collider_3d: segment_3d,
        collider_2d: segment_2d,
        shape: SharedShape::new(Segment {
            a: Point3::new(0.0, -1.0, 0.0),
            b: Point3::new(0.0, 1.0, 0.0),
        }),
    });

    let segment_animation = commands
        .build_animation()
        .from_component(Transform::from_xyz(0.0, 7.0, 0.0))
        .multiplied_with(|animation| animation.from_axial_function(Vec3::Z, f32::sin))
        .multiplied_with(|animation| {
            animation.from_component(Transform::default().looking_at(Vec3::ONE, Vec3::Z))
        })
        .animating_component(segment_3d)
        .with_time_source(timeline)
        .evaluate()
        .insert(Name::new("Segment Transform"))
        .id();

    commands.entity(segment_3d).add_child(segment_animation);

    // Playfield-intersecting triangle
    let mut mesh_triangle = Mesh::new(PrimitiveTopology::TriangleList);
    mesh_triangle.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.5, -0.5, -0.5], [-0.5, 0.5, -0.5], [-0.5, -0.5, 0.5]],
    );
    mesh_triangle.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    );
    mesh_triangle.set_indices(Some(Indices::U32(vec![0, 1, 2])));

    let triangle_3d = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle::<GameMaterial> {
            mesh: meshes.add(mesh_triangle),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(RenderLayers::layer(1))
        .id();
    let triangle_2d = commands.spawn().id();

    commands.add(AssemblePlaneCollider {
        collider_3d: triangle_3d,
        collider_2d: triangle_2d,
        shape: SharedShape::new(Triangle {
            a: Point3::new(0.5, -0.5, -0.5),
            b: Point3::new(-0.5, 0.5, -0.5),
            c: Point3::new(-0.5, -0.5, 0.5),
        }),
    });

    let triangle_animation = commands
        .build_animation()
        .from_component(Transform::from_xyz(10.0, -7.0, 0.0))
        .multiplied_with(|animation| animation.from_axial_function(Vec3::Z, f32::sin))
        .animating_component(triangle_3d)
        .with_time_source(timeline)
        .evaluate()
        .insert(Name::new("Triangle Transform"))
        .id();

    commands.entity(triangle_3d).add_child(triangle_animation);

    // Playfield-intersecting halfspace
    let halfspace_3d = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle::<GameMaterial> {
            mesh: meshes.add(
                Plane {
                    size: 10.0,
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(RenderLayers::layer(1))
        .id();

    let halfspace_2d = commands.spawn().id();

    commands.add(AssemblePlaneCollider {
        collider_3d: halfspace_3d,
        collider_2d: halfspace_2d,
        shape: SharedShape::new(HalfSpace::new(Unit::new_normalize(Vector3::new(
            0.0, 1.0, 0.0,
        )))),
    });

    let halfspace_animation = commands
        .build_animation()
        .from_component(Transform::from_xyz(0.0, -7.0, 0.0))
        .multiplied_with(|animation| animation.from_axial_function(Vec3::Z, |t| t.sin() * 2.0))
        .multiplied_with(|animation| {
            animation.from_component(Transform::from_rotation(Quat::from_axis_angle(
                Vec3::X,
                std::f32::consts::FRAC_PI_4,
            )))
        })
        .animating_component(halfspace_3d)
        .with_time_source(timeline)
        .evaluate()
        .insert(Name::new("Halfspace Transform"))
        .id();

    commands.entity(halfspace_3d).add_child(halfspace_animation);

    // Playfield-intersecting sphere
    let sphere_3d = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle::<GameMaterial> {
            mesh: meshes.add(
                Icosphere {
                    radius: 1.0,
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(RenderLayers::layer(1))
        .id();

    let sphere_2d = commands.spawn().id();

    commands.add(AssemblePlaneCollider {
        collider_3d: sphere_3d,
        collider_2d: sphere_2d,
        shape: SharedShape::new(Ball::new(1.0)),
    });

    let sphere_animation = commands
        .build_animation()
        .from_component(Transform::from_xyz(10.0, 7.0, 0.0))
        .multiplied_with(|animation| animation.from_axial_function(Vec3::Z, |t| t.sin() * 2.0))
        .animating_component(sphere_3d)
        .with_time_source(timeline)
        .evaluate()
        .insert(Name::new("Sphere Transform"))
        .id();

    commands.entity(sphere_3d).add_child(sphere_animation);

    // Playfield-intersecting box
    let box_3d = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle::<GameMaterial> {
            mesh: meshes.add(Cube::new(1.0).into()),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(RenderLayers::layer(1))
        .id();

    let box_2d = commands.spawn().id();

    commands.add(AssemblePlaneCollider {
        collider_3d: box_3d,
        collider_2d: box_2d,
        shape: SharedShape::new(Cuboid::new(Vector3::new(0.5, 0.5, 0.5))),
    });

    let box_animation = commands
        .build_animation()
        .from_component(Transform::from_xyz(-10.0, 7.0, 0.0))
        .multiplied_with(|animation| animation.from_axial_function(Vec3::Z, f32::cos))
        .multiplied_with(|animation| {
            animation.from_component(Transform::default().looking_at(Vec3::ONE, Vec3::Y))
        })
        .animating_component(box_3d)
        .with_time_source(timeline)
        .evaluate()
        .insert(Name::new("Box Transform"))
        .id();

    commands.entity(box_3d).add_child(box_animation);

    // Playfield-intersecting convex shape
    let convex_mesh: Mesh = Icosphere {
        radius: 0.75,
        subdivisions: 0,
    }
    .into();

    let convex_verts = convex_mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .into_iter()
        .map(|point| Point3::new(point[0], point[1], point[2]))
        .collect::<Vec<_>>();

    let convex_3d = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle::<GameMaterial> {
            mesh: meshes.add(convex_mesh),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(RenderLayers::layer(1))
        .id();

    let convex_2d = commands.spawn().id();

    commands.add(AssemblePlaneCollider {
        collider_3d: convex_3d,
        collider_2d: convex_2d,
        shape: SharedShape::new(ConvexPolyhedron::from_convex_hull(&convex_verts).unwrap()),
    });

    let convex_animation = commands
        .build_animation()
        .from_component(Transform::from_xyz(-10.0, -7.0, 0.0))
        .multiplied_with(|animation| animation.from_axial_function(Vec3::Z, f32::cos))
        .animating_component(convex_3d)
        .with_time_source(timeline)
        .evaluate()
        .insert(Name::new("Convex Transform"))
        .id();

    commands.entity(convex_3d).add_child(convex_animation);

    // Add children
    commands.entity(background).push_children(&[
        segment_3d,
        sphere_3d,
        halfspace_3d,
        triangle_3d,
        box_3d,
        convex_3d,
    ]);

    commands.entity(playfield).push_children(&[
        segment_2d,
        sphere_2d,
        halfspace_2d,
        triangle_2d,
        box_2d,
        convex_2d,
    ]);
}
