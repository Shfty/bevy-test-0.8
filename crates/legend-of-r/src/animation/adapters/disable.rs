use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, Component, Entity, Name, World, info_span},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
};

/// Adapter that can conditionally disable an animation
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Disable<T>
where
    T: 'static + Send + Sync + Default,
{
    pub disabled: bool,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Animate for Disable<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = Option<T>;

    fn animate(world: &mut World, animation: Entity, t: AnimationTime) -> Self::Type {
        info_span!("Disable").in_scope(|| {
            let data = Self::data(world, animation).unwrap();

            if data.disabled {
                None
            } else {
                Some(Self::handle::<T>(world, animation).animate(world, t))
            }
        })
    }
}

impl<T> TimelineAnimation for Disable<T>
where
    T: 'static + Send + Sync + Default,
{
    fn visit(world: &World, animation: Entity, timeline_ui: TimelineAnimationContext) {
        Self::handle::<T>(world, animation).visit(world, timeline_ui)
    }
}

pub trait DisableTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_disabled(self, disabled: bool) -> AnimationEntityBuilder<'a, I, Disable<T::Type>>;
}

impl<'w, 's, 'a, I, T> DisableTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_disabled(self, disabled: bool) -> AnimationEntityBuilder<'a, I, Disable<T::Type>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Disable"))
            .insert(Disable::<T::Type> {
                disabled,
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
