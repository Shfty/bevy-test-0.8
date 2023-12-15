use bevy::prelude::Commands;

use crate::prelude::GraphEdgeCommands;

pub trait Connect {
    fn connect<V, const N: usize, T>(&mut self, arc: GraphEdgeCommands<V, N, T>) -> &mut Self;
}

impl Connect for Commands<'_, '_> {
    fn connect<V, const N: usize, T>(&mut self, arc: GraphEdgeCommands<V, N, T>) -> &mut Self {
        arc.connect(self);
        self
    }
}
