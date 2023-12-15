use bevy::{math::{IVec4, UVec4, Vec4}, prelude::Color};

use crate::prelude::Fields4;

pub trait Destructure4: Fields4 {
    fn f0(&self) -> Self::F0;
    fn f1(&self) -> Self::F1;
    fn f2(&self) -> Self::F2;
    fn f3(&self) -> Self::F3;
}

impl<T0, T1, T2, T3> Destructure4 for (T0, T1, T2, T3)
where
    T0: Clone,
    T1: Clone,
    T2: Clone,
    T3: Clone,
{
    fn f0(&self) -> Self::F0 {
        self.0.clone()
    }

    fn f1(&self) -> Self::F1 {
        self.1.clone()
    }

    fn f2(&self) -> Self::F2 {
        self.2.clone()
    }

    fn f3(&self) -> Self::F3 {
        self.3.clone()
    }
}

impl<T> Destructure4 for [T; 4]
where
    T: Clone,
{
    fn f0(&self) -> Self::F0 {
        self[0].clone()
    }

    fn f1(&self) -> Self::F1 {
        self[1].clone()
    }

    fn f2(&self) -> Self::F2 {
        self[2].clone()
    }

    fn f3(&self) -> Self::F3 {
        self[3].clone()
    }
}

impl Destructure4 for Vec4 {
    fn f0(&self) -> f32 {
        self.x
    }

    fn f1(&self) -> f32 {
        self.y
    }

    fn f2(&self) -> f32 {
        self.z
    }

    fn f3(&self) -> f32 {
        self.w
    }
}

impl Destructure4 for IVec4 {
    fn f0(&self) -> i32 {
        self.x
    }

    fn f1(&self) -> i32 {
        self.y
    }

    fn f2(&self) -> i32 {
        self.z
    }

    fn f3(&self) -> i32 {
        self.w
    }
}

impl Destructure4 for UVec4 {
    fn f0(&self) -> u32 {
        self.x
    }

    fn f1(&self) -> u32 {
        self.y
    }

    fn f2(&self) -> u32 {
        self.z
    }

    fn f3(&self) -> u32 {
        self.w
    }
}

impl Destructure4 for Color {
    fn f0(&self) -> Self::F0 {
        self.r()
    }

    fn f1(&self) -> Self::F1 {
        self.g()
    }

    fn f2(&self) -> Self::F2 {
        self.b()
    }

    fn f3(&self) -> Self::F3 {
        self.a()
    }
}
