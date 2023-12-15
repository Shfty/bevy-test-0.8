use bevy::{
    ecs::system::CommandQueue,
    prelude::{default, Commands, Entity, World},
};

use crate::prelude::{
    EvaluateEdge, EvaluateEdgeMut, EvaluateVertex, GraphEdgeCommands, GraphThroughCommands, Input,
    Output, VertexInput, VertexOutput,
};

use std::{fmt::Debug, marker::PhantomData};

#[derive(Debug)]
pub struct GraphVertexCommands<V> {
    pub vertex: Entity,
    pub _phantom: PhantomData<V>,
}

impl<V> Copy for GraphVertexCommands<V> {}

impl<V> Clone for GraphVertexCommands<V> {
    fn clone(&self) -> Self {
        Self {
            vertex: self.vertex.clone(),
            _phantom: default(),
        }
    }
}

impl<V> GraphVertexCommands<V>
where
    V: 'static,
{
    pub fn input<const N: usize, T>(&self) -> GraphEdgeCommands<V, N, T>
    where
        V: VertexInput<Input<N, T>>,
        T: 'static + Send + Sync,
    {
        GraphEdgeCommands {
            vertex: self.vertex,
            commands: default(),
            _phantom: default(),
        }
    }

    pub fn output<const N: usize, T>(&self) -> GraphEdgeCommands<V, N, T>
    where
        V: VertexOutput<Output<N, T>>,
        T: 'static + Send + Sync,
    {
        GraphEdgeCommands {
            vertex: self.vertex,
            commands: default(),
            _phantom: default(),
        }
    }

    pub fn through<const IN: usize, const ON: usize, T>(&self) -> GraphThroughCommands<V, IN, ON, T>
    where
        V: VertexInput<Input<IN, T>>,
        V: VertexOutput<Output<ON, T>>,
        T: 'static + Send + Sync,
    {
        GraphThroughCommands {
            vertex: self.vertex,
            commands: default(),
            _phantom: default(),
        }
    }

    pub fn evaluate<const N: usize, T>(self, commands: &mut Commands) -> Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity>,
        T: 'static + Send + Sync,
    {
        commands
            .entity(self.vertex)
            .insert(EvaluateEdge::new::<V, Output<N, T>>());
        self
    }

    /// Mark this edge for system evaluation
    pub fn evaluate_mut<const N: usize, T>(self, commands: &mut Commands) -> Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity, Type = CommandQueue>,
        T: 'static + Send + Sync,
    {
        commands
            .entity(self.vertex)
            .insert(EvaluateEdgeMut::new::<V, Output<N, T>>());
        self
    }

    pub fn evaluate_with<const N: usize, T>(self, commands: &mut Commands) -> Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity>,
        T: 'static + Send + Sync,
    {
        commands
            .entity(self.vertex)
            .insert(EvaluateVertex::<V>::default());
        self
    }
}
