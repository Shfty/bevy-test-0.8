use std::marker::PhantomData;

use bevy::prelude::{default, Component, Entity, Query, With, World};

use crate::prelude::{EdgeEvaluateEdge, EdgeOut, VertexOutput};

/// Bevy component for evaluating a graph via a vertex and edge specified at system creation
#[derive(Debug, Copy, Clone, Component)]
pub struct EvaluateVertex<V> {
    _phantom: PhantomData<V>,
}

impl<V> Default for EvaluateVertex<V> {
    fn default() -> Self {
        Self {
            _phantom: default(),
        }
    }
}

/// Evaluate a vertex edge via type
pub fn evaluate_with<'a, V, E>(
    world: &'static World,
    query: Query<(Entity, &EdgeEvaluateEdge<E>), With<EvaluateVertex<V>>>,
) where
    V: VertexOutput<E, Context = World, Key = Entity>,
    E: EdgeOut + Component,
{
    for (entity, evaluate) in query.iter() {
        (evaluate.evaluate_edge)(world, entity);
    }
}
