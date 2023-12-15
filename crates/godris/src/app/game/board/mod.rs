//! A three-dimensional grid of cells

pub mod cell;
pub mod compute_instances;
pub mod grid_move;
pub mod material;
pub mod model_instance;
pub mod plugin;
pub mod position;
pub mod voxel;

use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    hierarchy::{BuildChildren, Parent},
    math::{IVec3, UVec3, Vec3, Vec4},
    pbr::AlphaMode,
    prelude::{
        debug, default, Added, Assets, Bundle, Commands, Component, Entity, Name, Query, Res,
        ResMut, Transform, Without,
    },
    reflect::Reflect,
    render::render_component::ExtractComponent,
    transform::TransformBundle,
    utils::{HashMap, HashSet},
};
use ecs_ex::{entity_default, WithName};

use crate::prelude::{BoardMaterial, CellState, Model};

use bevy_instancing::prelude::{InstanceBlock, InstanceBlockBundle, InstanceColor};

use self::{
    cell::{Cell, CellMesh, CollisionLayer},
    position::{BoardTransform, BoardTransformBundle},
    voxel::Voxel,
};

pub const BOARD_WIDTH: usize = 38;
pub const BOARD_HEIGHT: usize = 18;
pub const BOARD_DEPTH: usize = 38;

pub const BOARD_MIN_X: usize = 0;
pub const BOARD_MIN_Y: usize = 0;
pub const BOARD_MIN_Z: usize = 0;

pub const BOARD_MAX_X: usize = BOARD_WIDTH - 1;
pub const BOARD_MAX_Y: usize = BOARD_HEIGHT - 1;
pub const BOARD_MAX_Z: usize = BOARD_DEPTH - 1;

pub fn board_size() -> Vec3 {
    Vec3::new(BOARD_WIDTH as f32, BOARD_HEIGHT as f32, BOARD_DEPTH as f32)
}

pub fn board_min() -> Vec3 {
    Vec3::new(BOARD_MIN_X as f32, BOARD_MIN_Y as f32, BOARD_MIN_Z as f32)
}

pub fn board_max() -> Vec3 {
    Vec3::new(BOARD_MAX_X as f32, BOARD_MAX_Y as f32, BOARD_MAX_Z as f32)
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct Board {
    /// Board size in 3 dimensions
    pub size: UVec3,

    /// Cells below this coordinate won't be rendered
    pub visible_min: Option<UVec3>,

    /// Cells above this coordinate won't be rendered
    pub visible_max: Option<UVec3>,

    /// Internal GPU-compatible state for each cell
    pub cell_states: Vec<CellState>,

    /// Per-cell collision information
    pub cell_collision: Vec<Vec<(Entity, CollisionLayer)>>,

    /// Handle to opaque InstancedMeshes
    pub instance_block_opaque: Entity,

    /// Handle to transparent InstancedMeshes
    pub instance_block_transparent: Entity,

    /// Voxel entity -> cell index map
    /// Used for updating cells when a voxel moves or is removed
    pub voxel_to_cell: HashMap<Entity, usize>,
    pub cell_to_voxel: Vec<HashSet<Entity>>,

    // Command queues - deferral is necessary to provide queries to cell update functions
    pub pending_moves: Vec<(Entity, IVec3)>,
    pub pending_removes: Vec<Entity>,
}

impl ExtractComponent for Board {
    type Query = &'static Self;

    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        Board {
            size: item.size,
            visible_min: item.visible_min,
            instance_block_opaque: item.instance_block_opaque,
            instance_block_transparent: item.instance_block_transparent,
            cell_states: item.cell_states.clone(),
            ..default()
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            size: default(),
            visible_min: default(),
            visible_max: default(),
            cell_states: default(),
            cell_collision: default(),
            instance_block_opaque: entity_default(),
            instance_block_transparent: entity_default(),
            voxel_to_cell: default(),
            cell_to_voxel: default(),
            pending_moves: default(),
            pending_removes: default(),
        }
    }
}

impl MapEntities for Board {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        let iter = self.voxel_to_cell.drain().collect::<Vec<_>>();
        for (k, v) in iter {
            self.voxel_to_cell.insert(entity_map.get(k)?, v);
        }

        Ok(())
    }
}

impl Board {
    fn pos_to_index(&self, pos: IVec3) -> usize {
        ((pos.z * self.size.y as i32 * self.size.x as i32) + (pos.y * self.size.x as i32) + pos.x)
            as usize
    }

    pub fn cell_state(&self, pos: IVec3) -> Option<&CellState> {
        let index = self.pos_to_index(pos);
        self.cell_states.get(index)
    }

    pub fn cell_state_mut(&mut self, pos: IVec3) -> Option<&mut CellState> {
        let index = self.pos_to_index(pos);
        self.cell_states.get_mut(index)
    }

    pub fn move_voxel(&mut self, voxel: Entity, position: IVec3) {
        self.pending_moves.push((voxel, position));
    }

    pub fn update_cell(
        &mut self,
        index: usize,
        query_voxel: &Query<&Voxel>,
        query_color: &Query<&InstanceColor, Without<Cell>>,
        query_parent: &Query<&Parent>,
    ) {
        let color = if let Some(cell_state) = self.cell_states.get_mut(index) {
            &mut cell_state.color
        } else {
            return;
        };

        *color = Vec4::ZERO;
        self.cell_collision[index].clear();

        for entity in &self.cell_to_voxel[index] {
            if let Ok(voxel) = query_voxel.get(*entity) {
                // Update collision layers
                self.cell_collision[index].push((*entity, voxel.collision_layer));

                // Sum colors
                let color = query_color.get(*entity).unwrap();

                let mut cell_color = Vec4::new(color.r(), color.g(), color.b(), color.a());
                if let Ok(parent) = query_parent.get(*entity) {
                    if let Ok(parent_color) = query_color.get(**parent) {
                        cell_color *= Vec4::new(
                            parent_color.r(),
                            parent_color.g(),
                            parent_color.b(),
                            parent_color.a(),
                        );
                    }
                }

                self.cell_states[index].color += cell_color;
            }
        }
    }

    pub fn flush_moves(
        &mut self,
        query_voxel: &Query<&Voxel>,
        query_color: &Query<&InstanceColor, Without<Cell>>,
        query_parent: &Query<&Parent>,
    ) {
        for (entity, position) in self.pending_moves.drain(..).collect::<Vec<_>>() {
            if let Some(from_index) = self.voxel_to_cell.remove(&entity) {
                self.cell_to_voxel[from_index].remove(&entity);

                debug!("Voxel {entity:?} moved");
                self.update_cell(from_index, query_voxel, query_color, query_parent);
            } else {
                debug!("Voxel {entity:?} added");
            }

            let index = self.pos_to_index(position);
            if let Some(cell_to_voxel) = self.cell_to_voxel.get_mut(index) {
                cell_to_voxel.insert(entity);
                self.voxel_to_cell.insert(entity, index);
                self.update_cell(index, query_voxel, query_color, query_parent);
            }
        }
    }

    pub fn remove_voxel(&mut self, voxel: Entity) {
        self.pending_removes.push(voxel);
    }

    fn remove_voxel_impl(
        &mut self,
        voxel: Entity,
        query_voxel: &Query<&Voxel>,
        query_color: &Query<&InstanceColor, Without<Cell>>,
        query_parent: &Query<&Parent>,
    ) {
        if let Some(index) = self.voxel_to_cell.remove(&voxel) {
            self.cell_to_voxel[index].remove(&voxel);
            debug!("Voxel {voxel:?} removed from board");
            self.update_cell(index, query_voxel, query_color, query_parent);
        }
    }

    pub fn flush_removes(
        &mut self,
        query_voxel: &Query<&Voxel>,
        query_color: &Query<&InstanceColor, Without<Cell>>,
        query_parent: &Query<&Parent>,
    ) {
        for entity in self.pending_removes.drain(..).collect::<Vec<_>>() {
            self.remove_voxel_impl(entity, query_voxel, query_color, query_parent);
        }
    }

    pub fn intersect_model(
        &self,
        model: &Model,
        transform: BoardTransform,
        layer: CollisionLayer,
        blacklist: Option<&[Entity]>,
    ) -> bool {
        for (translation, _) in model.voxels.iter() {
            let candidate_translation = transform * *translation;

            let index = self.pos_to_index(candidate_translation);
            if let Some(layers) = self.cell_collision.get(index) {
                if layers.iter().any(|(entity, cell_layer)| {
                    !blacklist
                        .map(|blacklist| blacklist.contains(entity))
                        .unwrap_or_default()
                        && *cell_layer == layer
                }) {
                    return true;
                }
            } else {
                return true;
            }
        }

        false
    }
}

#[derive(Bundle)]
pub struct BoardBundle {
    pub name: Name,
    #[bundle]
    pub transform: TransformBundle,
    #[bundle]
    pub board_transform: BoardTransformBundle,
    pub board: Board,
}

impl Default for BoardBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Board"),
            transform: default(),
            board_transform: default(),
            board: default(),
        }
    }
}

pub fn board_added(
    mut instanced_materials: ResMut<Assets<BoardMaterial>>,
    cell_mesh: Res<CellMesh>,
    mut query: Query<(Entity, &mut Board, &mut Transform), Added<Board>>,
    mut commands: Commands,
) {
    for (board_entity, mut board, mut transform) in query.iter_mut() {
        // Actual cell count
        let cell_count = (board.size.x * board.size.y * board.size.z) as usize;

        // Actual cell count
        let visible_size =
            board.visible_max.unwrap_or(board.size) - board.visible_min.unwrap_or_default();

        let visible_cell_count = (visible_size.x * visible_size.y * visible_size.z) as usize;

        // Cell count padded to account for GPU storage buffer alignment
        let cell_count_gpu = visible_cell_count + 2560 + 160;

        board.cell_states = vec![default(); cell_count_gpu];
        board.cell_collision = vec![default(); cell_count];
        board.cell_to_voxel = vec![default(); cell_count];

        transform.translation -= Vec3::new(
            board.size.x as f32,
            board.size.y as f32,
            board.size.z as f32,
        ) / 2.0;

        commands.entity(board_entity).with_children(|children| {
            let opaque_material = instanced_materials.add(BoardMaterial {
                alpha_mode: AlphaMode::Opaque,
                ..default()
            });

            let transparent_material = instanced_materials.add(BoardMaterial {
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            board.instance_block_opaque = children
                .spawn()
                .with_name("Opaque Mesh Instance Block")
                .insert_bundle(InstanceBlockBundle {
                    material: opaque_material.clone(),
                    mesh_instance_block: InstanceBlock {
                        instance_count: cell_count_gpu,
                    },
                    mesh: cell_mesh.0.clone(),
                    ..default()
                })
                .id();

            board.instance_block_transparent = children
                .spawn()
                .with_name("Transparent Mesh Instance Block")
                .insert_bundle(InstanceBlockBundle {
                    material: transparent_material.clone(),
                    mesh_instance_block: InstanceBlock {
                        instance_count: cell_count_gpu,
                    },
                    mesh: cell_mesh.0.clone(),
                    ..default()
                })
                .id();
        });
    }
}

pub fn board_flush_moves(
    mut query_board: Query<&mut Board>,
    query_voxel: Query<&Voxel>,
    query_color: Query<&InstanceColor, Without<Cell>>,
    query_parent: Query<&Parent>,
) {
    for mut board in query_board.iter_mut() {
        board.flush_moves(&query_voxel, &query_color, &query_parent);
    }
}

pub fn board_flush_removes(
    mut query_board: Query<&mut Board>,
    query_voxel: Query<&Voxel>,
    query_color: Query<&InstanceColor, Without<Cell>>,
    query_parent: Query<&Parent>,
) {
    for mut board in query_board.iter_mut() {
        board.flush_removes(&query_voxel, &query_color, &query_parent);
    }
}

pub fn bresenham(mut from: IVec3, to: IVec3) -> impl Iterator<Item = IVec3> {
    let mut list_of_points = vec![];

    list_of_points.push(from);

    let delta = to - from;
    let delta_abs = delta.abs();
    let delta_sign = delta.signum();

    let (i0, i1, i2) = if delta_abs.x >= delta_abs.y && delta_abs.x >= delta_abs.z {
        (0, 1, 2)
    } else if delta_abs.y >= delta_abs.x && delta_abs.y >= delta_abs.z {
        (1, 0, 2)
    } else if delta_abs.z >= delta_abs.x && delta_abs.z >= delta_abs.y {
        (2, 1, 0)
    } else {
        return list_of_points.into_iter();
    };

    let mut p1 = 2 * delta_abs[i1] - delta_abs[i0];
    let mut p2 = 2 * delta_abs[i2] - delta_abs[i0];
    while from[i0] != to[i0] {
        from[i0] += delta_sign[i0];
        if p1 >= 0 {
            from[i1] += delta_sign[i1];
            p1 -= 2 * delta_abs[i0];
        }
        if p2 >= 0 {
            from[i2] += delta_sign[i2];
            p2 -= 2 * delta_abs[i0];
        }
        p1 += 2 * delta_abs[i1];
        p2 += 2 * delta_abs[i2];
        list_of_points.push(from);
    }

    list_of_points.into_iter()
}
