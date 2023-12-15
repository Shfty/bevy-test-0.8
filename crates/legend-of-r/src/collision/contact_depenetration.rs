use bevy::{
    ecs::system::Command,
    prelude::{
        default, Component, Entity, ParallelSystemDescriptorCoercion, Plugin, Query, ResMut, Vec3,
        With,
    },
};
use bevy_rapier2d::{
    na::Vector2,
    prelude::{systems::step_simulation, PhysicsStages, RapierContext, RigidBody},
};

pub struct ContactDepenetrationPlugin;

impl Plugin for ContactDepenetrationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(
            PhysicsStages::StepSimulation,
            contact_depenetration.after(step_simulation::<()>),
        );
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct ContactDepenetration {
    pub targets: Vec<Entity>,
}

pub struct InsertContactDepenetration {
    pub entity: Entity,
}

impl Command for InsertContactDepenetration {
    fn write(self, world: &mut bevy::prelude::World) {
        world.entity_mut(self.entity).insert(ContactDepenetration {
            targets: vec![self.entity],
        });
    }
}

pub fn contact_depenetration(
    mut context: ResMut<RapierContext>,
    query: Query<&ContactDepenetration, With<RigidBody>>,
) {
    for contact_depenetration in query.iter() {
        let targets = &contact_depenetration.targets;

        let contacts = contact_depenetration
            .targets
            .iter()
            .flat_map(|source| context.contacts_with(*source))
            .collect::<Vec<_>>();

        let mut delta: Vec3 = default();

        for contact_pair in contacts {
            if let Some((manifold, contact)) = contact_pair.find_deepest_contact() {
                let (body_1, body_2) = (manifold.rigid_body1(), manifold.rigid_body2());
                let normal = match (body_1, body_2) {
                    (Some(body), _) if targets.contains(&body) => manifold.local_n1(),
                    (_, Some(body)) if targets.contains(&body) => manifold.local_n2(),
                    _ => continue,
                };

                let dist = contact.dist();

                // If rapier computed a valid depenetration distance, apply it
                if dist < -f32::EPSILON {
                    delta += (contact.dist() * normal).extend(0.0);
                }
            }
        }

        for target in &contact_depenetration.targets {
            let body_handle = if let Some(handle) = context.entity2body().get(target) {
                *handle
            } else {
                continue;
            };

            let body = if let Some(body) = context.bodies.get_mut(body_handle) {
                body
            } else {
                continue;
            };

            body.set_translation(body.translation() + Vector2::new(delta.x, delta.y), true);
        }
    }

    context.propagate_modified_body_positions_to_colliders()
}
