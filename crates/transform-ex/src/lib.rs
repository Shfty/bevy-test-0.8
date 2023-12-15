pub mod euler_rotation;
pub mod projected_translation;
pub mod quat_ex;

use bevy::reflect::Reflect;
use bitmask::bitmask;
bitmask! {
    #[derive(Debug, Reflect)]
    pub mask TransformFieldMask: u8 where flags TransformField {
        Translation = 1,
        Rotation = 2,
        Scale = 4,
    }

}

impl Default for TransformFieldMask {
    fn default() -> Self {
        TransformFieldMask::all()
    }
}
