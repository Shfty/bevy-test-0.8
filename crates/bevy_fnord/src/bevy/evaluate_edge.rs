use bevy::{
    ecs::system::CommandQueue,
    prelude::{Component, Entity, Query, World},
};

use crate::prelude::{EdgeOut, VertexOutput};

/// Bevy component for evaluating a graph via a vertex and edge specified at component creation
#[derive(Copy, Clone, Component)]
pub struct EvaluateEdge {
    pub evaluate: fn(&World, Entity),
}

impl EvaluateEdge {
    pub fn new<V, E>() -> Self
    where
        V: VertexOutput<E, Context = World, Key = Entity>,
        E: EdgeOut,
    {
        EvaluateEdge {
            evaluate: |world, entity| {
                V::evaluate(world, entity);
            },
        }
    }
}

/// Evaluate an edge via function pointer
pub fn evaluate_edge(world: &World, query: Query<(Entity, &EvaluateEdge)>) {
    for (entity, evaluate_graph) in query.iter() {
        (evaluate_graph.evaluate)(world, entity);
    }
}

/// Bevy component for evaluating a graph via a vertex and edge specified at component creation
#[derive(Copy, Clone, Component)]
pub struct EvaluateEdgeMut {
    pub evaluate: fn(&World, Entity) -> CommandQueue,
}

impl EvaluateEdgeMut {
    pub fn new<V, E>() -> Self
    where
        V: VertexOutput<E, Context = World, Key = Entity, Type = CommandQueue>,
        E: EdgeOut,
    {
        EvaluateEdgeMut {
            evaluate: V::evaluate,
        }
    }
}

/// Evaluate an edge via function pointer
pub fn evaluate_edge_mut(world: &mut World) {
    let mut query = world.query::<(Entity, &EvaluateEdgeMut)>();

    let mut queues = vec![];
    for (entity, evaluate_graph) in query.iter(world) {
        let command_queue = (evaluate_graph.evaluate)(world, entity);
        queues.push(command_queue);
    }

    for mut queue in queues {
        queue.apply(world);
    }
}
