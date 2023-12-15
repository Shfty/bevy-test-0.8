use std::collections::BTreeSet;

use bevy::{
    gltf::{Gltf, GltfExtras},
    prelude::{
        AssetEvent, Assets, Component, CoreStage, Deref, DerefMut, Entity, EventReader,
        ParallelSystemDescriptorCoercion, Plugin, ResMut, With,
    },
    scene::Scene,
};

pub struct GltfJsonPlugin;

impl Plugin for GltfJsonPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(CoreStage::Last, gltf_json_insert)
            .add_system_to_stage(CoreStage::Last, gltf_json_remove.after(gltf_json_insert));
    }
}

#[derive(Debug, Default, Clone, Component, Deref, DerefMut)]
pub struct GltfExtrasJson(pub serde_json::Value);

/// When a Gltf asset is created or changed,
/// convert its GltfExtras component into a GltfExtrasJson
///
/// This allows later systems to consume extras data piecemeal without reserialization
pub fn gltf_json_insert(
    mut gltf_events: EventReader<AssetEvent<Gltf>>,
    gltf_assets: ResMut<Assets<Gltf>>,
    mut scene_assets: ResMut<Assets<Scene>>,
) {
    let handles = gltf_events
        .iter()
        .filter_map(|event| match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => Some(handle),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    for handle in handles {
        let gltf = if let Some(gltf) = gltf_assets.get(handle) {
            gltf
        } else {
            continue;
        };

        for handle in gltf.scenes.iter() {
            let scene = scene_assets.get_mut(handle).unwrap();

            let mut query = scene.world.query::<(Entity, &GltfExtras)>();

            for (entity, extras) in query
                .iter(&scene.world)
                .map(|(entity, extras)| (entity, extras.clone()))
                .collect::<Vec<_>>()
            {
                if let Ok(value) = serde_json::from_str(&extras.value) {
                    let mut entity = scene.world.entity_mut(entity);
                    entity.remove::<GltfExtras>();
                    entity.insert(GltfExtrasJson(value));
                }
            }
        }
    }
}

/// When all GltfExtrasJson consumers have run, remove to avoid blocking scene spawning
/// on type reflection and registration
pub fn gltf_json_remove(
    mut gltf_events: EventReader<AssetEvent<Gltf>>,
    gltf_assets: ResMut<Assets<Gltf>>,
    mut scene_assets: ResMut<Assets<Scene>>,
) {
    let handles = gltf_events
        .iter()
        .filter_map(|event| match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => Some(handle),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    for handle in handles {
        let gltf = if let Some(gltf) = gltf_assets.get(handle) {
            gltf
        } else {
            continue;
        };

        for handle in gltf.scenes.iter() {
            let scene = scene_assets.get_mut(handle).unwrap();
            let mut query = scene.world.query_filtered::<Entity, With<GltfExtrasJson>>();
            for entity in query.iter(&scene.world).collect::<Vec<_>>() {
                scene.world.entity_mut(entity).remove::<GltfExtrasJson>();
            }
        }
    }
}
