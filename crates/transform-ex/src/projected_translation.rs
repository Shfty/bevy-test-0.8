use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    math::Vec3,
    prelude::{Component, Entity, PerspectiveProjection, Plugin, Query, Res, Transform, Without},
    reflect::Reflect,
    render::camera::CameraProjection,
    window::Windows,
};

use result_system::ResultSystem;

pub struct ProjectedTranslationPlugin;

impl Plugin for ProjectedTranslationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<ProjectedTranslation>()
            .add_system(project_translation::<PerspectiveProjection>.result_system());
    }
}

#[derive(Debug, Copy, Clone, Reflect, Component)]
#[reflect(Component, MapEntities)]
pub struct ProjectedTranslation {
    pub projection_entity: Entity,
    pub depth: f32,
}

impl Default for ProjectedTranslation {
    fn default() -> Self {
        Self {
            projection_entity: Entity::from_raw(u32::MAX),
            depth: 1.0,
        }
    }
}

impl MapEntities for ProjectedTranslation {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.projection_entity = entity_map.get(self.projection_entity)?;
        Ok(())
    }
}

fn project_translation<T: Component + CameraProjection>(
    windows: Res<Windows>,
    mut query_mouse: Query<(&ProjectedTranslation, &mut Transform)>,
    query_camera: Query<(&Transform, &T), Without<ProjectedTranslation>>,
) -> Result<(), &'static str> {
    let window = windows
        .get_primary()
        .ok_or("Failed to get primary window")?;

    if let Some(cursor) = window.cursor_position() {
        let width = window.width();
        let height = window.height();

        for (mouse, mut transform) in query_mouse.iter_mut() {
            let (trx, proj) = query_camera
                .get(mouse.projection_entity)
                .or(Err("Failed to query projection entity"))?;

            let proj = proj.get_projection_matrix();

            let proj_depth = proj.project_point3(Vec3::new(0.0, 0.0, -mouse.depth)).z;

            let pos = proj.inverse().project_point3(Vec3::new(
                ((cursor.x / width) - 0.5) * 2.0,
                ((cursor.y / height) - 0.5) * 2.0,
                proj_depth,
            ));

            transform.translation = trx.rotation * pos
        }
    }

    Ok(())
}
