mod axial;
mod constant;
mod damping;
mod displacement;
mod foreign_constant;
mod length;
mod look_at;
mod max_projection;
mod multiply;
mod planar;

pub use axial::Axial;
pub use constant::Constant;
pub use damping::Damping;
pub use displacement::Displacement;
pub use foreign_constant::ForeignConstant;
pub use length::Length;
pub use look_at::{LookAt, LookAlong};
pub use max_projection::MaxProjection;
pub use multiply::Multiply;
pub use planar::Planar;

#[cfg(feature = "rapier")]
pub mod rapier;

use super::{TransformDerivative, Velocity};

use bevy::{
    prelude::{debug, App, Component, Deref, DerefMut, Entity, Plugin, World},
    reflect::{FromType, Reflect, TypeRegistry},
};

#[derive(Debug, Default, Copy, Clone)]
pub struct ForcePlugin;

impl Plugin for ForcePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Axial>()
            .register_type::<Constant>()
            .register_type::<Damping>()
            .register_type::<Displacement>()
            .register_type::<ForeignConstant>()
            .register_type::<Length>()
            .register_type::<LookAt>()
            .register_type::<Multiply>()
            .register_type::<Planar>()
            .register_type::<MaxProjection>();

        #[cfg(feature = "rapier")]
        app.register_type::<rapier::Depenetration>()
            .register_type::<rapier::Gravity>()
            .register_type::<rapier::Raycast>();
    }
}

/// A type that can calculate a TransformDerivative
pub trait Force {
    fn force(
        &self,
        world: &World,
        displacement: TransformDerivative,
    ) -> Option<TransformDerivative>;
}

/// Reflection adapter for Force
#[derive(Copy, Clone)]
pub struct ReflectForce {
    force: fn(&World, Entity, TransformDerivative) -> Option<TransformDerivative>,
}

impl<C: Component + Force + Reflect> FromType<C> for ReflectForce {
    fn from_type() -> Self {
        ReflectForce {
            force: |world, entity, displacement| world.get::<C>(entity)?.force(world, displacement),
        }
    }
}

impl ReflectForce {
    pub fn force(
        &self,
        world: &World,
        entity: Entity,
        displacement: TransformDerivative,
    ) -> Option<TransformDerivative> {
        (self.force)(world, entity, displacement)
    }
}

/// Utility struct for resolving `Component`s that implement `Force` from the given `Entity`s via reflection
#[derive(Debug, Default, Clone, Deref, DerefMut, Reflect)]
pub struct ForceEvaluator(pub Vec<Entity>);

impl ForceEvaluator {
    pub fn force(&self, world: &World) -> Option<TransformDerivative> {
        let type_registry = world.get_resource::<TypeRegistry>()?;
        let type_registry_read = type_registry.read();
        let reflects = type_registry_read
            .iter()
            .flat_map(|reg| {
                reg.data::<ReflectForce>()
                    .map(|reflect| (reg.short_name(), reflect))
            })
            .collect::<Vec<_>>();

        let mut displacement = TransformDerivative::default();
        for disp_entity in self.iter() {
            for (name, reflect) in reflects.iter().copied() {
                if let Some(disp) = reflect.force(world, *disp_entity, displacement) {
                    debug!(
                        "ForceEvaluator reflected {name:} displacement of {disp:?} with target {:?}",
                        disp_entity
                    );
                    displacement = disp;
                }
            }
        }

        Some(displacement)
    }
}
