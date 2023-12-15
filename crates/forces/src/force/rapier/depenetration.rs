use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, Entity, World},
    reflect::Reflect,
};
use bevy_rapier3d::{prelude::RapierContext, rapier::prelude::RigidBodyType};
use ecs_ex::entity_default;
use struct_derive::WithFields;

use crate::TransformDerivative;

use crate::{Force, ReflectForce};

/// The force necessary to move an body out of collision
#[derive(Debug, Copy, Clone, WithFields, Component, Reflect)]
#[reflect(Component, Force)]
pub struct Depenetration {
    pub target: Entity,
}

impl Default for Depenetration {
    fn default() -> Self {
        Depenetration {
            target: entity_default(),
        }
    }
}

impl Force for Depenetration {
    fn force(
        &self,
        world: &World,
        mut displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        let rapier_context = world.get_resource::<RapierContext>()?;

        for pair in rapier_context.contact_pairs() {
            let lhs = pair.collider1();
            let rhs = pair.collider2();

            let (other, is_lhs) = if lhs == self.target {
                (rhs, true)
            } else if rhs == self.target {
                (lhs, false)
            } else {
                continue;
            };

            let other_body = rapier_context.entity2body().get(&other);
            let ratio = if let Some(other) = other_body {
                match rapier_context
                    .bodies
                    .get(*other)
                    .expect("Other collider has no valid body")
                    .body_type()
                {
                    RigidBodyType::Fixed => 1.0,
                    RigidBodyType::KinematicPositionBased
                    | RigidBodyType::KinematicVelocityBased => 0.5,
                    RigidBodyType::Dynamic => 0.0,
                }
            } else {
                1.0
            };

            if pair.has_any_active_contacts() {
                for manifold in pair.manifolds() {
                    if let Some(contact) = manifold.find_deepest_contact() {
                        let moved = if is_lhs {
                            manifold.local_n2()
                        } else {
                            manifold.local_n1()
                        } * -contact.dist()
                            * ratio;
                        displacement.translation += moved;
                    }
                }
            }
        }

        Some(displacement)
    }
}
