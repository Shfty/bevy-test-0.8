use bevy::reflect::FromType;

#[derive(Debug, Default, Copy, Clone)]
pub struct ReflectBoolean;

impl FromType<bool> for ReflectBoolean {
    fn from_type() -> Self {
        ReflectBoolean
    }
}

