pub mod force;
pub mod integrators;
pub mod kinematic_constraint;
pub mod kinematic_force;

pub use force::{
    Axial, Constant, Damping, Displacement, Force, ForceEvaluator, ForcePlugin, ForeignConstant,
    Length, Multiply, Planar, ReflectForce,
};

pub use kinematic_constraint::{KinematicConstraintBuilder, KinematicConstraintPlugin};
pub use kinematic_force::{KinematicForceBuilder, KinematicForcePlugin, KinematicImpulseBuilder};

use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3,
    prelude::{App, Component, Deref, DerefMut, Plugin, SystemSet},
    reflect::Reflect,
};
use transform_ex::{TransformField, TransformFieldMask};

use self::integrators::IntegratorPlugin;

#[derive(Debug)]
pub struct ForcesPlugin {
    pub register_systems: bool,
}

impl Default for ForcesPlugin {
    fn default() -> Self {
        Self {
            register_systems: true,
        }
    }
}

impl Plugin for ForcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ForceDeltaTime>();

        app.register_type::<ForceDeltaTime>()
            .register_type::<TransformDerivative>()
            .register_type::<Velocity>()
            .register_type::<Acceleration>()
            .register_type::<ForceEvaluator>();

        app.add_plugin(ForcePlugin)
            .add_plugin(IntegratorPlugin {
                register_systems: self.register_systems,
            })
            .add_plugin(KinematicForcePlugin {
                register_systems: self.register_systems,
            })
            .add_plugin(KinematicConstraintPlugin {
                register_systems: self.register_systems,
            });
    }
}

impl ForcesPlugin {
    pub fn systems_prepare_integration() -> SystemSet {
        IntegratorPlugin::systems_prepare()
    }

    pub fn systems_solve_forces() -> SystemSet {
        KinematicForcePlugin::systems_solve_forces()
    }

    pub fn systems_integrate() -> SystemSet {
        IntegratorPlugin::systems_integrate()
    }

    pub fn systems_solve_constraints() -> SystemSet {
        KinematicConstraintPlugin::systems_solve_constraints()
    }
}

/// Representation of a change in transform, i.e. velocity, acceleration
#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct TransformDerivative {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl TransformDerivative {
    pub fn apply(mut self, f: impl Fn(Vec3) -> Vec3, fields: TransformFieldMask) -> Self {
        if fields.contains(TransformField::Translation) {
            self.translation = f(self.translation);
        }
        if fields.contains(TransformField::Rotation) {
            self.rotation = f(self.rotation);
        }
        if fields.contains(TransformField::Scale) {
            self.scale = f(self.scale);
        }
        self
    }
}

impl std::ops::Add<TransformDerivative> for TransformDerivative {
    type Output = Self;

    fn add(self, rhs: TransformDerivative) -> Self::Output {
        TransformDerivative {
            translation: self.translation + rhs.translation,
            rotation: self.rotation + rhs.rotation,
            scale: self.scale + rhs.scale,
        }
    }
}

impl std::ops::Sub<TransformDerivative> for TransformDerivative {
    type Output = Self;

    fn sub(self, rhs: TransformDerivative) -> Self::Output {
        TransformDerivative {
            translation: self.translation - rhs.translation,
            rotation: self.rotation - rhs.rotation,
            scale: self.scale - rhs.scale,
        }
    }
}

impl std::ops::Mul<TransformDerivative> for TransformDerivative {
    type Output = Self;

    fn mul(self, rhs: TransformDerivative) -> Self::Output {
        TransformDerivative {
            translation: self.translation * rhs.translation,
            rotation: self.rotation * rhs.rotation,
            scale: self.scale * rhs.scale,
        }
    }
}

impl std::ops::Div<TransformDerivative> for TransformDerivative {
    type Output = Self;

    fn div(self, rhs: TransformDerivative) -> Self::Output {
        TransformDerivative {
            translation: self.translation / rhs.translation,
            rotation: self.rotation / rhs.rotation,
            scale: self.scale / rhs.scale,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Deref, DerefMut, Component, Reflect)]
#[reflect(Component)]
pub struct Velocity(pub TransformDerivative);

#[derive(Debug, Default, Copy, Clone, Deref, DerefMut, Component, Reflect)]
#[reflect(Component)]
pub struct Acceleration(pub TransformDerivative);

/// The delta time used for force integration
#[derive(Debug, Copy, Clone, Deref, DerefMut, Reflect)]
pub struct ForceDeltaTime(f32);

impl Default for ForceDeltaTime {
    fn default() -> Self {
        Self(1.0 / 60.0)
    }
}
