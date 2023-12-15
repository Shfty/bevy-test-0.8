use bevy::{prelude::{Entity, World, Commands}, ecs::system::CommandQueue};

use crate::prelude::{GraphVertexCommands, Output, VertexOutput};

pub trait Evaluate<V> {
    fn evaluate<const N: usize, T>(&mut self, arc: GraphVertexCommands<V>) -> &mut Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity>,
        T: 'static + Send + Sync;

    fn evaluate_mut<const N: usize, T>(
        &mut self,
        arc: GraphVertexCommands<V>,
    ) -> &mut Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity, Type = CommandQueue>,
        T: 'static + Send + Sync;

    fn evaluate_with<const N: usize, T>(
        &mut self,
        arc: GraphVertexCommands<V>,
    ) -> &mut Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity, Type = CommandQueue>,
        T: 'static + Send + Sync;
}

impl<V> Evaluate<V> for Commands<'_, '_> {
    fn evaluate<const N: usize, T>(&mut self, arc: GraphVertexCommands<V>) -> &mut Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity>,
        T: 'static + Send + Sync,
    {
        arc.evaluate::<N, T>(self);
        self
    }

    fn evaluate_mut<const N: usize, T>(
        &mut self,
        arc: GraphVertexCommands<V>,
    ) -> &mut Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity, Type = CommandQueue>,
        T: 'static + Send + Sync,
    {
        arc.evaluate_mut::<N, T>(self);
        self
    }

    fn evaluate_with<const N: usize, T>(
        &mut self,
        arc: GraphVertexCommands<V>,
    ) -> &mut Self
    where
        V: VertexOutput<Output<N, T>, Context = World, Key = Entity, Type = CommandQueue>,
        T: 'static + Send + Sync,
    {
        arc.evaluate_with::<N, T>(self);
        self
    }
}

