use bevy::math::{IVec2, UVec2, Vec2};

use crate::prelude::Fields2;

pub trait Structure2: Fields2 {
    fn structure(f0: Self::F0, f1: Self::F1) -> Self;
}

impl<T0, T1> Structure2 for (T0, T1) {
    fn structure(f0: Self::F0, f1: Self::F1) -> Self {
        (f0, f1)
    }
}

impl<T> Structure2 for [T; 2] {
    fn structure(f0: Self::F0, f1: Self::F1) -> Self {
        [f0, f1]
    }
}

impl Structure2 for Vec2 {
    fn structure(f0: Self::F0, f1: Self::F1) -> Self {
        Vec2::new(f0, f1)
    }
}

impl Structure2 for IVec2 {
    fn structure(f0: Self::F0, f1: Self::F1) -> Self {
        IVec2::new(f0, f1)
    }
}

impl Structure2 for UVec2 {
    fn structure(f0: Self::F0, f1: Self::F1) -> Self {
        UVec2::new(f0, f1)
    }
}

