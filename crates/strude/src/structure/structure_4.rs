use bevy::{
    math::{IVec4, UVec4, Vec4},
    prelude::Color,
};

use crate::prelude::Fields4;

pub trait Structure4: Fields4 {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2, f3: Self::F3) -> Self;
}

impl<T0, T1, T2, T3> Structure4 for (T0, T1, T2, T3) {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2, f3: Self::F3) -> Self {
        (f0, f1, f2, f3)
    }
}

impl<T> Structure4 for [T; 4] {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2, f3: Self::F3) -> Self {
        [f0, f1, f2, f3]
    }
}

impl Structure4 for Vec4 {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2, f3: Self::F3) -> Self {
        Vec4::new(f0, f1, f2, f3)
    }
}

impl Structure4 for IVec4 {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2, f3: Self::F3) -> Self {
        IVec4::new(f0, f1, f2, f3)
    }
}

impl Structure4 for UVec4 {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2, f3: Self::F3) -> Self {
        UVec4::new(f0, f1, f2, f3)
    }
}

impl Structure4 for Color {
    fn structure(f0: Self::F0, f1: Self::F1, f2: Self::F2, f3: Self::F3) -> Self {
        Color::rgba(f0, f1, f2, f3)
    }
}
