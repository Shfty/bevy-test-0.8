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

/// Animation adapter that applies a temporal offset to some other animation
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Dilate<T>
where
    T: 'static + Send + Sync + Default,
{
    factor: f64,
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for Dilate<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Dilate").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let factor = data.factor;

            Self::handle::<T>(world, animation).animate(
                world,
                AnimationTime {
                    t: time.t * factor,
                    prev_t: time.prev_t * factor,
                    ..time
                },
            )
        })
    }
}

impl<T> TimelineAnimation for Dilate<T>
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

        for animation in animation_ui.plot_widgets.iter_mut() {
            match animation {
                AnimationWidget::TimelineAnimation(widget) => {
                    widget.min_t = widget.min_t.map(|min_t| min_t / data.factor);

                    widget.max_t = widget.max_t.map(|max_t| max_t / data.factor);

                    if let Some(max_t) = widget.max_t {
                        timeline_ui.length = timeline_ui.length.max(max_t)
                    }
                }
                AnimationWidget::Stop(widget) => widget.x /= data.factor,
                AnimationWidget::Dynamic(_) => (),
            }
        }
    }
}

pub trait DilateTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_dilation(self, factor: f64) -> AnimationEntityBuilder<'a, I, Dilate<T::Type>>;
}

impl<'w, 's, 'a, I, T> DilateTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_dilation(self, factor: f64) -> AnimationEntityBuilder<'a, I, Dilate<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Dilate"))
            .insert(Dilate::<T::Type> {
                factor,
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
