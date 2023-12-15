use bevy::prelude::{Component, Deref, DerefMut, Entity, Name, World};

use crate::prelude::{
    Animate, AnimationBuilder, AnimationBuilderTrait, AnimationEntityBuilder, AnimationTime,
    TimelineAnimation, TimelineAnimationContext,
};

/// Animation that reads a component from an entity
#[derive(Component)]
pub struct EntityComponentField<C, F>
where
    C: 'static + Send + Sync,
    F: 'static + Send + Sync,
{
    pub target: Entity,
    pub accessor: fn(&C) -> &F,
}

impl<C, F> Animate for EntityComponentField<C, F>
where
    C: Component,
    F: 'static + Send + Sync + Clone,
{
    type Type = F;

    fn animate(world: &mut World, animation: Entity, _: AnimationTime) -> Self::Type {
        let data = Self::data(world, animation).unwrap();
        let target = data.target;
        let accessor = data.accessor;

        let target = world.entity(target);
        let component = target.get::<C>().unwrap();
        accessor(&component).clone()
    }
}

impl<C, F> TimelineAnimation for EntityComponentField<C, F>
where
    C: Component,
    F: 'static + Send + Sync + Clone,
{
    fn visit(_world: &World, _animation: Entity, _timeline_ui: TimelineAnimationContext) {}
}

pub trait EntityComponentFieldTrait<'w, 's, 'a, I> {
    fn from_entity_component_field<C, F>(
        self,
        target: Entity,
        accessor: fn(&C) -> &F,
    ) -> AnimationEntityBuilder<'a, I, EntityComponentField<C, F>>
    where
        C: 'static + Send + Sync,
        F: 'static + Send + Sync + Clone;

    fn from_value_field<T>(
        self,
        value: T,
    ) -> AnimationEntityBuilder<'a, I, EntityComponentField<Value<T>, T>>
    where
        T: 'static + Send + Sync + Clone;
}

impl<'w, 's, 'a, I> EntityComponentFieldTrait<'w, 's, 'a, I> for AnimationBuilder<'a, I>
where
    I: AnimationBuilderTrait,
{
    fn from_entity_component_field<C, F>(
        self,
        target: Entity,
        accessor: fn(&C) -> &F,
    ) -> AnimationEntityBuilder<'a, I, EntityComponentField<C, F>>
    where
        C: 'static + Send + Sync,
        F: 'static + Send + Sync + Clone,
    {
        let mut commands = self.spawn();

        commands
            .insert(Name::new("Entity Component"))
            .insert(EntityComponentField::<C, F> { target, accessor });

        commands
    }

    fn from_value_field<T>(
        self,
        value: T,
    ) -> AnimationEntityBuilder<'a, I, EntityComponentField<Value<T>, T>>
    where
        T: 'static + Send + Sync + Clone,
    {
        let mut commands = self.spawn();

        let target = commands.id();

        commands
            .insert(Name::new("Value"))
            .insert(Value(value))
            .insert(Name::new("Entity Component"))
            .insert(EntityComponentField::<Value<T>, T> {
                target,
                accessor: |value| &value.0,
            });

        commands
    }
}

#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut, Component,
)]
pub struct Value<T>(T)
where
    T: 'static + Send + Sync;
