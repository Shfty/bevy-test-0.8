use bevy::{
    ecs::component::TableStorage,
    prelude::{default, Component, ReflectComponent},
    reflect::{
        erased_serde::Serialize, serde::Serializable, DynamicTupleStruct, FromType,
        GetTypeRegistration, Reflect, ReflectMut, ReflectRef, TupleStruct, TupleStructFieldIter,
        TypeRegistration,
    },
};

use crate::{phantom_tuple_n_reflect, prelude::Input};

impl<const N: usize, T> Component for Input<N, T>
where
    T: 'static + Send + Sync,
{
    type Storage = TableStorage;
}

phantom_tuple_n_reflect!(Input);
