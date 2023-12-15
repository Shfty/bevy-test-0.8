use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use bevy::{
    hierarchy::{Children, Parent},
    math::IVec3,
    prelude::{default, Bundle, Changed, Component, Deref, DerefMut, Entity, Query, With, Without}, reflect::Reflect,
    ecs::reflect::ReflectComponent,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
#[reflect(Component)]
pub enum BoardRotation {
    Identity,
    CW,
    Inverse,
    CCW,
}

impl Default for BoardRotation {
    fn default() -> Self {
        BoardRotation::Identity
    }
}

impl BoardRotation {
    pub fn cw(self) -> Self {
        match self {
            BoardRotation::Identity => BoardRotation::CW,
            BoardRotation::CW => BoardRotation::Inverse,
            BoardRotation::Inverse => BoardRotation::CCW,
            BoardRotation::CCW => BoardRotation::Identity,
        }
    }

    pub fn ccw(self) -> Self {
        match self {
            BoardRotation::Identity => BoardRotation::CCW,
            BoardRotation::CCW => BoardRotation::Inverse,
            BoardRotation::Inverse => BoardRotation::CW,
            BoardRotation::CW => BoardRotation::Identity,
        }
    }

    pub fn inverse(self) -> Self {
        match self {
            BoardRotation::Identity => BoardRotation::Inverse,
            BoardRotation::CW => BoardRotation::CCW,
            BoardRotation::Inverse => BoardRotation::Identity,
            BoardRotation::CCW => BoardRotation::CW,
        }
    }
}

impl Add<Self> for BoardRotation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            BoardRotation::Identity => self,
            BoardRotation::CW => self.cw(),
            BoardRotation::Inverse => self.inverse(),
            BoardRotation::CCW => self.ccw(),
        }
    }
}

impl AddAssign<Self> for BoardRotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Sub<Self> for BoardRotation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs {
            BoardRotation::Identity => self,
            BoardRotation::CW => self.ccw(),
            BoardRotation::Inverse => self.inverse(),
            BoardRotation::CCW => self.cw(),
        }
    }
}

impl SubAssign<Self> for BoardRotation {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Mul<IVec3> for BoardRotation {
    type Output = IVec3;

    fn mul(self, rhs: IVec3) -> Self::Output {
        match self {
            BoardRotation::Identity => rhs,
            BoardRotation::CW => IVec3::new(-rhs.z, rhs.y, rhs.x),
            BoardRotation::Inverse => IVec3::new(-rhs.x, rhs.y, -rhs.z),
            BoardRotation::CCW => IVec3::new(rhs.z, rhs.y, -rhs.x),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Component, Reflect)]
#[reflect(Component)]
pub struct BoardTransform {
    pub translation: IVec3,
    pub rotation: BoardRotation,
}

impl From<IVec3> for BoardTransform {
    fn from(translation: IVec3) -> Self {
        BoardTransform {
            translation,
            ..default()
        }
    }
}

impl From<BoardTransform> for IVec3 {
    fn from(trx: BoardTransform) -> Self {
        trx.translation
    }
}

impl Add<Self> for BoardTransform {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        BoardTransform {
            translation: self.translation.add(rhs.translation),
            rotation: self.rotation.add(rhs.rotation),
        }
    }
}

impl AddAssign<Self> for BoardTransform {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Sub<Self> for BoardTransform {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        BoardTransform {
            translation: self.translation.sub(rhs.translation),
            rotation: self.rotation.sub(rhs.rotation),
        }
    }
}

impl SubAssign<Self> for BoardTransform {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(rhs);
    }
}

impl Mul<Self> for BoardTransform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        BoardTransform {
            translation: self.translation + self.rotation * rhs.translation,
            rotation: self.rotation + rhs.rotation,
        }
    }
}

impl Mul<IVec3> for BoardTransform {
    type Output = IVec3;

    fn mul(self, rhs: IVec3) -> Self::Output {
        self.translation + (self.rotation * rhs)
    }
}

#[derive(Debug, Default, Copy, Clone, Deref, DerefMut, Component, Reflect)]
#[reflect(Component)]
pub struct GlobalBoardTransform(pub BoardTransform);

impl From<BoardTransform> for GlobalBoardTransform {
    fn from(transform: BoardTransform) -> Self {
        GlobalBoardTransform(transform)
    }
}

#[derive(Debug, Default, Copy, Clone, Bundle)]
pub struct BoardTransformBundle {
    pub position: BoardTransform,
    pub global_position: GlobalBoardTransform,
}

impl From<BoardTransform> for BoardTransformBundle {
    fn from(transform: BoardTransform) -> Self {
        BoardTransformBundle {
            position: transform,
            global_position: GlobalBoardTransform(transform),
        }
    }
}

pub fn propagate_global_positions(
    mut query_root: Query<
        (
            &BoardTransform,
            Changed<BoardTransform>,
            &mut GlobalBoardTransform,
            Option<&Children>,
        ),
        Without<Parent>,
    >,
    mut query_child: Query<
        (
            &BoardTransform,
            Changed<BoardTransform>,
            &mut GlobalBoardTransform,
            Option<&Children>,
        ),
        With<Parent>,
    >,
) {
    for (position, position_changed, mut global_position, children) in query_root.iter_mut() {
        if position_changed {
            global_position.translation = position.translation;
            global_position.rotation = position.rotation;
        }

        if let Some(children) = children {
            for child in children.iter() {
                propagate_global_positions_impl(
                    child,
                    &mut query_child,
                    *position,
                    position_changed,
                    1,
                )
            }
        }
    }
}

fn propagate_global_positions_impl(
    entity: &Entity,
    query: &mut Query<
        (
            &BoardTransform,
            Changed<BoardTransform>,
            &mut GlobalBoardTransform,
            Option<&Children>,
        ),
        With<Parent>,
    >,
    mut global: BoardTransform,
    mut changed: bool,
    depth: usize,
) {
    let (transform, position_changed, mut global_transform, children) =
        if let Ok(components) = query.get_mut(*entity) {
            components
        } else {
            return;
        };

    global = global * *transform;

    changed |= position_changed;
    if changed {
        global_transform.translation = global.translation;
        global_transform.rotation = global.rotation;
    }

    if let Some(children) = children.cloned() {
        for child in children.into_iter() {
            propagate_global_positions_impl(child, query, global, changed, depth + 1)
        }
    }
}
