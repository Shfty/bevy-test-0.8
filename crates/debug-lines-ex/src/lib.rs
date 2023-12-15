///! Extension plugin for bevy_prototype_debug_lines
///!
///! Allows lines to be defined using components
use anyhow::Result;
use bevy::{
    math::Quat,
    prelude::{
        Color, Component, Entity, GlobalTransform, Plugin, Query, ResMut, Transform, Vec3,
    },
};
use bevy_prototype_debug_lines::DebugLines;

use result_system::ResultSystem;
use transform_ex::quat_ex::QuatEx;

pub struct DebugLinesExPlugin;

impl Plugin for DebugLinesExPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(debug_lines_ex.result_system());
    }
}

macro_rules! impl_inline_builder {
    ($self_ty:ty, $field:ident, $field_ty:ty, $fn_name:ident) => {
        impl $self_ty {
            pub fn $fn_name(mut self, $field: $field_ty) -> Self {
                self.$field = $field;
                self
            }
        }
    };
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Point {
    pub parent: Option<Entity>,
    pub translation: Vec3,
    pub color: Color,
}

impl_inline_builder!(Point, parent, Option<Entity>, with_parent);
impl_inline_builder!(Point, translation, Vec3, with_translation);
impl_inline_builder!(Point, color, Color, with_color);

impl Point {
    pub fn point(&self, query_transform: &Query<&GlobalTransform>) -> Result<Vec3> {
        Ok(if let Some(parent) = self.parent {
            let parent_trx = query_transform.get(parent)?;
            parent_trx.mul_vec3(self.translation)
        } else {
            self.translation
        })
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct Line {
    pub points: Vec<Point>,
    pub duration: f32,
}

impl_inline_builder!(Line, points, Vec<Point>, with_points);
impl_inline_builder!(Line, duration, f32, with_duration);

impl Line {
    pub fn with_point(mut self, point: Point) -> Self {
        self.points.push(point);
        self
    }
}

pub fn debug_lines_ex(
    mut lines: ResMut<DebugLines>,
    query_line: Query<&Line>,
    query_transform: Query<&GlobalTransform>,
) -> Result<()> {
    for line in query_line.iter() {
        for window in line.points.windows(2) {
            let from = window[0];
            let to = window[1];

            lines.line_gradient(
                from.point(&query_transform)?,
                to.point(&query_transform)?,
                line.duration,
                from.color,
                to.color,
            );
        }
    }

    Ok(())
}

// Utility functions

pub fn circle_verts(count: usize, radius: f32) -> impl Iterator<Item = Vec3> {
    (0..=count).map(move |i| {
        let f = (i as f32 / count as f32) * std::f32::consts::TAU;
        Vec3::new(f.sin(), f.cos(), 0.0) * radius
    })
}

pub fn debug_circle(
    lines: &mut DebugLines,
    transform: Transform,
    radius: f32,
    res: usize,
    color: Color,
) {
    let verts = circle_verts(res, radius).collect::<Vec<_>>();
    for window in verts.windows(2) {
        let v0 = transform * window[0];
        let v1 = transform * window[1];
        lines.line_colored(v0, v1, 0.0, color);
    }
}

pub fn debug_cylinder(
    lines: &mut DebugLines,
    from: Vec3,
    to: Vec3,
    radius: f32,
    res: usize,
    color_from: Color,
    color_to: Color,
) {
    let delta = to - from;

    let mut circle_trx = Transform::default();
    circle_trx.translation = from;
    if from != to {
        let delta_norm = delta.normalize();
        let up = if delta_norm.dot(Vec3::Y).abs() < 0.9 {
            Vec3::Y
        } else if delta_norm.dot(Vec3::X).abs() < 0.9 {
            Vec3::X
        } else if delta_norm.dot(Vec3::Z).abs() < 0.9 {
            Vec3::Z
        } else {
            panic!("No valid up axis");
        };
        circle_trx.rotation = Quat::look_at(from, to, up);
    }

    debug_circle(lines, circle_trx, radius, res, color_from);
    for ofs in [
        circle_trx.local_y() * radius,
        circle_trx.local_x() * radius,
        -circle_trx.local_y() * radius,
        -circle_trx.local_x() * radius,
    ] {
        lines.line_gradient(from + ofs, to + ofs, 0.0, color_from, color_to);
    }
    circle_trx.translation += delta;
    debug_circle(lines, circle_trx, radius, res, color_to);
}

pub fn debug_arrow(
    lines: &mut DebugLines,
    from: Vec3,
    to: Vec3,
    radius: f32,
    res: usize,
    color: Color,
) {
    let delta = to - from;

    let mut circle_trx = Transform::default();
    circle_trx.translation = from;
    if from != to {
        let delta_norm = delta.normalize();
        let up = if delta_norm.dot(Vec3::Y).abs() < 0.9 {
            Vec3::Y
        } else if delta_norm.dot(Vec3::X).abs() < 0.9 {
            Vec3::X
        } else if delta_norm.dot(Vec3::Z).abs() < 0.9 {
            Vec3::Z
        } else {
            panic!("No valid up axis");
        };
        circle_trx.rotation = Quat::look_at(from, to, up);
    }

    debug_circle(lines, circle_trx, radius, res, color);
    for ofs in [
        circle_trx.local_y() * radius,
        circle_trx.local_x() * radius,
        -circle_trx.local_y() * radius,
        -circle_trx.local_x() * radius,
    ] {
        lines.line_colored(from + ofs, to, 0.0, color);
    }
}

pub fn debug_sphere(
    lines: &mut DebugLines,
    transform: Transform,
    radius: f32,
    res: usize,
    color: Color,
) {
    let verts_xy = circle_verts(res, radius).collect::<Vec<_>>();

    let verts_xz = verts_xy
        .iter()
        .map(|vert| Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2) * *vert)
        .collect::<Vec<_>>();

    let verts_yz = verts_xy
        .iter()
        .map(|vert| Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2) * *vert)
        .collect::<Vec<_>>();

    for verts in [verts_xy, verts_xz, verts_yz] {
        for window in verts.windows(2) {
            let v0 = transform * window[0];
            let v1 = transform * window[1];
            lines.line_colored(v0, v1, 0.0, color);
        }
    }
}
