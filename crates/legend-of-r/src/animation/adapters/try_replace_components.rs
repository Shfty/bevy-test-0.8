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

/// Animation adapter that takes the result of a doubly-nested option animation and applies it to an entity,
/// removing the component if the inner option is None
#[derive(Default, Component, Reflect)]
#[reflect(IntegrationBlacklist, Component)]
pub struct TryReplaceComponents<T>
where
    T: 'static + Send + Sync + Default,
{
    pub targets: Vec<Entity>,
    #[reflect(ignore)]
    pub predicate: Option<Box<dyn Send + Sync + Fn(&World, Entity, AnimationTime) -> bool>>,
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

/// Option<Value> implementation
impl<T> Animate for TryReplaceComponents<Option<Option<T>>>
where
    T: Default + Clone + Component,
{
    type Type = ();

    fn animate(world: &mut World, animation: Entity, t: AnimationTime) -> Self::Type {
        info_span!("TryReplaceComponents").in_scope(|| {
            let data = Self::data(world, animation).unwrap();
            let targets = data.targets.clone();

            match Self::handle::<Option<Option<T>>>(world, animation).animate(world, t) {
                Some(component) => match component {
                    Some(component) => {
                        for target in targets {
                            let data = Self::data(world, animation).unwrap();
                            if let Some(predicate) = &data.predicate {
                                if !predicate(world, target, t) {
                                    continue;
                                }
                            }
                            world.entity_mut(target).insert(component.clone());
                        }
                    }
                    None => {
                        for target in targets {
                            let data = Self::data(world, animation).unwrap();
                            if let Some(predicate) = &data.predicate {
                                if !predicate(world, target, t) {
                                    continue;
                                }
                            }
                            world.entity_mut(target).remove::<T>();
                        }
                    }
                },
                _ => (),
            }
        })
    }
}

impl<T> TimelineAnimation for TryReplaceComponents<Option<Option<T>>>
where
    T: Default + Clone + Component,
{
    fn visit(world: &World, animation: Entity, ctx: TimelineAnimationContext) {
        Self::handle::<Option<Option<T>>>(world, animation).visit(world, ctx);
    }
}

/// Extension trait for constructing a by-option [`AnimateEntity`]
pub trait TryReplaceComponentsTrait<'w, 's, 'a, C, T, V>: Sized
where
    C: AnimationBuilderTrait,
    T: TimelineAnimation<Type = Option<Option<V>>>,
    V: 'static + Send + Sync + Default,
{
    fn try_replacing_component(
        self,
        target: Entity,
    ) -> AnimationEntityBuilder<'a, C, TryReplaceComponents<T::Type>> {
        self.try_replacing_components([target])
    }

    fn try_replacing_components<I>(
        self,
        entities: I,
    ) -> AnimationEntityBuilder<'a, C, TryReplaceComponents<T::Type>>
    where
        I: IntoIterator<Item = Entity>;
}

impl<'w, 's, 'a, C, T, V> TryReplaceComponentsTrait<'w, 's, 'a, C, T, V>
    for AnimationEntityBuilder<'a, C, T>
where
    C: AnimationBuilderTrait,
    T: TimelineAnimation<Type = Option<Option<V>>>,
    V: 'static + Send + Sync + Default,
{
    fn try_replacing_components<I>(
        self,
        entities: I,
    ) -> AnimationEntityBuilder<'a, C, TryReplaceComponents<T::Type>>
    where
        I: IntoIterator<Item = Entity>,
    {
        let child = self.id();
        let handle = self.handle();

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Try Replace Components"))
            .insert(TryReplaceComponents::<T::Type> {
                targets: entities.into_iter().collect(),
                ..default()
            })
            .insert(AnimationHandles::from(handle))
            .add_child(child);

        commands
    }
}
