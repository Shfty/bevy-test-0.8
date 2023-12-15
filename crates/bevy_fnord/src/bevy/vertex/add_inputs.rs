use bevy::ecs::system::EntityCommands;

use crate::prelude::{Cell, Cons, EdgeArc, EdgeEvaluateEdge, EvaluateInEdge};

pub trait AddInputs {
    fn add_inputs(commands: &mut EntityCommands);
}

impl<E, N> AddInputs for Cell![E, N]
where
    E: EvaluateInEdge,
    E::Type: 'static + Send + Sync,
    N: AddInputs,
{
    fn add_inputs(commands: &mut EntityCommands) {
        commands
            .insert(EdgeArc::<E>::default())
            .insert(EdgeEvaluateEdge::<E>::input());

        N::add_inputs(commands);
    }
}

impl AddInputs for Cons![] {
    fn add_inputs(_: &mut EntityCommands) {}
}

