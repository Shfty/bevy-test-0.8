mod analog;
mod camera_focus;
mod camera_pitch;
mod camera_projection;
mod camera_yaw;
mod camera_zoom;
mod digital;
mod gravity;
mod hard_drop;

use bevy::{
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
        system::{CommandQueue, EntityCommands},
    },
    math::{Quat, Vec3},
    prelude::{default, Commands, Component, Entity},
    reflect::Reflect,
};
use bevy_fnord::prelude::{AddGraphVertex, Evaluator};

use crate::prelude::{
    BoardTransform, GridMove, InputReader, MoveType, OrbitCamera, PushPendingGridMoves,
};

use ecs_ex::{entity_default, WithName};

use std::f32::consts::FRAC_PI_2;

#[derive(Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct InputController {
    pub controller: Entity,
    pub target: Option<Entity>,
    pub digital: Entity,
    pub digital_autorepeat: Entity,
    pub analog_x: Entity,
    pub analog_y: Entity,
    pub gravity: Entity,
}

impl MapEntities for InputController {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.controller = entity_map.get(self.controller)?;
        self.target = if let Some(target) = self.target {
            Some(entity_map.get(target)?)
        } else {
            None
        };
        self.digital = entity_map.get(self.digital)?;
        self.digital_autorepeat = entity_map.get(self.digital_autorepeat)?;
        self.analog_x = entity_map.get(self.analog_x)?;
        self.analog_y = entity_map.get(self.analog_y)?;
        self.gravity = entity_map.get(self.gravity)?;
        Ok(())
    }
}

impl Default for InputController {
    fn default() -> Self {
        Self {
            controller: entity_default(),
            target: default(),
            digital: entity_default(),
            digital_autorepeat: entity_default(),
            analog_x: entity_default(),
            analog_y: entity_default(),
            gravity: entity_default(),
        }
    }
}

pub fn auto_repeat_moves(
    move_type: MoveType,
) -> impl Fn(Vec3, bool, Option<Entity>) -> CommandQueue {
    move |value: Vec3, tick: bool, target: Option<Entity>| {
        let mut queue = CommandQueue::default();

        if !tick {
            return queue;
        }

        let target = if let Some(target) = target {
            target
        } else {
            return queue;
        };

        queue.push(PushPendingGridMoves {
            pending_grid_moves: target,
            moves: vec![GridMove {
                delta: BoardTransform {
                    translation: (value.abs().ceil() * value.signum()).as_ivec3(),
                    ..default()
                },
                move_type,
                ..default()
            }],
        });

        queue
    }
}

impl InputController {
    pub fn spawn<'w, 's, 'a>(
        &mut self,
        commands: &'a mut Commands<'w, 's>,
        controller: &InputReader,
        camera_orbit: Entity,
        camera_strafe: Entity,
    ) -> EntityCommands<'w, 's, 'a> {
        let controller_entity = commands.spawn().with_name("Input Controller").id();

        *self = Self {
            digital: commands.spawn().with_name("Digital").id(),
            digital_autorepeat: commands.spawn().with_name("Digital Autorepeat").id(),
            analog_x: commands.spawn().with_name("Analog X").id(),
            analog_y: commands.spawn().with_name("Analog Y").id(),
            gravity: commands.spawn().with_name("Gravity").id(),
            ..*self
        };

        // ECS query vertices
        let vertex_camera_rotation = commands.add_graph_vertex(Evaluator::new(move |world| {
            Quat::from_axis_angle(Vec3::Y, world.get::<OrbitCamera>(camera_orbit).unwrap().yaw)
        }));

        let vertex_camera_rotation_snapped =
            commands.add_graph_vertex(Evaluator::new(move |world| {
                let mut yaw = world.get::<OrbitCamera>(camera_orbit).unwrap().yaw;
                yaw = (yaw / FRAC_PI_2).round() * FRAC_PI_2;
                Quat::from_axis_angle(Vec3::Y, yaw)
            }));

        let vertex_controller_target = commands.add_graph_vertex(Evaluator::new(move |world| {
            let controller = world.entity(controller_entity);
            let controller = controller.get::<InputController>().unwrap();
            controller.target
        }));

        // Digital
        digital::digital(
            commands,
            controller.digital_x,
            controller.digital_y,
            controller.rotate_cw,
            controller.rotate_ccw,
            self.digital_autorepeat,
            vertex_camera_rotation_snapped,
            vertex_controller_target,
        );

        // Analog
        analog::analog_axes(
            commands,
            controller.analog_x,
            controller.analog_y,
            self.analog_x,
            self.analog_y,
            vertex_camera_rotation,
            vertex_controller_target,
        );

        // Gravity
        gravity::gravity(
            commands,
            controller.soft_drop,
            controller_entity,
            self.gravity,
            vertex_controller_target,
        );

        // Hard drop
        hard_drop::hard_drop(
            commands,
            controller_entity,
            controller.hard_drop,
            vertex_controller_target,
        );

        // Camera Yaw
        camera_yaw::camera_yaw(commands, controller.camera_x, camera_orbit);

        // Camera Pitch
        camera_pitch::camera_pitch(commands, controller.camera_y, camera_orbit);

        // Camera Zoom
        camera_zoom::camera_zoom(
            commands,
            controller.camera_zoom_in,
            controller.camera_zoom_out,
            camera_orbit,
        );
        // Camera Focus
        camera_focus::camera_focus(commands, controller.switch_focus, camera_strafe);

        // Camera Projection
        camera_projection::camera_projection(commands, controller.switch_projection, camera_orbit);

        let mut entity_commands = commands.entity(controller_entity);
        entity_commands.insert(*self);
        entity_commands
    }
}
