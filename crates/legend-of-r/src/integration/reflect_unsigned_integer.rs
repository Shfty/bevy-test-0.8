use bevy::reflect::FromType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ReflectUnsignedInteger {
    pub min: u128,
    pub max: u128,
}

impl FromType<u8> for ReflectUnsignedInteger {
    fn from_type() -> Self {
        ReflectUnsignedInteger {
            min: u8::MIN as u128,
            max: u8::MAX as u128,
        }
    }
}

impl FromType<u16> for ReflectUnsignedInteger {
    fn from_type() -> Self {
        ReflectUnsignedInteger {
            min: u16::MIN as u128,
            max: u16::MAX as u128,
        }
    }
}

impl FromType<u32> for ReflectUnsignedInteger {
    fn from_type() -> Self {
        ReflectUnsignedInteger {
            min: u32::MIN as u128,
            max: u32::MAX as u128,
        }
    }
}

impl FromType<u64> for ReflectUnsignedInteger {
    fn from_type() -> Self {
        ReflectUnsignedInteger {
            min: u64::MIN as u128,
            max: u64::MAX as u128,
        }
    }
}

impl FromType<u128> for ReflectUnsignedInteger {
    fn from_type() -> Self {
        ReflectUnsignedInteger {
            min: u128::MIN,
            max: u128::MAX,
        }
    }
}

impl FromType<usize> for ReflectUnsignedInteger {
    fn from_type() -> Self {
        ReflectUnsignedInteger {
            min: usize::MIN as u128,
            max: usize::MAX as u128,
        }
    }
}

