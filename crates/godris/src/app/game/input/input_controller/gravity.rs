use bevy::{
    ecs::system::CommandQueue,
    math::IVec3,
    prelude::{default, info, Commands, Entity},
};
use bevy_fnord::prelude::{
    AddGraphVertex, Cache, Connect, Evaluate, Evaluator, Function, GraphVertexCommands, Output,
    VertexOutput,
};

use crate::prelude::{
    AutorepeatSetTickRate, BoardTransform, DelayedAutoRepeat, GridMove, InputController,
    InputFloat, LockTimer, MoveType, PushPendingGridMoves, DAS_RATE,
};

pub fn gravity<V>(
    commands: &mut Commands,
    soft_drop_input: Entity,
    controller_entity: Entity,
    gravity_entity: Entity,
    vertex_controller_target: GraphVertexCommands<V>,
) where
    V: VertexOutput<Output<0, Option<Entity>>>,
{
    let mut gravity_autorepeat = DelayedAutoRepeat::new(0., 2.);
    gravity_autorepeat.start();

    // Update gravity repeat rate based on soft drop input and lock timer
    let vertex_soft_drop_input = commands.add_graph_vertex(Evaluator::new(move |world| {
        world.get::<InputFloat>(soft_drop_input).unwrap().float() > 0.0
    }));

    let vertex_lock_timer_active = commands.add_graph_vertex(Evaluator::new(move |world| {
        let controller = if let Some(controller) = world.get::<InputController>(controller_entity) {
            controller
        } else {
            return false;
        };

        let entity = if let Some(value) = controller.target {
            value
        } else {
            return false;
        };

        let lock_timer = if let Some(lock_timer) = world.get::<LockTimer>(entity) {
            lock_timer
        } else {
            return false;
        };

        !lock_timer.paused()
    }));

    let vertex_auto_repeat_rate_gravity = commands.add_graph_vertex(Function::new(
        move |value: bool, lock_timer_active: bool| {
            let mut queue = CommandQueue::default();

            queue.push(AutorepeatSetTickRate {
                autorepeat: gravity_entity,
                tick_rate: if value {
                    DAS_RATE
                } else if lock_timer_active {
                    0.0
                } else {
                    2.0
                },
            });

            queue
        },
    ));

    commands
        .connect(
            vertex_soft_drop_input.output::<0, bool>()
                | vertex_auto_repeat_rate_gravity.input::<0, bool>(),
        )
        .connect(
            vertex_lock_timer_active.output::<0, bool>()
                | vertex_auto_repeat_rate_gravity.input::<1, bool>(),
        );

    // Start / stop autorepeat based on controller target validity
    let vertex_controller_target_valid = commands.add_graph_vertex(Evaluator::new(move |world| {
        let controller = if let Some(controller) = world.get::<InputController>(controller_entity) {
            controller
        } else {
            return false;
        };

        !controller.target.is_some()
    }));

    let vertex_controller_target_valid_cache = commands.add_graph_vertex(Cache::<bool>::default());

    let vertex_auto_repeat_active = commands.add_graph_vertex(Function::new(
        move |auto_repeat_active: bool, auto_repeat_changed: bool| {
            let mut queue = CommandQueue::default();

            if auto_repeat_changed {
                if auto_repeat_active {
                    info!("Stopping gravity autorepeat");
                    queue.push(crate::prelude::AutorepeatStop {
                        autorepeat: gravity_entity,
                    });
                } else {
                    info!("Starting gravity autorepeat");
                    queue.push(crate::prelude::AutorepeatStart {
                        autorepeat: gravity_entity,
                    });
                }
            }

            queue
        },
    ));

    commands
        .connect(
            vertex_controller_target_valid.output::<0, bool>()
                | vertex_controller_target_valid_cache.through::<0, 0, bool>()
                | vertex_auto_repeat_active.input::<0, bool>(),
        )
        .connect(
            vertex_controller_target_valid_cache.output::<1, bool>()
                | vertex_auto_repeat_active.input::<1, bool>(),
        );

    // Send a move whenever the autorepeat timer ticks
    let vertex_auto_repeat_tick = commands.add_graph_vertex(Evaluator::new(move |world| {
        world
            .entity(gravity_entity)
            .get::<DelayedAutoRepeat>()
            .unwrap()
            .just_finished()
    }));

    let vertex_auto_repeat_moves_gravity = commands.add_graph_vertex(Function::new({
        move |tick: bool, soft_drop: bool, target: Option<Entity>| {
            let mut queue = CommandQueue::default();

            if !tick {
                return queue;
            }

            let target = if let Some(target) = target {
                target
            } else {
                return queue;
            };

            queue.push(PushPendingGridMoves {
                pending_grid_moves: target,
                moves: vec![GridMove {
                    delta: BoardTransform {
                        translation: -IVec3::Y,
                        ..default()
                    },
                    move_type: if soft_drop {
                        MoveType::Lock
                    } else {
                        MoveType::Land
                    },
                    ..default()
                }],
            });

            queue
        }
    }));

    commands
        .connect(
            vertex_auto_repeat_tick.output::<0, bool>()
                | vertex_auto_repeat_moves_gravity.input::<0, bool>(),
        )
        .connect(
            vertex_soft_drop_input.output::<0, bool>()
                | vertex_auto_repeat_moves_gravity.input::<1, bool>(),
        )
        .connect(
            vertex_controller_target.output::<0, Option<Entity>>()
                | vertex_auto_repeat_moves_gravity.input::<2, Option<Entity>>(),
        );

    commands
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_rate_gravity)
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_active)
        .evaluate_mut::<0, CommandQueue>(vertex_auto_repeat_moves_gravity);

    commands.entity(gravity_entity).insert(gravity_autorepeat);
}
