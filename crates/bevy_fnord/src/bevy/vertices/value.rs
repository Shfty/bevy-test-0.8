use std::marker::PhantomData;

use bevy::prelude::{debug, default, Component, Entity, World};

use crate::{
    prelude::{Edges, Out, VertexOutput},
    Cons,
};

pub struct ByCopy;
pub struct ByClone;

/// Vertex that copies T upon evaluation
#[derive(Debug, Default, Clone, Component)]
pub struct Value<O: 'static + Send + Sync, T: 'static + Send + Sync> {
    pub value: T,
    _phantom: PhantomData<O>,
}

impl<O, T> Value<O, T>
where
    O: 'static + Send + Sync,
    T: 'static + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Value {
            value,
            _phantom: default(),
        }
    }
}

impl<T> Edges for Value<ByCopy, T>
where
    T: 'static + Send + Sync + Copy,
{
    type Inputs = Cons![];
    type Outputs = Cons![(Self, Out<T>)];
}

impl<T> VertexOutput<Out<T>> for Value<ByCopy, T>
where
    T: 'static + Send + Sync + Copy,
{
    type Context = World;
    type Key = Entity;
    type Type = T;

    fn evaluate(world: &World, entity: Entity) -> T {
        world
            .get::<Value<ByCopy, T>>(entity)
            .expect("Invalid Value<ByCopy, T> Vertex")
            .value
    }
}

impl<T> Edges for Value<ByClone, T>
where
    T: 'static + Send + Sync + Clone,
{
    type Inputs = Cons![];
    type Outputs = Cons![(Self, Out<T>)];
}

impl<T> VertexOutput<Out<T>> for Value<ByClone, T>
where
    T: 'static + Send + Sync + Clone,
{
    type Context = World;
    type Key = Entity;
    type Type = T;

    fn evaluate(world: &World, entity: Entity) -> T {
        debug!(
            "Evaluate Value {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        world
            .get::<Value<ByClone, T>>(entity)
            .expect("Invalid Value<ByClone, T> Vertex")
            .value
            .clone()
    }
}
