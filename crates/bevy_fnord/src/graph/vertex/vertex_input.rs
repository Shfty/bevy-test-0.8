use crate::prelude::EdgeIn;

/// Marks T as an input edge for Self
pub trait VertexInput<E>: 'static + Send + Sync
where
    E: EdgeIn,
    E::Type: 'static + Send + Sync,
{
    type Type;
}
