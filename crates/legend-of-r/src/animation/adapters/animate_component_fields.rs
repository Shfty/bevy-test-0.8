use bevy::prelude::{Component, Entity, Name, World, info_span};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    TimelineAnimation, TimelineAnimationContext,
};

/// Animation adapter that takes the result of an animation and applies it to the
/// field of a component based on a provided accessor and mutator
#[derive(Component)]
pub struct AnimateComponentFields<C, F>
where
    C: 'static + Send + Sync,
    F: 'static + Send + Sync,
{
    pub targets: Vec<Entity>,
    pub accessor: fn(&C) -> &F,
    pub mutator: fn(&mut C) -> &mut F,
}

impl<C, F> Animate for AnimateComponentFields<C, F>
where
    C: Component,
    F: 'static + Send + Sync + Clone + PartialEq,
{
    type Type = ();

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("AnimateComponentFields").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let targets = data.targets.clone();
            let accessor = data.accessor;
            let mutator = data.mutator;

            let value = Self::handle::<F>(world, animation).animate(world, time);

            for target in targets {
                let value = value.clone();

                let mut entity = world.entity_mut(target);
                if let Some(entity_component) = entity.get::<C>() {
                    let field = accessor(&entity_component);
                    if *field != value {
                        *mutator(&mut entity.get_mut::<C>().unwrap()) = value;
                    }
                }
            }
        })
    }
}

impl<C, F> TimelineAnimation for AnimateComponentFields<C, F>
where
    C: Component,
    F: 'static + Send + Sync + Clone + PartialEq,
{
    fn visit(world: &World, animation: Entity, timeline_ui: TimelineAnimationContext) {
        Self::handle::<F>(world, animation).visit(world, timeline_ui);
    }
}

/// Extension trait for constructing an AnimateComponentField
pub trait AnimateComponentFieldsTrait<'w, 's, 'a, I, T>: Sized
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn animating_component_field<C>(
        self,
        entity: Entity,
        accessor: fn(&C) -> &T::Type,
        mutator: fn(&mut C) -> &mut T::Type,
    ) -> AnimationEntityBuilder<'a, I, AnimateComponentFields<C, T::Type>>
    where
        C: Component,
    {
        self.animating_component_fields_impl([entity], accessor, mutator)
    }

    fn animating_component_fields<C, Iter>(
        self,
        entities: Iter,
        accessor: fn(&C) -> &T::Type,
        mutator: fn(&mut C) -> &mut T::Type,
    ) -> AnimationEntityBuilder<'a, I, AnimateComponentFields<C, T::Type>>
    where
        C: Component,
        Iter: IntoIterator<Item = Entity>,
    {
        self.animating_component_fields_impl(entities, accessor, mutator)
    }

    fn animating_component_fields_impl<C, Iter>(
        self,
        entity: Iter,
        accessor: fn(&C) -> &T::Type,
        mutator: fn(&mut C) -> &mut T::Type,
    ) -> AnimationEntityBuilder<'a, I, AnimateComponentFields<C, T::Type>>
    where
        C: Component,
        Iter: IntoIterator<Item = Entity>;
}

impl<'w, 's, 'a, I, T> AnimateComponentFieldsTrait<'w, 's, 'a, I, T>
    for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn animating_component_fields_impl<C, Iter>(
        self,
        targets: Iter,
        accessor: fn(&C) -> &T::Type,
        mutator: fn(&mut C) -> &mut T::Type,
    ) -> AnimationEntityBuilder<'a, I, AnimateComponentFields<C, T::Type>>
    where
        C: Component,
        Iter: IntoIterator<Item = Entity>,
    {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Animate Component Fields"))
            .insert(AnimateComponentFields::<C, T::Type> {
                targets: targets.into_iter().collect(),
                accessor,
                mutator,
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
