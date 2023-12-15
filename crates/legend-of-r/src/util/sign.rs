use bevy::prelude::Vec3;

pub trait Sign {
    fn sign(self) -> Self;
}

impl Sign for f32 {
    fn sign(self) -> Self {
        if self != 0.0 {
            self.signum()
        } else {
            0.0
        }
    }
}

impl Sign for Vec3 {
    fn sign(self) -> Self {
        Vec3::new(self.x.sign(), self.y.sign(), self.z.sign())
    }
}

