use bevy::{prelude::{default, Commands, Component}, hierarchy::ChildBuilder};

use crate::prelude::{AddInputs, AddOutputs, Edges, GraphVertexCommands};

pub trait AddGraphVertex<'w, 's, 'a> {
    fn add_graph_vertex<V>(&'a mut self, vertex: V) -> GraphVertexCommands<V>
    where
        V: Edges + Component;

    fn init_graph_vertex_new<V>(&'a mut self) -> GraphVertexCommands<V>
    where
        V: Default + Edges + Component,
    {
        self.add_graph_vertex(default())
    }
}

impl<'w, 's, 'a> AddGraphVertex<'w, 's, 'a> for Commands<'w, 's> {
    fn add_graph_vertex<V>(&'a mut self, vertex: V) -> GraphVertexCommands<V>
    where
        V: Edges + Component,
    {
        let mut commands = self.spawn();

        commands.insert(vertex);

        V::Inputs::add_inputs(&mut commands);
        V::Outputs::add_outputs(&mut commands);

        GraphVertexCommands {
            vertex: commands.id(),
            _phantom: default(),
        }
    }
}

impl<'w, 's, 'a> AddGraphVertex<'w, 's, 'a> for ChildBuilder<'w, 's, 'a> {
    fn add_graph_vertex<V>(&'a mut self, vertex: V) -> GraphVertexCommands<V>
    where
        V: Edges + Component,
    {
        let mut commands = self.spawn();

        commands.insert(vertex);

        V::Inputs::add_inputs(&mut commands);
        V::Outputs::add_outputs(&mut commands);

        GraphVertexCommands {
            vertex: commands.id(),
            _phantom: default(),
        }
    }
}
