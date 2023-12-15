use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{debug, default, Component, Entity, World},
    reflect::Reflect,
};

use crate::{
    prelude::{Edges, EvaluateInEdge, In, Output, VertexInput, VertexOutput},
    Cons,
};
use strude::prelude::Destructure2 as Destructure2Trait;

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Destructure2<T>
where
    T: 'static + Send + Sync + Destructure2Trait,
{
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for Destructure2<T>
where
    T: 'static + Send + Sync + Destructure2Trait,
{
    fn default() -> Self {
        Self {
            _phantom: default(),
        }
    }
}

impl<T> Edges for Destructure2<T>
where
    T: 'static + Send + Sync + Destructure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Inputs = Cons![In<T>];
    type Outputs = Cons![(Self, Output<0, T::F0>), (Self, Output<1, T::F1>)];
}

impl<T> VertexInput<In<T>> for Destructure2<T>
where
    T: 'static + Send + Sync + Destructure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Type = T;
}

impl<'a, T> VertexOutput<Output<0, T::F0>> for Destructure2<T>
where
    T: 'static + Send + Sync + Destructure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F0;

    fn evaluate(world: &World, entity: Entity) -> T::F0 {
        In::<T>::evaluate_in(world, entity).f0()
    }
}

impl<'a, T> VertexOutput<Output<1, T::F1>> for Destructure2<T>
where
    T: 'static + Send + Sync + Destructure2Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F1;

    fn evaluate(world: &World, entity: Entity) -> T::F1 {
        debug!(
            "Evaluate Destructure2 {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        In::<T>::evaluate_in(world, entity).f1()
    }
}
