use bevy::{
    ecs::component::TableStorage,
    prelude::{default, Component, ReflectComponent},
    reflect::{
        erased_serde::Serialize, serde::Serializable, DynamicTupleStruct, FromType,
        GetTypeRegistration, Reflect, ReflectMut, ReflectRef, TupleStruct, TupleStructFieldIter,
        TypeRegistration,
    },
};

use crate::{phantom_tuple_n_reflect, prelude::Output};

impl<const N: usize, T> Component for Output<N, T>
where
    T: 'static + Send + Sync,
{
    type Storage = TableStorage;
}

phantom_tuple_n_reflect!(Output);
