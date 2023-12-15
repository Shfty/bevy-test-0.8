use bevy::{math::{IVec4, UVec4, Vec4}, prelude::Color};

pub trait Fields4 {
    type F0;
    type F1;
    type F2;
    type F3;
}

impl<T0, T1, T2, T3> Fields4 for (T0, T1, T2, T3) {
    type F0 = T0;
    type F1 = T1;
    type F2 = T2;
    type F3 = T3;
}

impl<T> Fields4 for [T; 4] {
    type F0 = T;
    type F1 = T;
    type F2 = T;
    type F3 = T;
}

impl Fields4 for Vec4 {
    type F0 = f32;
    type F1 = f32;
    type F2 = f32;
    type F3 = f32;
}

impl Fields4 for IVec4 {
    type F0 = i32;
    type F1 = i32;
    type F2 = i32;
    type F3 = i32;
}

impl Fields4 for UVec4 {
    type F0 = u32;
    type F1 = u32;
    type F2 = u32;
    type F3 = u32;
}

impl Fields4 for Color {
    type F0 = f32;
    type F1 = f32;
    type F2 = f32;
    type F3 = f32;
}

