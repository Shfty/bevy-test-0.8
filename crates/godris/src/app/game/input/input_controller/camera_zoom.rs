use bevy::{
    ecs::system::CommandQueue,
    prelude::{Commands, Entity},
};
use bevy_fnord::prelude::{AddGraphVertex, Connect, Evaluate, Evaluator, Function};

use crate::prelude::{AdjustZoom, InputFloat};

pub fn camera_zoom(
    commands: &mut Commands,
    camera_zoom_in_input: Entity,
    camera_zoom_out_input: Entity,
    camera_orbit: Entity,
) {
    let vertex_camera_zoom_in_input = commands.add_graph_vertex({
        Evaluator::new(move |world| {
            world
                .get::<InputFloat>(camera_zoom_in_input)
                .unwrap()
                .float()
        })
    });

    let vertex_camera_zoom_out_input = commands.add_graph_vertex({
        Evaluator::new(move |world| {
            world
                .get::<InputFloat>(camera_zoom_out_input)
                .unwrap()
                .float()
        })
    });

    let vertex_camera_zoom =
        commands.add_graph_vertex(Function::new(move |zoom_in: f32, zoom_out: f32| {
            let mut queue = CommandQueue::default();
            queue.push(AdjustZoom {
                camera: camera_orbit,
                delta: zoom_out - zoom_in,
            });
            queue
        }));

    commands
        .connect(
            vertex_camera_zoom_in_input.output::<0, f32>() | vertex_camera_zoom.input::<0, f32>(),
        )
        .connect(
            vertex_camera_zoom_out_input.output::<0, f32>() | vertex_camera_zoom.input::<1, f32>(),
        );

    commands.evaluate_mut::<0, CommandQueue>(vertex_camera_zoom);
}
