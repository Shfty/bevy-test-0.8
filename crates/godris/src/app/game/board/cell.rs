//! A single cell within a [`Board`]

use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec4,
    prelude::{Added, Bundle, Component, Handle, Mesh, Query, Reflect, Res},
    reflect::FromReflect,
};
use bytemuck::{Pod, Zeroable};

use crate::prelude::BoardTransform;

pub struct CellMesh(pub Handle<Mesh>);

#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect, FromReflect,
)]
pub struct CollisionLayer(pub usize);

#[derive(Debug, Default, Copy, Clone, Reflect, FromReflect, Pod, Zeroable)]
#[repr(C)]
pub struct CellState {
    pub color: Vec4,
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Cell;

#[derive(Default, Bundle)]
pub struct CellBundle {
    pub cell: Cell,
    pub transform: BoardTransform,
}

pub fn cell_mesh_added(
    cell_mesh_resource: Res<CellMesh>,
    mut query: Query<&mut Handle<Mesh>, Added<Cell>>,
) {
    for mut cell_mesh in query.iter_mut() {
        *cell_mesh = cell_mesh_resource.0.clone();
    }
}
