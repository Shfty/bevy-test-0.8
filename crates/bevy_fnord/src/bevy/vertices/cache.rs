use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{debug, default, Component, Entity, World},
    reflect::Reflect,
};

use crate::{
    prelude::{
        AddInputs, AddOutputs, Edges, EvaluateInEdge, In, Out, Output, VertexInput, VertexOutput,
    },
    Cons,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub type Value<T> = Output<0, T>;
pub type Changed = Output<1, bool>;

/// Graph vertex that caches whatever value passes through it
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Cache<T: 'static + Send + Sync> {
    #[reflect(ignore)]
    pub value: Arc<Mutex<Option<T>>>,
    #[reflect(ignore)]
    pub changed: AtomicBool,
}

impl<T> Default for Cache<T>
where
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            value: default(),
            changed: default(),
        }
    }
}

impl<T> Cache<T>
where
    T: 'static + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Cache {
            value: Arc::new(Mutex::new(Some(value))),
            changed: AtomicBool::new(false),
        }
    }
}

impl<T> Cache<T>
where
    T: 'static + Send + Sync,
{
    pub fn value(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.lock().unwrap().clone()
    }

    pub fn changed(&self) -> bool {
        self.changed.load(Ordering::Relaxed)
    }
}

impl<T> Edges for Cache<T>
where
    T: 'static + Send + Sync + Clone + PartialEq,
    Cons![In<T>]: AddInputs,
    Cons![(Self, Output<0, T>), (Self, Output<1, bool>)]: AddOutputs,
{
    type Inputs = Cons![In<T>];
    type Outputs = Cons![(Self, Output<0, T>), (Self, Output<1, bool>)];
}

impl<T> VertexInput<In<T>> for Cache<T> where T: 'static + Send + Sync {
    type Type = T;
}

impl<T> VertexOutput<Out<T>> for Cache<T>
where
    T: 'static + Send + Sync + Clone + PartialEq + std::fmt::Debug,
{
    type Context = World;
    type Key = Entity;
    type Type = T;

    fn evaluate(world: &World, entity: Entity) -> T {
        debug!(
            "Evaluate Cache {} for {entity:?}",
            std::any::type_name::<Self>()
        );

        let output = world.get::<Cache<T>>(entity).expect("Invalid Value Vertex");
        output.changed.store(false, Ordering::Relaxed);

        let value = In::<T>::evaluate_in(world, entity);
        let mut mutex = output.value.lock().unwrap();
        if mutex.as_ref() != Some(&value) {
            *mutex = Some(value.clone());
            output.changed.store(true, Ordering::Relaxed);
        }
        value
    }
}

impl<T> VertexOutput<Changed> for Cache<T>
where
    T: 'static + Send + Sync + Clone + PartialEq,
{
    type Context = World;
    type Key = Entity;
    type Type = bool;

    fn evaluate(world: &World, entity: Entity) -> bool {
        debug!(
            "Evaluate Cache {} for {entity:?}",
            std::any::type_name::<Self>()
        );

        let output = world.get::<Cache<T>>(entity).expect("Invalid Value Vertex");
        output.changed.load(Ordering::Relaxed)
    }
}
