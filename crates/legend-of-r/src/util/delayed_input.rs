use std::{
    collections::VecDeque,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    time::{Duration, Instant},
};

use bevy::{
    ecs::schedule::SystemLabelId,
    prelude::{
        default, CoreStage, Input, ParallelSystemDescriptorCoercion, Plugin, Res, ResMut,
        SystemSet,
    },
};

pub struct DelayedInputPlugin<T> {
    pub after_label: Option<SystemLabelId>,
    pub delay: Duration,
    pub _phantom: PhantomData<T>,
}

impl<T> Default for DelayedInputPlugin<T> {
    fn default() -> Self {
        Self {
            after_label: default(),
            delay: Duration::from_secs(1),
            _phantom: default(),
        }
    }
}

impl<T> Plugin for DelayedInputPlugin<T>
where
    T: 'static + Send + Sync + Debug + Copy + Eq + Hash,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(DelayedInputs::<T> {
            delay: self.delay.clone(),
            ..default()
        });

        app.insert_resource(Input::<Delayed<T>>::default());

        if let Some(after_label) = self.after_label {
            app.add_system_to_stage(
                CoreStage::PreUpdate,
                delayed_inputs::<T>.after(after_label),
            );
        } else {
            app.add_system_to_stage(CoreStage::PreUpdate, delayed_inputs::<T>);
        }

        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::default()
                .with_system(delayed_input_clear::<T>)
                .with_system(delayed_input_press::<T>.after(delayed_input_clear::<T>))
                .with_system(delayed_input_release::<T>.after(delayed_input_clear::<T>))
                .after(delayed_inputs::<T>),
        );
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Delayed<T>(pub T);

#[derive(Debug, Clone)]
pub struct DelayedInputs<T> {
    pub delay: Duration,
    pub presses: VecDeque<(Instant, T)>,
    pub releases: VecDeque<(Instant, T)>,
}

impl<T> Default for DelayedInputs<T> {
    fn default() -> Self {
        Self {
            delay: default(),
            presses: default(),
            releases: default(),
        }
    }
}

fn delayed_inputs<T: 'static + Send + Sync + Copy + Eq + Hash>(
    mut delayed_input: ResMut<DelayedInputs<T>>,
    input: Res<Input<T>>,
) {
    for press in input.get_just_pressed() {
        delayed_input.presses.push_back((Instant::now(), *press));
    }

    for release in input.get_just_released() {
        delayed_input.releases.push_back((Instant::now(), *release));
    }
}

fn delayed_input_clear<T: 'static + Send + Sync + Copy + Eq + Hash>(
    mut delayed: ResMut<Input<Delayed<T>>>,
) {
    delayed.clear()
}

fn delayed_input_press<T: 'static + Send + Sync + Debug + Copy + Eq + Hash>(
    mut delayed_inputs: ResMut<DelayedInputs<T>>,
    mut delayed: ResMut<Input<Delayed<T>>>,
) {
    loop {
        if let Some(front) = delayed_inputs.presses.front().copied() {
            if Instant::now().duration_since(front.0) > delayed_inputs.delay {
                let (_, front) = delayed_inputs.presses.pop_front().unwrap();
                delayed.press(Delayed(front));
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn delayed_input_release<T: 'static + Send + Sync + Debug + Copy + Eq + Hash>(
    mut delayed_inputs: ResMut<DelayedInputs<T>>,
    mut delayed: ResMut<Input<Delayed<T>>>,
) {
    loop {
        if let Some(front) = delayed_inputs.releases.front().copied() {
            if Instant::now().duration_since(front.0) > delayed_inputs.delay {
                let (_, front) = delayed_inputs.releases.pop_front().unwrap();
                delayed.release(Delayed(front));
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

