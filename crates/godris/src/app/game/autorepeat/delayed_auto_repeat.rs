use bevy::{
    core::{Time, Timer},
    prelude::{Component, Query, Res},
};

use crate::prelude::TICK_BASELINE;

use std::time::Duration;

#[derive(Debug, Copy, Clone, Component)]
pub enum AutorepeatState {
    Stopped,
    Delay,
    Ticking,
}

impl Default for AutorepeatState {
    fn default() -> Self {
        AutorepeatState::Stopped
    }
}

#[derive(Debug, Clone, Component)]
pub struct DelayedAutoRepeat {
    pub state: AutorepeatState,
    pub delay_timer: Timer,
    pub tick_timer: Timer,
}

impl DelayedAutoRepeat {
    pub fn new(delay_frames: f32, tick_hz: f32) -> Self {
        Self {
            delay_timer: Timer::from_seconds((1.0 / TICK_BASELINE) * delay_frames, false),
            tick_timer: Timer::from_seconds(1.0 / tick_hz, true),
            state: AutorepeatState::Stopped,
        }
    }

    pub fn tick_rate(&self) -> f32 {
        self.tick_timer.duration().as_secs_f32()
    }

    pub fn set_tick_rate(&mut self, tick_hz: f32) {
        self.tick_timer.set_duration(if tick_hz > 0.0 {
            Duration::from_secs_f32(1.0 / tick_hz)
        } else {
            Duration::MAX
        });
    }
}

impl Default for DelayedAutoRepeat {
    fn default() -> Self {
        Self::new(14., 60.)
    }
}

impl DelayedAutoRepeat {
    pub fn start(&mut self) {
        self.state = match self.state {
            AutorepeatState::Stopped => {
                self.delay_timer.unpause();
                AutorepeatState::Delay
            }
            _ => self.state,
        }
    }

    pub fn stop(&mut self) {
        self.delay_timer.pause();
        self.delay_timer.reset();

        self.tick_timer.pause();
        self.tick_timer.reset();

        self.state = AutorepeatState::Stopped;
    }

    pub fn tick(&mut self, time: &Time) {
        match self.state {
            AutorepeatState::Stopped => (),
            AutorepeatState::Delay => {
                self.delay_timer.tick(time.delta());
                if self.delay_timer.just_finished() {
                    self.state = AutorepeatState::Ticking;
                    self.tick_timer.unpause();
                }
            }
            AutorepeatState::Ticking => {
                self.tick_timer.tick(time.delta());
            }
        }
    }

    pub fn just_finished(&self) -> bool {
        match self.state {
            AutorepeatState::Stopped => false,
            AutorepeatState::Delay => false,
            AutorepeatState::Ticking => self.tick_timer.just_finished(),
        }
    }
}

pub fn delayed_auto_repeat(time: Res<Time>, mut query: Query<&mut DelayedAutoRepeat>) {
    for mut autorepeat in query.iter_mut() {
        autorepeat.tick(&time);
    }
}
