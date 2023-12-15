use std::borrow::Cow;

use bevy::reflect::FromType;

#[derive(Debug, Default, Copy, Clone)]
pub struct ReflectString;

impl FromType<String> for ReflectString {
    fn from_type() -> Self {
        ReflectString
    }
}

impl FromType<Cow<'static, str>> for ReflectString {
    fn from_type() -> Self {
        ReflectString
    }
}

