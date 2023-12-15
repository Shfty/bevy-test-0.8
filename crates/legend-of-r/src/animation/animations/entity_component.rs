use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    default_entity, Animate, AnimationBuilder, AnimationBuilderTrait, AnimationEntityBuilder,
    AnimationTime, TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
};

/// Animation that reads a component from an entity
#[derive(Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct EntityComponent<T>
where
    T: 'static + Send + Sync,
{
    pub target: Entity,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for EntityComponent<T>
where
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            target: default_entity(),
            _phantom: default(),
        }
    }
}

impl<T> Animate for EntityComponent<T>
where
    T: Clone + Component,
{
    type Type = T;

    fn animate(world: &mut World, animation: Entity, _: AnimationTime) -> Self::Type {
        let data = Self::data(world, animation).unwrap();

        let target = world.entity(data.target);
        target.get::<Self::Type>().unwrap().clone()
    }
}

impl<T> TimelineAnimation for EntityComponent<T>
where
    T: Default + Clone + Component,
{
    fn visit(_world: &World, _animation: Entity, _timeline_ui: TimelineAnimationContext) {}
}

pub trait EntityComponentTrait<'w, 's, 'a, I> {
    fn from_entity_component<T>(
        self,
        target: Entity,
    ) -> AnimationEntityBuilder<'a, I, EntityComponent<T>>
    where
        T: Component;

    fn from_component<T>(self, value: T) -> AnimationEntityBuilder<'a, I, EntityComponent<T>>
    where
        T: Component;
}

impl<'w, 's, 'a, I> EntityComponentTrait<'w, 's, 'a, I> for AnimationBuilder<'a, I>
where
    I: AnimationBuilderTrait,
{
    fn from_entity_component<T>(
        self,
        target: Entity,
    ) -> AnimationEntityBuilder<'a, I, EntityComponent<T>>
    where
        T: Component,
    {
        let mut commands = self.spawn();

        commands
            .insert(Name::new("Entity Component"))
            .insert(EntityComponent::<T> {
                target,
                ..default()
            });

        commands
    }

    fn from_component<T>(self, value: T) -> AnimationEntityBuilder<'a, I, EntityComponent<T>>
    where
        T: Component,
    {
        let mut commands = self.spawn();

        let target = commands.id();

        commands
            .insert(Name::new("Value"))
            .insert(value)
            .insert(Name::new("Entity Component"))
            .insert(EntityComponent::<T> {
                target,
                ..default()
            });

        commands
    }
}
