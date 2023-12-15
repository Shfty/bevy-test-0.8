use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{
        default, info_span, Component, Entity, Name, ParallelCommands, Query, With, Without, World,
    },
    reflect::Reflect,
};

use crate::{
    animation::{AnimationTag, TaggedAnimation},
    prelude::{
        Animate, AnimationBuilderTrait, AnimationEntityBuilder, AnimationHandles, AnimationTime,
        TimelineAnimation, TimelineAnimationContext, ReflectIntegrationBlacklist
    },
};

/// Adapter indicating that an animation should be evaluated automatically via system
#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component, IntegrationBlacklist)]
pub struct Evaluate;

impl Animate for Evaluate {
    type Type = ();

    fn animate(world: &mut World, animation: Entity, _: AnimationTime) -> Self::Type {
        info_span!("Animation")
            .in_scope(|| Self::handle(world, animation).animate(world, default()))
    }
}

impl TimelineAnimation for Evaluate {
    fn visit(
        world: &World,
        animation: Entity,
        TimelineAnimationContext {
            timeline_ui,
            animation_ui,
        }: TimelineAnimationContext,
    ) {
        animation_ui.name = world
            .entity(animation)
            .get::<Name>()
            .map(|name| name.to_string())
            .or_else(|| Some(format!("Animation {animation:?}")))
            .unwrap();

        let handle = world
            .entity(animation)
            .get::<AnimationHandles<()>>()
            .unwrap()[0];

        handle.visit(
            world,
            TimelineAnimationContext {
                timeline_ui,
                animation_ui,
            },
        );
    }
}

pub trait EvaluateTrait<'w, 's, 'a, I> {
    fn evaluate(self) -> AnimationEntityBuilder<'a, I, Evaluate>;
}

impl<'w, 's, 'a, I, T> EvaluateTrait<'w, 's, 'a, I> for AnimationEntityBuilder<'a, I, T>
where
    I: AnimationBuilderTrait,
    T: TimelineAnimation<Type = ()>,
{
    fn evaluate(self) -> AnimationEntityBuilder<'a, I, Evaluate> {
        let handle = self.handle();
        let child = handle.animation;

        let mut commands = self.spawn();

        commands
            .insert(Name::new("Animation"))
            .insert(Evaluate)
            .insert(AnimationHandles {
                animations: vec![handle],
            })
            .add_child(child);

        commands
    }
}

/// System for driving animations
pub fn evaluate(
    query_animation: Query<Entity, (With<Evaluate>, Without<TaggedAnimation>)>,
    par_commands: ParallelCommands,
) {
    query_animation.par_for_each(32, |entity| {
        par_commands.command_scope(|mut commands| {
            commands.add(move |world: &mut World| {
                Evaluate::animate(world, entity, default());
            })
        })
    });
}

/// System for driving animations predicated by a tag
pub fn evaluate_tagged<T>(
    query_animation: Query<Entity, (With<Evaluate>, With<AnimationTag<T>>)>,
    par_commands: ParallelCommands,
) where
    T: Component,
{
    query_animation.par_for_each(32, |entity| {
        par_commands.command_scope(|mut commands| {
            commands.add(move |world: &mut World| {
                Evaluate::animate(world, entity, default());
            })
        })
    });
}
