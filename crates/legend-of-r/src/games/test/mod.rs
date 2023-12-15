use bevy::prelude::{info, Entity, Plugin, Query};

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(animation());
    }
}

fn animation() -> impl FnMut(Query<Entity>) {
    move |query| {
        for entity in query.iter() {
            info!("Closure system, entity {entity:?}")
        }
    }
}
