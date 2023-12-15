use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, Component, Entity, Name, World, info_span},
    reflect::Reflect,
};
use bevy_egui::egui::Color32;

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    AnimationWidget, StopWidget, TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
};

/// Animation adapter that loops another animation at a specified interval
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Repeat<T>
where
    T: 'static + Send + Sync + Default,
{
    interval: f64,
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for Repeat<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Repeat").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let interval = data.interval;

            Self::handle::<T>(world, animation).animate(
                world,
                AnimationTime {
                    t: time.t % interval,
                    prev_t: time.prev_t % interval,
                    ..time
                },
            )
        })
    }
}

impl<T> TimelineAnimation for Repeat<T>
where
    T: 'static + Send + Sync + Default,
{
    fn visit(
        world: &World,
        animation: Entity,
        TimelineAnimationContext {
            timeline_ui,
            animation_ui,
        }: TimelineAnimationContext,
    ) {
        Self::handle::<T>(world, animation).visit(
            world,
            TimelineAnimationContext {
                timeline_ui,
                animation_ui,
            },
        );

        let data = Self::data(world, animation).unwrap();

        if let Some(AnimationWidget::TimelineAnimation(animation)) = animation_ui
            .plot_widgets
            .iter_mut()
            .find(|animation| matches!(animation, AnimationWidget::TimelineAnimation(_)))
        {
            animation.max_t = None;
        } else {
            return;
        };

        let y = -(animation_ui.index as f64);
        let timestamp = timeline_ui.timestamp % data.interval;
        animation_ui.add_stop(StopWidget {
            x: timestamp,
            y,
            color: Color32::WHITE,
        });
    }
}

pub trait RepeatTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_repeat(self, interval: f64) -> AnimationEntityBuilder<'a, I, Repeat<T::Type>>;
}

impl<'w, 's, 'a, I, T> RepeatTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_repeat(self, interval: f64) -> AnimationEntityBuilder<'a, I, Repeat<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Loop"))
            .insert(Repeat::<T::Type> {
                interval,
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
