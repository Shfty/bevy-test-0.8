use bevy::math::{IVec2, UVec2, Vec2};

use crate::prelude::Fields2;

pub trait Destructure2: Fields2 {
    fn f0(&self) -> Self::F0;
    fn f1(&self) -> Self::F1;
}

impl<T0, T1> Destructure2 for (T0, T1)
where
    T0: Clone,
    T1: Clone,
{
    fn f0(&self) -> Self::F0 {
        self.0.clone()
    }

    fn f1(&self) -> Self::F1 {
        self.1.clone()
    }
}

impl<T> Destructure2 for [T; 2]
where
    T: Clone,
{
    fn f0(&self) -> Self::F0 {
        self[0].clone()
    }

    fn f1(&self) -> Self::F1 {
        self[1].clone()
    }
}

impl Destructure2 for Vec2 {
    fn f0(&self) -> f32 {
        self.x
    }

    fn f1(&self) -> f32 {
        self.y
    }
}

impl Destructure2 for IVec2 {
    fn f0(&self) -> i32 {
        self.x
    }

    fn f1(&self) -> i32 {
        self.y
    }
}

impl Destructure2 for UVec2 {
    fn f0(&self) -> u32 {
        self.x
    }

    fn f1(&self) -> u32 {
        self.y
    }
}

