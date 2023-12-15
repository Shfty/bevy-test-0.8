use bevy::prelude::{Component, Entity, World};

use crate::prelude::EvaluateOutEdge;

/// A connection between two edges
#[derive(Copy, Clone, Component)]
pub struct GraphArcEvaluate<T>
where
    T: 'static + Send + Sync,
{
    pub evaluate: fn(&World, Entity) -> T,
}

impl<V> GraphArcEvaluate<V>
where
    V: 'static + Send + Sync,
{
    pub fn new<E: EvaluateOutEdge<Type = V>>() -> Self {
        Self {
            evaluate: E::evaluate_out,
        }
    }
}

