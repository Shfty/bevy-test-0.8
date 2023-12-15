use bevy::{
    prelude::{debug, Commands, Component, Entity, Query, Res},
    render::{
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
        renderer::RenderDevice,
    },
};

use crate::prelude::Board;

/// Buffer of mesh instance data
#[derive(Component)]
pub struct BoardBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

pub fn prepare_board_buffer(
    mut commands: Commands,
    query: Query<(Entity, &Board)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, board) in query.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("board buffer"),
            contents: bytemuck::cast_slice(board.cell_states.as_slice()),
            usage: BufferUsages::STORAGE,
        });

        debug!("Preparing board buffer");
        commands.entity(entity).insert(BoardBuffer {
            buffer,
            length: board.cell_states.len(),
        });
    }
}
