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
use strude::prelude::Destructure4 as Destructure4Trait;

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
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

impl<T> Edges for Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Inputs = Cons![In<T>];
    type Outputs = Cons![
        (Self, Output<0, T::F0>),
        (Self, Output<1, T::F1>),
        (Self, Output<2, T::F2>),
        (Self, Output<3, T::F3>),
    ];
}

impl<T> VertexInput<In<T>> for Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Type = T;
}

impl<T> VertexOutput<Output<0, T::F0>> for Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F0;

    fn evaluate(world: &World, entity: Entity) -> T::F0 {
        In::<T>::evaluate_in(world, entity).f0()
    }
}

impl<T> VertexOutput<Output<1, T::F1>> for Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F1;

    fn evaluate(world: &World, entity: Entity) -> T::F1 {
        In::<T>::evaluate_in(world, entity).f1()
    }
}

impl<T> VertexOutput<Output<2, T::F2>> for Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F2;

    fn evaluate(world: &World, entity: Entity) -> T::F2 {
        In::<T>::evaluate_in(world, entity).f2()
    }
}

impl<T> VertexOutput<Output<3, T::F3>> for Destructure4<T>
where
    T: 'static + Send + Sync + Default + Destructure4Trait,
    T::F0: 'static + Send + Sync,
    T::F1: 'static + Send + Sync,
    T::F2: 'static + Send + Sync,
    T::F3: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Type = T::F3;

    fn evaluate(world: &World, entity: Entity) -> T::F3 {
        debug!(
            "Evaluate Destructure4 {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        In::<T>::evaluate_in(world, entity).f3()
    }
}
