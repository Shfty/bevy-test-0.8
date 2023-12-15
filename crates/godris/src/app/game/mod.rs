// TODO: Implement StandardInstancedMaterial
//       * Initial goal: Make pbr.wgsl work with instances
//         * Should be able to encode most StandardMaterial params as instance data
//       * Secondary goal: Make shadows work
//         * This may be better left until posting on the bevy github
//
// TODO: Investigate spliting SpecializedInstancedMaterial::Key into InstanceKey and PipelineKey
//       * Currently coupling pipelines to mutually compatible sets of materials
//         * i.e. Materials that share both a bind group and parameter values
//       * In theory pipelines should be able to key separately, resulting in less duplication
//
// TODO: Optimization pass for instanced mesh batcher
//
// TODO: Use BufferVec to cache batch data inside InstanceMeta
//       * Avoid clearing every draw
//         * Instead, prune empty buffers
//       * Should be able to avoid unnecessary writes on frames where the
//         buffer data for a given key doesn't change
//         * Seems important, since the act of writing data to the GPU is what indirect avoids
//
// TODO: Specialization for compute shaders that operate on a MeshInstanceBlock
//       * Effectively a generalization of the current board compute module
//       * Should resemble SpecializedInstancedMaterial
//       * InstanceComputeKernel?
//
// TODO: Uniform support for instance buffer
//       * Need to split instances into
//         (std::mem::size_of::<InstanceData>() / MAX_UNIFORM_BUFFER_SIZE) sub-batches,
//         introduce additional draw calls
//
// FIXME: ModelInstance colors should be pushed to children on change
//        * Currently checking parent manually in cell update
//        * Doesn't work for animating next piece timer
//        * Also need to investigate why this works for the lock timer
//          * Something is causing on-change to fire every frame during countdown
//          * This doesn't seem right, need to figure out the cause and fix it
//
// TODO: Refactor InputReader as an input graph
//       * Merge inputs from different device types
//         * ex. WASD + D-Pad, Mouse + Right Stick, etc.
//       * Provide vertex entities for InputController to use
//
// TODO: Refactor graph creation to allow for parent-child hierarchy
//       * More structure makes the inspector more useful
//
// TODO: Mouse + Keyboard input
//       * Mouse for absolute pointing
//         * Won't work if camera is locked to piece, need to fall back on relative movement
//         * Raycast into the scene to determine target XZ
//           * Auto-shift toward it at max analog speed
//       * WASD for digital finesse
//       * Space for hard drop
//       * Left click soft drop
//       * Right click spin camera
//       * Middle click projection
//       * Shift camera focus
//       * Mouse wheel zoom
//
// TODO: Consider persisting XZ position across next piece spawns
//       * Would allow focusing on a specific area of the world without having to
//         move from the center each time
//       * Not good for Tetris due to move palette, gravity increase and focus on line clears,
//         but makes sense here due to structured building goals
//
// TODO: Flag to break models into a set of per-voxel unit models on lock
//       * Off-ground models should continue to move post break
//       * Should cause them to inherit movers etc
//       * Enables wetrix-style heightmap addition
//         * Should also enable heightmap subtraction
//           * Use a 'subtract' flag that causes a model to delete
//             voxels in the direction of failed moves
//       * May be wiser to make voxels the thing that moves,
//         have model instances act as a local-space group for rotation
//
// TODO: Proper timed A-to-B lerp for camera projection
//
// TODO: Experiment with camera rotation spring
//       * May be desirable to lerp back to a default rotation
//         * Perhaps only lerp pitch?
//       * Good excuse to separate OrbitCamera into yaw and pitch controllers
//
// TODO: Visual cue for lock timer
//       * Fade ghost piece to fully transparent over course of lock timer
//       * Flash to double brightness and then quickly fade back down when locking
//
// TODO: Next piece queue
//       * Position above where next piece will spawn
//
// TODO: Improved guidline wireframes
//       * Calculate an XZ profile of the piece and draw one line per corner
//
// TODO: Piece holding machinery
//       * One or more slots to store pieces in
//         * Certain pieces cannot be reserved (ex. disasters, dungeons, etc)
//
// TODO: Marching cubes cell mode
//       * Switch between cube permutation meshes based on neighbours
//
// TODO: Cellular automata water simulation
//
// TODO: Flag to allow terrain to push non-terrain pieces upward
//       * Dropping a terrin piece on a house should shift the house up, not bury it
//       * Special types of terrain could be a hazardous exception to this rule
//
// FIXME: Padding for board storage arrays uses a magic number
//        * Currently dialing in by hand when the board size is changed
//        * Should figure out the maths behind it and implement based on struct sizes
//
// FIXME: Missing system order constraint causes ghost piece to intermittently lag behind on lock
//        * Active piece locks, ghost piece is still at its previous position
//        * Could this be one of the timer systems firing on the same frame as the lock?
//
// FIXME: Orthographic projection appears to have a slight perspective component
//        * Doesn't appear to be tied to interpolation, but should test to make sure
//
// FIXME: Half-pixel offset interferes with wall kicks
//        * Pieces with evenly-numbered dimensions refuse to wall kick in certain situations
//        * Less a problem now pieces are symmetrical, but still an implementation flaw
//
// FIXME: Softlock - next piece failed to trigger during a lengthy play session
//        * Seems random and difficult to reproduce
//
// TODO: World systems
//       * Modify terrain
//         * Raise by dropping more terrain pieces on it
//         * Lower via a spatial piece type
//         * Specialized piece types
//           * Bomb for area clearing
//           * Drills for tunneling
//           * Magic seed to sprout a beanstalk
//           * etc.
//       * Buildings
//         * Houses
//           * Allow Villagers to regain HP
//           * Without a house, a Villager will eventually die
//         * Town hall
//           * Pushes houses into the piece queue
//         * Castle
//           * Royal Castle
//             * After a time, pushes a Knight into the piece queue if one doesn't already exist
//           * Villain's Lair
//             * Endgame dungeon
//             * Occasionally pushes a Disaster or Boss into the piece queue
//         * Dungeon
//           * Creates Monsters
//           * Contains EXP and loot to strengthen Hero units
//         * Blacksmith
//           * Strengthens Hero units with better equipment
//         * Bakery
//           * Buffs Villagers and Heroes to be more productive
//         * Goblin Settlement
//           * Harasses villages, kidnaps Villagers and puts them on the wheel of pain
//           * If a particularly hardy but undeveloped Villager is kidnapped
//             and allowed to suffer a while, pushes a Barbarian into the piece queue
//           * Weak villagers will die after a few sessions on the wheel
//         * Roads?
//           * Feels intuitive from gameplay testing, but might be too much micromanagement
//           * Roads as a suggestion rather than a constraint?
//       * Characters
//         * Villager
//           * Helpless but industrious
//         * Heroes
//           * Barbarian
//             * Affected by more types of trap
//             * 'Muscularity' stat to govern posing, wrestling moves, and attack bonuses
//               * Focus on self improvement via monster bashing
//           * Wizard
//             * Physically weak
//             * High ranged damage
//             * 'Wisdom' stat
//               * Raised through study, finding artifacts
//           * Knight
//             * Middling stats
//             * Flat 1000 point bonus whenever he shouts 'Tally Ho!'
//               * 'Valor' stat to govern frequency / behaviour?
//                 * Raise by helping the citizenry and doing other knightly things
//       * Wildlife
//         * Creatures
//           * Sheep
//             * Should have floaty animation like Space Station Silicon Valley
//           * Piggy
//             * Rotund, prone to rolling
//           * Wolf
//             * Preys on Sheep, Piggy, Villager
//             * Favours lone targets
//             * Boids algorithm to stay close to other wolves
//           * Lemmy
//             * Lemmings character - green hair and a blue jumpsuit
//             * Walks in a straight line, eventually off the edge of the board
//             * Occasionally switches mode to encourage reactive play
//               * Hopping over walls, moving or destroying blocks, spontaneously exploding, etc
//               * Need to counter with different building strategies
//                 * Crush with a block to prevent self-destruct
//                 * Obstruct with a wall to force a rotation
//                 * Obstruct with a ceiling to prevent climbing?
//             * Periodic bonus for keeping it alive
//         * Monsters
//           * Prey on Creatures, Villagers, Heroes
//           * Bosses
//             * Dragon
//       * Foliage
//         * Provides material for creating Buildings?
//           * Needs to be simple to avoid getting too deep into the god game side of things
//           * Water nourishes earth as it flows over it
//             * Nourished earth trees and other flora to grow
//             * Too much water will wash away flora
//           * Sun dries earth
//             * Dry earth slows growth, eventually becomes barren
//           * Certain monsters favour different kinds of terrain
//         * Trees
//         * Rocks
//       * Precipitation cycle
//         * Waterfalls off the edge of the board
//         * Rainbows for large lakes
//         * Ye Auspicious Rubber Duck of Legend
//       * Disasters
//         * Unavoidable piece drops
//           * Decision making needs to be more interesting than Wetrix corner bombing
//         * Tornado
//           * Blows away light structures and characters
//           * Can be harnessed with a windmill
//         * Earthquake
//           * Pulls the board in two, swallowing up structures caught in the middle
//           * May spawn or reveal buried treasure
//         * Meteor
//           * Large damaging AoE
//           * May contain otherworldly treasure
//         * Biblical Flood
//           * Max-height water generator that lasts a long time
//           * Can be repurposed as a water feature with prior planning
//             * Use to drive point-lucrative lake and river systems while it lasts
//         * Magic Nuke
//         * Fruit Drops from Blitter Boy
//           * Damaging AoE on impact + plants exotic seeds that can be grown
//             into a resource with proper watering
//         * Emperor of Mankind crashes his ship into the board and proceeds to
//           hijack the ecosystem
//           * Destroys the Lair and damages the Castle on the way in
//           * Initially helpful, but becomes a Villain in his own right long-term
//
// TODO: Gameplay loop
//       * Quest system
//         * Quest entries are prayers from characters in the world
//           * Characters can also dedicate an item to their god,
//             i.e. push a piece into the queue
//             These may range from harmless offerings from regular villagers,
//             to dangerous calamities from misguided fanatics
//         * Main quests to drive the core loop
//           * Multiple possible main quests depending on player actions
//             * Generally culminating in the defeat of a Villain unit in its Lair
//               * Variant to make rescuing a princess the primary goal
//             * Special quest branch that turns the tables,
//               puts the player on the villain's side
//         * Side quests to encourage experimentation
//           * Civic planning
//           * Hydroengineering
//             * "We need an aqueduct!" quest entry just before village is washed away by flood
//           * Building a boss fight arena for the Barbarian to hang out in
//           * Digging for lost treasure
//           * The Legend of Excalibur
//             * Various investigation phases culminating in an excalibur pedestal piece drop
//             * Big buff for Knight and points bonus
//       * 'New Game Plus' as fundamental loop mechanic to scale difficulty and scoring potential
//         * Automatically triggered at the end of a given main quest
//           * ex. Hero activates a magic artifact that triggers a disaster to reset the game
//           * Can prevent via interaction to stay in current cycle and open up 'post-game' content
//       * Different phases of the game that affect the piece randomizer
//         * Allows each set of systems to be focused and get some breathing room
//         * Ex. precipitation phase, wildlife phase, hero phase, etc etc
//         * Potentially give player some control over this?
//           * Eases the design / implementation of a branching game structure
//           * Could give each mode a timer bar that drains while active,
//             recharges while inactive, and forces a switch on depletion
//             * Potential for tying different hold pieces to each mode
//           * Alternately, set up as items that appear on the board and has to be
//             crushed with an appropriately sized piece
//             * Upon activation, switches out the piece pool to some other mode
//           * Or, tie to building some specific configuration of common pieces
//             * i.e. Some idol that switches 'season' (piece set) and crumbles after N turns
//             * Could have a 'default' piece set that branches out into special themed ones,
//               which expire after a time and revert to the default, or can branch further
//               * Affords the player a good amount of control, like a fighting game stance switch combo
//       * True ending involves getting all three heroes and forming a party
//         * Should only be feasible to create one hero per playthrough
//           * One hero full stop, or one endgame-viable hero?
//         * Avert the new game plus calamity and finish the game with a gigantic points bonus
//
// Godris
// ------
// Play as a god whose hands are tied by a never-ending cycle of prayers and falling blocks.
// Build a prosperous kingdom of high adventure, or cackle as you burn it to the ground.
//

pub mod assets;
pub mod autorepeat;
pub mod bag_randomizer;
pub mod board;
pub mod camera;
pub mod ghost_piece;
pub mod input;
pub mod lock_timer;

#[cfg(test)]
pub mod tests;

use std::{fs::File, time::Duration};

use bevy::{
    core::{Time, Timer},
    ecs::{event::Events, reflect::ReflectComponent, system::AsSystemLabel},
    hierarchy::{BuildChildren, Children},
    math::{IVec3, Quat, UVec3},
    pbr::{DirectionalLight, DirectionalLightBundle},
    prelude::{
        default, info, AssetServer, Assets, Color, Commands, Component, CoreStage, Deref, DerefMut,
        Entity, Gamepad, Handle, Msaa, OrthographicProjection, ParallelSystemDescriptorCoercion,
        PerspectiveProjection, Plugin, Query, Res, ResMut, StartupStage, SystemSet, Transform,
        With,
    },
    reflect::Reflect,
    transform::TransformBundle,
    window::Windows,
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use ecs_ex::{entity_default, WithName};
use ron::ser::PrettyConfig;

use crate::prelude::{
    board_size, ghost_piece::ghost_piece_position, ghost_piece_color, model_instance_added,
    move_model_instances, sound_effects, BagRandomizer, Board, BoardBundle, BoardMaterial,
    BoardPlugin, BoardRotation, BoardTransform, CameraFocus, CameraPlugin, CameraZoom, CellMesh,
    CollisionLayer, FailedGridMoves, GhostPiece, GridMovesBundle, InputController, InputPlugin,
    InputReader, LerpCameraBundle, LerpCameraProjection, LockTimer, Model, ModelInstance,
    ModelInstanceBundle, ModelPlugin, MoveType, OrbitCamera, SoundEffect, SoundEffectsPlugin,
    SuccessfulGridMoves, BOARD_DEPTH, BOARD_HEIGHT, BOARD_MAX_X, BOARD_MAX_Y, BOARD_MAX_Z,
    BOARD_WIDTH, MODELS_TETROMINOS, SOUND_DROP, SOUND_INSTALL, SOUND_MINO_1, SOUND_MINO_2,
    SOUND_MINO_3, SOUND_MINO_4, SOUND_MINO_5, SOUND_MINO_6, SOUND_MINO_7, SOUND_PIECE_HIT,
    SOUND_PIECE_MOVE, SOUND_ROTATE,
};

use bevy_instancing::prelude::{IndirectRenderingPlugin, MeshInstanceBundle, ColorInstanceBundle};

pub const TICK_BASELINE: f32 = 60.0;
pub const DAS_DELAY: f32 = 7.0;
pub const DAS_RATE: f32 = TICK_BASELINE / 2.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Msaa { samples: 4 });

        app.add_plugin(DebugLinesPlugin::with_depth_test(true))
            .add_plugin(ModelPlugin)
            .add_plugin(SoundEffectsPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(IndirectRenderingPlugin)
            .add_plugin(BoardPlugin)
            .add_plugin(CameraPlugin);
    }
}

pub struct GameSandboxPlugin;

impl Plugin for GameSandboxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Game systems
        app.add_startup_system(setup_window_title)
            .add_startup_system_to_stage(StartupStage::PostStartup, setup_gameplay_sandbox);

        app.add_system(land_pieces.after(move_model_instances))
            .add_system(lock_pieces.after(land_pieces))
            .add_system(next_piece.after(lock_pieces).before(model_instance_added));

        app.add_system(ghost_piece_color.after(lock_pieces));
        app.add_system(ghost_piece_position.after(move_model_instances));
        app.add_system(piece_debug_lines.after(ghost_piece_position));

        app.add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::default()
                .with_system(sound_move)
                .with_system(sound_hard_drop)
                .with_system(sound_soft_drop)
                .before(sound_effects.as_system_label()),
        );
    }
}

const GENERATE_MODELS: bool = true;
pub fn generate_model(
    asset_server: &AssetServer,
    models: &mut Assets<Model>,
    path: &str,
    voxels: Vec<(IVec3, Color)>,
    half_offset: IVec3,
) -> Handle<Model> {
    if GENERATE_MODELS {
        let model = Model {
            voxels,
            half_offset,
        };
        let file = File::create("assets/".to_string() + path).unwrap();
        ron::ser::to_writer_pretty(file, &model, PrettyConfig::default()).unwrap();
        models.add(model)
    } else {
        info!("Queueing load from {path:}");
        asset_server.load(path)
    }
}

pub fn spawn_camera(input_reader: InputReader, commands: &mut Commands) -> (Entity, Entity) {
    let mut camera_orbit = entity_default();
    let camera_strafe = commands
        .spawn()
        .with_name("Camera Strafe")
        .insert_bundle(TransformBundle::default())
        .insert(CameraFocus::default())
        .with_children(|children| {
            camera_orbit = children
                .spawn()
                .with_name("Camera Orbit")
                .insert_bundle(
                    LerpCameraBundle::<PerspectiveProjection, OrthographicProjection> {
                        projection: LerpCameraProjection {
                            rhs: OrthographicProjection {
                                scale: 0.04,
                                ..default()
                            },
                            factor: 0.0,
                            target_factor: 0.0,
                            speed_in: 25.0,
                            speed_out: 0.25,
                            ..default()
                        },
                        ..default()
                    },
                )
                .insert(OrbitCamera {
                    input_yaw: input_reader.camera_x,
                    input_pitch: input_reader.camera_y,
                    yaw: -std::f32::consts::FRAC_PI_4 + 0.0001,
                    pitch: -std::f32::consts::FRAC_PI_8,
                    ..default()
                })
                .insert(CameraZoom {
                    zoom_out: input_reader.camera_zoom_out,
                    zoom_in: input_reader.camera_zoom_in,
                    fov: 60.0_f32.to_radians(),
                    min: 20.0_f32.to_radians(),
                    max: 60.0_f32.to_radians(),
                })
                .id();
        })
        .id();

    (camera_strafe, camera_orbit)
}

fn setup_window_title(mut windows: ResMut<Windows>) {
    // Set window title
    if let Some(window) = windows.get_primary_mut() {
        window.set_title("Godris".into());
    }
}

pub fn setup_gameplay_sandbox(
    asset_server: Res<AssetServer>,
    cell_mesh: Res<CellMesh>,
    mut instanced_materials: ResMut<Assets<BoardMaterial>>,
    mut models: ResMut<Assets<Model>>,
    mut commands: Commands,
) {
    let board = commands
        .spawn()
        .with_name("Board")
        .insert_bundle(BoardBundle {
            board: Board {
                size: UVec3::new(BOARD_WIDTH as u32, BOARD_HEIGHT as u32, BOARD_DEPTH as u32),
                visible_min: Some(UVec3::new(1, 0, 1)),
                visible_max: Some(board_size().as_uvec3() - UVec3::new(1, 0, 1)),
                ..default()
            },
            ..default()
        })
        .id();

    // Directional Light
    commands
        .spawn()
        .with_name("Directional Light")
        .insert_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 4000.,
                ..default()
            },
            transform: Transform {
                // Workaround: Pointing straight up or down prevents directional shadow from rendering
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 * 0.6),
                ..default()
            },
            ..default()
        });

    // Reference Cube
    commands
        .spawn()
        .with_name("Reference Cube")
        .insert_bundle(ColorInstanceBundle {
            instance_bundle: MeshInstanceBundle {
                mesh: cell_mesh.0.clone(),
                transform: Transform::from_xyz(0.0, BOARD_HEIGHT as f32, 0.0).into(),
                material: instanced_materials.add(default()),
                ..default()
            },
            ..default()
        });

    // Plane models
    let model_unit = generate_model(
        &asset_server,
        &mut models,
        "model/unit.model.ron",
        vec![(IVec3::ZERO, Color::WHITE)],
        IVec3::new(0, 0, 0),
    );

    let model_xz_plane = generate_model(
        &asset_server,
        &mut models,
        "model/xz_plane.model.ron",
        (0..BOARD_WIDTH)
            .flat_map(move |x| {
                (0..BOARD_DEPTH)
                    .map(move |z| (IVec3::new(x as i32, 0, z as i32), Color::rgb(0.2, 0.5, 0.2)))
            })
            .collect(),
        default(),
    );

    let model_xy_plane = generate_model(
        &asset_server,
        &mut models,
        "model/xy_plane.model.ron",
        (0..BOARD_WIDTH)
            .flat_map(move |x| {
                (0..BOARD_HEIGHT)
                    .map(move |y| (IVec3::new(x as i32, y as i32, 0), Color::rgb(0.5, 0.5, 0.5)))
            })
            .collect(),
        default(),
    );

    let model_yz_plane = generate_model(
        &asset_server,
        &mut models,
        "model/yz_plane.model.ron",
        (0..BOARD_HEIGHT)
            .flat_map(move |y| {
                (0..BOARD_DEPTH)
                    .map(move |z| (IVec3::new(0, y as i32, z as i32), Color::rgb(0.5, 0.5, 0.5)))
            })
            .collect(),
        default(),
    );

    let model_wetrix_i = generate_model(
        &asset_server,
        &mut models,
        "model/wetrix/i_block.model.ron",
        (-4..5)
            .flat_map(move |x| {
                (-1..2).map(move |z| (IVec3::new(x, 0, z), Color::hex("8be9fd").unwrap()))
            })
            .collect(),
        IVec3::new(0, 0, 0),
    );

    let model_wetrix_l = generate_model(
        &asset_server,
        &mut models,
        "model/wetrix/l_block.model.ron",
        (-4..5)
            .flat_map(move |x| {
                (-4..5).flat_map(move |z| {
                    if x >= -1 && z >= -1 {
                        return None;
                    }

                    Some((IVec3::new(x, 0, z), Color::hex("ffb86c").unwrap()))
                })
            })
            .collect(),
        IVec3::new(0, 0, 0),
    );

    let model_wetrix_o = generate_model(
        &asset_server,
        &mut models,
        "model/wetrix/o_block.model.ron",
        (-4..5)
            .flat_map(move |x| {
                (-4..5).flat_map(move |z| {
                    if (x >= -1 && x <= 1) && (z >= -1 && z <= 1) {
                        return None;
                    }

                    Some((IVec3::new(x, 0, z), Color::hex("f1fa8c").unwrap()))
                })
            })
            .collect(),
        IVec3::new(0, 0, 0),
    );

    let model_wetrix_t = generate_model(
        &asset_server,
        &mut models,
        "model/wetrix/t_block.model.ron",
        (-4..5)
            .flat_map(move |x| {
                (-4..5).flat_map(move |z| {
                    if z >= -1 && (x < -1 || x > 1) {
                        return None;
                    }

                    Some((IVec3::new(x, 0, z), Color::hex("bd93f9").unwrap()))
                })
            })
            .collect(),
        IVec3::new(0, 0, 0),
    );

    let model_cube_3 = generate_model(
        &asset_server,
        &mut models,
        "model/cube_3.model.ron",
        (-1..2)
            .flat_map(move |x| {
                (0..3).flat_map(move |y| {
                    (-1..2).flat_map(move |z| Some((IVec3::new(x, y, z), Color::ANTIQUE_WHITE)))
                })
            })
            .collect(),
        IVec3::new(0, 0, 0),
    );

    // Floor
    commands
        .spawn()
        .with_name("Floor")
        .insert_bundle(ModelInstanceBundle {
            model_instance: ModelInstance { board, ..default() },
            model: model_xz_plane.clone(),
            ..default()
        });

    // Back wall
    commands
        .spawn()
        .with_name("Back Wall")
        .insert_bundle(ModelInstanceBundle {
            model_instance: ModelInstance { board, ..default() },
            model: model_xy_plane.clone(),
            ..default()
        });

    // Front wall
    commands
        .spawn()
        .with_name("Front Wall")
        .insert_bundle(ModelInstanceBundle {
            transform: BoardTransform {
                translation: IVec3::Z * BOARD_MAX_Z as i32,
                ..default()
            }
            .into(),
            model_instance: ModelInstance { board, ..default() },
            model: model_xy_plane.clone(),
            ..default()
        });

    // Left wall
    commands
        .spawn()
        .with_name("Left Wall")
        .insert_bundle(ModelInstanceBundle {
            model_instance: ModelInstance { board, ..default() },
            model: model_yz_plane.clone(),
            ..default()
        });

    // Right wall
    commands
        .spawn()
        .with_name("Right Wall")
        .insert_bundle(ModelInstanceBundle {
            transform: BoardTransform {
                translation: IVec3::X * BOARD_MAX_X as i32,
                ..default()
            }
            .into(),
            model_instance: ModelInstance { board, ..default() },
            model: model_yz_plane.clone(),
            ..default()
        });

    // Controller input
    let mut input_reader = InputReader {
        gamepad: Gamepad(0),
        ..default()
    };
    let input_reader_entity = input_reader.spawn(commands.spawn()).id();

    // Camera
    let (camera_strafe, camera_orbit) = spawn_camera(input_reader, &mut commands);

    // Ghost Piece
    commands
        .spawn()
        .with_name("Ghost Piece")
        .insert_bundle(ModelInstanceBundle {
            model_instance: ModelInstance {
                board,
                collision_layer: CollisionLayer(1),
                ..default()
            },
            model: model_unit.clone(),
            transform: BoardTransform {
                translation: IVec3::new(
                    BOARD_WIDTH as i32 / 2,
                    BOARD_HEIGHT as i32 / 2,
                    BOARD_DEPTH as i32 / 2,
                ),
                ..default()
            }
            .into(),
            color: Color::rgba(1.0, 1.0, 1.0, 0.5).into(),
        })
        .insert(GhostPiece);

    // Next piece timer
    commands
        .spawn()
        .with_name("Next Piece Timer")
        .insert(NextPieceTimer::default());

    // Controller moves
    let mut controller_moves = InputController {
        controller: input_reader_entity,
        ..default()
    };
    controller_moves
        .spawn(&mut commands, &input_reader, camera_orbit, camera_strafe)
        .with_name("Controller Moves");

    // Bag Randomizer
    let voxels = MODELS_TETROMINOS
        .into_iter()
        .map(|path| asset_server.load::<Model, _>(*path));

    //commands.spawn().insert(BagRandomizer::new(voxels, 2));
    commands
        .spawn()
        .with_name("Bag Randomizer")
        .insert(BagRandomizer::new(
            vec![
                model_cube_3,
                model_unit,
                model_wetrix_i,
                model_wetrix_l,
                model_wetrix_o,
                model_wetrix_t,
            ],
            2,
        ));
}

fn sound_move(mut sound_effects: ResMut<Events<SoundEffect>>, query: Query<&SuccessfulGridMoves>) {
    let mut moved = false;
    let mut rotated = false;
    for successful in query.iter() {
        if successful.iter().any(|grid_move| {
            grid_move.move_type == MoveType::Hit && grid_move.delta.translation != IVec3::ZERO
        }) {
            moved = true;
        }

        if successful.iter().any(|grid_move| {
            grid_move.move_type == MoveType::Hit
                && grid_move.delta.rotation != BoardRotation::Identity
        }) {
            rotated = true;
        }

        if moved && rotated {
            break;
        }
    }

    if moved {
        sound_effects.send(SOUND_PIECE_MOVE.into());
    }

    if rotated {
        sound_effects.send(SOUND_ROTATE.into());
    }
}

fn sound_hard_drop(mut sound_effects: ResMut<Events<SoundEffect>>, query: Query<&FailedGridMoves>) {
    for failed in query.iter() {
        if failed
            .iter()
            .any(|grid_move| grid_move.move_type == MoveType::Land)
        {
            sound_effects.send(SOUND_DROP.into());
            break;
        }
    }
}

fn sound_soft_drop(mut sound_effects: ResMut<Events<SoundEffect>>, query: Query<&FailedGridMoves>) {
    for failed in query.iter() {
        if failed
            .iter()
            .any(|grid_move| grid_move.move_type == MoveType::Lock)
        {
            sound_effects.send(SOUND_PIECE_HIT.into());
            break;
        }
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Component, Reflect)]
#[reflect(Component)]
pub struct NextPieceTimer(pub Timer);

impl Default for NextPieceTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.52, false))
    }
}

fn next_piece(
    time: Res<Time>,
    mut sound_effects: ResMut<Events<SoundEffect>>,
    mut commands: Commands,
    models: Res<Assets<Model>>,
    query_board: Query<(Entity, &Board, &BoardTransform)>,
    mut query_timer: Query<&mut NextPieceTimer>,
    mut query_controller_moves: Query<&mut InputController>,
    mut query_camera_focus: Query<&mut CameraFocus>,
    mut query_bag_randomizer: Query<&mut BagRandomizer<Handle<Model>>>,
    mut query_ghost_piece: Query<(&mut ModelInstance, &mut Handle<Model>), With<GhostPiece>>,
) {
    let mut timer = query_timer.iter_mut().next().unwrap();

    if timer.paused() {
        return;
    }

    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    let mut bag_randomizer = query_bag_randomizer.iter_mut().next().unwrap();
    let model_handle = bag_randomizer.next().unwrap();
    let model = models.get(model_handle.clone()).unwrap();

    let (board_entity, board, board_transform) = query_board.iter().next().unwrap();
    let transform = BoardTransform {
        translation: IVec3::new(
            (BOARD_WIDTH / 2) as i32,
            BOARD_MAX_Y as i32 - 3,
            (BOARD_DEPTH / 2) as i32,
        ),
        ..default()
    };

    if board.intersect_model(model, *board_transform * transform, CollisionLayer(0), None) {
        return;
    }

    let model_instance = commands
        .spawn()
        .with_name("Piece")
        .insert_bundle(ModelInstanceBundle {
            transform: transform.into(),
            model_instance: ModelInstance {
                board: board_entity,
                ..default()
            },
            model: model_handle.clone(),
            ..default()
        })
        .insert_bundle(GridMovesBundle::default())
        .insert(LockTimer::default())
        .id();

    let mut controller_moves = query_controller_moves.iter_mut().next().unwrap();
    controller_moves.target = Some(model_instance);

    let mut camera_focus = query_camera_focus.iter_mut().next().unwrap();
    camera_focus.target = model_instance;

    if let Some(path) = match rand::random::<u8>() % 7 {
        0 => Some(SOUND_MINO_1),
        1 => Some(SOUND_MINO_2),
        2 => Some(SOUND_MINO_3),
        3 => Some(SOUND_MINO_4),
        4 => Some(SOUND_MINO_5),
        5 => Some(SOUND_MINO_6),
        6 => Some(SOUND_MINO_7),
        _ => None,
    } {
        sound_effects.send(path.into());
    }

    timer.pause();
    timer.reset();

    let (mut ghost_piece, mut ghost_piece_model) = query_ghost_piece.iter_mut().next().unwrap();
    ghost_piece.model_loaded = false;
    *ghost_piece_model = model_handle;
}

pub fn land_pieces(
    models: Res<Assets<Model>>,
    mut sound_effects: ResMut<Events<SoundEffect>>,
    query_board: Query<&Board>,
    mut query_controller_moves: Query<&mut InputController>,
    mut query_moves: Query<(
        &ModelInstance,
        &Handle<Model>,
        &BoardTransform,
        &SuccessfulGridMoves,
        &Children,
        &mut LockTimer,
    )>,
    mut query_next_piece_timer: Query<&mut NextPieceTimer>,
) {
    for mut controller_moves in query_controller_moves.iter_mut() {
        let target = if let Some(target) = controller_moves.target {
            target
        } else {
            continue;
        };

        let (model_instance, model, board_transform, grid_moves, children, mut lock_timer) =
            if let Ok(components) = query_moves.get_mut(target) {
                components
            } else {
                continue;
            };

        let model = models.get(model).unwrap();

        let board = query_board.get(model_instance.board).unwrap();

        if lock_timer.paused() {
            if board.intersect_model(
                model,
                *board_transform
                    + BoardTransform {
                        translation: -IVec3::Y,
                        ..default()
                    },
                CollisionLayer(0),
                Some(&children),
            ) {
                if grid_moves
                    .iter()
                    .any(|grid_move| grid_move.move_type == MoveType::Land)
                {
                    sound_effects.send(SOUND_DROP.into());
                    lock_timer.unpause();
                } else if grid_moves
                    .iter()
                    .any(|grid_move| grid_move.move_type == MoveType::Lock)
                {
                    let mut next_piece_timer = query_next_piece_timer.iter_mut().next().unwrap();
                    lock_piece(
                        &mut controller_moves,
                        &mut lock_timer,
                        &mut next_piece_timer,
                        &mut sound_effects,
                    )
                }
            }
        } else {
            if !board.intersect_model(
                model,
                *board_transform
                    + BoardTransform {
                        translation: -IVec3::Y,
                        ..default()
                    },
                CollisionLayer(0),
                Some(&children),
            ) {
                lock_timer.pause();
                lock_timer.reset();
            }
        }
    }
}

fn lock_piece(
    controller_moves: &mut InputController,
    lock_timer: &mut Timer,
    next_piece_timer: &mut Timer,
    sound_effects: &mut Events<SoundEffect>,
) {
    controller_moves.target = None;
    sound_effects.send(SoundEffect {
        path: SOUND_INSTALL,
    });

    lock_timer.reset();
    lock_timer.pause();

    next_piece_timer.unpause();
}

pub fn lock_pieces(
    time: Res<Time>,
    mut sound_effects: ResMut<Events<SoundEffect>>,
    mut query_controller_moves: Query<&mut InputController>,
    mut query_moves: Query<(&FailedGridMoves, &mut LockTimer)>,
    mut query_next_piece_timer: Query<&mut NextPieceTimer>,
) {
    for mut controller_moves in query_controller_moves.iter_mut() {
        let target = if let Some(target) = controller_moves.target {
            target
        } else {
            continue;
        };

        let (failed_moves, mut lock_timer) = if let Ok(failed_moves) = query_moves.get_mut(target) {
            failed_moves
        } else {
            continue;
        };

        let mut next_piece_timer = query_next_piece_timer.iter_mut().next().unwrap();

        if failed_moves
            .iter()
            .any(|grid_move| grid_move.move_type == MoveType::Lock)
        {
            lock_piece(
                &mut controller_moves,
                &mut lock_timer,
                &mut next_piece_timer,
                &mut sound_effects,
            );
            return;
        }

        if lock_timer.paused() {
            if !failed_moves
                .iter()
                .any(|grid_move| grid_move.move_type == MoveType::Land)
            {
                continue;
            }

            lock_timer.unpause();
        } else {
            lock_timer.tick(Duration::from_secs_f32(time.delta_seconds()));

            if !lock_timer.finished() {
                continue;
            }

            lock_piece(
                &mut controller_moves,
                &mut lock_timer,
                &mut next_piece_timer,
                &mut sound_effects,
            );
        }
    }
}

pub fn piece_debug_lines(
    mut lines: ResMut<DebugLines>,
    query_moves: Query<&InputController>,
    query_board_transform: Query<&BoardTransform>,
    query_ghost_piece: Query<Entity, With<GhostPiece>>,
) {
    let ghost_piece = query_ghost_piece.iter().next().unwrap();
    let transform = *query_board_transform.get(ghost_piece).unwrap();

    for moves in query_moves.iter() {
        let target = if let Some(target) = moves.target {
            target
        } else {
            continue;
        };

        let target = if let Ok(target) = query_board_transform.get(target) {
            target
        } else {
            continue;
        };

        let board_size = board_size();
        let from = target.translation.as_vec3() - board_size * 0.5;
        let to = transform.translation.as_vec3() - board_size * 0.5;
        lines.line_gradient(from, to, 0.0, Color::LIME_GREEN, Color::WHITE);
    }
}
