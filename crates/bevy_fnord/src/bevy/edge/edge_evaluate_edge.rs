use bevy::prelude::{Component, Entity, World};

use crate::prelude::{EdgeType, EvaluateInEdge, EvaluateOutEdge, VertexOutput};

/// Reference to the Arc connected to a given Edge
#[derive(Component)]
pub struct EdgeEvaluateEdge<T>
where
    T: EdgeType,
{
    pub evaluate_edge: fn(&World, Entity) -> T::Type,
}

impl<E> EdgeEvaluateEdge<E>
where
    E: EvaluateInEdge,
    E::Type: 'static + Send + Sync,
{
    pub fn input() -> Self {
        EdgeEvaluateEdge {
            evaluate_edge: E::evaluate_in,
        }
    }
}

impl<E> EdgeEvaluateEdge<E>
where
    E: EvaluateOutEdge,
{
    pub fn output<V>() -> Self
    where
        V: VertexOutput<E, Context = World, Key = Entity, Type = E::Type>,
    {
        EdgeEvaluateEdge {
            evaluate_edge: E::evaluate_out,
        }
    }
}
