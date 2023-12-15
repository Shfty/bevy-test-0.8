use bevy::{
    math::{EulerRot, Quat, Vec3},
    prelude::{Component, CoreStage, Plugin, Query, StartupStage, Transform},
    reflect::Reflect,
    ecs::reflect::ReflectComponent,
};

pub struct EulerRotationPlugin;

impl Plugin for EulerRotationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<EulerRotation>()
            .add_startup_system_to_stage(StartupStage::PostStartup, euler_rotation_setup)
            .add_system_to_stage(CoreStage::PostUpdate, euler_rotation_apply);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EulerRotation {
    pub rotation: Vec3,
}

impl Default for EulerRotation {
    fn default() -> Self {
        Self {
            rotation: Vec3::ZERO,
        }
    }
}

pub fn euler_rotation_setup(mut flycams: Query<(&Transform, &mut EulerRotation)>) {
    for (transform, mut euler_rotation) in flycams.iter_mut() {
        let (x, y, z) = transform.rotation.to_euler(EulerRot::YXZ);
        *euler_rotation = EulerRotation {
            rotation: Vec3::new(x, y, z),
            ..*euler_rotation
        };
    }
}

fn euler_rotation_apply(mut transforms: Query<(&EulerRotation, &mut Transform)>) {
    for (euler_rotation, mut transform) in transforms.iter_mut() {
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            euler_rotation.rotation.x,
            euler_rotation.rotation.y,
            0.0,
        );
    }
}
