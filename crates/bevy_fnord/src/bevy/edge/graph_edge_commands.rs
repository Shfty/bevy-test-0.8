use bevy::prelude::{default, Commands, Entity};

use crate::prelude::{ArcCommand, Input, Output};

use std::{marker::PhantomData, ops::BitOr};

pub struct GraphEdgeCommands<V, const N: usize, T> {
    pub vertex: Entity,
    pub commands: Vec<Box<dyn FnOnce(&mut Commands)>>,
    pub _phantom: PhantomData<(V, T)>,
}

impl<V, const N: usize, T> GraphEdgeCommands<V, N, T> {
    pub fn connect(self, commands: &mut Commands) {
        for command in self.commands {
            command(commands)
        }
    }
}

impl<IV, const IN: usize, OV, const ON: usize, T> BitOr<GraphEdgeCommands<IV, IN, T>>
    for GraphEdgeCommands<OV, ON, T>
where
    T: 'static + Send + Sync,
{
    type Output = GraphEdgeCommands<IV, IN, T>;

    fn bitor(mut self, mut rhs: GraphEdgeCommands<IV, IN, T>) -> Self::Output {
        let vertex_in = rhs.vertex;
        let vertex_out = self.vertex;
        self.commands.push(Box::new(move |commands: &mut Commands| {
            let arc = commands.spawn().id();
            commands.add(ArcCommand::<Output<ON, T>, Input<IN, T>> {
                vertex_out,
                vertex_in,
                arc,
                ..default()
            });
        }));
        rhs.commands.extend(self.commands);
        rhs
    }
}

pub struct GraphThroughCommands<V, const ON: usize, const IN: usize, T> {
    pub vertex: Entity,
    pub commands: Vec<Box<dyn FnOnce(&mut Commands)>>,
    pub _phantom: PhantomData<(V, T)>,
}

impl<IV, const IN: usize, OV, const ON: usize, T, const X: usize>
    BitOr<GraphThroughCommands<IV, IN, X, T>> for GraphEdgeCommands<OV, ON, T>
where
    T: 'static + Send + Sync,
{
    type Output = GraphThroughCommands<IV, IN, X, T>;

    fn bitor(mut self, mut rhs: GraphThroughCommands<IV, IN, X, T>) -> Self::Output {
        let vertex_in = rhs.vertex;
        let vertex_out = self.vertex;
        self.commands.push(Box::new(move |commands: &mut Commands| {
            let arc = commands.spawn().id();
            commands.add(ArcCommand::<Output<ON, T>, Input<IN, T>> {
                vertex_out,
                vertex_in,
                arc,
                ..default()
            });
        }));
        rhs.commands.extend(self.commands);
        rhs
    }
}

impl<IV, const IN: usize, OV, const ON: usize, T, const X: usize>
    BitOr<GraphEdgeCommands<IV, IN, T>> for GraphThroughCommands<OV, X, ON, T>
where
    T: 'static + Send + Sync,
{
    type Output = GraphEdgeCommands<IV, IN, T>;

    fn bitor(mut self, mut rhs: GraphEdgeCommands<IV, IN, T>) -> Self::Output {
        let vertex_in = rhs.vertex;
        let vertex_out = self.vertex;
        self.commands.push(Box::new(move |commands: &mut Commands| {
            let arc = commands.spawn().id();
            commands.add(ArcCommand::<Output<ON, T>, Input<IN, T>> {
                vertex_out,
                vertex_in,
                arc,
                ..default()
            });
        }));
        rhs.commands.extend(self.commands);
        rhs
    }
}

impl<IV, const IN: usize, OV, const ON: usize, T, const X: usize>
    BitOr<GraphThroughCommands<IV, X, IN, T>> for GraphThroughCommands<OV, ON, X, T>
where
    T: 'static + Send + Sync,
{
    type Output = GraphThroughCommands<IV, X, IN, T>;

    fn bitor(mut self, mut rhs: GraphThroughCommands<IV, X, IN, T>) -> Self::Output {
        let vertex_in = rhs.vertex;
        let vertex_out = self.vertex;
        self.commands.push(Box::new(move |commands: &mut Commands| {
            let arc = commands.spawn().id();
            commands.add(ArcCommand::<Output<ON, T>, Input<IN, T>> {
                vertex_out,
                vertex_in,
                arc,
                ..default()
            });
        }));
        rhs.commands.extend(self.commands);
        rhs
    }
}
