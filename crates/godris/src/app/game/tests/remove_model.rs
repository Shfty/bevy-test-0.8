use crate::prelude::{
    generate_model, setup_board, Board, BoardBundle, BoardTransform, Model, ModelInstance,
    ModelInstanceBundle,
};

use super::*;

fn test_remove_model() {
    let mut app = test_base();

    app.add_startup_system_to_stage(StartupStage::Startup, setup_board)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_remove_model)
        .add_system(remove_timer);

    app.run();
}

fn setup_remove_model(
    asset_server: Res<AssetServer>,
    mut models: ResMut<Assets<Model>>,
    mut commands: Commands,
) {
    // Board
    let board = commands
        .spawn()
        .insert_bundle(BoardBundle {
            board: Board {
                size: UVec3::new(4, 4, 4),
                ..default()
            },
            ..default()
        })
        .id();

    // Model instance
    let model_unit = generate_model(
        &asset_server,
        &mut models,
        "model/unit.model.ron",
        vec![(IVec3::ZERO, Color::rgba(1.0, 1.0, 1.0, 0.5))],
        IVec3::new(0, 0, 0),
    );

    commands
        .spawn()
        .insert(Name::new("Unit Model Instance"))
        .insert_bundle(ModelInstanceBundle {
            model_instance: ModelInstance { board, ..default() },
            model: model_unit.clone(),
            transform: BoardTransform {
                translation: IVec3::ONE,
                ..default()
            }
            .into(),
            ..default()
        })
        .insert(RemoveTimer(Timer::from_seconds(1.0, false)));
}
