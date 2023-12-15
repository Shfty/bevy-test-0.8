use bevy::math::{IVec2, UVec2, Vec2};

pub trait Fields2 {
    type F0;
    type F1;
}

impl<T0, T1> Fields2 for (T0, T1) {
    type F0 = T0;
    type F1 = T1;
}

impl<T> Fields2 for [T; 2] {
    type F0 = T;
    type F1 = T;
}

impl Fields2 for Vec2 {
    type F0 = f32;
    type F1 = f32;
}

impl Fields2 for IVec2 {
    type F0 = i32;
    type F1 = i32;
}

impl Fields2 for UVec2 {
    type F0 = u32;
    type F1 = u32;
}
