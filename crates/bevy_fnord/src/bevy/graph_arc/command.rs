use std::marker::PhantomData;

use bevy::{
    ecs::system::Command,
    prelude::{debug, default, info, Commands, Component, Entity, World},
};
use ecs_ex::entity_default;

use crate::prelude::{EdgeArc, EdgeIn, EdgeOut, GraphArcEvaluate, GraphArcOutEdge};

#[derive(Debug, Copy, Clone)]
pub struct ArcCommand<I, O> {
    pub vertex_out: Entity,
    pub vertex_in: Entity,
    pub arc: Entity,
    pub _phantom: PhantomData<(I, O)>,
}

impl<O, I> Default for ArcCommand<O, I> {
    fn default() -> Self {
        Self {
            vertex_out: entity_default(),
            vertex_in: entity_default(),
            arc: entity_default(),
            _phantom: Default::default(),
        }
    }
}

impl<O, I> Command for ArcCommand<O, I>
where
    O: EdgeOut + Component,
    I: EdgeIn<Type = O::Type> + Component,
    O::Type: 'static + Send + Sync,
{
    fn write(self, world: &mut World) {
        info!(
            "Inserting Arc<{}> with target edge {} on {:?}",
            std::any::type_name::<O::Type>(),
            std::any::type_name::<O>(),
            self.vertex_out
        );

        let arc = world
            .entity_mut(self.arc)
            .insert(GraphArcOutEdge {
                edge_out: self.vertex_out,
            })
            .insert(GraphArcEvaluate::<O::Type>::new::<O>())
            .id();

        debug!("Getting EdgeArc from {:?}", self.vertex_out);
        let mut edge_in = world
            .get_mut::<EdgeArc<O>>(self.vertex_out)
            .expect("Invalid Out EdgeArc");

        edge_in.arc = arc;

        debug!("Getting EdgeArc from {:?}", self.vertex_in);
        let mut edge_out = world
            .get_mut::<EdgeArc<I>>(self.vertex_in)
            .expect("Invalid In EdgeArc");

        edge_out.arc = arc;
    }
}

pub trait AddGraphArc {
    fn add_graph_arc<O, I>(&mut self, edge_in: Entity, edge_out: Entity)
    where
        O: EdgeOut + Component,
        O::Type: 'static + Send + Sync,
        I: EdgeIn<Type = O::Type> + Component;
}

impl AddGraphArc for Commands<'_, '_> {
    fn add_graph_arc<O, I>(&mut self, vertex_in: Entity, vertex_out: Entity)
    where
        O: EdgeOut + Component,
        O::Type: 'static + Send + Sync,
        I: EdgeIn<Type = O::Type> + Component,
    {
        let arc = self.spawn().id();
        self.add(ArcCommand::<O, I> {
            vertex_out,
            vertex_in,
            arc,
            ..default()
        })
    }
}
