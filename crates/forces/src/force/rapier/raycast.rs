use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    prelude::{Component, Entity, Transform, World},
    reflect::Reflect,
};
use bevy_rapier3d::prelude::{InteractionGroups, RapierContext};
use ecs_ex::ForeignComponent;
use struct_derive::WithFields;

use crate::TransformDerivative;

use crate::{Force, ReflectForce};

#[derive(Debug, Clone, Reflect)]
pub enum RaycastFilter {
    None,
    Blacklist(Vec<Entity>),
    Whitelist(Vec<Entity>),
}

impl Default for RaycastFilter {
    fn default() -> Self {
        RaycastFilter::None
    }
}

/// The force necessary to move an body out of collision
#[derive(Debug, Default, Clone, WithFields, Component, Reflect)]
#[reflect(Component, MapEntities, Force)]
pub struct Raycast {
    pub from: ForeignComponent<Transform>,
    pub to: ForeignComponent<Transform>,
    pub target: ForeignComponent<Transform>,
    pub solid: bool,
    pub filter: RaycastFilter,
}

impl MapEntities for Raycast {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.from.map_entities(entity_map)?;
        Ok(())
    }
}

impl Force for Raycast {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let rapier_context = world.get_resource::<RapierContext>()?;

        let from = self.from.get(world)?.translation;
        let to = self.to.get(world)?.translation;
        let target = self.target.get(world)?.translation;

        let delta = to - from;
        let max_toi = delta.length();
        let dir = delta.normalize();

        let result = rapier_context.cast_ray(
            from.into(),
            dir.into(),
            max_toi,
            self.solid,
            InteractionGroups::all(),
            Some(&|entity| match &self.filter {
                RaycastFilter::None => true,
                RaycastFilter::Blacklist(blacklist) => !blacklist.contains(&entity),
                RaycastFilter::Whitelist(whitelist) => whitelist.contains(&entity),
            }),
        );

        let toi = if let Some((_, toi)) = result {
            toi
        } else {
            max_toi
        };

        let hit_point = from + dir * toi;
        let delta = hit_point - target;
        displacement.translation += delta;

        Some(displacement)
    }
}
