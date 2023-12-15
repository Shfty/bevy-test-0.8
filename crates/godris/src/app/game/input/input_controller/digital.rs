use std::sync::Mutex;

use bevy::{
    ecs::system::CommandQueue,
    math::{IVec3, Quat, Vec3},
    prelude::{default, Commands, Entity},
};
use bevy_fnord::prelude::{
    AddGraphVertex, Cache, Connect, Destructure3, Evaluate, Evaluator, Function,
    GraphVertexCommands, Output, VertexOutput,
};

use crate::prelude::{
    AutorepeatSetTickRate, AutorepeatStart, AutorepeatStop, BoardRotation, BoardTransform,
    DelayedAutoRepeat, GridMove, InputFloat, MoveType, PushPendingGridMoves, DAS_DELAY, DAS_RATE,
};

use super::auto_repeat_moves;

fn rising_edge_f32(deadzone: f32) -> impl FnOnce(f32) -> bool {
    move |f| f.abs() > deadzone
}

fn rising_edge_vec3(deadzone: f32) -> impl FnOnce(Vec3) -> bool {
    move |vec| vec.length() > deadzone
}

pub fn digital<V, V2>(
    commands: &mut Commands,
    digital_x_input: Entity,
    digital_y_input: Entity,
    rotate_cw_input: Entity,
    rotate_ccw_input: Entity,
    digital_autorepeat: Entity,
    vertex_camera_rotation_snapped: GraphVertexCommands<V>,
    vertex_controller_target: GraphVertexCommands<V2>,
) where
    V: VertexOutput<Output<0, Quat>>,
    V2: VertexOutput<Output<0, Option<Entity>>>,
{
    // Read X / Y inputs and cache them
    let vertex_digital_x = commands.add_graph_vertex({
        Evaluator::new(move |world| world.get::<InputFloat>(digital_x_input).unwrap().float())
    });

    let vertex_digital_y = commands.add_graph_vertex({
        Evaluator::new(move |world| world.get::<InputFloat>(digital_y_input).unwrap().float())
    });

    let vertex_cache_x = commands.add_graph_vertex(Cache::<f32>::default());
    let vertex_cache_y = commands.add_graph_vertex(Cache::<f32>::default());

    commands
        .connect(vertex_digital_x.output::<0, f32>() | vertex_cache_x.input::<0, f32>())
        .connect(vertex_digital_y.output::<0, f32>() | vertex_cache_y.input::<0, f32>());

    // Compose XY into a Vec3(x, 0.0, y) via 4-way gate
    let active_axis = Mutex::new(Vec3::ZERO);
    let vertex_active_axis = commands.add_graph_vertex(Function::new(
        move |x: f32, x_changed: bool, y: f32, y_changed| {
            if x_changed && x != 0.0 {
                *active_axis.lock().unwrap() = Vec3::X
            } else if y_changed && y != 0.0 {
                *active_axis.lock().unwrap() = Vec3::Z
            } else if x == 0.0 && y == 0.0 {
                *active_axis.lock().unwrap() = Vec3::ZERO
            }

            *active_axis.lock().unwrap()
        },
    ));

    commands
        .connect(vertex_cache_x.output::<0, f32>() | vertex_active_axis.input::<0, f32>())
        .connect(vertex_cache_x.output::<1, bool>() | vertex_active_axis.input::<1, bool>())
        .connect(vertex_cache_y.output::<0, f32>() | vertex_active_axis.input::<2, f32>())
        .connect(vertex_cache_y.output::<1, bool>() | vertex_active_axis.input::<3, bool>());

    let vertex_active_axis_cache = commands.add_graph_vertex(Cache::<Vec3>::default());

    commands.connect(
        vertex_active_axis.output::<0, Vec3>() | vertex_active_axis_cache.input::<0, Vec3>(),
    );

    let vertex_vec_xy =
        commands.add_graph_vertex(Function::new(move |x: f32, y: f32, active_axis: Vec3| {
            Vec3::new(x, 0.0, y) * active_axis
        }));

    commands
        .connect(vertex_digital_x.output::<0, f32>() | vertex_vec_xy.input::<0, f32>())
        .connect(vertex_digital_y.output::<0, f32>() | vertex_vec_xy.input::<1, f32>())
        .connect(vertex_active_axis.output::<0, Vec3>() | vertex_vec_xy.input::<2, Vec3>());

    // Rotate the vector and manually round to 1.0
    let vertex_transform_vec = commands.add_graph_vertex(Function::new(|vec: Vec3, quat: Quat| {
        if vec.length() > 0.0 {
            let vec = vec.normalize();
            let vec = quat.inverse() * vec;
            Vec3::new(
                match vec.x {
                    x if x < -0.5 => -1.0,
                    x if x > 0.5 => 1.0,
                    _ => 0.0,
                },
                0.0,
                match vec.z {
                    z if z < -0.5 => 1.0,
                    z if z > 0.5 => -1.0,
                    _ => 0.0,
                },
            )
        } else {
            vec
        }
    }));

    commands
        .connect(vertex_vec_xy.output::<0, Vec3>() | vertex_transform_vec.input::<0, Vec3>())
        .connect(
            vertex_camera_rotation_snapped.output::<0, Quat>()
                | vertex_transform_vec.input::<1, Quat>(),
        );

    // Cache the rotated vector and split it into individual components
    let vertex_cache_xy = commands.add_graph_vertex(Cache::<Vec3>::default());
    let vertex_destructure_xy = commands.add_graph_vertex(Destructure3::<Vec3>::default());

    commands.connect(
        vertex_transform_vec.output::<0, Vec3>()
            | vertex_cache_xy.through::<0, 0, Vec3>()
            | vertex_destructure_xy.input::<0, Vec3>(),
    );

    // Cache the individual components
    let vertex_cache_x = commands.add_graph_vertex(Cache::<f32>::default());
    let vertex_cache_y = commands.add_graph_vertex(Cache::<f32>::default());

    commands
        .connect(vertex_destructure_xy.output::<0, f32>() | vertex_cache_x.input::<0, f32>())
        .connect(vertex_destructure_xy.output::<2, f32>() | vertex_cache_y.input::<0, f32>());

    // Merge the unsigned rotate axes into a single signed axis
    let vertex_rotate_cw_input = commands.add_graph_vertex({
        let rotate_cw = rotate_cw_input;
        Evaluator::new(move |world| world.get::<InputFloat>(rotate_cw).unwrap().float())
    });

    let vertex_rotate_ccw_input = commands.add_graph_vertex({
        let rotate_ccw = rotate_ccw_input;
        Evaluator::new(move |world| world.get::<InputFloat>(rotate_ccw).unwrap().float())
    });

    let vertex_rotate_merge =
        commands.add_graph_vertex(Function::new(|rotate_cw: f32, rotate_ccw: f32| {
            rotate_cw - rotate_ccw
        }));

    commands
        .connect(vertex_rotate_cw_input.output::<0, f32>() | vertex_rotate_merge.input::<0, f32>())
        .connect(
            vertex_rotate_ccw_input.output::<0, f32>() | vertex_rotate_merge.input::<1, f32>(),
        );

    // Cache the signed rotate axis
    let vertex_rotate_cache = commands.add_graph_vertex(Cache::<f32>::default());

    commands
        .connect(vertex_rotate_merge.output::<0, f32>() | vertex_rotate_cache.output::<0, f32>());

    // Convert the cached X value into a relative transform
    let vertex_digital_transform_x =
        commands.add_graph_vertex(Function::new(move |x: f32, changed: bool| BoardTransform {
            translation: if changed {
                IVec3::X * x.round() as i32
            } else {
                default()
            },
            ..default()
        }));

    commands
        .connect(vertex_cache_x.output::<0, f32>() | vertex_digital_transform_x.input::<0, f32>())
        .connect(
            vertex_cache_x.output::<1, bool>() | vertex_digital_transform_x.input::<1, bool>(),
        );

    // Convert the cached Z value into an IVec3 and compose it with the relative transform
    let vertex_digital_transform_z = commands.add_graph_vertex(Function::new(
        move |transform: BoardTransform, z: f32, changed: bool| BoardTransform {
            translation: transform.translation
                + if changed {
                    IVec3::Z * z.round() as i32
                } else {
                    default()
                },
            ..transform
        },
    ));

    commands
        .connect(
            vertex_digital_transform_x.output::<0, BoardTransform>()
                | vertex_digital_transform_z.input::<0, BoardTransform>(),
        )
        .connect(vertex_cache_y.output::<0, f32>() | vertex_digital_transform_z.input::<1, f32>())
        .connect(
            vertex_cache_y.output::<1, bool>() | vertex_digital_transform_z.input::<2, bool>(),
        );

    // Convert the signed rotate axis into a BoardRotation and compose it with the transform
    let vertex_digital_rotate = commands.add_graph_vertex(Function::new(
        move |transform: BoardTransform, rotate: f32, changed: bool| BoardTransform {
            rotation: if changed {
                if rotate < 0.0 {
                    BoardRotation::CCW
                } else if rotate > 0.0 {
                    BoardRotation::CW
                } else {
                    BoardRotation::Identity
                }
            } else {
                default()
            },
            ..transform
        },
    ));

    commands
        .connect(
            vertex_digital_transform_z.output::<0, BoardTransform>()
                | vertex_digital_rotate.input::<0, BoardTransform>(),
        )
        .connect(vertex_rotate_cache.output::<0, f32>() | vertex_digital_rotate.input::<1, f32>())
        .connect(
            vertex_rotate_cache.output::<1, bool>() | vertex_digital_rotate.input::<2, bool>(),
        );

    // Push the relative transform as a grid move
    let vertex_digital = commands.add_graph_vertex(Function::new(
        move |transform: BoardTransform, target: Option<Entity>| {
            let mut queue = CommandQueue::default();

            let target = if let Some(target) = target {
                target
            } else {
                return queue;
            };

            if transform != BoardTransform::default() {
                queue.push(PushPendingGridMoves {
                    pending_grid_moves: target,
                    moves: vec![GridMove {
                        delta: transform,
                        move_type: MoveType::Hit,
                    }],
                })
            }
            queue
        },
    ));

    commands
        .connect(
            vertex_digital_rotate.output::<0, BoardTransform>()
                | vertex_digital.input::<0, BoardTransform>(),
        )
        .connect(
            vertex_controller_target.output::<0, Option<Entity>>()
                | vertex_digital.input::<1, Option<Entity>>(),
        );

    // Update the autorepeat tick rate based on digital deflection
    let vertex_auto_repeat_rate_digital =
        commands.add_graph_vertex(Function::new(move |value: Vec3| {
            let mut queue = CommandQueue::default();
            queue.push(AutorepeatSetTickRate {
                autorepeat: digital_autorepeat,
                tick_rate: value.length() * DAS_RATE,
            });
            queue
        }));

    commands.connect(
        vertex_cache_xy.output::<0, Vec3>() | vertex_auto_repeat_rate_digital.input::<0, Vec3>(),
    );

    // Start or stop the autorepeat based on digital deflection
    let vertex_auto_repeat_trigger_digital = commands.add_graph_vertex(Function::new(
        move |active_axis: Vec3, active_axis_changed: bool| {
            let mut queue = CommandQueue::default();
            if active_axis == Vec3::ZERO {
                queue.push(AutorepeatStop {
                    autorepeat: digital_autorepeat,
                })
            } else {
                if active_axis_changed {
                    queue.push(AutorepeatStop {
                        autorepeat: digital_autorepeat,
                    });
                    queue.push(AutorepeatStart {
                        autorepeat: digital_autorepeat,
                    });
                }
            }
            queue
        },
    ));

    commands
        .connect(
            vertex_active_axis_cache.output::<0, Vec3>()
                | vertex_auto_repeat_trigger_digital.input::<0, Vec3>(),
        )
        .connect(
            vertex_active_axis_cache.output::<1, bool>()
                | vertex_auto_repeat_trigger_digital.input::<1, bool>(),
        );

    // Send moves whenever the autorepeat ticks
    let vertex_auto_repeat_tick = commands.add_graph_vertex(Evaluator::new(move |world| {
        world
            .entity(digital_autorepeat)
            .get::<DelayedAutoRepeat>()
            .unwrap()
            .just_finished()
    }));

    let vertex_auto_repeat_moves_digital =
        commands.add_graph_vertex(Function::new(auto_repeat_moves(MoveType::Hit)));

    commands
        .connect(
            vertex_cache_xy.output::<0, Vec3>()
                | vertex_auto_repeat_moves_digital.input::<0, Vec3>(),
        )
        .connect(
            vertex_auto_repeat_tick.output::<0, bool>()
                | vertex_auto_repeat_moves_digital.input::<1, bool>(),
        )
        .connect(
            vertex_controller_target.output::<0, Option<Entity>>()
                | vertex_auto_repeat_moves_digital.input::<2, Option<Entity>>(),
        );

    // Mark vertices for evaluation
    commands
        .evaluate_mut::<0, CommandQueue>(vertex_digital)
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_rate_digital)
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_trigger_digital)
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_moves_digital);

    // Insert autorepeat component
    commands
        .entity(digital_autorepeat)
        .insert(DelayedAutoRepeat::new(DAS_DELAY, DAS_RATE));
}
