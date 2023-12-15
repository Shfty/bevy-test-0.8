use bevy::{
    ecs::system::CommandQueue,
    prelude::{Commands, Entity},
};
use bevy_fnord::prelude::{AddGraphVertex, Cache, Connect, Evaluate, Evaluator, Function};

use crate::prelude::{InputFloat, ToggleProjection};

pub fn camera_projection(
    commands: &mut Commands,
    switch_projection_input: Entity,
    camera_orbit: Entity,
) {
    let vertex_camera_projection_input = commands.add_graph_vertex({
        Evaluator::new(move |world| {
            world
                .get::<InputFloat>(switch_projection_input)
                .unwrap()
                .float()
                > 0.0
        })
    });

    let vertex_camera_projection_cache = commands.add_graph_vertex(Cache::<bool>::default());

    let vertex_camera_projection =
        commands.add_graph_vertex(Function::new(move |input: bool, changed: bool| {
            let mut queue = CommandQueue::default();
            if changed && input {
                queue.push(ToggleProjection {
                    camera: camera_orbit,
                });
            }
            queue
        }));

    commands
        .connect(
            vertex_camera_projection_input.output::<0, bool>()
                | vertex_camera_projection_cache.input::<0, bool>(),
        )
        .connect(
            vertex_camera_projection_cache.output::<0, bool>()
                | vertex_camera_projection.input::<0, bool>(),
        )
        .connect(
            vertex_camera_projection_cache.output::<1, bool>()
                | vertex_camera_projection.input::<1, bool>(),
        );

    commands.evaluate_mut::<0, CommandQueue>(vertex_camera_projection);
}
