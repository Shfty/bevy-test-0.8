use std::ops::{BitOr, BitAnd};

use bevy::{
    ecs::system::CommandQueue,
    math::{Quat, Vec3},
    prelude::{Commands, Entity},
};
use bevy_fnord::prelude::{
    value::Value, AddGraphVertex, ByCopy, Cache, Connect, Destructure3, Evaluate, Evaluator,
    Function, GraphVertexCommands, Output, VertexOutput,
};

use crate::prelude::{
    AutorepeatSetTickRate, AutorepeatStart, AutorepeatStop, DelayedAutoRepeat, InputFloat,
    MoveType, DAS_RATE,
};

use super::auto_repeat_moves;

const DEADZONE: f32 = 0.02;

pub fn analog_axis<V, V2>(
    commands: &mut Commands,
    analog_input: Entity,
    axis: Vec3,
    deadzone: f32,
    vertex_cache: GraphVertexCommands<V>,
    controller_target: GraphVertexCommands<V2>,
) where
    V: VertexOutput<Output<0, f32>>,
    V: VertexOutput<Output<1, bool>>,
    V2: VertexOutput<Output<0, Option<Entity>>>,
{
    commands
        .entity(analog_input)
        .insert(DelayedAutoRepeat::new(0.0, DAS_RATE));

    let vertex_auto_repeat_rate_x = commands.add_graph_vertex(Function::new(move |value: f32| {
        let mut queue = CommandQueue::default();
        queue.push(AutorepeatSetTickRate {
            autorepeat: analog_input,
            tick_rate: value.abs() * DAS_RATE,
        });
        queue
    }));

    let vertex_auto_repeat_trigger_x =
        commands.add_graph_vertex(Function::new(move |value: f32| {
            let mut queue = CommandQueue::default();
            if value.abs() > deadzone {
                queue.push(AutorepeatStart {
                    autorepeat: analog_input,
                });
            } else {
                queue.push(AutorepeatStop {
                    autorepeat: analog_input,
                })
            }
            queue
        }));

    let vertex_auto_repeat_tick = commands.add_graph_vertex(Evaluator::new(move |world| {
        world
            .entity(analog_input)
            .get::<DelayedAutoRepeat>()
            .unwrap()
            .just_finished()
    }));

    let vertex_rising_edge = commands.add_graph_vertex(Function::new(move |value: f32| value.abs() > deadzone));
    let vertex_rising_edge_cache = commands.add_graph_vertex(Cache::<bool>::default());

    commands.connect(vertex_cache.output::<0, f32>() | vertex_rising_edge.input::<0, f32>());
    commands.connect(vertex_rising_edge.output::<0, bool>() | vertex_rising_edge_cache.input::<0, bool>());

    let vertex_and = commands.add_graph_vertex(Function::new(BitAnd::bitand));
    let vertex_or = commands.add_graph_vertex(Function::new(BitOr::bitor));

    commands.connect(vertex_rising_edge_cache.output::<0, bool>() | vertex_and.input::<0, bool>());
    commands.connect(vertex_rising_edge_cache.output::<1, bool>() | vertex_and.input::<1, bool>());

    commands.connect(vertex_auto_repeat_tick.output::<0, bool>() | vertex_or.input::<0, bool>());
    commands.connect(vertex_and.output::<0, bool>() | vertex_or.input::<1, bool>());

    let vertex_analog_x_proj = commands.add_graph_vertex(Function::new(move |x: f32| axis * x));

    let vertex_auto_repeat_moves_analog_x =
        commands.add_graph_vertex(Function::new(auto_repeat_moves(MoveType::Hit)));

    commands
        .connect(vertex_cache.output::<0, f32>() | vertex_auto_repeat_rate_x.input::<0, f32>())
        .connect(vertex_cache.output::<0, f32>() | vertex_auto_repeat_trigger_x.input::<0, f32>())
        .connect(vertex_cache.output::<0, f32>() | vertex_analog_x_proj.input::<0, f32>())
        .connect(
            vertex_analog_x_proj.output::<0, Vec3>()
                | vertex_auto_repeat_moves_analog_x.input::<0, Vec3>(),
        )
        .connect(
            vertex_or.output::<0, bool>() | vertex_auto_repeat_moves_analog_x.input::<1, bool>(),
        )
        .connect(
            controller_target.output::<0, Option<Entity>>()
                | vertex_auto_repeat_moves_analog_x.input::<2, Option<Entity>>(),
        );

    commands
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_rate_x)
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_trigger_x)
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_moves_analog_x);
}

pub fn analog_axes<V, V2>(
    commands: &mut Commands,
    analog_x_input: Entity,
    analog_y_input: Entity,
    analog_x_autorepeat: Entity,
    analog_y_autorepeat: Entity,
    vertex_camera_rotation: GraphVertexCommands<V>,
    vertex_controller_target: GraphVertexCommands<V2>,
) where
    V: VertexOutput<Output<0, Quat>> + Clone,
    V2: VertexOutput<Output<0, Option<Entity>>> + Clone,
{
    let vertex_analog_x = commands.add_graph_vertex({
        let analog_x = analog_x_input;
        Evaluator::new(move |world| world.get::<InputFloat>(analog_x).unwrap().float())
    });

    let vertex_analog_y = commands.add_graph_vertex({
        let analog_y = analog_y_input;
        Evaluator::new(move |world| world.get::<InputFloat>(analog_y).unwrap().float())
    });

    let vertex_zero = commands.add_graph_vertex(Value::<ByCopy, f32>::new(0.0));

    let vertex_vec_xy = commands.add_graph_vertex(Function::new(Vec3::new));

    let vertex_transform_xy = commands.add_graph_vertex(Function::new(|vec: Vec3, yaw: Quat| {
        let len = vec.length();
        if len > 0.0 {
            let len = len.powf(2.0).min(1.0);
            yaw.inverse() * vec.normalize() * len
        } else {
            vec
        }
    }));

    let vertex_destructure_xy = commands.add_graph_vertex(Destructure3::<Vec3>::default());

    let vertex_cache_x = commands.add_graph_vertex(Cache::<f32>::default());

    let vertex_cache_y = commands.add_graph_vertex(Cache::<f32>::default());

    commands
        .connect(vertex_analog_x.output::<0, f32>() | vertex_vec_xy.input::<0, f32>())
        .connect(vertex_zero.output::<0, f32>() | vertex_vec_xy.input::<1, f32>())
        .connect(vertex_analog_y.output::<0, f32>() | vertex_vec_xy.input::<2, f32>())
        .connect(vertex_vec_xy.output::<0, Vec3>() | vertex_transform_xy.input::<0, Vec3>())
        .connect(
            vertex_camera_rotation.output::<0, Quat>() | vertex_transform_xy.input::<1, Quat>(),
        )
        .connect(vertex_transform_xy.output::<0, Vec3>() | vertex_destructure_xy.input::<0, Vec3>())
        .connect(vertex_destructure_xy.output::<0, f32>() | vertex_cache_x.input::<0, f32>())
        .connect(vertex_destructure_xy.output::<2, f32>() | vertex_cache_y.input::<0, f32>());

    // Analog X
    analog_axis(
        commands,
        analog_x_autorepeat,
        Vec3::X,
        DEADZONE,
        vertex_cache_x,
        vertex_controller_target.clone(),
    );

    // Analog Y
    analog_axis(
        commands,
        analog_y_autorepeat,
        -Vec3::Z,
        DEADZONE,
        vertex_cache_y,
        vertex_controller_target,
    );
}
