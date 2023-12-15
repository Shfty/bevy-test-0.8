use std::fmt::Display;

use bevy::{
    ecs::{schedule::ParallelSystemDescriptor, system::AsSystemLabel},
    prelude::{error, In, IntoChainSystem, IntoSystem, ParallelSystemDescriptorCoercion},
};

pub trait ResultSystem<P, M, T, E> {
    type ResultSystem;
    fn result_system(self) -> Self::ResultSystem;
}

impl<Params, Marker, T, E, F> ResultSystem<Params, Marker, T, E> for F
where
    T: 'static,
    E: 'static + Display,
    F: 'static + Copy + IntoSystem<(), Result<T, E>, Params> + AsSystemLabel<Marker>,
{
    type ResultSystem = ParallelSystemDescriptor;

    fn result_system(self) -> Self::ResultSystem {
        self.chain(handle_error).label(self.as_system_label())
    }
}

pub fn handle_error<T, E: Display>(In(result): In<Result<T, E>>) {
    if let Err(e) = result {
        error!("{e:}")
    }
}
