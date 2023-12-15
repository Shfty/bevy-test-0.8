use std::marker::PhantomData;

use bevy::{
    core::Name,
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
        system::AsSystemLabel,
    },
    prelude::{
        debug, default, App, Commands, Component, Entity, ParallelSystemDescriptorCoercion,
        ParamSet, Plugin, Query, SystemSet, World,
    },
    reflect::Reflect,
};
use struct_derive::WithFields;

use crate::{
    force::Force as ForceBase,
    integrators::{integrate_begin, prepare_accelerations},
    Acceleration, Velocity,
};
use ecs_ex::{entity_default, ForeignQuery};

use super::ForceEvaluator;

#[derive(Debug, Copy, Clone)]
pub struct KinematicForcePlugin {
    pub register_systems: bool,
}

impl Default for KinematicForcePlugin {
    fn default() -> Self {
        Self {
            register_systems: true,
        }
    }
}

pub const LABEL_SOLVE_FORCES: &'static str = "kinematic_force_solve_forces";

impl KinematicForcePlugin {
    pub fn systems_solve_forces() -> SystemSet {
        SystemSet::default()
            .with_system(apply_kinematic_forces)
            .with_system(apply_kinematic_impulses.after(apply_kinematic_forces))
            .label(LABEL_SOLVE_FORCES)
    }
}

impl Plugin for KinematicForcePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicForce>()
            .register_type::<KinematicImpulse>();

        if self.register_systems {
            app.add_system_set(
                Self::systems_solve_forces()
                    .after(prepare_accelerations.as_system_label())
                    .before(integrate_begin.as_system_label()),
            );
        }
    }
}

/// Applies a set of `Force`s as acceleration or velocity
#[derive(Debug, Default, Component, WithFields, Reflect)]
#[reflect(Component, MapEntities)]
pub struct Kinematic<T: 'static + Default + Component + Reflect + Send + Sync> {
    pub target: ForeignQuery<&'static mut T>,
    pub force_evaluator: ForceEvaluator,
    #[reflect(ignore)]
    #[with_fields(ignore)]
    ty: PhantomData<T>,
}

/// Apply as acceleration
#[derive(Debug, Default, Copy, Clone, Reflect)]
pub struct Force;

/// Apply as velocity
#[derive(Debug, Default, Copy, Clone, Reflect)]
pub struct Impulse;

/// Applies a set of `Force`s as acceleration
pub type KinematicForce = Kinematic<Acceleration>;

/// Applies a set of `Force`s as velocity
pub type KinematicImpulse = Kinematic<Velocity>;

impl<T: Default + Component + Reflect> MapEntities for Kinematic<T> {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.target.map_entities(entity_map)?;
        Ok(())
    }
}

pub struct KinematicBuilder<'c, 'w, 's, T> {
    commands: &'c mut Commands<'w, 's>,
    target: Entity,
    forces: Vec<Entity>,
    _ty: T,
}

impl<'c, 'w, 's, T> KinematicBuilder<'c, 'w, 's, T>
where
    T: Default + Component + Reflect,
{
    pub fn new(commands: &'c mut Commands<'w, 's>) -> Self {
        KinematicBuilder {
            commands,
            target: entity_default(),
            forces: default(),
            _ty: default(),
        }
    }

    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = target;
        self
    }

    pub fn with_force<F: Component + ForceBase>(mut self, force: F) -> Self {
        self.forces.push(self.commands.spawn().insert(force).id());
        self
    }

    pub fn with_foreign_force(mut self, force: Entity) -> Self {
        self.forces.push(force);
        self
    }

    pub fn spawn(self) -> Entity {
        self.commands
            .spawn()
            .insert(
                Kinematic::<T>::default()
                    .with_target(self.target)
                    .with_force_evaluator(ForceEvaluator(self.forces)),
            )
            .id()
    }
}

pub type KinematicForceBuilder<'c, 'w, 's> = KinematicBuilder<'c, 'w, 's, Acceleration>;
pub type KinematicImpulseBuilder<'c, 'w, 's> = KinematicBuilder<'c, 'w, 's, Velocity>;

pub fn apply_kinematic_forces(
    query_force: Query<(Entity, &KinematicForce)>,
    query_name: Query<&Name>,
    mut set: ParamSet<(&World, Query<&mut Acceleration>)>,
) {
    for (entity, kinematic_force) in query_force.iter() {
        if let Some(force) = kinematic_force.force_evaluator.force(set.p0()) {
            if let Ok(mut acceleration) = kinematic_force.target.query_mut(&mut set.p1()) {
                let entity_name = if let Ok(name) = query_name.get(*kinematic_force.target) {
                    name.to_string()
                } else {
                    format!("{:?}", entity)
                };

                debug!("Applying Kinematic Force: {force:?} to {entity_name:?}");
                acceleration.translation += force.translation;
                acceleration.rotation += force.rotation;
                acceleration.scale += force.scale;
            }
        }
    }
}

pub fn apply_kinematic_impulses(
    query_force: Query<(Entity, &KinematicImpulse)>,
    query_name: Query<&Name>,
    mut set: ParamSet<(&World, Query<&mut Velocity>)>,
) {
    for (entity, kinematic_force) in query_force.iter() {
        if let Some(force) = kinematic_force.force_evaluator.force(set.p0()) {
            if let Ok(mut velocity) = kinematic_force.target.query_mut(&mut set.p1()) {
                let entity_name = if let Ok(name) = query_name.get(*kinematic_force.target) {
                    name.to_string()
                } else {
                    format!("{:?}", entity)
                };

                debug!("Applying Kinematic Impulse: {force:?} to {entity_name:?}");
                velocity.translation += force.translation;
                velocity.rotation += force.rotation;
                velocity.scale += force.scale;
            }
        }
    }
}
