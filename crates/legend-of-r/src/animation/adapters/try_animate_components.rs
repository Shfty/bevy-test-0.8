use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    ReflectIntegrationBlacklist, TimelineAnimation, TimelineAnimationContext,
};

#[derive(Debug, Copy, Clone, Reflect)]
pub enum TryAnimateComponentsMode {
    /// If the underlying animation returns Some, component is inserted
    /// If the underlying animation returns None, do nothing
    Update,
    /// If the underlying animation returns Some and component doesn't exist, component is inserted
    /// If the underlying animation returns None and component exists, component is removed
    Direct,
}

impl Default for TryAnimateComponentsMode {
    fn default() -> Self {
        TryAnimateComponentsMode::Update
    }
}

/// Animation adapter that takes the result of an option animation and applies it to an entity
#[derive(Default, Component, Reflect)]
#[reflect(IntegrationBlacklist, Component)]
pub struct TryAnimateComponents<T>
where
    T: 'static + Send + Sync + Default,
{
    pub targets: Vec<Entity>,
    #[reflect(ignore)]
    pub predicate: Option<Box<dyn Send + Sync + Fn(&World, Entity, AnimationTime) -> bool>>,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Animate for TryAnimateComponents<Option<T>>
where
    T: Default + Clone + PartialEq + Component,
{
    type Type = ();

    fn animate(world: &mut World, animation: Entity, t: AnimationTime) -> Self::Type {
        info_span!("TryAnimateComponents").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let targets = data.targets.clone();

            if let Some(component) = Self::handle::<Option<T>>(world, animation).animate(world, t) {
                for target in targets {
                    let data = Self::data(world, animation).unwrap();
                    if let Some(predicate) = &data.predicate {
                        if !predicate(world, target, t) {
                            continue;
                        }
                    }
                    let mut entity = world.entity_mut(target);
                    if let Some(entity_component) = entity.get::<T>() {
                        if *entity_component != component {
                            *entity.get_mut::<T>().unwrap() = component.clone();
                        }
                    } else {
                        world.entity_mut(target).insert(component.clone());
                    }
                }
            }
        })
    }
}

impl<T> TimelineAnimation for TryAnimateComponents<Option<T>>
where
    T: Default + Clone + PartialEq + Component,
{
    fn visit(world: &World, animation: Entity, ctx: TimelineAnimationContext) {
        Self::handle::<Option<T>>(world, animation).visit(world, ctx);
    }
}

/// Extension trait for constructing a by-option [`AnimateEntity`]
pub trait TryAnimateComponentsTrait<'w, 's, 'a, C, T, V>: Sized
where
    C: AnimationBuilderTrait,
    T: TimelineAnimation<Type = Option<V>>,
    V: 'static + Send + Sync + Default,
{
    fn try_animating_component(
        self,
        target: Entity,
    ) -> AnimationEntityBuilder<'a, C, TryAnimateComponents<T::Type>> {
        self.try_animating_components([target])
    }

    fn try_animating_components<I>(
        self,
        entities: I,
    ) -> AnimationEntityBuilder<'a, C, TryAnimateComponents<T::Type>>
    where
        I: IntoIterator<Item = Entity>;
}

impl<'w, 's, 'a, C, T, V> TryAnimateComponentsTrait<'w, 's, 'a, C, T, V>
    for AnimationEntityBuilder<'a, C, T>
where
    C: AnimationBuilderTrait,
    T: TimelineAnimation<Type = Option<V>>,
    V: 'static + Send + Sync + Default,
{
    fn try_animating_components<I>(
        self,
        entities: I,
    ) -> AnimationEntityBuilder<'a, C, TryAnimateComponents<T::Type>>
    where
        I: IntoIterator<Item = Entity>,
    {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Try Animate Components"))
            .insert(TryAnimateComponents::<T::Type> {
                targets: entities.into_iter().collect(),
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
