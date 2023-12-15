use crate::prelude::EdgeType;

pub trait EdgeIn: EdgeType + Sized
where
    Self::Type: 'static + Send + Sync,
{
}
