use bevy::{
    ecs::system::CommandQueue,
    prelude::{Commands, Entity},
};
use bevy_fnord::prelude::{AddGraphVertex, Cache, Connect, Evaluate, Evaluator, Function};

use crate::prelude::{InputFloat, ToggleFocus};

pub fn camera_focus(commands: &mut Commands, switch_focus_input: Entity, camera_strafe: Entity) {
    let vertex_camera_focus_input = commands.add_graph_vertex({
        let camera_focus = switch_focus_input;
        Evaluator::new(move |world| world.get::<InputFloat>(camera_focus).unwrap().float() > 0.0)
    });

    let vertex_camera_focus_cache = commands.add_graph_vertex(Cache::<bool>::default());

    let vertex_camera_focus =
        commands.add_graph_vertex(Function::new(move |input: bool, changed: bool| {
            let mut queue = CommandQueue::default();
            if changed && input {
                queue.push(ToggleFocus {
                    camera: camera_strafe,
                });
            }
            queue
        }));

    commands
        .connect(
            vertex_camera_focus_input.output::<0, bool>()
                | vertex_camera_focus_cache.input::<0, bool>(),
        )
        .connect(
            vertex_camera_focus_cache.output::<0, bool>() | vertex_camera_focus.input::<0, bool>(),
        )
        .connect(
            vertex_camera_focus_cache.output::<1, bool>() | vertex_camera_focus.input::<1, bool>(),
        );

    commands.evaluate_mut::<0, CommandQueue>(vertex_camera_focus);
}
