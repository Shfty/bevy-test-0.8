use bevy::reflect::FromType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ReflectSignedInteger {
    pub min: i128,
    pub max: i128,
}

impl FromType<i8> for ReflectSignedInteger {
    fn from_type() -> Self {
        ReflectSignedInteger {
            min: i8::MIN as i128,
            max: i8::MAX as i128,
        }
    }
}

impl FromType<i16> for ReflectSignedInteger {
    fn from_type() -> Self {
        ReflectSignedInteger {
            min: i16::MIN as i128,
            max: i16::MAX as i128,
        }
    }
}

impl FromType<i32> for ReflectSignedInteger {
    fn from_type() -> Self {
        ReflectSignedInteger {
            min: i32::MIN as i128,
            max: i32::MAX as i128,
        }
    }
}

impl FromType<i64> for ReflectSignedInteger {
    fn from_type() -> Self {
        ReflectSignedInteger {
            min: i64::MIN as i128,
            max: i64::MAX as i128,
        }
    }
}

impl FromType<i128> for ReflectSignedInteger {
    fn from_type() -> Self {
        ReflectSignedInteger {
            min: i128::MIN,
            max: i128::MAX,
        }
    }
}

impl FromType<isize> for ReflectSignedInteger {
    fn from_type() -> Self {
        ReflectSignedInteger {
            min: isize::MIN as i128,
            max: isize::MAX as i128,
        }
    }
}

