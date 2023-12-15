pub mod debug_colors;
pub mod fixed_timestep;
pub mod interpolation;
//pub mod rigs;

use bevy::prelude::{Plugin, Query, ResMut, Transform};
use bevy_prototype_debug_lines::DebugLines;
use fixed_timestep::RapierFixedTimestepPlugin;
use interpolation::TransformInterpolationPlugin;

pub struct RapierExPlugin {
    pub fixed_dt: Option<f64>,
}

impl Plugin for RapierExPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if let Some(fixed_dt) = self.fixed_dt {
            app.add_plugin(RapierFixedTimestepPlugin { dt: fixed_dt });
        }
        app.add_plugin(TransformInterpolationPlugin);

        app.add_system(transform_debug_lines);
    }
}

pub fn transform_debug_lines(mut lines: ResMut<DebugLines>, query: Query<&Transform>) {
    for transform in query.iter() {
        lines.line_colored(
            transform.translation,
            transform.translation + transform.local_x() * 0.5,
            0.0,
            debug_colors::AXIS_X,
        );
        lines.line_colored(
            transform.translation,
            transform.translation + transform.local_y() * 0.5,
            0.0,
            debug_colors::AXIS_Y,
        );
        lines.line_colored(
            transform.translation,
            transform.translation + transform.local_z() * 0.5,
            0.0,
            debug_colors::AXIS_Z,
        );
    }
}
