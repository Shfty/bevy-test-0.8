use bevy::{
    hierarchy::Children,
    math::IVec3,
    prelude::{default, Assets, Color, Component, Handle, Query, Res, With, Without},
};
use bevy_instancing::prelude::InstanceColor;
use lerp::Lerp;

use crate::prelude::{
    Board, BoardTransform, CollisionLayer, InputController, LockTimer, Model, NextPieceTimer,
};

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct GhostPiece;

pub fn ghost_piece_position(
    models: Res<Assets<Model>>,
    query_board: Query<&Board>,
    query_controller: Query<&InputController>,
    query_controller_target: Query<(&BoardTransform, &Handle<Model>), Without<GhostPiece>>,
    query_children: Query<&Children>,
    mut query_ghost_piece: Query<&mut BoardTransform, With<GhostPiece>>,
) {
    let controller = query_controller.iter().next().unwrap();
    let board = query_board.iter().next().unwrap();

    let mut ghost_piece_transform = query_ghost_piece.iter_mut().next().unwrap();

    if let Some(target) = controller.target {
        if let Ok((target_transform, model)) = query_controller_target.get(target) {
            let children = query_children.get(target);

            let model = models.get(model).unwrap();

            let mut target_transform = *target_transform;

            loop {
                if board.intersect_model(
                    model,
                    BoardTransform {
                        translation: IVec3::new(
                            target_transform.translation.x,
                            target_transform.translation.y - 1,
                            target_transform.translation.z,
                        ),
                        ..target_transform
                    },
                    CollisionLayer(0),
                    children.ok().map(|children| &**children),
                ) {
                    break;
                }

                target_transform.translation.y -= 1;
            }

            *ghost_piece_transform = target_transform;
        }
    }
}

pub fn ghost_piece_color(
    query_controller: Query<&InputController>,
    query_controller_target: Query<&LockTimer, Without<GhostPiece>>,
    query_next_piece_timer: Query<&NextPieceTimer>,
    mut query_ghost_piece: Query<&mut InstanceColor, With<GhostPiece>>,
) {
    let controller = query_controller.iter().next().unwrap();
    let next_piece_timer = query_next_piece_timer.iter().next().unwrap();

    let mut ghost_piece_color = query_ghost_piece.iter_mut().next().unwrap();

    let (lock_timer_active, lock_timer_percent) = if let Some(target) = controller.target {
        if let Ok(lock_timer) = query_controller_target.get(target) {
            (!lock_timer.paused(), lock_timer.percent())
        } else {
            (false, default())
        }
    } else {
        (false, default())
    };

    let next_piece_timer_active = !next_piece_timer.paused();

    let target_color = if lock_timer_active {
        Color::WHITE.lerp(Color::rgba(0.0, 0.0, 0.0, 0.0), lock_timer_percent)
    } else if next_piece_timer_active {
        (Color::rgba(2.0, 2.0, 2.0, 1.0)).lerp(
            Color::rgba(0.0, 0.0, 0.0, 0.0),
            (next_piece_timer.percent() * 2.0).min(1.0),
        )
    } else {
        Color::rgba(1.0, 1.0, 1.0, 0.3)
    };

    if **ghost_piece_color != target_color {
        **ghost_piece_color = target_color;
    }
}
