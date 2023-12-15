use bevy::prelude::{Component, Entity, World};

use crate::prelude::{EdgeType, EvaluateOutEdge, VertexOutput};

/// Reference to the Arc connected to a given Edge
#[derive(Component)]
pub struct EdgeEvaluateVertex<T>
where
    T: EdgeType,
{
    pub evaluate_vertex: fn(&World, Entity) -> T::Type,
}

impl<E> EdgeEvaluateVertex<E>
where
    E: EvaluateOutEdge,
{
    pub fn new<V>() -> Self
    where
        V: VertexOutput<E, Context = World, Key = Entity, Type = E::Type>,
    {
        EdgeEvaluateVertex {
            evaluate_vertex: V::evaluate,
        }
    }
}
