use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    AnimationWidget, ReflectIntegrationBlacklist, TimelineAnimation, TimelineAnimationContext,
};

/// Animation adapter for playing an animation before a specific time
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Before<T>
where
    T: 'static + Send + Sync + Default,
{
    before: f64,
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for Before<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = Option<T>;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Before").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let before = data.before;

            let handle = Self::handle::<T>(world, animation);
            if time.t < before {
                Some(handle.animate(world, time))
            } else if time.t >= before && time.prev_t < before {
                Some(handle.animate(world, AnimationTime { t: before, ..time }))
            } else {
                None
            }
        })
    }
}

impl<T> TimelineAnimation for Before<T>
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

        let timeline_animation =
            if let Some(AnimationWidget::TimelineAnimation(timeline_animation)) = animation_ui
                .plot_widgets
                .iter_mut()
                .find(|widget| matches!(widget, AnimationWidget::TimelineAnimation(_)))
            {
                timeline_animation
            } else {
                return;
            };

        let data = Self::data(world, animation).unwrap();
        timeline_animation.max_t = Some(data.before);

        timeline_ui.length = timeline_ui.length.max(data.before);
    }
}

pub trait BeforeTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: Default,
{
    fn before(self, before: f64) -> AnimationEntityBuilder<'a, I, Before<T::Type>>;
}

impl<'w, 's, 'a, I, T> BeforeTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: Default,
{
    fn before(self, before: f64) -> AnimationEntityBuilder<'a, I, Before<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();
        commands
            .insert(Name::new("Before"))
            .insert(Before::<T::Type> {
                before,
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
