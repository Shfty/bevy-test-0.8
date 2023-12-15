use bevy::{
    ecs::system::Command,
    prelude::{
        default, Component, Entity, ParallelSystemDescriptorCoercion, Plugin, Query, ResMut, Vec2,
    },
};
use bevy_rapier2d::{
    na::Vector2,
    prelude::{Collider, CollisionGroups, PhysicsStages, QueryFilter, RapierContext},
};

use crate::prelude::contact_depenetration;

pub struct ShapecastDepenetrationPlugin;

impl Plugin for ShapecastDepenetrationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(
            PhysicsStages::StepSimulation,
            shapecast_depenetration.after(contact_depenetration),
        );
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct ShapecastDepenetration {
    pub targets: Vec<Entity>,
}

pub struct InsertShapecastDepenetration {
    pub entity: Entity,
}

impl Command for InsertShapecastDepenetration {
    fn write(self, world: &mut bevy::prelude::World) {
        world
            .entity_mut(self.entity)
            .insert(ShapecastDepenetration {
                targets: vec![self.entity],
            });
    }
}

pub fn shapecast_depenetration(
    mut context: ResMut<RapierContext>,
    query_depenetration: Query<(&ShapecastDepenetration, Option<&CollisionGroups>)>,
) {
    for (shapecast_depenetration, collision_groups) in query_depenetration.iter() {
        let mut colliders = shapecast_depenetration
            .targets
            .iter()
            .flat_map(|target| context.entity2body().get(target))
            .flat_map(|handle| context.bodies.get(*handle))
            .flat_map(|body| {
                body.colliders()
                    .into_iter()
                    .flat_map(|handle| context.colliders.get(*handle))
                    .map(|collider| {
                        (
                            Vec2::from(*collider.translation()),
                            collider.rotation().angle(),
                            Collider::from(collider.shared_shape().clone()),
                        )
                    })
            })
            .collect::<Vec<_>>();

        let average_translation = colliders
            .iter()
            .map(|(translation, _, _)| translation)
            .sum::<Vec2>()
            / colliders.len() as f32;

        colliders
            .iter_mut()
            .for_each(|(translation, _, _)| *translation -= average_translation);

        if colliders.len() == 0 {
            continue;
        }

        let collider = Collider::compound(colliders);

        let collider_radius = collider.raw.compute_local_bounding_sphere().radius;

        let mut filter = QueryFilter::default();
        if let Some(groups) = collision_groups {
            filter = filter.groups((*groups).into());
        }

        for target in shapecast_depenetration.targets.iter() {
            filter = filter.exclude_rigid_body(*target);
        }

        // Check whether we're still intersecting
        let intersection = if let Some(intersection) =
            context.intersection_with_shape(average_translation, default(), &collider, filter)
        {
            intersection
        } else {
            continue;
        };

        let collider_body_handle =
            if let Some(handle) = context.entity2collider().get(&intersection) {
                handle
            } else {
                continue;
            };

        let collider_body = if let Some(collider) = context.colliders.get(*collider_body_handle) {
            collider
        } else {
            continue;
        };

        let local = average_translation - Vec2::from(*collider_body.translation());
        let (point, feature) = collider_body
            .shape()
            .project_local_point_and_get_feature(&local.into());

        let normal = if let Some(normal) = collider_body
            .shape()
            .feature_normal_at_point(feature, &point.point.into())
        {
            normal
        } else {
            continue;
        };

        let normal = Vec2::new(normal.x, normal.y);

        let radius = collider_radius
            + match collider.as_typed_shape() {
                bevy_rapier2d::prelude::ColliderView::HalfSpace(_) => 0.0,
                _ => collider.raw.compute_local_bounding_sphere().radius,
            };

        let vel = -normal * radius;
        let from = average_translation - vel;

        if let Some((_, toi)) = context.cast_shape(
            from,
            default(),
            vel,
            &collider,
            1.0,
            filter.predicate(&|entity| entity == intersection),
        ) {
            let to = from + (vel * (1.0 - toi.toi));
            let delta = from - to;

            for handle in shapecast_depenetration
                .targets
                .iter()
                .flat_map(|target| context.entity2body().get(target))
                .map(|handle| *handle)
                .collect::<Vec<_>>()
            {
                if let Some(body) = context.bodies.get_mut(handle) {
                    body.set_translation(*body.translation() + Vector2::from(delta), true);
                }
            }

            break;
        }
    }

    context.propagate_modified_body_positions_to_colliders();
}
