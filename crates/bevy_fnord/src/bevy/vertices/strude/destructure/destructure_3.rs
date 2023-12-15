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
use strude::prelude::Destructure3 as Destructure3Trait;

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Destructure3<T>
where
    T: 'static + Send + Sync + Destructure3Trait,
{
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for Destructure3<T>
where
    T: 'static + Send + Sync + Destructure3Trait,
{
    fn default() -> Self {
        Destructure3::<T> {
            _phantom: default(),
        }
    }
}

impl<T> Edges for Destructure3<T>
where
    T: 'static + Send + Sync + Destructure3Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
{
    type Inputs = Cons![In<T>];
    type Outputs = Cons![
        (Self, Output<0, T::F0>),
        (Self, Output<1, T::F1>),
        (Self, Output<2, T::F2>)
    ];
}

impl<T> VertexInput<In<T>> for Destructure3<T>
where
    T: 'static + Send + Sync + Destructure3Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
{
    type Type = T;
}

impl<T> VertexOutput<Output<0, T::F0>> for Destructure3<T>
where
    T: 'static + Send + Sync + Destructure3Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F0;

    fn evaluate(world: &World, entity: Entity) -> T::F0 {
        In::<T>::evaluate_in(world, entity).f0()
    }
}

impl<T> VertexOutput<Output<1, T::F1>> for Destructure3<T>
where
    T: 'static + Send + Sync + Destructure3Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F1;

    fn evaluate(world: &World, entity: Entity) -> T::F1 {
        In::<T>::evaluate_in(world, entity).f1()
    }
}

impl<T> VertexOutput<Output<2, T::F2>> for Destructure3<T>
where
    T: 'static + Send + Sync + Destructure3Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F2;

    fn evaluate(world: &World, entity: Entity) -> T::F2 {
        debug!(
            "Evaluate Destructure3 {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        In::<T>::evaluate_in(world, entity).f2()
    }
}
