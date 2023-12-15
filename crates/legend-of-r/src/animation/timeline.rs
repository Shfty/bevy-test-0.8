use std::{collections::BTreeMap, fmt::Debug, time::Duration};

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{
        default, info, Commands, Component, CoreStage, Entity, Plugin, Query, Res, With, World,
    },
    reflect::Reflect,
    time::{Stopwatch, Time},
};

use bevy_egui::{
    egui::{
        self,
        plot::{Line, LinkedAxisGroup, Plot, PlotPoint, PlotPoints, PlotUi, Polygon, Text},
        Align, Align2, CollapsingHeader, Color32, Layout,
    },
    EguiContext,
};
use bevy_inspector_egui::egui::Rect;

use crate::prelude::{default_entity, Animate, Evaluate};

use super::timeline_input::TimelineInputPlugin;

#[derive(Debug, Default, Copy, Clone)]
pub struct TimelinePlugin {
    pub ui: bool,
}

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(TimelineInputPlugin);

        app.register_type::<Timeline>()
            .register_type::<TimelineTime>();

        app.add_system_to_stage(CoreStage::First, timeline);

        if self.ui {
            app.init_resource::<TimelineUiState>();
            app.add_system_to_stage(CoreStage::PreUpdate, timeline_ui::<()>);
        }
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Timeline {
    pub tick_rate: f64,
    pub scrub_rate: f64,
    pub t: f64,
    pub paused: bool,
    pub cached_paused: bool,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            tick_rate: 1.0,
            scrub_rate: 0.0,
            t: default(),
            paused: default(),
            cached_paused: default(),
        }
    }
}

impl Timeline {
    pub fn paused() -> Self {
        let mut stopwatch = Stopwatch::new();
        stopwatch.pause();
        Timeline { ..default() }
    }
}

/// Marker type for
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct TimelineTime {
    pub timeline: Entity,
}

impl Default for TimelineTime {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
        }
    }
}

impl TimelineTime {
    pub fn new(timeline: Entity) -> Self {
        TimelineTime { timeline }
    }
}

pub fn timeline(time: Res<Time>, mut query: Query<&mut Timeline>) {
    for mut timeline in query.iter_mut() {
        let elapsed = timeline.t;
        let mut delta = time.delta_seconds_f64() * timeline.scrub_rate;

        if !timeline.paused {
            delta += time.delta_seconds_f64() * timeline.tick_rate;
        }

        let new_t = (elapsed + delta).max(0.0);

        if new_t != elapsed {
            timeline.t = new_t;
        }
    }
}

/// Context passed to TimelineAnimation implementors during UI construction
#[derive(Debug)]
pub struct TimelineAnimationContext<'a> {
    pub timeline_ui: &'a mut TimelineUi,
    pub animation_ui: &'a mut AnimationUi,
}

pub trait TimelineAnimation: Animate {
    fn visit(_world: &World, _animation: Entity, _timeline_ui: TimelineAnimationContext);
}

/// Type alias for referencing an instance of TimelineAnimation::visit
pub type VisitPointer = fn(&World, Entity, TimelineAnimationContext);

/// Struct for constructing UI from a timeline
#[derive(Debug)]
pub struct TimelineUi {
    pub timeline: Entity,
    pub timestamp: f64,
    pub paused: bool,
    pub cached_paused: bool,
    pub length: f64,
    pub animations: Vec<AnimationUi>,
}

impl Default for TimelineUi {
    fn default() -> Self {
        Self {
            timeline: default_entity(),
            timestamp: default(),
            paused: default(),
            cached_paused: default(),
            length: default(),
            animations: default(),
        }
    }
}

pub enum AnimationWidget {
    TimelineAnimation(TimelineAnimationWidget),
    Stop(StopWidget),
    Dynamic(Box<dyn Send + Sync + PlotWidget>),
}

impl Debug for AnimationWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimelineAnimation(arg0) => {
                f.debug_tuple("TimelineAnimation").field(arg0).finish()
            }
            Self::Stop(arg0) => f.debug_tuple("Stop").field(arg0).finish(),
            Self::Dynamic(_) => f.debug_tuple("Dynamic").finish(),
        }
    }
}

impl AnimationWidget {
    pub fn timeline_animation(widget: TimelineAnimationWidget) -> Self {
        AnimationWidget::TimelineAnimation(widget)
    }

    pub fn stop(widget: StopWidget) -> Self {
        AnimationWidget::Stop(widget)
    }

    pub fn dynamic<T>(widget: T) -> Self
    where
        T: 'static + Send + Sync + PlotWidget,
    {
        AnimationWidget::Dynamic(Box::new(widget))
    }

    pub fn plot_ui(&mut self, plot_ui: &mut PlotUi) {
        match self {
            AnimationWidget::TimelineAnimation(widget) => widget.plot_ui(plot_ui),
            AnimationWidget::Stop(widget) => widget.plot_ui(plot_ui),
            AnimationWidget::Dynamic(widget) => widget.plot_ui(plot_ui),
        }
    }
}

impl<T> From<T> for AnimationWidget
where
    T: 'static + Send + Sync + PlotWidget,
{
    fn from(t: T) -> Self {
        AnimationWidget::dynamic(t)
    }
}

/// Struct for constructing UI from an animation
#[derive(Debug)]
pub struct AnimationUi {
    pub index: usize,
    pub name: String,
    pub plot_widgets: Vec<AnimationWidget>,
}

impl Default for AnimationUi {
    fn default() -> Self {
        Self {
            index: default(),
            name: default(),
            plot_widgets: default(),
        }
    }
}

impl AnimationUi {
    pub fn add_timeline_animation(&mut self, t: TimelineAnimationWidget) {
        self.plot_widgets
            .push(AnimationWidget::timeline_animation(t))
    }

    pub fn add_stop(&mut self, t: StopWidget) {
        self.plot_widgets.push(AnimationWidget::stop(t))
    }

    pub fn add_dynamic<T>(&mut self, t: T)
    where
        T: 'static + Send + Sync + PlotWidget,
    {
        self.plot_widgets.push(AnimationWidget::dynamic(t));
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TimelineUiState {
    pub rect: Rect,
}

impl Default for TimelineUiState {
    fn default() -> Self {
        Self {
            rect: Rect {
                min: default(),
                max: default(),
            },
        }
    }
}

/// Timeline user interface system
pub fn timeline_ui<T>(
    world: &World,
    ui_state: Res<TimelineUiState>,
    query_timeline: Query<(Entity, &Timeline)>,
    query_animation: Query<Entity, With<Evaluate>>,
    mut commands: Commands,
) where
    T: 'static + Send + Sync,
{
    let mut timeline_data = if ui_state.rect.height() > 0.0 {
        query_timeline
            .iter()
            .map(|(timeline_entity, timeline)| {
                let timestamp = timeline.t;
                let paused = timeline.paused;
                let cached_paused = timeline.cached_paused;
                let mut timeline_ui = TimelineUi {
                    timeline: timeline_entity,
                    timestamp,
                    paused,
                    cached_paused,
                    length: timestamp,
                    ..default()
                };

                for (i, animation_entity) in query_animation.iter().enumerate() {
                    let mut animation_ui = AnimationUi {
                        index: i,
                        ..default()
                    };

                    Evaluate::visit(
                        world,
                        animation_entity,
                        TimelineAnimationContext {
                            timeline_ui: &mut timeline_ui,
                            animation_ui: &mut animation_ui,
                        },
                    );

                    timeline_ui.animations.push(animation_ui);
                }

                (timeline_entity, timeline_ui)
            })
            .collect::<BTreeMap<_, _>>()
    } else {
        default()
    };

    timeline_data.iter_mut().for_each(|(_, timeline_ui)| {
        for animation_ui in timeline_ui.animations.iter_mut() {
            let timeline_animation = if let Some(AnimationWidget::TimelineAnimation(animation)) =
                animation_ui
                    .plot_widgets
                    .iter_mut()
                    .find(|widget| matches!(widget, AnimationWidget::TimelineAnimation(_)))
            {
                animation
            } else {
                continue;
            };

            timeline_animation.timeline_length = timeline_ui.length;
        }
    });

    commands.add(move |world: &mut World| {
        let mut egui_context = world.resource_mut::<EguiContext>();

        let (responses, rect) = egui::TopBottomPanel::bottom("timeline_panel")
            .show(egui_context.ctx_mut(), |ui| {
                let responses = timeline_data
                    .into_iter()
                    .flat_map(|(timeline_entity, timeline_ui)| {
                        let response =
                            CollapsingHeader::new(egui::RichText::new("Timeline").heading())
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(7.0);

                                        ui.vertical(|ui| {
                                            let linked = LinkedAxisGroup::x();

                                            let bounds = Plot::new(timeline_entity)
                                                .width(ui.available_width() - 7.0)
                                                .height(224.0)
                                                .include_x(-2.5)
                                                .show_axes([true, false])
                                                .show_y(false)
                                                .link_axis(linked.clone())
                                                .show(ui, |plot_ui| {
                                                    for mut animation_ui in timeline_ui.animations {
                                                        for widget in
                                                            animation_ui.plot_widgets.iter_mut()
                                                        {
                                                            widget.plot_ui(plot_ui);
                                                        }
                                                    }

                                                    plot_ui.plot_bounds()
                                                })
                                                .inner;

                                            // Draw drag handle
                                            let mut timestamp = None;
                                            let mut paused = None;
                                            let mut cached_paused = None;

                                            ui.horizontal(|ui| {
                                                ui.style_mut().spacing.slider_width =
                                                    ui.available_width();
                                                let mut ts = timeline_ui.timestamp;
                                                let response = ui.add_sized(
                                                    ui.available_size(),
                                                    egui::Slider::new(
                                                        &mut ts,
                                                        bounds.min()[0]..=bounds.max()[0],
                                                    )
                                                    .show_value(false),
                                                );

                                                ts = ts.max(0.0);

                                                if response.changed() {
                                                    timestamp = Some(ts);
                                                }

                                                if response.drag_started() {
                                                    info!("Paused? {}", timeline_ui.paused);
                                                    paused = Some(true);
                                                    cached_paused = Some(timeline_ui.paused);
                                                } else if response.drag_released() {
                                                    paused = Some(timeline_ui.cached_paused);
                                                }

                                                response
                                            });

                                            let mut scrub_rate = 0.0;

                                            ui.horizontal(|ui| {
                                                if ui.button("\u{23ee}").clicked() {
                                                    timestamp = Some(0.0);
                                                }
                                                // Play / Pause buttons
                                                if timeline_ui.paused {
                                                    if ui.button("\u{25b6}").clicked() {
                                                        paused = Some(false);
                                                    }
                                                } else {
                                                    if ui.button("\u{23f8}").clicked() {
                                                        paused = Some(true);
                                                    }
                                                }

                                                if ui.button("\u{23ed}").clicked() {
                                                    timestamp = Some(timeline_ui.length);
                                                }

                                                // Time readout
                                                let duration =
                                                    Duration::from_secs_f64(timeline_ui.timestamp);
                                                ui.label(format!(
                                                    "{:02}:{:02}:{:03}",
                                                    duration.as_secs() / 60,
                                                    duration.as_secs() % 60,
                                                    duration.as_millis() % 1000
                                                ));

                                                // Play rate scrubber
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        let scrubber = ui.add(
                                                            egui::Slider::new(
                                                                &mut scrub_rate,
                                                                -10.0..=10.0,
                                                            )
                                                            .show_value(false),
                                                        );

                                                        if scrubber.drag_started() {
                                                            paused = Some(true);
                                                        } else if scrubber.drag_released() {
                                                            paused = Some(false);
                                                        }

                                                        ui.label(format!("{:1.2}x", scrub_rate,));
                                                    },
                                                )
                                                .response;
                                            });

                                            (
                                                timeline_entity,
                                                timestamp,
                                                scrub_rate,
                                                paused,
                                                cached_paused,
                                            )
                                        })
                                    })
                                });

                        response.body_returned.map(|returned| returned.inner.inner)
                    })
                    .collect::<Vec<_>>();

                (responses, ui.min_rect())
            })
            .inner;

        *world.resource_mut::<TimelineUiState>() = TimelineUiState { rect };

        for (timeline_entity, timestamp, scrub_rate, paused, cached_paused) in responses {
            let mut entity = world.entity_mut(timeline_entity);
            let mut timeline = entity.get_mut::<Timeline>().unwrap();

            if let Some(timestamp) = timestamp {
                timeline.t = timestamp;
            }

            timeline.scrub_rate = scrub_rate;

            match paused {
                Some(true) => {
                    info!("Pausing");
                    timeline.paused = true
                }
                Some(false) => {
                    info!("Unpausing");
                    timeline.paused = false
                }
                _ => (),
            }

            if let Some(cached_paused) = cached_paused {
                timeline.cached_paused = cached_paused;
            }
        }
    });
}

#[derive(Debug, Default, Clone)]
pub struct TimelineAnimationWidget {
    pub name: String,
    pub y: f32,
    pub min_t: Option<f64>,
    pub max_t: Option<f64>,
    pub timeline_length: f64,
    pub disabled: bool,
}

impl PlotWidget for TimelineAnimationWidget {
    fn plot_ui(&mut self, plot_ui: &mut PlotUi) {
        plot_ui.text(
            Text::new(PlotPoint::new(-0.05, self.y - 0.5), &self.name).anchor(Align2::RIGHT_CENTER),
        );

        plot_ui.polygon(
            Polygon::new(PlotPoints::Owned(vec![
                PlotPoint::new(self.min_t.unwrap_or(0.0), self.y),
                PlotPoint::new(self.max_t.unwrap_or(self.timeline_length), self.y),
                PlotPoint::new(self.max_t.unwrap_or(self.timeline_length), self.y - 1.0),
                PlotPoint::new(self.min_t.unwrap_or(0.0), self.y - 1.0),
            ]))
            .color(if self.disabled {
                Color32::GRAY
            } else {
                Color32::TRANSPARENT
            })
            .fill_alpha(1.0),
        );
    }
}

#[derive(Debug, Default, Clone)]
pub struct StopWidget {
    pub x: f64,
    pub y: f64,
    pub color: Color32,
}

impl PlotWidget for StopWidget {
    fn plot_ui(&mut self, plot_ui: &mut PlotUi) {
        plot_ui.line(
            Line::new(PlotPoints::Owned(vec![
                PlotPoint::new(self.x, self.y),
                PlotPoint::new(self.x, self.y - 1.0),
            ]))
            .color(self.color),
        );
    }
}

pub trait PlotWidget {
    fn plot_ui(&mut self, plot_ui: &mut PlotUi);
}

impl<F> PlotWidget for F
where
    F: FnMut(&mut PlotUi),
{
    fn plot_ui(&mut self, plot_ui: &mut PlotUi) {
        self(plot_ui)
    }
}

pub trait PlotTrait {
    fn add<T>(&mut self, t: &mut T)
    where
        T: PlotWidget;
}

impl PlotTrait for PlotUi {
    fn add<T>(&mut self, t: &mut T)
    where
        T: PlotWidget,
    {
        t.plot_ui(self)
    }
}
