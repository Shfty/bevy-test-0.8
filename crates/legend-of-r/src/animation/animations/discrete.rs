use std::{collections::BTreeSet, fmt::Debug, iter::FromIterator};

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, Component, Entity, Name, World},
    reflect::Reflect,
};
use bevy_egui::egui::Color32;

use crate::prelude::{
    Animate, AnimationBuilder, AnimationBuilderTrait, AnimationEntityBuilder, AnimationTime,
    AnimationWidget, FloatOrd64, ReflectIntegrationBlacklist, StopWidget, TimelineAnimation,
    TimelineAnimationContext,
};

/// Identifier for relating discrete stops that should be pruned as a group if they all occur in the future
pub type DeterminismId = usize;

/// Resource for providing an incrementing DeterminismID
#[derive(Debug, Default, Copy, Clone)]
pub struct Determinisms {
    head: DeterminismId,
}

impl Determinisms {
    pub fn next(&mut self) -> DeterminismId {
        let id = self.head;
        self.head += 1;
        id
    }
}

impl Iterator for Determinisms {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct DiscreteStop<T> {
    pub t: f64,
    pub value: T,
    pub determinism: DeterminismId,
    pub disabled: bool,
}

/// Animation that applies specific values at specific points in time
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Discrete<T>
where
    T: 'static + Send + Sync + Default + Clone,
{
    pub t: f64,
    pub prev_t: f64,
    #[reflect(ignore)]
    pub _stops: Vec<DiscreteStop<T>>,
    pub wants_update: bool,
}

impl<T> Default for Discrete<T>
where
    T: 'static + Send + Sync + Default + Clone,
{
    fn default() -> Self {
        Self {
            t: default(),
            prev_t: default(),
            _stops: default(),
            wants_update: true,
        }
    }
}

impl<T> FromIterator<(f64, T)> for Discrete<T>
where
    T: 'static + Send + Sync + Default + Clone,
{
    fn from_iter<I: IntoIterator<Item = (f64, T)>>(iter: I) -> Self {
        let mut _stops = iter
            .into_iter()
            .map(|(stop, value)| DiscreteStop {
                t: stop,
                value,
                ..default()
            })
            .collect::<Vec<_>>();

        Discrete {
            _stops,
            wants_update: true,
            ..default()
        }
    }
}

impl<T> Discrete<T>
where
    T: 'static + Send + Sync + Default + Clone,
{
    /// Sort and deduplicate the underlying stops
    pub fn update_stops(&mut self) -> &mut Self {
        self._stops
            .sort_unstable_by(|lhs, rhs| FloatOrd64(lhs.t).cmp(&FloatOrd64(rhs.t)));
        self._stops.dedup_by(|lhs, rhs| lhs.t.eq(&rhs.t));
        self.wants_update = false;
        self
    }

    pub fn stop_at(&self, t: f64) -> Option<&DiscreteStop<T>> {
        self.stops().rev().find(|stop| stop.t <= t)
    }

    pub fn stop_at_mut(&mut self, t: f64) -> Option<&mut DiscreteStop<T>> {
        self.stops_mut().rev().find(|stop| stop.t <= t)
    }

    pub fn stops_before(&self, t: f64) -> impl DoubleEndedIterator<Item = &DiscreteStop<T>> {
        self._stops.iter().rev().filter(move |stop| stop.t <= t)
    }

    pub fn stops_after(&self, t: f64) -> impl DoubleEndedIterator<Item = &DiscreteStop<T>> {
        self._stops.iter().filter(move |stop| stop.t >= t)
    }

    pub fn stops_forward(&self) -> impl DoubleEndedIterator<Item = &DiscreteStop<T>> {
        self.stops_after(self.prev_t)
    }

    pub fn stops_backward(&self) -> impl DoubleEndedIterator<Item = &DiscreteStop<T>> {
        self.stops_before(self.prev_t)
    }

    pub fn stops(&self) -> impl DoubleEndedIterator<Item = &DiscreteStop<T>> {
        self._stops.iter()
    }

    pub fn stops_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut DiscreteStop<T>> {
        self.wants_update = true;
        self._stops.iter_mut()
    }

    pub fn insert_stop(&mut self, stop: DiscreteStop<T>) -> &mut Self {
        self._stops.push(stop);
        self.wants_update = true;
        self
    }

    pub fn remove_stops_after(&mut self, t: f64) -> &mut Self {
        self._stops.retain(|stop| stop.t <= t);
        self
    }

    pub fn try_fire_stop_forward(&mut self, t: f64) -> Option<T> {
        if self.t >= t {
            self.stop_at(t)
                .map(|stop| (!stop.disabled).then(|| stop.value.clone()))
                .flatten()
        } else {
            None
        }
    }

    pub fn try_fire_stop_backward(&mut self, t: f64) -> Option<T> {
        if self.t <= t {
            self.stop_at(t)
                .map(|stop| (!stop.disabled).then(|| stop.value.clone()))
                .flatten()
        } else {
            None
        }
    }

    pub fn prune_non_deterministic(&mut self, t: f64) {
        let future_determinisms = self
            .stops()
            .map(|DiscreteStop { determinism, .. }| determinism)
            .map(|group| {
                (
                    *group,
                    self.stops()
                        .filter(|DiscreteStop { determinism: g, .. }| g == group)
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, stops)| stops.iter().all(|stop| stop.t > t))
            .map(|(group, _)| group)
            .collect::<BTreeSet<_>>();

        self._stops
            .retain(|DiscreteStop { determinism, .. }| !future_determinisms.contains(determinism));
    }

    pub fn animate_impl(&mut self, time: AnimationTime) -> Option<T> {
        if self.wants_update {
            self.update_stops();
        }

        self.prev_t = self.t;
        self.t = time.t;

        let mut value = None;

        if (time.prev_paused && !time.paused) || (!time.paused && self.t < self.prev_t) {
            // If we unpaused, or rewound without pausing, prune non-deterministic events
            self.prune_non_deterministic(time.t);
        }

        if self.t > self.prev_t {
            for stop in self.stops_forward().map(|stop| stop.t).collect::<Vec<_>>() {
                if let Some(stop_value) = self.try_fire_stop_forward(stop) {
                    value = Some(stop_value);
                }
            }
        } else if self.t < self.prev_t {
            for stop in self.stops_backward().map(|stop| stop.t).collect::<Vec<_>>() {
                if let Some(stop_value) = self.try_fire_stop_backward(stop) {
                    value = Some(stop_value)
                }
            }

            if value.is_some() {
                if let Some(DiscreteStop { value: v, .. }) = self
                    .stops_backward()
                    .find(|stop| time.t >= stop.t && self.prev_t > stop.t && !stop.disabled)
                {
                    value = Some(v.clone());
                }
            }
        }

        value
    }
}

impl<T> Animate for Discrete<T>
where
    T: 'static + Send + Sync + Default + Clone,
{
    type Type = Option<T>;

    fn animate(world: &mut World, animation: Entity, t: AnimationTime) -> Self::Type {
        let mut data = Self::data_mut(world, animation).unwrap();
        data.animate_impl(t)
    }
}

impl<T> TimelineAnimation for Discrete<T>
where
    T: 'static + Send + Sync + Default + Clone,
{
    fn visit(world: &World, animation: Entity, context: TimelineAnimationContext) {
        let data = Self::data(world, animation).unwrap();
        let y = -(context.animation_ui.index as f64);

        let timeline_animation = if let Some(AnimationWidget::TimelineAnimation(animation)) =
            context
                .animation_ui
                .plot_widgets
                .iter_mut()
                .find(|animation| matches!(animation, AnimationWidget::TimelineAnimation(_)))
        {
            animation
        } else {
            return;
        };

        let min_t = data.stops().next();
        timeline_animation.min_t = min_t.map(|stop| stop.t);

        let max_t = data.stops().last();
        timeline_animation.max_t = max_t.map(|stop| stop.t);

        for stop in data.stops() {
            context.timeline_ui.length = context.timeline_ui.length.max(stop.t);

            context.animation_ui.add_stop(StopWidget {
                x: stop.t,
                y,
                color: stop
                    .disabled
                    .then_some(Color32::GRAY)
                    .unwrap_or(Color32::WHITE),
            });
        }
    }
}

pub trait DiscreteStopsTrait<'w, 's, 'a, I, C, T>
where
    I: IntoIterator<Item = DiscreteStop<T>>,
    T: 'static + Send + Sync + Default + Clone,
{
    fn from_discrete_stops(self, stops: I) -> AnimationEntityBuilder<'a, C, Discrete<T>>;
}

impl<'w, 's, 'a, I, T, C> DiscreteStopsTrait<'w, 's, 'a, I, C, T> for AnimationBuilder<'a, C>
where
    I: IntoIterator<Item = DiscreteStop<T>>,
    T: 'static + Send + Sync + Default + Clone,
    C: AnimationBuilderTrait,
{
    fn from_discrete_stops(self, stops: I) -> AnimationEntityBuilder<'a, C, Discrete<T>> {
        let mut commands = self.spawn();

        commands
            .insert(Name::new("Discrete Stops"))
            .insert(Discrete {
                _stops: stops.into_iter().collect(),
                ..default()
            });

        commands
    }
}
