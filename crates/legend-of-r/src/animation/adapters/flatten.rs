use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
};
/// Animation adapter for flattening nested option types
#[derive(Default, Copy, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Flatten<T>
where
    T: 'static + Send + Sync + Default,
{
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> Animate for Flatten<T>
where
    T: 'static + Send + Sync + Default,
{
    type Type = Option<T>;

    fn animate(world: &mut World, animation: Entity, t: AnimationTime) -> Self::Type {
        info_span!("Flatten").in_scope(|| {
            if let Some(Some(result)) =
                Self::handle::<Option<Option<T>>>(world, animation).animate(world, t)
            {
                Some(result)
            } else {
                None
            }
        })
    }
}

impl<T> TimelineAnimation for Flatten<T>
where
    T: 'static + Send + Sync + Default,
{
    fn visit(world: &World, animation: Entity, context: TimelineAnimationContext) {
        Self::handle::<Option<Option<T>>>(world, animation).visit(world, context);
    }
}

pub trait FlattenTrait<'w, 's, 'a, I, T, V>
where
    T: Animate<Type = Option<Option<V>>>,
    V: 'static + Send + Sync + Default,
{
    fn flatten(self) -> AnimationEntityBuilder<'a, I, Flatten<V>>;
}

impl<'w, 's, 'a, I, T, V> FlattenTrait<'w, 's, 'a, I, T, V> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation<Type = Option<Option<V>>>,
    V: 'static + Send + Sync + Default,
{
    fn flatten(self) -> AnimationEntityBuilder<'a, I, Flatten<V>> {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Flatten"))
            .insert(Flatten::<V>::default())
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
