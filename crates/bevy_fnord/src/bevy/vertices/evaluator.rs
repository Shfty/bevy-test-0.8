use bevy::prelude::{debug, Component, Entity, World};

use crate::{
    prelude::{Edges, Out, VertexOutput},
    Cons,
};

/// Vertex that calls a function that takes &World and produces some value T
#[derive(Copy, Clone, Component)]
pub struct Evaluator<F = fn(&World), T = ()>
where
    F: 'static + Send + Sync + Fn(&World) -> T,
    T: 'static + Send + Sync,
{
    f: F,
}

impl<F, T> Evaluator<F, T>
where
    F: 'static + Send + Sync + Fn(&World) -> T,
    T: 'static + Send + Sync,
{
    pub fn new(f: F) -> Self {
        Evaluator { f }
    }
}

impl<F, T> Edges for Evaluator<F, T>
where
    F: 'static + Send + Sync + Fn(&World) -> T,
    T: 'static + Send + Sync,
{
    type Inputs = Cons![];
    type Outputs = Cons![(Self, Out<T>)];
}

impl<F, T> VertexOutput<Out<T>> for Evaluator<F, T>
where
    F: 'static + Send + Sync + Fn(&World) -> T,
    T: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T;

    fn evaluate(world: &World, entity: Entity) -> T {
        debug!(
            "Evaluate Evaluator {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        let evaluator = world.get::<Self>(entity).expect("Invalid Evaluator Vertex");
        (evaluator.f)(world)
    }
}
