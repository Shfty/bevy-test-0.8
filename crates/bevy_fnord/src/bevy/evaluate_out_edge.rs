use bevy::prelude::{debug, Entity, World};

use crate::prelude::{EdgeEvaluateVertex, EdgeOut};

pub trait EvaluateOutEdge: 'static + EdgeOut {
    fn evaluate_out(world: &World, entity: Entity) -> Self::Type {
        debug!("Evaluate Out {} {entity:?}", std::any::type_name::<Self>());

        let evaluate = world
            .get::<EdgeEvaluateVertex<Self>>(entity)
            .unwrap_or_else(|| panic!("No EdgeEvaluateVertex for edge {entity:?}"));
        debug!("Out Edge Valid");

        (evaluate.evaluate_vertex)(world, entity)
    }
}

impl<T> EvaluateOutEdge for T where T: 'static + EdgeOut {}
