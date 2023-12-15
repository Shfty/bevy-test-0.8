use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    math::Vec3,
    prelude::{default, Component, Transform, World},
    reflect::Reflect,
};
use transform_ex::{TransformField, TransformFieldMask};

use crate::TransformDerivative;
use ecs_ex::ForeignComponent;

use super::{Force, ReflectForce};

/// Length displacement, i.e. a length spring
#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component, MapEntities, Force)]
pub struct Length {
    pub from: ForeignComponent<Transform>,
    pub to: ForeignComponent<Transform>,
    pub length: f32,
    pub fields: TransformFieldMask,
}

impl Default for Length {
    fn default() -> Self {
        Self {
            from: default(),
            to: default(),
            length: 1.0,
            fields: default(),
        }
    }
}

impl MapEntities for Length {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.from.map_entities(entity_map)?;
        self.to.map_entities(entity_map)?;
        Ok(())
    }
}

impl Force for Length {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let from = self.from.get(world)?;
        let to = self.to.get(world)?;

        if self.fields.contains(TransformField::Translation) {
            displacement.translation = length_disp(from.translation, to.translation, self.length);
        }
        if self.fields.contains(TransformField::Rotation) {
            displacement.rotation = length_disp(
                from.rotation.to_scaled_axis(),
                to.rotation.to_scaled_axis(),
                self.length,
            );
        }
        if self.fields.contains(TransformField::Scale) {
            displacement.scale = length_disp(from.scale, to.scale, self.length);
        }

        Some(displacement)
    }
}

fn length_disp(from: Vec3, to: Vec3, length: f32) -> Vec3 {
    let delta = to - from;
    let delta_norm = delta.normalize();
    let delta_len = delta.length();
    let length_disp = delta_len - length;
    delta_norm * length_disp
}
