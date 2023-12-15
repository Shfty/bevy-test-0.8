use ::bevy::{
    core::CorePlugin,
    gilrs::GilrsPlugin,
    input::{gamepad::gamepad_event_system, InputPlugin},
    log::{Level, LogPlugin},
    math::Vec2,
    prelude::{
        default, App, Commands, CoreStage, Gamepad, GamepadAxisType,
        ParallelSystemDescriptorCoercion,
    },
};

use crate::prelude::{
    evaluate_edge::evaluate_edge, AddGraphVertex, Cache, Connect, EdgeArc, Evaluator, Function,
    GraphArcOutEdge, In, Input, Log, Out, Output,
};

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{info, Component, DynamicScene, EventReader, GamepadEvent, Query},
    reflect::Reflect,
};

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct GamepadAxis {
    #[reflect(ignore)]
    pub gamepad: Gamepad,
    #[reflect(ignore)]
    pub axis_type: GamepadAxisType,
    pub value: f32,
}

impl Default for GamepadAxis {
    fn default() -> Self {
        Self {
            gamepad: Gamepad(0),
            axis_type: GamepadAxisType::LeftStickX,
            value: default(),
        }
    }
}

pub fn gamepad_axis(mut events: EventReader<GamepadEvent>, mut query: Query<&mut GamepadAxis>) {
    for GamepadEvent(gamepad, event) in events.iter() {
        for mut input_axis in query.iter_mut() {
            if input_axis.gamepad != *gamepad {
                continue;
            }

            match event {
                bevy::prelude::GamepadEventType::AxisChanged(axis_type, value) => {
                    if input_axis.axis_type != *axis_type {
                        continue;
                    }

                    input_axis.value = *value;
                }
                _ => (),
            }
        }
    }
}

fn gamepad_graph(mut commands: Commands) {
    // Left stick inputs
    let left_stick_x_axis = commands
        .spawn()
        .insert(GamepadAxis {
            gamepad: Gamepad(1),
            axis_type: GamepadAxisType::LeftStickX,
            ..default()
        })
        .id();

    let left_stick_y_axis = commands
        .spawn()
        .insert(GamepadAxis {
            gamepad: Gamepad(1),
            axis_type: GamepadAxisType::LeftStickY,
            ..default()
        })
        .id();

    // Graph inputs
    let left_stick_x = commands.add_graph_vertex(Evaluator::new(move |world| {
        world
            .get::<GamepadAxis>(left_stick_x_axis)
            .expect("No Left Stick X GamepadAxis")
            .value
    }));

    let left_stick_y = commands.add_graph_vertex(Evaluator::new(move |world| {
        world
            .get::<GamepadAxis>(left_stick_y_axis)
            .expect("No Left Stick Y GamepadAxis")
            .value
    }));

    // Structure into a Vec2
    let vec2_new = commands.add_graph_vertex(Function::new(Vec2::new));

    // Apply power curve
    let pow_exponent = commands.add_graph_vertex(Function::new(|| 2.0f32)); //.id();
    let vec2_pow = commands.add_graph_vertex(Function::new(|v: Vec2, e: f32| {
        if v.length() > 0.0 {
            v.normalize() * v.length().powf(e)
        } else {
            v
        }
    }));

    // Log to console
    let log = commands.add_graph_vertex(Log::<Vec2>::new(Level::INFO));

    // Store to component
    let cache = commands.init_graph_vertex_new::<Cache<Vec2>>();

    // Connections
    commands
        .connect(left_stick_x.output::<0, f32>() | vec2_new.input::<0, f32>())
        .connect(left_stick_y.output::<0, f32>() | vec2_new.input::<1, f32>())
        .connect(pow_exponent.output::<0, f32>() | vec2_pow.input::<1, f32>())
        .connect(
            vec2_new.output::<0, Vec2>()
                | vec2_pow.through::<0, 0, Vec2>()
                | log.through::<0, 0, Vec2>()
                | cache.input::<0, Vec2>(),
        );

    // Evaluate
    cache.evaluate::<0, Vec2>(&mut commands);
}

#[test]
pub fn test_gamepad() {
    println!();
    let mut app = App::new();

    app.add_plugin(CorePlugin)
        //.add_plugin(::bevy::app::ScheduleRunnerPlugin)
        .add_plugin(LogPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(GilrsPlugin);

    app.add_system_to_stage(
        CoreStage::PreUpdate,
        gamepad_axis.after(gamepad_event_system),
    );

    app.add_startup_system(gamepad_graph)
        .add_system(evaluate_edge);
    //.add_system(evaluate_with::<Cache<Vec2>, Output<Vec2>>);

    app.run();
}

fn test_serialize() {
    println!();
    let mut app = App::new();

    app.add_plugin(CorePlugin)
        .add_plugin(::bevy::app::ScheduleRunnerPlugin)
        .add_plugin(LogPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(GilrsPlugin);

    app.register_type::<GamepadAxis>();

    app.register_type::<GraphArcOutEdge>();

    app.register_type::<EdgeArc<In<f32>>>();
    app.register_type::<EdgeArc<Out<f32>>>();

    app.register_type::<EdgeArc<In<Vec2>>>();
    app.register_type::<EdgeArc<Out<Vec2>>>();

    app.register_type::<Log<Vec2>>();
    app.register_type::<Cache<Vec2>>();

    app.register_type::<In<f32>>();
    app.register_type::<In<Vec2>>();

    app.register_type::<Out<f32>>();
    app.register_type::<Out<Vec2>>();

    app.register_type::<Output<0, f32>>();
    app.register_type::<Output<1, f32>>();

    app.register_type::<Input<0, f32>>();
    app.register_type::<Input<1, f32>>();

    app.register_type::<Input<0, f32>>();
    app.register_type::<Input<1, f32>>();

    app.register_type::<Input<0, Vec2>>();
    app.register_type::<Input<1, Vec2>>();

    app.register_type::<Out<f32>>();
    app.register_type::<Out<Vec2>>();

    app.add_startup_system(gamepad_graph);
    app.update();

    let type_registry = app.world.resource::<bevy::reflect::TypeRegistryArc>();
    let scene = DynamicScene::from_world(&app.world, type_registry);
    let ron = scene.serialize_ron(type_registry).unwrap();

    info!("Ron:");
    for line in ron.split('\n') {
        info!("{line:}");
    }
}
