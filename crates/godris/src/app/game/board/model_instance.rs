use bevy::{
    hierarchy::{BuildChildren, Children, DespawnRecursiveExt, Parent},
    math::IVec3,
    prelude::{
        default, info, warn, Assets, Bundle, Changed, Commands, Component, Entity, Handle, Query,
        Res, Without,
    },
};
use bevy_instancing::prelude::InstanceColor;
use ecs_ex::entity_default;

use crate::prelude::{
    Board, BoardRotation, BoardTransform, BoardTransformBundle, FailedGridMoves,
    Model, PendingGridMoves, SuccessfulGridMoves, Voxel, VoxelBundle, CollisionLayer, 
};

#[derive(Debug, Clone, Component)]
pub struct ModelInstance {
    pub board: Entity,
    pub collision_layer: CollisionLayer,
    pub model_loaded: bool,
}

impl Default for ModelInstance {
    fn default() -> Self {
        Self {
            board: entity_default(),
            collision_layer: default(),
            model_loaded: false,
        }
    }
}

#[derive(Debug, Default, Clone, Bundle)]
pub struct ModelInstanceBundle {
    #[bundle]
    pub transform: BoardTransformBundle,
    pub color: InstanceColor,
    pub model_instance: ModelInstance,
    pub model: Handle<Model>,
}

pub fn model_instance_added(
    models: Res<Assets<Model>>,
    mut query_model_instance: Query<(Entity, &mut ModelInstance, &Handle<Model>)>,
    mut commands: Commands,
) {
    for (model_instance_entity, mut model_instance, model) in query_model_instance.iter_mut() {
        if model_instance.model_loaded {
            continue;
        }

        let model = if let Some(model) = models.get(model.clone()) {
            model
        } else {
            warn!("Model {:?} not loaded", model);
            continue;
        };

        commands.entity(model_instance_entity).despawn_descendants();

        commands
            .entity(model_instance.board)
            .add_child(model_instance_entity);

        for (translation, color) in model.voxels.iter().copied() {
            commands
                .entity(model_instance_entity)
                .with_children(|children| {
                    children.spawn().insert_bundle(VoxelBundle {
                        position_bundle: BoardTransform {
                            translation,
                            ..default()
                        }
                        .into(),
                        voxel: Voxel {
                            board: model_instance.board,
                            collision_layer: model_instance.collision_layer,
                            ..default()
                        },
                        mesh_instance_color: color.into(),
                    });
                });
        }

        info!("Model instance board {:?}", model_instance.board);

        model_instance.model_loaded = true;

        info!("Model instance {model_instance_entity:?} added");
    }
}

pub fn clear_succesful_moves(mut query: Query<&mut SuccessfulGridMoves>) {
    for mut successful in query.iter_mut() {
        successful.clear()
    }
}

pub fn clear_failed_moves(mut query: Query<&mut FailedGridMoves>) {
    for mut failed in query.iter_mut() {
        failed.clear()
    }
}

pub fn move_model_instances(
    models: Res<Assets<Model>>,
    mut query_model_instance: Query<
        (
            &Handle<Model>,
            &Parent,
            &Children,
            &mut PendingGridMoves,
            &mut BoardTransform,
            Option<&mut SuccessfulGridMoves>,
            Option<&mut FailedGridMoves>,
        ),
        (Changed<PendingGridMoves>, Without<Board>, Without<Voxel>),
    >,
    query_parent: Query<&Parent>,
    query_board: Query<(&Board, &BoardTransform)>,
) {
    for (model, parent, children, mut moves, mut model_transform, succesful_moves, failed_moves) in
        query_model_instance.iter_mut()
    {
        let model = models.get(model.clone()).unwrap();

        let (board, board_transform) = if let Some(components) =
            hierarchy_ex::walk_up(**parent, &query_parent, |entity| {
                query_board.get(entity).ok()
            }) {
            components
        } else {
            continue;
        };

        let mut transform = *model_transform;

        let mut succesful = vec![];
        let mut failed = vec![];
        let mut iter = moves.drain(..);
        'moves: while let Some(move_data) = iter.next() {
            // Rotate
            if move_data.delta.rotation != BoardRotation::Identity {
                let candidate_rotation = transform.rotation + move_data.delta.rotation;

                // If no intersection, apply and continue
                if !board.intersect_model(
                    model,
                    *board_transform
                        * BoardTransform {
                            translation: transform.translation,
                            rotation: candidate_rotation,
                        },
                    CollisionLayer(0),
                    Some(&**children),
                ) {
                    match move_data.delta.rotation {
                        BoardRotation::CW => {
                            transform.translation += transform.rotation * model.half_offset
                        }
                        BoardRotation::CCW => {
                            transform.translation +=
                                (transform.rotation + BoardRotation::CW) * model.half_offset
                        }
                        _ => (),
                    }
                    transform.rotation += move_data.delta.rotation;
                } else {
                    // An intersection occurred, try to kick out of it
                    let aabb = transform.rotation * model.aabb();
                    let aabb_rotated = candidate_rotation * model.aabb();
                    let size_delta = aabb_rotated.size() - aabb.size();
                    let kick = size_delta.max(IVec3::ZERO);

                    let mut did_fail = false;
                    'kicks: for kick in [kick, -kick] {
                        let mut kick_unit = kick.abs().min(IVec3::ONE) * kick.signum();
                        'kick: loop {
                            if !board.intersect_model(
                                model,
                                *board_transform
                                    * BoardTransform {
                                        translation: transform.translation + kick_unit,
                                        rotation: candidate_rotation,
                                    },
                                CollisionLayer(0),
                                Some(&**children),
                            ) {
                                transform.translation += transform.rotation * model.half_offset;
                                transform.translation += kick_unit;
                                transform.rotation += move_data.delta.rotation;
                                break 'kicks;
                            }

                            if kick_unit == kick {
                                did_fail = true;
                                break 'kick;
                            }

                            kick_unit += kick.abs().min(IVec3::ONE) * kick.signum();
                        }
                    }

                    if did_fail {
                        failed.push(move_data);
                        break 'moves;
                    }
                }
            }

            // Translate
            let mut axes = vec![];
            if move_data.delta.translation.x != 0 {
                axes.push((0, IVec3::X));
            }

            if move_data.delta.translation.y != 0 {
                axes.push((1, IVec3::Y));
            }

            if move_data.delta.translation.z != 0 {
                axes.push((2, IVec3::Z));
            }

            let mut axes = axes.into_iter().collect::<Vec<_>>();

            let mut pending_translation = move_data.delta.translation;

            'axes: loop {
                if axes.len() == 0 {
                    succesful.push(move_data);
                    break 'axes;
                }

                if let Some((i, idx)) = axes.iter().enumerate().find_map(|(idx, (i, axis))| {
                    let candidate_translation =
                        transform.translation + (*axis * pending_translation[*i]);

                    if board.intersect_model(
                        model,
                        *board_transform
                            * BoardTransform {
                                translation: candidate_translation,
                                rotation: transform.rotation,
                            },
                        CollisionLayer(0),
                        Some(&**children),
                    ) {
                        None
                    } else {
                        Some((i, idx))
                    }
                }) {
                    transform.translation[*i] += pending_translation[*i];
                    pending_translation[*i] = 0;
                    axes.remove(idx);
                } else {
                    failed.push(move_data);
                    break 'axes;
                };
            }
        }

        failed.extend(iter);

        if let Some(mut succesful_moves) = succesful_moves {
            succesful_moves.extend(succesful.into_iter());
        }

        if let Some(mut failed_moves) = failed_moves {
            failed_moves.extend(failed.into_iter());
        }

        *model_transform = transform;
    }
}
