use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{default, info_span, Component, Entity, Name, World},
    reflect::Reflect,
};

use crate::prelude::{
    Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
    TimelineAnimation, TimelineAnimationContext,
};

/// Animation adapter that takes the result of an animation and applies it to an entity
#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct AnimateComponents<T>
where
    T: 'static + Send + Sync + Default,
{
    pub targets: Vec<Entity>,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

/// Value implementation
impl<T> Animate for AnimateComponents<T>
where
    T: Default + Clone + PartialEq + Component,
{
    type Type = ();

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type {
        info_span!("AnimateComponents").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let targets = data.targets.clone();

            let component = Self::handle::<T>(world, animation).animate(world, time);

            for target in targets {
                let mut entity = world.entity_mut(target);
                let component = component.clone();
                if let Some(entity_component) = entity.get::<T>() {
                    if *entity_component != component {
                        *entity.get_mut::<T>().unwrap() = component;
                    }
                } else {
                    world.entity_mut(target).insert(component);
                }
            }
        })
    }
}

impl<T> TimelineAnimation for AnimateComponents<T>
where
    T: Default + Clone + PartialEq + Component,
{
    fn visit(world: &World, animation: Entity, timeline_ui: TimelineAnimationContext) {
        Self::handle::<T>(world, animation).visit(world, timeline_ui);
    }
}

/// Extension trait for constructing a by-value [`AnimateEntity`]
pub trait AnimateComponentsTrait<'w, 's, 'a, C, T>: Sized
where
    T: Animate,
    T::Type: 'static + Send + Sync + Default,
{
    fn animating_component(
        self,
        entity: Entity,
    ) -> AnimationEntityBuilder<'a, C, AnimateComponents<T::Type>> {
        self.animating_components([entity])
    }

    fn animating_components<I>(
        self,
        entity: I,
    ) -> AnimationEntityBuilder<'a, C, AnimateComponents<T::Type>>
    where
        I: IntoIterator<Item = Entity>;
}

impl<'w, 's, 'a, C, T> AnimateComponentsTrait<'w, 's, 'a, C, T> for AnimationEntityBuilder<'a, C, T>
where
    C: AnimationBuilderTrait,
    T: TimelineAnimation,
    T::Type: 'static + Send + Sync + Default,
{
    fn animating_components<I>(
        self,
        targets: I,
    ) -> AnimationEntityBuilder<'a, C, AnimateComponents<T::Type>>
    where
        I: IntoIterator<Item = Entity>,
    {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Animate Components"))
            .insert(AnimateComponents::<T::Type> {
                targets: targets.into_iter().collect(),
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
