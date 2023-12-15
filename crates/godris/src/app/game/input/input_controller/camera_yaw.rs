use bevy::{
    ecs::system::CommandQueue,
    prelude::{Commands, Entity},
};
use bevy_fnord::prelude::{AddGraphVertex, Connect, Evaluate, Evaluator, Function};

use crate::prelude::{InputFloat, YawCamera};

use std::f32::consts::TAU;

pub fn camera_yaw(commands: &mut Commands, camera_x_input: Entity, camera_orbit: Entity) {
    let vertex_camera_x_input = commands.add_graph_vertex({
        let camera_x = camera_x_input;
        Evaluator::new(move |world| world.get::<InputFloat>(camera_x).unwrap().float())
    });

    let vertex_camera_yaw = commands.add_graph_vertex(Function::new(move |input: f32| {
        let mut queue = CommandQueue::default();
        queue.push(YawCamera {
            camera: camera_orbit,
            delta: (input.abs().powf(2.0)) * input.signum() * -TAU,
        });
        queue
    }));

    commands
        .connect(vertex_camera_x_input.output::<0, f32>() | vertex_camera_yaw.input::<0, f32>());
    commands.evaluate_mut::<0, CommandQueue>(vertex_camera_yaw);
}
