use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
    time::Time,
};
use bevy_egui::egui::{
    plot::{PlotUi, VLine},
    Color32,
};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    Timeline, TimelineAnimation, TimelineAnimationContext, TimelineAnimationWidget, TimelineTime,
};

/// Adapter that pulls from some source of time and uses it to drive an animation
#[derive(Clone, Component, Reflect)]
#[reflect(Component)]
pub struct TimeSource<C, T>
where
    C: Default + Reflect,
    T: 'static + Send + Sync,
{
    pub source: C,
    prev_t: f64,
    prev_paused: bool,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<C, T> Default for TimeSource<C, T>
where
    C: Default + Reflect,
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            source: default(),
            prev_t: default(),
            prev_paused: default(),
            _phantom: default(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct WorldTime;

impl<T> Animate for TimeSource<WorldTime, T>
where
    T: 'static + Send + Sync,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, _: AnimationTime) -> Self::Type {
        info_span!("TimeSource").in_scope(|| {
            let time = world.resource::<Time>();
            let t = time.seconds_since_startup();

            let mut data = Self::data_mut(world, animation).unwrap();
            let prev_t = data.prev_t;
            data.prev_t = t;

            Self::handle::<T>(world, animation).animate(
                world,
                AnimationTime {
                    t,
                    prev_t,
                    paused: false,
                    prev_paused: false,
                },
            )
        })
    }
}

impl<T> Animate for TimeSource<TimelineTime, T>
where
    T: 'static + Send + Sync,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, _: AnimationTime) -> Self::Type {
        let data = Self::data(world, animation).unwrap();
        let source = data.source;
        let timeline = world.entity(source.timeline).get::<Timeline>().unwrap();
        let t = timeline.t;
        let paused = timeline.paused;

        let mut data = Self::data_mut(world, animation).unwrap();
        let prev_t = data.prev_t;
        data.prev_t = t;
        let prev_paused = data.prev_paused;
        data.prev_paused = paused;

        Self::handle::<T>(world, animation).animate(world, AnimationTime { t, prev_t, paused, prev_paused })
    }
}

impl<T> TimelineAnimation for TimeSource<WorldTime, T>
where
    T: 'static + Send + Sync,
{
    fn visit(_world: &World, _animation: Entity, _timeline_ui: TimelineAnimationContext) {
        unimplemented!()
    }
}

impl<T> TimelineAnimation for TimeSource<TimelineTime, T>
where
    T: 'static + Send + Sync,
{
    fn visit(world: &World, animation: Entity, context: TimelineAnimationContext) {
        let data = Self::data(world, animation).unwrap();

        if data.source.timeline != context.timeline_ui.timeline {
            return;
        }

        context
            .animation_ui
            .add_timeline_animation(TimelineAnimationWidget {
                name: context.animation_ui.name.clone(),
                y: -(context.animation_ui.index as f32),
                min_t: None,
                max_t: None,
                timeline_length: context.timeline_ui.length,
                disabled: false,
            });

        let timestamp = context.timeline_ui.timestamp;
        context
            .animation_ui
            .add_dynamic(move |plot_ui: &mut PlotUi| {
                plot_ui.vline(VLine::new(timestamp).color(Color32::WHITE))
            });

        Self::handle::<T>(world, animation).visit(world, context);
    }
}

pub trait TimeSourceTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_time_source<C>(
        self,
        source: C,
    ) -> AnimationEntityBuilder<'a, I, TimeSource<C, T::Type>>
    where
        C: Default + Reflect;
}

impl<'w, 's, 'a, I, T> TimeSourceTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_time_source<C>(self, source: C) -> AnimationEntityBuilder<'a, I, TimeSource<C, T::Type>>
    where
        C: Default + Reflect,
    {
        let handle = self.handle();
        let child = handle.animation;

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Time Source"))
            .insert(TimeSource::<C, T::Type> {
                source,
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
