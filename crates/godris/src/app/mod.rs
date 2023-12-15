pub mod game;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::App,
    DefaultPlugins,
};
use bevy_inspector_egui::WorldInspectorPlugin;

use crate::prelude::GamePlugin;

use clap::{ArgEnum, Parser};

use self::game::GameSandboxPlugin;

#[derive(Debug, Copy, Clone, ArgEnum)]
enum AppMode {
    Run,
    DumpEcsSchedule,
    DumpRenderSchedule,
    DumpRenderGraph,
}

#[derive(Parser)]
pub struct Godris {
    #[clap(arg_enum, default_value = "run")]
    app_mode: AppMode,

    #[clap(skip)]
    app: App,
}

impl Godris {
    pub fn run() {
        let mut app: Self = Parser::parse();
        app.plugins();
        match app.app_mode {
            AppMode::Run => app.app.run(),
            AppMode::DumpEcsSchedule => bevy_mod_debugdump::print_schedule(&mut app.app),
            AppMode::DumpRenderSchedule => bevy_mod_debugdump::print_render_schedule(&mut app.app),
            AppMode::DumpRenderGraph => bevy_mod_debugdump::print_render_graph(&mut app.app),
        }
    }

    fn plugins<'a>(&mut self) {
        self.app
            .add_plugins_with(DefaultPlugins, |plugins| match self.app_mode {
                AppMode::DumpEcsSchedule
                | AppMode::DumpRenderSchedule
                | AppMode::DumpRenderGraph => plugins.disable::<bevy::log::LogPlugin>(),
                _ => plugins,
            })
            .add_plugin(GamePlugin)
            .add_plugin(GameSandboxPlugin)
            .add_plugin(WorldInspectorPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default());
    }
}
