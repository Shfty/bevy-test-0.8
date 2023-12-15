use bevy::{math::{IVec3, UVec3, Vec3, Quat}, prelude::{Color, Transform}};

pub trait Fields3 {
    type F0;
    type F1;
    type F2;
}

impl<T0, T1, T2> Fields3 for (T0, T1, T2) {
    type F0 = T0;
    type F1 = T1;
    type F2 = T2;
}

impl<T> Fields3 for [T; 3] {
    type F0 = T;
    type F1 = T;
    type F2 = T;
}

impl Fields3 for Vec3 {
    type F0 = f32;
    type F1 = f32;
    type F2 = f32;
}

impl Fields3 for IVec3 {
    type F0 = i32;
    type F1 = i32;
    type F2 = i32;
}

impl Fields3 for UVec3 {
    type F0 = u32;
    type F1 = u32;
    type F2 = u32;
}

impl Fields3 for Color {
    type F0 = f32;
    type F1 = f32;
    type F2 = f32;
}

impl Fields3 for Transform {
    type F0 = Vec3;
    type F1 = Quat;
    type F2 = Vec3;
}
