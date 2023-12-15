//pub mod function_0;
//pub mod function_1;
//pub mod function_2;
//pub mod function_3;

use bevy::prelude::{default, Component};

use bevy_fnord_macro::vertex;

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Copy, Clone, Component)]
pub struct Function<const N: usize, F, P, R> {
    pub f: F,
    pub _phantom: PhantomData<(P, R)>,
}

impl<const N: usize, F, P, R> Deref for Function<N, F, P, R> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.f
    }
}

impl<const N: usize, F, P, R> DerefMut for Function<N, F, P, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.f
    }
}

impl<const N: usize, F, P, R> Function<N, F, P, R> {
    pub fn new(f: F) -> Self {
        Function {
            f,
            _phantom: default(),
        }
    }
}

use crate::Cons;

vertex!();
vertex!(P);
vertex!(P0, P1);
vertex!(P0, P1, P2);
vertex!(P0, P1, P2, P3);
vertex!(P0, P1, P2, P3, P4);
vertex!(P0, P1, P2, P3, P4, P5);
vertex!(P0, P1, P2, P3, P4, P5, P6);

/*
impl<F, R> Edges for Function<0, F, (), R>
where
    F: 'static + Send + Sync + Fn() -> R,
    R: 'static + Send + Sync,
{
    type EdgeConstructor = BevyEdgeConstructor;

    const INPUTS: Inputs<Self> = &[];
    const OUTPUTS: Outputs<Self> = &[output::<Self, Out<R>>];
}

impl<'a, T, R> VertexOutput<Out<R>> for Function<0, T, (), R>
where
    T: 'static + Send + Sync + Fn() -> R,
    R: 'static + Send + Sync,
{
    type Context = World;
    type Key = Entity;
    type Return = R;

    fn evaluate(world: &World, entity: Entity) -> R {
        debug!(
            "Evaluate Function0 {} for {entity:?}",
            std::any::type_name::<Self>()
        );

        let f = world
            .get::<Function<0, T, (), R>>(entity)
            .expect("Invalid Function Vertex");
        f()
    }
}
*/
