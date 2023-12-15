use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Neg,
};

use bevy::{
    prelude::{Deref, DerefMut},
    reflect::Reflect,
};

/// A wrapper for floats that implements [`Ord`], [`Eq`], and [`Hash`] traits.
///
/// This is a work around for the fact that the IEEE 754-2008 standard,
/// implemented by Rust's [`f32`] type,
/// doesn't define an ordering for [`NaN`](f32::NAN),
/// and `NaN` is not considered equal to any other `NaN`.
///
/// Wrapping a float with `FloatOrd` breaks conformance with the standard
/// by sorting `NaN` as less than all other numbers and equal to any other `NaN`.
#[derive(Debug, Default, Copy, Clone, PartialOrd, Deref, DerefMut, Reflect)]
pub struct FloatOrd64(pub f64);

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for FloatOrd64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or_else(|| {
            if self.0.is_nan() && !other.0.is_nan() {
                Ordering::Less
            } else if !self.0.is_nan() && other.0.is_nan() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
    }
}

impl PartialEq for FloatOrd64 {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_nan() && other.0.is_nan() {
            true
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for FloatOrd64 {}

impl Hash for FloatOrd64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_nan() {
            // Ensure all NaN representations hash to the same value
            state.write(&f64::to_ne_bytes(f64::NAN));
        } else if self.0 == 0.0 {
            // Ensure both zeroes hash to the same value
            state.write(&f64::to_ne_bytes(0.0f64));
        } else {
            state.write(&f64::to_ne_bytes(self.0));
        }
    }
}

impl Neg for FloatOrd64 {
    type Output = FloatOrd64;

    fn neg(self) -> Self::Output {
        FloatOrd64(-self.0)
    }
}
