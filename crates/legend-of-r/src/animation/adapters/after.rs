use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimatePointer, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles,
    AnimationTime, AnimationWidget, ReflectIntegrationBlacklist, TimelineAnimation,
    TimelineAnimationContext,
};

/// Animation adapter for playing an animation after a specific time
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct After<T>
where
    T: 'static + Send + Sync + Default,
{
    after: f64,
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for After<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = Option<T>;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("After").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let after = data.after;

            let handle = Self::handle::<T>(world, animation);
            if time.t >= after {
                Some(handle.animate(world, time))
            } else if time.t < after && time.prev_t >= after {
                Some(handle.animate(world, AnimationTime { t: after, ..time }))
            } else {
                None
            }
        })
    }
}

impl<T> TimelineAnimation for After<T>
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
        timeline_animation.min_t = Some(data.after);
    }
}

pub trait AfterTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: Default,
    AnimatePointer<T>: Copy,
{
    fn after(self, after: f64) -> AnimationEntityBuilder<'a, I, After<T::Type>>;
}

impl<'w, 's, 'a, I, T> AfterTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: Default,
{
    fn after(self, after: f64) -> AnimationEntityBuilder<'a, I, After<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();
        commands
            .insert(Name::new("After"))
            .insert(After::<T::Type> { after, ..default() })
            .insert(AnimationHandles {
                animations: vec![handle],
            })
            .add_child(child);

        commands
    }
}
