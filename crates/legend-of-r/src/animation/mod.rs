pub mod adapters;
pub mod animations;
pub mod dynamic_animation;
pub mod float_ord_64;
pub mod timeline;
pub mod timeline_input;

use std::marker::PhantomData;

use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{
        default, App, BuildChildren, BuildWorldChildren, Bundle, Commands, Component, CoreStage,
        Deref, DerefMut, Entity, Mut, Plugin, World,
    },
    reflect::Reflect,
};

use crate::prelude::{
    adapters::offset::Offset, default_entity, evaluate, evaluate_tagged, After, AnimateComponents,
    AxialFunction, Before, Curve, Dilate, Disable, Discrete, Discretize, EntityComponent, Evaluate,
    Flatten, Multiply, Repeat, Sequence, TimelineAnimation, TimelineAnimationContext,
    TimelinePlugin, TryAnimateComponents, TryReplaceComponents, VisitPointer,
};

use self::animations::discrete::Determinisms;

#[derive(Debug, Default, Copy, Clone)]
pub struct AnimationPlugin {
    timeline_ui: bool,
}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<AxialFunction>()
            .register_type::<Evaluate>();

        app.add_plugin(TimelinePlugin {
            ui: self.timeline_ui,
        });

        app.init_resource::<Determinisms>();

        app.add_system(evaluate)
            .add_system_to_stage(CoreStage::First, evaluate_tagged::<First>)
            .add_system_to_stage(CoreStage::PreUpdate, evaluate_tagged::<PreUpdate>)
            .add_system_to_stage(CoreStage::Update, evaluate_tagged::<Update>)
            .add_system_to_stage(CoreStage::PostUpdate, evaluate_tagged::<PostUpdate>)
            .add_system_to_stage(CoreStage::Last, evaluate_tagged::<Last>);
    }
}

impl AnimationPlugin {
    pub fn with_ui() -> Self {
        AnimationPlugin { timeline_ui: true }
    }
}

/// Bundle for composing an AnimationTag<T> and TaggedAnimation
#[derive(Bundle)]
pub struct AnimationTagBundle<T>
where
    T: 'static + Send + Sync,
{
    pub tag: AnimationTag<T>,
    pub tagged: TaggedAnimation,
}

impl<T> Default for AnimationTagBundle<T>
where
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            tag: default(),
            tagged: default(),
        }
    }
}

/// Untyped component for identifying animations that have an AnimationTag
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
#[reflect(Component)]
pub struct TaggedAnimation;

/// Strongly typed component for identifying an animation as executing in a certain stage
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct AnimationTag<T>
where
    T: 'static + Send + Sync,
{
    #[reflect(ignore)]
    pub _phantom: PhantomData<T>,
}

impl<T> Default for AnimationTag<T>
where
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            _phantom: default(),
        }
    }
}

impl<T> Copy for AnimationTag<T> where T: 'static + Send + Sync {}

impl<T> Clone for AnimationTag<T>
where
    T: 'static + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            _phantom: self._phantom.clone(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct First;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct PreUpdate;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct Update;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct PostUpdate;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct Last;

pub trait AnimationTagTrait {
    fn tagged<T>(self) -> Self
    where
        T: 'static + Send + Sync;
}

impl<I, T> AnimationTagTrait for AnimationEntityBuilder<'_, I, T>
where
    I: AnimationBuilderTrait,
{
    fn tagged<U>(mut self) -> Self
    where
        U: 'static + Send + Sync,
    {
        self.inner
            .insert_bundle(self.entity, AnimationTagBundle::<U>::default());
        self
    }
}

/// Extension trait for registering built-in animation wrappers around T
pub trait RegisterAnimationType {
    fn register_animation_type<T>(&mut self) -> &mut Self
    where
        T: 'static + Send + Sync + Default + Clone;
}

impl RegisterAnimationType for App {
    fn register_animation_type<T>(&mut self) -> &mut Self
    where
        T: 'static + Send + Sync + Default + Clone,
    {
        self.register_type::<EntityComponent<T>>()
            .register_type::<Discrete<T>>()
            .register_type::<Multiply<T>>()
            .register_type::<Offset<T>>()
            .register_type::<Dilate<T>>()
            .register_type::<Curve<T>>()
            .register_type::<Repeat<T>>()
            .register_type::<Flatten<T>>()
            .register_type::<Repeat<Option<T>>>()
            .register_type::<Before<T>>()
            .register_type::<Before<Option<T>>>()
            .register_type::<After<T>>()
            .register_type::<After<Option<T>>>()
            .register_type::<Sequence<T>>()
            .register_type::<Discretize<T>>()
            .register_type::<Disable<T>>()
            .register_type::<AnimateComponents<T>>()
            .register_type::<TryAnimateComponents<Option<T>>>()
            .register_type::<TryReplaceComponents<Option<Option<T>>>>()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct AnimationTime {
    pub t: f64,
    pub prev_t: f64,
    pub paused: bool,
    pub prev_paused: bool,
}

/// Trait for producing a value from the world at a given time
pub trait Animate: 'static + Send + Sync + Sized {
    type Type: 'static + Send + Sync;

    fn animate(world: &mut World, animation: Entity, time: AnimationTime) -> Self::Type;

    fn data(world: &World, animation: Entity) -> Option<&Self>
    where
        Self: Component,
    {
        let entity = world.entity(animation);
        entity.get::<Self>()
    }

    fn data_mut(world: &mut World, animation: Entity) -> Option<Mut<Self>>
    where
        Self: Component,
    {
        world.get_mut::<Self>(animation)
    }

    fn handle<T>(world: &World, animation: Entity) -> AnimationHandle<T>
    where
        T: 'static + Send + Sync,
    {
        Self::handles(world, animation)[0]
    }

    fn handles<T>(world: &World, animation: Entity) -> &AnimationHandles<T>
    where
        T: 'static + Send + Sync,
    {
        world
            .entity(animation)
            .get::<AnimationHandles<T>>()
            .unwrap()
    }
}

/// Type alias for referencing an instance of Animate::animate
pub type AnimatePointer<T> = fn(&mut World, Entity, AnimationTime) -> T;

/// Wrapper struct for pointing to an instance of Animate::animate on an entity
pub struct AnimationHandle<T> {
    pub animation: Entity,
    pub animate: AnimatePointer<T>,
    pub visit: VisitPointer,
}

impl<T> Copy for AnimationHandle<T> {}

impl<T> Clone for AnimationHandle<T> {
    fn clone(&self) -> Self {
        Self {
            animation: self.animation.clone(),
            animate: self.animate.clone(),
            visit: self.visit.clone(),
        }
    }
}

impl<T> Default for AnimationHandle<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            animation: default_entity(),
            animate: |_, _, _| default(),
            visit: |_, _, _| (),
        }
    }
}

impl<T> AnimationHandle<T>
where
    T: 'static + Send + Sync,
{
    pub fn animate(&self, world: &mut World, t: AnimationTime) -> T {
        (self.animate)(world, self.animation, t)
    }

    pub fn visit(&self, world: &World, timeline_ui: TimelineAnimationContext) {
        (self.visit)(world, self.animation, timeline_ui)
    }
}

/// Wrapper struct for referencing multiple animation handles
#[derive(Default, Clone, Deref, DerefMut, Component)]
pub struct AnimationHandles<T>
where
    T: 'static + Send + Sync,
{
    pub animations: Vec<AnimationHandle<T>>,
}

impl<T> From<AnimationHandle<T>> for AnimationHandles<T>
where
    T: 'static + Send + Sync,
{
    fn from(animation: AnimationHandle<T>) -> Self {
        AnimationHandles {
            animations: vec![animation],
        }
    }
}

/// Wrapper type for creating animations
#[derive(Deref, DerefMut)]
pub struct AnimationBuilder<'a, T> {
    inner: &'a mut T,
}

impl<'a, I> AnimationBuilder<'a, I>
where
    I: AnimationBuilderTrait,
{
    pub fn spawn<T>(self) -> AnimationEntityBuilder<'a, I, T> {
        let entity = self.inner.spawn();
        AnimationEntityBuilder {
            entity,
            inner: self,
            _phantom: default(),
        }
    }
}

pub trait AnimationBuilderTrait {
    fn spawn(&mut self) -> Entity;

    fn insert<U>(&mut self, entity: Entity, component: U) -> &mut Self
    where
        U: Component;

    fn insert_bundle<U>(&mut self, entity: Entity, bundle: U) -> &mut Self
    where
        U: Bundle;

    fn add_child(&mut self, entity: Entity, child: Entity) -> &mut Self;
}

pub trait BuildAnimation<'a>: Sized {
    fn build_animation(&'a mut self) -> AnimationBuilder<'a, Self>;
}

impl<'w, 's, 'a> BuildAnimation<'a> for Commands<'w, 's> {
    fn build_animation(&'a mut self) -> AnimationBuilder<'a, Self> {
        AnimationBuilder { inner: self }
    }
}

impl<'w, 's, 'a> BuildAnimation<'a> for World {
    fn build_animation(&'a mut self) -> AnimationBuilder<'a, Self> {
        AnimationBuilder { inner: self }
    }
}

/// AnimationBuilder wrapper for creating animation adapters
pub struct AnimationEntityBuilder<'a, I, T> {
    entity: Entity,
    inner: AnimationBuilder<'a, I>,
    _phantom: PhantomData<T>,
}

impl<'a, I, T> AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
{
    pub fn spawn<U>(self) -> AnimationEntityBuilder<'a, I, U> {
        self.inner.spawn()
    }

    #[must_use]
    pub fn id(&self) -> Entity {
        self.entity
    }

    pub fn insert<U>(&mut self, component: U) -> &mut Self
    where
        U: Component,
    {
        self.inner.inner.insert(self.entity, component);
        self
    }

    pub fn insert_bundle<U>(&mut self, bundle: U) -> &mut Self
    where
        U: Bundle,
    {
        self.inner.inner.insert_bundle(self.entity, bundle);
        self
    }

    pub fn handle(&self) -> AnimationHandle<T::Type>
    where
        T: TimelineAnimation,
        T::Type: 'static + Send + Sync + Default,
    {
        AnimationHandle {
            animation: self.id(),
            animate: T::animate,
            visit: T::visit,
        }
    }

    pub fn add_child(&mut self, child: Entity) -> &mut Self {
        self.inner.add_child(self.entity, child);
        self
    }
}

impl<'w, 's> AnimationBuilderTrait for Commands<'w, 's> {
    fn spawn(&mut self) -> Entity {
        self.spawn().id()
    }

    fn insert<U>(&mut self, entity: Entity, component: U) -> &mut Self
    where
        U: Component,
    {
        self.entity(entity).insert(component);
        self
    }

    fn insert_bundle<U>(&mut self, entity: Entity, bundle: U) -> &mut Self
    where
        U: Bundle,
    {
        self.entity(entity).insert_bundle(bundle);
        self
    }

    fn add_child(&mut self, entity: Entity, child: Entity) -> &mut Self {
        self.entity(entity).push_children(&[child]);
        self
    }
}

impl AnimationBuilderTrait for World {
    fn spawn(&mut self) -> Entity {
        self.spawn().id()
    }

    fn insert<U>(&mut self, entity: Entity, component: U) -> &mut Self
    where
        U: Component,
    {
        self.entity_mut(entity).insert(component);
        self
    }

    fn insert_bundle<U>(&mut self, entity: Entity, bundle: U) -> &mut Self
    where
        U: Bundle,
    {
        self.entity_mut(entity).insert_bundle(bundle);
        self
    }

    fn add_child(&mut self, entity: Entity, child: Entity) -> &mut Self {
        self.entity_mut(entity).push_children(&[child]);
        self
    }
}
