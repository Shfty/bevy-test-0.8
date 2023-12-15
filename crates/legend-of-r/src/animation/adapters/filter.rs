use std::marker::PhantomData;

use bevy::prelude::{default, info_span, Component, Entity, Name, World};

use crate::animation::{
    timeline::TimelineAnimation, Animate, AnimationBuilderTrait, AnimationEntityBuilder,
    AnimationHandles, AnimationTime,
};

#[derive(Component)]
pub struct Filter<P, T>
where
    P: 'static + Send + Sync + Fn(&World, Entity, AnimationTime) -> bool,
    T: 'static + Send + Sync,
{
    pub predicate: P,
    pub _phantom: PhantomData<T>,
}

impl<P, T> Animate for Filter<P, T>
where
    P: 'static + Send + Sync + Clone + Fn(&World, Entity, AnimationTime) -> bool,
    T: 'static + Send + Sync,
{
    type Type = Option<T>;

    fn animate(
        world: &mut World,
        animation: Entity,
        time: crate::animation::AnimationTime,
    ) -> Self::Type {
        info_span!("Filter").in_scope(|| {
            let data = Self::data(world, animation).unwrap().clone();

            if (data.predicate)(world, animation, time) {
                Some(Self::handle::<T>(world, animation).animate(world, time))
            } else {
                None
            }
        })
    }
}

impl<P, T> TimelineAnimation for Filter<P, T>
where
    P: 'static + Send + Sync + Clone + Fn(&World, Entity, AnimationTime) -> bool,
    T: 'static + Send + Sync,
{
    fn visit(
        world: &World,
        animation: Entity,
        timeline_ui: crate::animation::timeline::TimelineAnimationContext,
    ) {
        Self::handle::<T>(world, animation).visit(world, timeline_ui)
    }
}

pub trait FilterTrait<'w, 's, 'a, I, T>
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_filter<P>(self, predicate: P) -> AnimationEntityBuilder<'a, I, Filter<P, T::Type>>
    where
        P: 'static + Send + Sync + Clone + Fn(&World, Entity, AnimationTime) -> bool;
}

impl<'w, 's, 'a, I, T> FilterTrait<'w, 's, 'a, I, T> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn with_filter<P>(self, predicate: P) -> AnimationEntityBuilder<'a, I, Filter<P, T::Type>>
    where
        P: 'static + Send + Sync + Clone + Fn(&World, Entity, AnimationTime) -> bool,
    {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();
        commands
            .insert(Name::new("Filter"))
            .insert(Filter::<P, T::Type> {
                predicate,
                _phantom: default(),
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
