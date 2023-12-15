use bevy::prelude::{Mat3, Quat, Vec3};

pub trait QuatEx {
    fn from_axes(right: Vec3, up: Vec3, forward: Vec3) -> Quat;

    fn look_to_up(up: Vec3, forward: Vec3) -> Quat {
        // Infer right vector from provided up and forward
        let right = up.cross(forward).normalize();

        // Recaculate forward vector to ensure orthonormalization
        let forward = right.cross(up);

        // Build rotation
        Self::from_axes(right, up, forward)
    }

    fn look_to_right(right: Vec3, up: Vec3) -> Quat {
        let forward = right.cross(up).normalize();
        let up = forward.cross(right);
        Self::from_axes(right, up, forward)
    }

    fn look_to_forward(forward: Vec3, up: Vec3) -> Quat {
        // Infer right vector from provided up and forward
        let right = up.cross(forward).normalize();

        // Recaculate up vector to ensure orthonormalization
        let up = forward.cross(right);

        // Build rotation
        Self::from_axes(right, up, forward)
    }

    fn look_at(from: Vec3, to: Vec3, up: Vec3) -> Quat {
        let delta = from - to;
        if delta != Vec3::ZERO {
            Self::look_to_forward(delta.normalize(), up)
        } else {
            Quat::IDENTITY
        }
    }

    fn delta_rotation(scaled_axis: Vec3, dt: f32) -> Quat {
        let mut half_angle = scaled_axis * dt * 0.5;
        let len = half_angle.length();
        if len > 0.0 {
            half_angle *= len.sin() / len;
            Quat::from_xyzw(half_angle.x, half_angle.y, half_angle.z, len.cos())
        } else {
            Quat::from_xyzw(half_angle.x, half_angle.y, half_angle.z, 1.0)
        }
    }

    // Calculate delta rotation by taylor series expansion
    // Cheaper than using delta_rotation, but requires normalization
    fn delta_rotation_approx(scaled_axis: Vec3, dt: f32) -> Quat {
        let half_angle = scaled_axis * dt * 0.5;
        Quat::from_xyzw(half_angle.x, half_angle.y, half_angle.z, 1.0)
    }
}

impl QuatEx for Quat {
    #[inline]
    fn from_axes(right: Vec3, up: Vec3, forward: Vec3) -> Quat {
        Quat::from_mat3(&Mat3::from_cols(right, up, forward))
    }
}
