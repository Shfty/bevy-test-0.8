use crate::prelude::{EdgeIn, EdgeType};
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

pub type In<T> = Input<0, T>;

/// Basic input edge
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Input<const N: usize, T>(PhantomData<T>);

impl<const N: usize, T> Default for Input<N, T> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

impl<const N: usize, T> EdgeType for Input<N, T>
where
    T: 'static + Send + Sync,
{
    type Type = T;
}

impl<const N: usize, T> EdgeIn for Input<N, T> where T: 'static + Send + Sync {}
