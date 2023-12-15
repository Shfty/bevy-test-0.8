use bevy::{
    ecs::system::CommandQueue,
    math::IVec3,
    prelude::{default, Commands, Entity},
};
use bevy_fnord::prelude::{
    AddGraphVertex, Cache, Connect, Evaluate, Evaluator, Function, GraphVertexCommands,
    Output, VertexOutput,
};

use crate::prelude::{
    BoardTransform, GridMove, InputFloat, LockTimer, MoveType, PushPendingGridMoves, BOARD_HEIGHT,
};

use super::InputController;

pub fn hard_drop<V>(
    commands: &mut Commands,
    controller_entity: Entity,
    hard_drop_input: Entity,
    vertex_controller_target: GraphVertexCommands<V>,
) where
    V: VertexOutput<Output<0, Option<Entity>>>,
{
    let vertex_hard_drop_input = commands.add_graph_vertex({
        Evaluator::new(move |world| world.get::<InputFloat>(hard_drop_input).unwrap().float() > 0.0)
    });

    let vertex_hard_drop_cache = commands.add_graph_vertex(Cache::<bool>::default());

    commands.connect(
        vertex_hard_drop_input.output::<0, bool>() | vertex_hard_drop_cache.input::<0, bool>(),
    );

    let vertex_lock_timer_active = commands.add_graph_vertex(Evaluator::new(move |world| {
        let controller = if let Some(controller) = world.get::<InputController>(controller_entity) {
            controller
        } else {
            return false;
        };

        let target = if let Some(target) = controller.target {
            target
        } else {
            return false;
        };

        if let Some(timer) = world.get::<LockTimer>(target) {
            !timer.paused()
        } else {
            false
        }
    }));

    let vertex_hard_drop = commands.add_graph_vertex(Function::new(
        |value: bool, changed: bool, lock_timer_active: bool, target: Option<Entity>| {
            let mut queue = CommandQueue::default();

            if lock_timer_active {
                return queue;
            }

            let target = if let Some(target) = target {
                target
            } else {
                return queue;
            };

            if value && changed {
                queue.push(PushPendingGridMoves {
                    pending_grid_moves: target,
                    moves: std::iter::repeat(GridMove {
                        delta: BoardTransform {
                            translation: -IVec3::Y,
                            ..default()
                        },
                        move_type: MoveType::Land,
                    })
                    .take(BOARD_HEIGHT)
                    .collect(),
                });
            }

            queue
        },
    ));

    commands
        .connect(vertex_hard_drop_cache.output::<0, bool>() | vertex_hard_drop.input::<0, bool>());
    commands
        .connect(vertex_hard_drop_cache.output::<1, bool>() | vertex_hard_drop.input::<1, bool>());
    commands.connect(
        vertex_lock_timer_active.output::<0, bool>() | vertex_hard_drop.input::<2, bool>(),
    );
    commands.connect(
        vertex_controller_target.output::<0, Option<Entity>>()
            | vertex_hard_drop.input::<3, Option<Entity>>(),
    );
    commands.evaluate_mut::<0, CommandQueue>(vertex_hard_drop);
}
