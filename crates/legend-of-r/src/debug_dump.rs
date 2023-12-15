use std::{fs::File, io::Write};

use bevy::{
    prelude::{info, App, Plugin},
    tasks::IoTaskPool,
};

pub struct DebugDumpPlugin;

impl Plugin for DebugDumpPlugin {
    fn build(&self, app: &mut App) {
        let pool = IoTaskPool::get();

        info!("Dumping system schedule...");
        let system_schedule = bevy_mod_debugdump::get_schedule(app);
        info!("Dumping render schedule...");
        let render_schedule = bevy_mod_debugdump::get_render_schedule(app);
        info!("Dumping render graph...");
        let render_graph = bevy_mod_debugdump::get_render_schedule(app);
        pool.scope(move |pool| {
            pool.spawn(async move {
                let mut file = File::create("system_schedule.dot").unwrap();
                file.write_all(system_schedule.as_bytes()).unwrap();
                info!("Wrote system schedule to file");
            });

            pool.spawn(async move {
                let mut file = File::create("render_schedule.dot").unwrap();
                file.write_all(render_schedule.as_bytes()).unwrap();
                info!("Wrote render schedule to file");
            });

            pool.spawn(async move {
                let mut file = File::create("render_graph.dot").unwrap();
                file.write_all(render_graph.as_bytes()).unwrap();
                info!("Wrote render graph to file");
            });
        });
    }
}

