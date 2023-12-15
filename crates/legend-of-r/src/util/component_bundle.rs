use bevy::prelude::{Bundle, Component};

/// Wrapper type to lift a component into a bundle
#[derive(Debug, Default, Copy, Clone, Bundle)]
pub struct ComponentBundle<T>
where
    T: Component,
{
    pub component: T,
}
