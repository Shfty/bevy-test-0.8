use bevy::{math::{IVec3, UVec3, Vec3, Quat}, prelude::Transform};

use crate::prelude::Fields3;

pub trait Destructure3: Fields3 {
    fn f0(&self) -> Self::F0;
    fn f1(&self) -> Self::F1;
    fn f2(&self) -> Self::F2;
}

impl<T0, T1, T2> Destructure3 for (T0, T1, T2)
where
    T0: Clone,
    T1: Clone,
    T2: Clone,
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
}

impl<T> Destructure3 for [T; 3]
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
}

impl Destructure3 for Vec3 {
    fn f0(&self) -> f32 {
        self.x
    }

    fn f1(&self) -> f32 {
        self.y
    }

    fn f2(&self) -> f32 {
        self.z
    }
}

impl Destructure3 for IVec3 {
    fn f0(&self) -> i32 {
        self.x
    }

    fn f1(&self) -> i32 {
        self.y
    }

    fn f2(&self) -> i32 {
        self.z
    }
}

impl Destructure3 for UVec3 {
    fn f0(&self) -> u32 {
        self.x
    }

    fn f1(&self) -> u32 {
        self.y
    }

    fn f2(&self) -> u32 {
        self.z
    }
}

impl Destructure3 for Transform {
    fn f0(&self) -> Vec3 {
        self.translation
    }

    fn f1(&self) -> Quat {
        self.rotation
    }

    fn f2(&self) -> Vec3 {
        self.scale
    }
}

