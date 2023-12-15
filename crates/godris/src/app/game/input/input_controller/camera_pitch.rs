use bevy::{
    ecs::system::CommandQueue,
    prelude::{Commands, Entity},
};
use bevy_fnord::prelude::{AddGraphVertex, Connect, Evaluate, Evaluator, Function};

use crate::prelude::{InputFloat, PitchCamera};

use std::f32::consts::PI;

pub fn camera_pitch(commands: &mut Commands, camera_y_input: Entity, camera_orbit: Entity) {
    let vertex_camera_y_input = commands.add_graph_vertex({
        let camera_y = camera_y_input;
        Evaluator::new(move |world| world.get::<InputFloat>(camera_y).unwrap().float())
    });

    let vertex_camera_pitch = commands.add_graph_vertex(Function::new(move |input: f32| {
        let mut queue = CommandQueue::default();
        queue.push(PitchCamera {
            camera: camera_orbit,
            delta: (input.abs().powf(2.0)) * input.signum() * PI,
        });
        queue
    }));

    commands
        .connect(vertex_camera_y_input.output::<0, f32>() | vertex_camera_pitch.input::<0, f32>());
    commands.evaluate_mut::<0, CommandQueue>(vertex_camera_pitch);
}
