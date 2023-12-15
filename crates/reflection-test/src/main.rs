// TODO: Reimplement integrators
//       * Ideally, should be capable of supporting RK4
//       * Therefore, force evaluation must take place inside the integrator
//       * Split ForceEvaluator out as its own component
//       * Implement integrators as systems over Query<Integrator, ForceEvaluator>
//       * Remove existing force / impulse application
//       * How to account for impulses?
//         * Integrator only understands forces
//         * May be better to model by using Multiply to offset against DT
//
// TODO: Replace existing rapier-ex implementation

use forces::{
    integrators::verlet::VelocityVerlet, Acceleration, Constant, Damping, Displacement,
    ForcesPlugin, KinematicConstraintBuilder, KinematicForceBuilder, Multiply, TransformDerivative,
    Velocity,
};

use bevy::{
    core::Name,
    log::LogPlugin,
    math::Vec3,
    prelude::{default, info, App, Commands, CoreStage, Entity, Query, Transform},
    MinimalPlugins,
};

use ecs_ex::{SpawnComponent, WithName};

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(LogPlugin);
    app.add_plugin(ForcesPlugin::default());

    app.add_startup_system(setup);

    app.add_system_to_stage(CoreStage::PostUpdate, print_translation);

    app.run();
}

fn setup(mut commands: Commands) {
    // Transform entities
    let a = commands
        .spawn()
        .insert(Transform::default())
        .insert(Velocity::default())
        .insert(Acceleration::default())
        .insert(VelocityVerlet::default())
        .with_name("a")
        .id();

    let b = commands
        .spawn()
        .insert(Transform::from_xyz(5.0, 0.0, 0.0))
        .insert(Velocity::default())
        .insert(Acceleration::default())
        .insert(VelocityVerlet::default())
        .with_name("b")
        .id();

    let middle = commands
        .spawn_component(Transform::default())
        .with_name("middle")
        .id();

    let above = commands
        .spawn_component(Transform::default())
        .with_name("above")
        .id();

    // Shared Forces
    let half_tension = commands
        .spawn_component(Multiply::default().with_multiplier(0.5))
        .id();

    let y_offset = commands
        .spawn_component(Constant::default().with_constant(TransformDerivative {
            translation: Vec3::Y,
            ..default()
        }))
        .id();

    // Kinematic Forces
    let _spring_damper_force = KinematicForceBuilder::new(&mut commands)
        .with_target(a)
        .with_force(
            Displacement::default()
                .with_from(a)
                .with_to(b),
        )
        //.with_force(Planar(Vec3::ONE.normalize()))
        //.with_foreign_force(half_tension)
        .with_force(Damping::default().with_target(a))
        .spawn();

    let _y_motor_force = KinematicForceBuilder::new(&mut commands)
        .with_target(b)
        .with_foreign_force(y_offset)
        .with_force(Multiply::default().with_multiplier(0.01))
        .spawn();

    // Kinematic Constraints
    let middle_to_a_constraint = KinematicConstraintBuilder::new(&mut commands)
        .with_target(middle)
        .with_force(Displacement::default().with_from(middle).with_to(a))
        .with_foreign_force(half_tension)
        .spawn();

    let middle_to_b_constraint = KinematicConstraintBuilder::new(&mut commands)
        .with_target(middle)
        .with_force(Displacement::default().with_from(middle).with_to(b))
        .with_foreign_force(half_tension)
        .spawn();

    let _above_to_middle_constraint = KinematicConstraintBuilder::new(&mut commands)
        .with_target(above)
        .with_force(Displacement::default().with_from(above).with_to(middle))
        .with_foreign_force(y_offset)
        .with_dependency(middle_to_a_constraint)
        .with_dependency(middle_to_b_constraint)
        .spawn();
}

pub fn print_translation(query: Query<(Entity, &Transform, Option<&Name>)>) {
    for (entity, transform, name) in query.iter() {
        let name = if let Some(name) = name {
            name.to_string()
        } else {
            format!("{:?}", entity)
        };

        info!("{name:}: {:?}", transform.translation);
    }

    println!();
}
