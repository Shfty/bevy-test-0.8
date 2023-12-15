use serde::Serialize;

use crate::prelude::{EdgeOut, EdgeType};

use std::marker::PhantomData;

/// Output edge with index N and type T
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Output<const N: usize, T>(PhantomData<T>);

impl<const N: usize, T> Default for Output<N, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<const N: usize, T> EdgeOut for Output<N, T> where T: 'static + Send + Sync {}

impl<const N: usize, T> EdgeType for Output<N, T>
where
    T: 'static + Send + Sync,
{
    type Type = T;
}

/// Convenience type for hiding parameter N in single-output vertices
pub type Out<T> = Output<0, T>;

/// Convenience type for hiding parameter T in no-output vertices
pub type NoOutput = Out<()>;
