pub mod archive;
pub mod background;
pub mod camera;
pub mod collision_group;
pub mod damage;
pub mod enemy;
pub mod entity_pool;
pub mod lives;
pub mod plane_collider;
pub mod player_input;
pub mod playfield;
pub mod shift_speed;
pub mod ship;
pub mod vulcan;

use bevy::prelude::{default, Plugin, Transform};

use bevy_rapier2d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::{DebugRenderMode, RapierDebugRenderPlugin},
};

use crate::prelude::{
    Alive, AnimationPlugin, BackgroundPlugin, CameraPlugin, ContactDepenetrationPlugin,
    DamagePlugin, Disable, HitPoints, LivesPlugin, PlaneColliderPlugin, PlayerInputPlugin,
    PlayfieldPlugin, RegisterAnimationType, ShapecastDepenetrationPlugin, ShipPlugin, VulcanPlugin,
    ShiftSpeedPlugin,
};

pub struct ShmupPlugin;

impl Plugin for ShmupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(bevy_egui::EguiPlugin)
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
            .add_plugin(RapierDebugRenderPlugin {
                mode: DebugRenderMode::COLLIDER_SHAPES,
                ..default()
            })
            .add_plugin(ContactDepenetrationPlugin)
            .add_plugin(ShapecastDepenetrationPlugin)
            .add_plugin(CameraPlugin)
            .add_plugin(PlaneColliderPlugin)
            .add_plugin(PlayerInputPlugin)
            .add_plugin(BackgroundPlugin)
            .add_plugin(PlayfieldPlugin)
            .add_plugin(AnimationPlugin::with_ui())
            .add_plugin(ShiftSpeedPlugin)
            .add_plugin(LivesPlugin)
            .add_plugin(VulcanPlugin)
            .add_plugin(DamagePlugin)
            .add_plugin(ShipPlugin);

        app.register_animation_type::<()>();
        app.register_animation_type::<HitPoints>();
        app.register_animation_type::<Transform>();
        app.register_animation_type::<Alive>();
        app.register_animation_type::<Disable<Alive>>();
    }
}
