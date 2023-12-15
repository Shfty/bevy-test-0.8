use crate::prelude::EdgeOut;

/// Marks T as an output edge for Self
pub trait VertexOutput<T>: 'static + Send + Sync
where
    T: EdgeOut,
{
    type Context;
    type Key;
    type Type;

    fn evaluate(context: &Self::Context, key: Self::Key) -> Self::Type;
}
