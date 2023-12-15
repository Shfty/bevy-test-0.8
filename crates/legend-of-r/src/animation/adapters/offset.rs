use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimatePointer, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles,
    AnimationTime, AnimationWidget, TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
};

/// Animation adapter that applies a temporal offset to some other animation
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Offset<T>
where
    T: 'static + Send + Sync + Default,
    AnimatePointer<T>: Copy,
{
    offset: f64,
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for Offset<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("Offset").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let offset = data.offset;

            Self::handle::<T>(world, animation).animate(
                world,
                AnimationTime {
                    t: time.t + offset,
                    prev_t: time.prev_t + offset,
                    ..time
                },
            )
        })
    }
}

impl<T> TimelineAnimation for Offset<T>
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
                    widget.min_t = widget.min_t.map(|min_t| min_t - data.offset);

                    widget.max_t = widget.max_t.map(|max_t| max_t - data.offset);

                    if let Some(max_t) = widget.max_t {
                        timeline_ui.length = timeline_ui.length.max(max_t)
                    }
                }
                AnimationWidget::Stop(widget) => widget.x -= data.offset,
                AnimationWidget::Dynamic(_) => (),
            }
        }
    }
}

pub trait OffsetTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_offset(self, offset: f64) -> AnimationEntityBuilder<'a, I, Offset<T::Type>>;
}

impl<'w, 's, 'a, I, T> OffsetTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_offset(self, offset: f64) -> AnimationEntityBuilder<'a, I, Offset<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();
        commands
            .insert(Name::new("Offset"))
            .insert(Offset::<T::Type> {
                offset,
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
