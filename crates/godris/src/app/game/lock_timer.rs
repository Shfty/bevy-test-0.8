use std::time::Duration;

use bevy::{prelude::{Deref, DerefMut, Component}, core::Timer};

#[derive(Debug, Clone, Deref, DerefMut, Component)]
pub struct LockTimer(pub Timer);

impl Default for LockTimer {
    fn default() -> Self {
        let mut timer = Timer::default();
        timer.set_duration(Duration::from_secs_f32(0.4));
        timer.set_repeating(true);
        timer.pause();
        LockTimer(timer)
    }
}

