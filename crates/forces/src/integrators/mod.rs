use bevy::{
    ecs::system::AsSystemLabel,
    prelude::{ParallelSystemDescriptorCoercion, Plugin, Query, SystemSet},
};

use super::Acceleration;

pub mod euler;
pub mod verlet;

#[derive(Debug, Copy, Clone)]
pub struct IntegratorPlugin {
    pub register_systems: bool,
}

impl Default for IntegratorPlugin {
    fn default() -> Self {
        Self {
            register_systems: true,
        }
    }
}

impl Plugin for IntegratorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<euler::ExplicitEuler>()
            .register_type::<euler::ImplicitEuler>()
            .register_type::<verlet::Verlet>()
            .register_type::<verlet::StormerVerlet>()
            .register_type::<verlet::VelocityVerlet>();

        if self.register_systems {
            app.add_system_set(Self::systems_prepare());
            app.add_system_set(
                Self::systems_integrate().after(prepare_accelerations.as_system_label()),
            );
        }
    }
}

impl IntegratorPlugin {
    pub fn systems_prepare() -> SystemSet {
        SystemSet::default().with_system(prepare_accelerations)
    }

    /// Returns a SystemSet encapsulating force functionality
    pub fn systems_integrate() -> SystemSet {
        SystemSet::default()
            .with_system(integrate_begin.before(integrate_end))
            .with_system(
                euler::explicit_euler
                    .after(integrate_begin)
                    .before(integrate_end),
            )
            .with_system(
                euler::implicit_euler
                    .after(integrate_begin)
                    .before(integrate_end),
            )
            .with_system(verlet::verlet.after(integrate_begin).before(integrate_end))
            .with_system(
                verlet::stormer_verlet
                    .after(integrate_begin)
                    .before(integrate_end),
            )
            .with_system(
                verlet::velocity_verlet
                    .after(integrate_begin)
                    .before(integrate_end),
            )
            .with_system(integrate_end.after(integrate_begin))
    }
}

pub fn prepare_accelerations(mut query: Query<&mut Acceleration>) {
    for mut acceleration in query.iter_mut() {
        **acceleration = Default::default();
    }
}

pub fn integrate_begin() {}
pub fn integrate_end() {}
