use std::marker::PhantomData;

use bevy::prelude::{default, info_span, Component, Entity, Name, World};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    TimelineAnimation, TimelineAnimationContext,
};

/// Animation adapter that takes the result of an animation and applies it to the
/// field of a component based on a provided accessor and mutator
#[derive(Component)]
pub struct TryAnimateComponentFields<C, T, A, M>
where
    C: 'static + Send + Sync,
    T: 'static + Send + Sync,
    A: 'static + Send + Sync + Clone + Fn(&C) -> &T,
    M: 'static + Send + Sync + Clone + Fn(&mut C) -> &mut T,
{
    pub targets: Vec<Entity>,
    pub accessor: A,
    pub mutator: M,
    pub _phantom: PhantomData<(C, T)>,
}

impl<C, T, A, M> Animate for TryAnimateComponentFields<C, T, A, M>
where
    C: 'static + Send + Sync + Component,
    T: 'static + Send + Sync + Clone + PartialEq,
    A: 'static + Send + Sync + Clone + Fn(&C) -> &T,
    M: 'static + Send + Sync + Clone + Fn(&mut C) -> &mut T,
{
    type Type = ();

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("TryAnimateComponentFields").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let targets = data.targets.clone();
            let accessor = data.accessor.clone();
            let mutator = data.mutator.clone();

            let value = Self::handle::<Option<T>>(world, animation).animate(world, time);

            for target in targets {
                if let Some(value) = value.clone() {
                    let mut entity = world.entity_mut(target);
                    if let Some(entity_component) = entity.get::<C>() {
                        let field = accessor(&entity_component);
                        if *field != value {
                            *mutator(&mut entity.get_mut::<C>().unwrap()) = value;
                        }
                    }
                }
            }
        })
    }
}

impl<C, T, A, M> TimelineAnimation for TryAnimateComponentFields<C, T, A, M>
where
    C: 'static + Send + Sync + Component,
    T: 'static + Send + Sync + Clone + PartialEq,
    A: 'static + Send + Sync + Clone + Fn(&C) -> &T,
    M: 'static + Send + Sync + Clone + Fn(&mut C) -> &mut T,
{
    fn visit(world: &World, animation: Entity, timeline_ui: TimelineAnimationContext) {
        Self::handle::<Option<T>>(world, animation).visit(world, timeline_ui);
    }
}

/// Extension trait for constructing an TryAnimateComponentField
pub trait TryAnimateComponentFieldsTrait<'w, 's, 'a, I, T, V>: Sized
where
    T: TimelineAnimation<Type = Option<V>>,
    T::Type: 'static + Send + Sync + Default,
    V: 'static + Send + Sync,
{
    fn try_animating_component_field<C, A, M>(
        self,
        entity: Entity,
        accessor: A,
        mutator: M,
    ) -> AnimationEntityBuilder<'a, I, TryAnimateComponentFields<C, V, A, M>>
    where
        C: Component,
        A: 'static + Send + Sync + Clone + Fn(&C) -> &V,
        M: 'static + Send + Sync + Clone + Fn(&mut C) -> &mut V,
    {
        self.try_animating_component_fields_impl([entity], accessor, mutator)
    }

    fn try_animating_component_fields<C, Iter, A, M>(
        self,
        entity: Iter,
        accessor: A,
        mutator: M,
    ) -> AnimationEntityBuilder<'a, I, TryAnimateComponentFields<C, V, A, M>>
    where
        C: Component,
        Iter: IntoIterator<Item = Entity>,
        A: 'static + Send + Sync + Clone + Fn(&C) -> &V,
        M: 'static + Send + Sync + Clone + Fn(&mut C) -> &mut V,
    {
        self.try_animating_component_fields_impl(entity, accessor, mutator)
    }

    fn try_animating_component_fields_impl<C, Iter, A, M>(
        self,
        entity: Iter,
        accessor: A,
        mutator: M,
    ) -> AnimationEntityBuilder<'a, I, TryAnimateComponentFields<C, V, A, M>>
    where
        C: Component,
        Iter: IntoIterator<Item = Entity>,
        A: 'static + Send + Sync + Clone + Fn(&C) -> &V,
        M: 'static + Send + Sync + Clone + Fn(&mut C) -> &mut V;
}

impl<'w, 's, 'a, I, T, V> TryAnimateComponentFieldsTrait<'w, 's, 'a, I, T, V>
    for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation<Type = Option<V>>,
    T::Type: 'static + Send + Sync + Default,
    V: 'static + Send + Sync,
{
    fn try_animating_component_fields_impl<C, Iter, A, M>(
        self,
        targets: Iter,
        accessor: A,
        mutator: M,
    ) -> AnimationEntityBuilder<'a, I, TryAnimateComponentFields<C, V, A, M>>
    where
        C: Component,
        Iter: IntoIterator<Item = Entity>,
        A: 'static + Send + Sync + Clone + Fn(&C) -> &V,
        M: 'static + Send + Sync + Clone + Fn(&mut C) -> &mut V,
    {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Animate Component Fields"))
            .insert(TryAnimateComponentFields::<C, V, A, M> {
                targets: targets.into_iter().collect(),
                accessor,
                mutator,
                _phantom: default(),
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
