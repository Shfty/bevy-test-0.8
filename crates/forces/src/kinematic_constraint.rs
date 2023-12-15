use bevy::{
    core::Name,
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
        system::AsSystemLabel,
    },
    math::Quat,
    prelude::{
        debug, default, error, App, Commands, Component, Entity, ParallelSystemDescriptorCoercion,
        ParamSet, Plugin, Query, ResMut, SystemSet, Transform, World,
    },
    reflect::Reflect,
};

use bevy_rapier3d::plugin::RapierContext;
use ecs_ex::entity_default;

use crate::{integrators::integrate_end, Force, ForceEvaluator};

#[derive(Debug, Copy, Clone)]
pub struct KinematicConstraintPlugin {
    pub register_systems: bool,
}

impl Default for KinematicConstraintPlugin {
    fn default() -> Self {
        Self {
            register_systems: true,
        }
    }
}

impl KinematicConstraintPlugin {
    /// Returns a SystemSet encapsulating constraint functionality.
    /// Must run after force integration.
    pub fn systems_solve_constraints() -> SystemSet {
        SystemSet::default()
            .with_system(apply_kinematic_constraints)
            .with_system(finalize_kinematic_constraints.after(apply_kinematic_constraints))
    }
}

impl Plugin for KinematicConstraintPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicConstraint>();

        if self.register_systems {
            app.add_system_set(
                Self::systems_solve_constraints().after(integrate_end.as_system_label()),
            );
        }
    }
}

/// Applies a set of `Force`s directly to a Transform by iteratively resolving a dependency graph
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct KinematicConstraint {
    pub target: Entity,
    pub force_evaluator: ForceEvaluator,
    pub dependencies: Vec<Entity>,
}

impl KinematicConstraint {
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = target;
        self
    }

    pub fn with_dependency(mut self, dependency: Entity) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn with_dependencies(mut self, dependencies: impl Iterator<Item = Entity>) -> Self {
        self.dependencies.extend(dependencies);
        self
    }

    pub fn with_force_evaluator(mut self, force_evaluator: ForceEvaluator) -> Self {
        self.force_evaluator = force_evaluator;
        self
    }
}

impl Default for KinematicConstraint {
    fn default() -> Self {
        Self {
            target: entity_default(),
            force_evaluator: default(),
            dependencies: vec![],
        }
    }
}

impl MapEntities for KinematicConstraint {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.target = entity_map.get(self.target)?;
        Ok(())
    }
}

pub struct KinematicConstraintBuilder<'c, 'w, 's> {
    commands: &'c mut Commands<'w, 's>,
    target: Entity,
    forces: Vec<Entity>,
    dependencies: Vec<Entity>,
}

impl<'c, 'w, 's> KinematicConstraintBuilder<'c, 'w, 's> {
    pub fn new(commands: &'c mut Commands<'w, 's>) -> Self {
        KinematicConstraintBuilder {
            commands,
            target: entity_default(),
            forces: default(),
            dependencies: default(),
        }
    }

    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = target;
        self
    }

    pub fn with_force<F: Component + Force>(mut self, force: F) -> Self {
        self.forces.push(self.commands.spawn().insert(force).id());
        self
    }

    pub fn with_foreign_force(mut self, force: Entity) -> Self {
        self.forces.push(force);
        self
    }

    pub fn with_dependency(mut self, dependency: Entity) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn spawn(self) -> Entity {
        self.commands
            .spawn()
            .insert(
                KinematicConstraint::default()
                    .with_target(self.target)
                    .with_force_evaluator(ForceEvaluator(self.forces))
                    .with_dependencies(self.dependencies.into_iter()),
            )
            .id()
    }
}

pub fn apply_kinematic_constraints(
    query_constraint: Query<(Entity, &KinematicConstraint)>,
    query_name: Query<&Name>,
    mut set: ParamSet<(&World, Query<&mut Transform>, ResMut<RapierContext>)>,
) {
    let mut unsolved_constraints: Vec<Entity> =
        query_constraint.iter().map(|(entity, _)| entity).collect();

    let mut solved_constraints: Vec<Entity> = vec![];

    let mut iter = 0;
    while unsolved_constraints.len() > 0 {
        debug!("Solver iteration {iter:}");
        // Stop iteration with an error if limit is exceeded
        if iter >= 40 {
            error!("Constraint solver exceeded iteration limit {}", 40);
            break;
        }
        iter += 1;

        // Find constraints whose dependencies are part of the solved set
        let mut solvable_constraints: Vec<Entity> = default();
        for entity in unsolved_constraints.drain(..).collect::<Vec<_>>() {
            let (_, constraint) = query_constraint.get(entity).unwrap();
            if constraint
                .dependencies
                .iter()
                .all(|dependency| solved_constraints.contains(dependency))
            {
                solvable_constraints.push(entity);
            } else {
                unsolved_constraints.push(entity);
            }
        }

        if solvable_constraints.is_empty() {
            error!("No solvable constraints at iteration {iter:}\nSolved constraints: {solved_constraints:#?}");
            break;
        }

        // Solve constraints
        for entity in solvable_constraints.iter() {
            let (_, kinematic_constraint) = query_constraint.get(*entity).unwrap();

            if let Some(force) = kinematic_constraint.force_evaluator.force(set.p0()) {
                if let Ok(mut transform) = set.p1().get_mut(kinematic_constraint.target) {
                    let entity_name = if let Ok(name) = query_name.get(kinematic_constraint.target)
                    {
                        name.to_string()
                    } else {
                        format!("{:?}", entity)
                    };

                    debug!("Applying Kinematic Constraint: {force:?} to {entity_name:?}");
                    transform.translation += force.translation;
                    transform.rotation *= Quat::from_scaled_axis(force.rotation);
                    transform.scale += force.scale;

                    // Apply to rapier bodies to ensure debug draw is up-to-date
                    let transform = *transform;
                    let mut rapier_context = set.p2();
                    if let Some(handle) = rapier_context
                        .entity2body()
                        .get(&kinematic_constraint.target)
                    {
                        let handle = *handle;
                        if let Some(body) = rapier_context.bodies.get_mut(handle) {
                            body.set_translation(
                                bevy_rapier3d::rapier::math::Vector::from(transform.translation),
                                false,
                            );
                            body.set_rotation(
                                bevy_rapier3d::rapier::math::AngVector::from(
                                    transform.rotation.to_scaled_axis(),
                                ),
                                false,
                            )
                        }
                    } else if let Some(handle) = rapier_context
                        .entity2collider()
                        .get(&kinematic_constraint.target)
                    {
                        let handle = *handle;
                        if let Some(body) = rapier_context.colliders.get_mut(handle) {
                            body.set_translation(bevy_rapier3d::rapier::math::Vector::from(
                                transform.translation,
                            ));
                            body.set_rotation(bevy_rapier3d::rapier::math::AngVector::from(
                                transform.rotation.to_scaled_axis(),
                            ))
                        }
                    }
                }
            }
        }

        // Add solved constraints to the set
        solved_constraints.extend(solvable_constraints.into_iter());
    }
}
pub fn finalize_kinematic_constraints(mut rapier_context: ResMut<RapierContext>) {
    rapier_context.propagate_modified_body_positions_to_colliders()
}
