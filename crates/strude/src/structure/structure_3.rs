use bevy::{
    math::{IVec3, UVec3, Vec3, Quat},
    prelude::{Color, Transform},
};

use crate::prelude::Fields3;

pub trait Structure3: Fields3 {
    fn structure(f0: Self::F0, f1: Self::F1, f1: Self::F2) -> Self;
}

impl<T0, T1, T2> Structure3 for (T0, T1, T2) {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2) -> Self {
        (f0, f1, f2)
    }
}

impl<T> Structure3 for [T; 3] {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2) -> Self {
        [f0, f1, f2]
    }
}

impl Structure3 for Vec3 {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2) -> Self {
        Vec3::new(f0, f1, f2)
    }
}

impl Structure3 for IVec3 {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2) -> Self {
        IVec3::new(f0, f1, f2)
    }
}

impl Structure3 for UVec3 {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2) -> Self {
        UVec3::new(f0, f1, f2)
    }
}

impl Structure3 for Color {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2) -> Self {
        Color::rgb(f0, f1, f2)
    }
}

impl Structure3 for Transform {
    fn structure(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Transform {
            translation,
            rotation,
            scale,
        }
    }
}
