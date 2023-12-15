use bevy::{
    ecs::system::EntityCommands,
    prelude::{Entity, World},
};

use crate::prelude::{
    Cell, Cons, EdgeArc, EdgeEvaluateEdge, EdgeEvaluateVertex, EdgeOut, VertexOutput,
};

pub trait AddOutputs {
    fn add_outputs(commands: &mut EntityCommands);
}

impl<V, E, N> AddOutputs for Cell![(V, E), N]
where
    V: VertexOutput<E, Context = World, Key = Entity>,
    E: 'static + Send + Sync + EdgeOut<Type = V::Type>,
    N: AddOutputs,
{
    fn add_outputs(commands: &mut EntityCommands) {
        commands
            .insert(EdgeArc::<E>::default())
            .insert(EdgeEvaluateEdge::<E>::output::<V>())
            .insert(EdgeEvaluateVertex::<E>::new::<V>());

        N::add_outputs(commands);
    }
}

impl AddOutputs for Cons![] {
    fn add_outputs(_: &mut EntityCommands) {}
}
