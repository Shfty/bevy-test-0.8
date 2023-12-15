use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
};
use bevy_egui::egui::Color32;

use crate::prelude::{
    Animate, AnimatePointer, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles,
    AnimationTime, AnimationWidget, Discrete, StopWidget, TimelineAnimation,
    TimelineAnimationContext,
};

/// Adapter that discretizes a continuous animation by evaluating it only when a stop is passed
#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Discretize<T>
where
    T: 'static + Send + Sync + Default,
{
    pub discrete: Discrete<f64>,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Animate for Discretize<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = Option<T>;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Discretize").in_scope(|| {
            let mut data = Self::data_mut(world, animation).unwrap();

            if let Some(t) = data.discrete.animate_impl(time) {
                Some(
                    Self::handle::<T>(world, animation).animate(world, AnimationTime { t, ..time }),
                )
            } else {
                None
            }
        })
    }
}

impl<T> TimelineAnimation for Discretize<T>
where
    T: 'static + Send + Sync + Default,
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

        let min_t = data.discrete.stops().next().map(|stop| stop.t);
        timeline_animation.min_t = min_t;

        let max_t = data.discrete.stops().last().map(|stop| stop.t);
        timeline_animation.max_t = max_t;

        for stop in data.discrete.stops() {
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

pub trait DiscretizeTrait<'w, 's, 'a, C, T>
where
    T: Animate,
    T::Type: Default,
    AnimatePointer<T>: Copy,
{
    fn discretized_to<I>(self, stops: I) -> AnimationEntityBuilder<'a, C, Discretize<T::Type>>
    where
        I: IntoIterator<Item = f64>;
}

impl<'w, 's, 'a, C, T> DiscretizeTrait<'w, 's, 'a, C, T> for AnimationEntityBuilder<'a, C, T>
where
    C: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: Default,
{
    fn discretized_to<I>(self, stops: I) -> AnimationEntityBuilder<'a, C, Discretize<T::Type>>
    where
        I: IntoIterator<Item = f64>,
    {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Discretize"))
            .insert(Discretize::<T::Type> {
                discrete: stops.into_iter().map(|stop| (stop, stop)).collect(),
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
