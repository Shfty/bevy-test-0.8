use std::{fmt::Debug, marker::PhantomData};

use bevy::{
    ecs::reflect::ReflectComponent,
    log::Level,
    prelude::{debug, default, error, info, trace, warn, Component, Entity, World},
    reflect::Reflect,
};

use crate::{prelude::{
    Edges,
    EvaluateInEdge, In, Out, VertexInput, VertexOutput,
}, Cons};

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Log<T>
where
    T: 'static + Send + Sync + Debug,
{
    #[reflect(ignore)]
    pub level: Level,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for Log<T>
where
    T: 'static + Send + Sync + Debug,
{
    fn default() -> Self {
        Self {
            level: Level::INFO,
            _phantom: default(),
        }
    }
}

impl<T> Log<T>
where
    T: 'static + Send + Sync + Debug,
{
    pub fn new(level: Level) -> Self {
        Log { level, ..default() }
    }

    pub fn log(&self, args: std::fmt::Arguments) {
        match self.level {
            Level::TRACE => trace!("{args:}"),
            Level::DEBUG => debug!("{args:}"),
            Level::INFO => info!("{args:}"),
            Level::WARN => warn!("{args:}"),
            Level::ERROR => error!("{args:}"),
        };
    }
}

impl<T> Edges for Log<T>
where
    T: 'static + Send + Sync + Debug,
{
    type Inputs = Cons![In<T>];
    type Outputs = Cons![(Self, Out<T>)];
}

impl<T> VertexInput<In<T>> for Log<T> where T: 'static + Send + Sync + Debug {
    type Type = T;
}

impl<'a, T> VertexOutput<Out<T>> for Log<T>
where
    T: 'static + Send + Sync + Debug,
{
    type Context = World;
    type Key = Entity;
    type Type = T;

    fn evaluate(world: &World, entity: Entity) -> T {
        debug!(
            "Evaluate Log {} for {entity:?}",
            std::any::type_name::<Self>()
        );
        let log = world.get::<Log<T>>(entity).expect("Invalid Log Vertex");

        let value = In::<T>::evaluate_in(world, entity);

        log.log(format_args!("{value:?}"));

        value
    }
}
