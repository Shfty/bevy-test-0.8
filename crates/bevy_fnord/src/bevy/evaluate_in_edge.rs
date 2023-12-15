use bevy::prelude::{debug, Component, Entity, World};

use crate::prelude::{EdgeArc, EdgeIn, GraphArcEvaluate, GraphArcOutEdge};

pub trait EvaluateInEdge: EdgeIn + Component
where
    Self::Type: 'static + Send + Sync,
{
    fn evaluate_in(world: &World, entity: Entity) -> Self::Type {
        debug!("Evaluate In {} {entity:?}", std::any::type_name::<Self>());

        let edge_arc = world
            .get::<EdgeArc<Self>>(entity)
            .unwrap_or_else(|| panic!("No EdgeArc for Edge {entity:?}"));
        debug!("In EdgeArc Valid");
        debug!("Edge arc: {:?}", edge_arc.arc);

        let evaluate = world
            .get::<GraphArcEvaluate<Self::Type>>(edge_arc.arc)
            .unwrap_or_else(|| panic!("No GraphArcEvaluate for {:?}", edge_arc.arc));
        debug!("In GraphArcEvaluate Valid");

        let edge_out = world
            .get::<GraphArcOutEdge>(edge_arc.arc)
            .unwrap_or_else(|| panic!("No GraphArcOutEdge for {:?}", edge_arc.arc));
        debug!("In GraphArcOutEdge Valid");
        debug!("Edge out: {:?}", edge_out.edge_out);

        (evaluate.evaluate)(world, edge_out.edge_out)
    }
}

impl<T> EvaluateInEdge for T
where
    T: EdgeIn + Component,
    T::Type: 'static + Send + Sync,
{
}
