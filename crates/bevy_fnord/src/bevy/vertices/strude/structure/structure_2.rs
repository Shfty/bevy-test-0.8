use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{debug, default, Component, Entity, World},
    reflect::Reflect,
};

use std::marker::PhantomData;

use crate::{
    prelude::{Edges, EvaluateInEdge, Input, Out, VertexInput, VertexOutput},
    Cons,
};
use strude::prelude::Structure2 as Structure2Trait;

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Structure2<T>
where
    T: 'static + Send + Sync + Structure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for Structure2<T>
where
    T: 'static + Send + Sync + Structure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            _phantom: default(),
        }
    }
}

impl<T> Edges for Structure2<T>
where
    T: 'static + Send + Sync + Structure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Inputs = Cons![Input<0, T::F0>, Input<1, T::F1>];
    type Outputs = Cons![(Self, Out<T>)];
}

impl<T> VertexInput<Input<0, T::F0>> for Structure2<T>
where
    T: 'static + Send + Sync + Structure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Type = T::F0;
}

impl<T> VertexInput<Input<1, T::F1>> for Structure2<T>
where
    T: 'static + Send + Sync + Structure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Type = T::F1;
}

impl<'a, T> VertexOutput<Out<T>> for Structure2<T>
where
    T: 'static + Send + Sync + Structure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T;

    fn evaluate(world: &World, entity: Entity) -> T {
        debug!(
            "Evaluate Structure2 {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        T::structure(
            Input::<0, T::F0>::evaluate_in(world, entity),
            Input::<1, T::F1>::evaluate_in(world, entity),
        )
    }
}
