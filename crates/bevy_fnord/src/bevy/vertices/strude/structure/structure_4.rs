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
use strude::prelude::Structure4 as Structure4Trait;

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            _phantom: default(),
        }
    }
}

impl<T> Edges for Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Inputs = Cons![Input<0, T::F0>, Input<1, T::F1>, Input<2, T::F2>, Input<3, T::F3>];
    type Outputs = Cons![(Self, Out<T>)];
}

impl<T> VertexInput<Input<0, T::F0>> for Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Type = T::F0;
}

impl<T> VertexInput<Input<1, T::F1>> for Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Type = T::F1;
}

impl<T> VertexInput<Input<2, T::F2>> for Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Type = T::F2;
}

impl<T> VertexInput<Input<3, T::F3>> for Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Type = T::F3;
}

impl<T> VertexOutput<Out<T>> for Structure4<T>
where
    T: 'static + Send + Sync + Structure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T;

    fn evaluate(world: &World, entity: Entity) -> T {
        debug!(
            "Evaluate Structure4 {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        T::structure(
            Input::<0, T::F0>::evaluate_in(world, entity),
            Input::<1, T::F1>::evaluate_in(world, entity),
            Input::<2, T::F2>::evaluate_in(world, entity),
            Input::<3, T::F3>::evaluate_in(world, entity),
        )
    }
}
