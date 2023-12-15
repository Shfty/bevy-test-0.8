use bevy::reflect::FromType;

#[derive(Debug, Default, Copy, Clone)]
pub struct ReflectFloat;

impl FromType<f32> for ReflectFloat {
    fn from_type() -> Self {
        ReflectFloat
    }
}

impl FromType<f64> for ReflectFloat {
    fn from_type() -> Self {
        ReflectFloat
    }
}

