use bevy::{
    ecs::system::Command,
    prelude::{
        default, Bundle, Camera3d, Commands, Component, Entity, Name,
        ParallelSystemDescriptorCoercion, PerspectiveProjection, Plugin, Query, Transform, Vec3,
        Vec4, With, Res,
    },
    render::{
        camera::CameraProjection,
        primitives::{Aabb, Frustum, Plane, Sphere},
    },
    transform::{transform_propagate_system, TransformBundle},
};
use bevy_rapier2d::{
    na::{Point2, Unit, Vector2},
    prelude::{Collider, PhysicsStages},
    rapier::prelude::{
        Ball as Ball2d, ConvexPolygon, HalfSpace as HalfSpace2d, Segment as Segment2d,
        SharedShape as SharedShape2d, Triangle as Triangle2d,
    },
};
use parry3d::shape::{
    Ball as Ball3d, ConvexPolyhedron, Cuboid as Cuboid3d, HalfSpace as HalfSpace3d,
    Segment as Segment3d, SharedShape as SharedShape3d, Triangle as Triangle3d,
    TypedShape as TypedShape3d,
};

use crate::prelude::{
    camera::{CameraPivotSource, CustomPerspectiveProjection, PlaneTransformSource},
    default_entity,
    playfield::Playfield, AspectRatio,
};

pub struct PlaneColliderPlugin;

impl Plugin for PlaneColliderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(
            PhysicsStages::SyncBackend,
            plane_collider.before(transform_propagate_system),
        );
    }
}

#[derive(Clone, Component)]
pub struct PlaneCollider3d {
    pub shape: SharedShape3d,
    pub collider_2d: Entity,
}

impl Default for PlaneCollider3d {
    fn default() -> Self {
        Self {
            shape: SharedShape3d::new(Ball3d::new(1.0)),
            collider_2d: default_entity(),
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct PlaneCollider2d {
    pub collider_3d: Entity,
}

impl Default for PlaneCollider2d {
    fn default() -> Self {
        Self {
            collider_3d: default_entity(),
        }
    }
}

#[derive(Bundle)]
pub struct PlaneCollider3dBundle {
    pub name: Name,
    pub plane_collider_3d: PlaneCollider3d,
    #[bundle]
    pub transform_bundle: TransformBundle,
}

impl Default for PlaneCollider3dBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Plane Collider 3D"),
            plane_collider_3d: default(),
            transform_bundle: default(),
        }
    }
}

#[derive(Bundle)]
pub struct PlaneCollider2dBundle {
    pub name: Name,
    pub plane_collider_2d: PlaneCollider2d,
    #[bundle]
    pub transform_bundle: TransformBundle,
}

impl Default for PlaneCollider2dBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Plane Collider 2D"),
            plane_collider_2d: default(),
            transform_bundle: default(),
        }
    }
}

pub struct AssemblePlaneCollider {
    pub collider_3d: Entity,
    pub collider_2d: Entity,
    pub shape: SharedShape3d,
}

impl Command for AssemblePlaneCollider {
    fn write(self, world: &mut bevy::prelude::World) {
        world
            .entity_mut(self.collider_3d)
            .insert_bundle(PlaneCollider3dBundle {
                plane_collider_3d: PlaneCollider3d {
                    shape: self.shape,
                    collider_2d: self.collider_2d,
                },
                ..default()
            });

        world
            .entity_mut(self.collider_2d)
            .insert_bundle(PlaneCollider2dBundle {
                plane_collider_2d: PlaneCollider2d {
                    collider_3d: self.collider_3d,
                },
                ..default()
            });
    }
}

/// Calculate the intersection between a line and a +Z plane centered at the origin
fn line_plane_intersection(v0: Vec3, v1: Vec3) -> Vec<Vec3> {
    // Depth sort
    let (v0, v1) = if v0.z <= v1.z { (v0, v1) } else { (v1, v0) };

    let delta = v1 - v0;
    let dir = delta.normalize();

    let dot = dir.dot(Vec3::Z);

    if dot == 0.0 {
        if v0.dot(Vec3::Z) == 0.0 {
            return vec![v0, v1];
        }
    }
    // Line is intersecting, determine if segment if intersecting
    else if v0.z < 0.0 && v1.z > 0.0 {
        let point = v0 + dir * (-v0.dot(Vec3::Z) / dot);
        return vec![point];
    }

    vec![]
}

/// Calculate the intersections between a set of lines and a +Z plane centered at the origin
fn convex_plane_intersection(lines: impl IntoIterator<Item = (Vec3, Vec3)>) -> Vec<Vec3> {
    lines
        .into_iter()
        .flat_map(|(v0, v1)| line_plane_intersection(v0, v1))
        .collect()
}

#[derive(Debug, Default, Copy, Clone)]
pub struct CrossSectionContext {
    pub plane_local: Transform,
    pub projected_translation: Vec3,
    pub scalar: f32,
}

impl CrossSectionContext {
    fn plane_local_to_world(&self, v: Vec3) -> Vec3 {
        ((v - self.plane_local.translation) * self.scalar) + self.projected_translation
    }
}

pub trait CrossSection {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)>;
}

impl CrossSection for Segment3d {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        let v0 = self.a;
        let v1 = self.b;

        let v0 = context.plane_local * Vec3::new(v0.x, v0.y, v0.z);
        let v1 = context.plane_local * Vec3::new(v1.x, v1.y, v1.z);

        let points = line_plane_intersection(v0, v1);

        match points.len() {
            0 => None,
            1 => {
                let v = context.plane_local_to_world(points[0]);

                Some((
                    Transform::from_xyz(v.x, v.y, 0.0),
                    SharedShape2d::new(Segment2d {
                        a: default(),
                        b: default(),
                    }),
                ))
            }
            2 => {
                let v0 = context.plane_local_to_world(points[0]);
                let v1 = context.plane_local_to_world(points[1]);

                Some((
                    default(),
                    SharedShape2d::new(Segment2d {
                        a: Point2::new(v0.x, v0.y),
                        b: Point2::new(v1.x, v1.y),
                    }),
                ))
            }
            _ => unreachable!(),
        }
    }
}

impl CrossSection for Triangle3d {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        let points = [
            context.plane_local * Vec3::new(self.a.x, self.a.y, self.a.z),
            context.plane_local * Vec3::new(self.b.x, self.b.y, self.b.z),
            context.plane_local * Vec3::new(self.c.x, self.c.y, self.c.z),
        ];

        let lines = [
            (points[0], points[1]),
            (points[1], points[2]),
            (points[2], points[0]),
        ];

        let intersections = convex_plane_intersection(lines)
            .into_iter()
            .map(|point| context.plane_local_to_world(point))
            .collect::<Vec<_>>();

        match intersections.len() {
            0 => None,
            1 => {
                let v = intersections[0];

                Some((
                    Transform::from_xyz(v.x, v.y, 0.0),
                    SharedShape2d::new(Segment2d {
                        a: default(),
                        b: default(),
                    }),
                ))
            }
            2 => {
                let v0 = intersections[0];
                let v1 = intersections[1];

                Some((
                    default(),
                    SharedShape2d::new(Segment2d {
                        a: Point2::new(v0.x, v0.y),
                        b: Point2::new(v1.x, v1.y),
                    }),
                ))
            }
            3 => {
                let v0 = intersections[0];
                let v1 = intersections[1];
                let v2 = intersections[2];

                let a = Point2::new(v0.x, v0.y);
                let b = Point2::new(v1.x, v1.y);
                let c = Point2::new(v2.x, v2.y);

                Some((default(), SharedShape2d::new(Triangle2d { a, b, c })))
            }
            _ => None,
        }
    }
}

impl CrossSection for HalfSpace3d {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        let normal_3d =
            context.plane_local.rotation * Vec3::new(self.normal.x, self.normal.y, self.normal.z);

        let dot = normal_3d.dot(Vec3::Z);
        let dist = context.plane_local.translation.z;
        let normal_2d = Vec3::new(normal_3d.x, normal_3d.y, 0.0).normalize();

        if normal_2d.is_nan() || normal_2d == Vec3::ZERO {
            return None;
        }

        let fac = dist * normal_2d * dot;

        Some((
            context
                .plane_local
                .with_translation(context.projected_translation + fac),
            SharedShape2d::new(HalfSpace2d::new(Unit::new_normalize(Vector2::new(
                normal_2d.x,
                normal_2d.y,
            )))),
        ))
    }
}

impl CrossSection for Ball3d {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        let radius = self.radius * context.scalar;
        let dist = context.plane_local.translation.z;
        let depth = (radius - dist.abs()).max(0.0);
        let fac = (depth / radius).sin();

        if fac > 0.0 {
            Some((
                context
                    .plane_local
                    .with_translation(context.projected_translation),
                SharedShape2d::new(Ball2d::new(fac * radius)),
            ))
        } else {
            None
        }
    }
}

impl CrossSection for Cuboid3d {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        let half = self.half_extents;

        let points = [
            context.plane_local * Vec3::new(-half.x, -half.y, -half.z),
            context.plane_local * Vec3::new(half.x, -half.y, -half.z),
            context.plane_local * Vec3::new(half.x, -half.y, half.z),
            context.plane_local * Vec3::new(-half.x, -half.y, half.z),
            context.plane_local * Vec3::new(-half.x, half.y, -half.z),
            context.plane_local * Vec3::new(half.x, half.y, -half.z),
            context.plane_local * Vec3::new(half.x, half.y, half.z),
            context.plane_local * Vec3::new(-half.x, half.y, half.z),
        ];

        let lines = [
            (points[0], points[1]),
            (points[1], points[2]),
            (points[2], points[3]),
            (points[3], points[0]),
            (points[4], points[5]),
            (points[5], points[6]),
            (points[6], points[7]),
            (points[7], points[4]),
            (points[0], points[4]),
            (points[1], points[5]),
            (points[2], points[6]),
            (points[3], points[7]),
        ];

        let polygon_points = convex_plane_intersection(lines);

        let polygon_points = polygon_points
            .into_iter()
            .map(|point| context.plane_local_to_world(point))
            .map(|point| Point2::new(point.x, point.y))
            .collect::<Vec<_>>();

        if polygon_points.len() > 2 {
            Some((
                default(),
                SharedShape2d::new(
                    ConvexPolygon::from_convex_hull(&polygon_points[..]).unwrap_or_else(|| {
                        panic!("Failed to build Cuboid cross-section from {points:#?}")
                    }),
                ),
            ))
        } else {
            None
        }
    }
}

impl CrossSection for ConvexPolyhedron {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        let polyhedron_points = self.points();
        let polygon_points = convex_plane_intersection(self.edges().into_iter().map(|edge| {
            let v0 = polyhedron_points[edge.vertices[0] as usize];
            let v1 = polyhedron_points[edge.vertices[1] as usize];

            let v0 = context.plane_local * Vec3::new(v0.x, v0.y, v0.z);
            let v1 = context.plane_local * Vec3::new(v1.x, v1.y, v1.z);

            (v0, v1)
        }));

        let polygon_points = polygon_points
            .into_iter()
            .map(|point| context.plane_local_to_world(point))
            .map(|point| Point2::new(point.x, point.y))
            .collect::<Vec<_>>();

        if polygon_points.len() > 2 {
            Some((
                default(),
                SharedShape2d::new(
                    ConvexPolygon::from_convex_hull(&polygon_points[..])
                        .expect("Failed to build ConvexPolyhedron cross-section from {points:#?}"),
                ),
            ))
        } else {
            None
        }
    }
}

impl CrossSection for TypedShape3d<'_> {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        match self {
            TypedShape3d::Segment(segment) => segment.cross_section(context),
            TypedShape3d::Triangle(triangle) => triangle.cross_section(context),
            TypedShape3d::HalfSpace(halfspace) => halfspace.cross_section(context),
            TypedShape3d::Ball(ball) => ball.cross_section(context),
            TypedShape3d::Cuboid(cuboid) => cuboid.cross_section(context),
            TypedShape3d::ConvexPolyhedron(convex) => convex.cross_section(context),
            _ => unimplemented!(),
        }
    }
}

impl CrossSection for SharedShape3d {
    fn cross_section(&self, context: CrossSectionContext) -> Option<(Transform, SharedShape2d)> {
        self.as_typed_shape().cross_section(context)
    }
}

fn plane_collider(
    aspect: Res<AspectRatio>,
    query_playfield: Query<&Playfield>,
    query_camera_pivot: Query<&Transform, With<CameraPivotSource>>,
    query_plane_transform: Query<&Transform, With<PlaneTransformSource>>,
    query_camera_projection: Query<&CustomPerspectiveProjection, With<Camera3d>>,
    query_collider_3d: Query<(&Transform, &PlaneCollider3d)>,
    mut query_collider_2d: Query<(), With<PlaneCollider2d>>,
    mut commands: Commands,
) {
    let playfield = if let Some(playfield) = query_playfield.iter().next() {
        playfield
    } else {
        return
    };

    let camera_pivot = query_camera_pivot.iter().next().unwrap();
    let playfield_half_size = playfield.half_size(**aspect, camera_pivot.translation.z);

    let plane_transform = query_plane_transform.iter().next().unwrap();
    let custom_projection = query_camera_projection.iter().next().unwrap();
    let custom_projection = CustomPerspectiveProjection {
        perspective: PerspectiveProjection {
            aspect_ratio: custom_projection.target_aspect,
            ..custom_projection.perspective
        },
        target_aspect: 1.0,
    };

    let projection = custom_projection.perspective.get_projection_matrix();
    let projection_offset = custom_projection.get_projection_matrix();

    let plane_transform_inv = Transform::from_matrix(plane_transform.compute_matrix().inverse());

    let frustum = Frustum {
        planes: [
            Plane::new(Vec3::NEG_X.extend(playfield_half_size.x)),
            Plane::new(Vec3::X.extend(playfield_half_size.x)),
            Plane::new(Vec3::NEG_Y.extend(playfield_half_size.y)),
            Plane::new(Vec3::Y.extend(playfield_half_size.y)),
            Plane::new(Vec3::NEG_Z.extend(0.0)),
            Plane::new(Vec3::Z.extend(0.0)),
        ],
    };

    for (collider_transform, plane_collider) in query_collider_3d.iter() {
        let plane_local = plane_transform_inv * *collider_transform;

        // Calculate broad phase collision shapes
        let collider_aabb = plane_collider.shape.compute_local_aabb();

        let sphere = Sphere {
            center: plane_local.translation.into(),
            radius: collider_aabb.bounding_sphere().radius,
        };

        let collider_aabb = Aabb {
            half_extents: collider_aabb.half_extents().into(),
            ..default()
        };

        // Sphere check
        if !frustum.intersects_sphere(&sphere, true) {
            commands
                .entity(plane_collider.collider_2d)
                .remove::<Collider>();
            continue;
        }

        // OBB check
        if !frustum.intersects_obb(&collider_aabb, &plane_local.compute_matrix(), true) {
            commands
                .entity(plane_collider.collider_2d)
                .remove::<Collider>();
            continue;
        }

        // Project translation onto plane
        let projected_translation = projection
            * Transform::from_xyz(
                -camera_pivot.translation.x,
                -camera_pivot.translation.y,
                -playfield_half_size.max_element(),
            )
            .compute_matrix()
            * Transform::from_rotation(camera_pivot.rotation.inverse()).compute_matrix()
            * Vec4::new(
                plane_local.translation.x,
                plane_local.translation.y,
                0.0,
                1.0,
            );

        let mut projected_translation = projected_translation.truncate() / projected_translation.w;

        projected_translation.x *= playfield_half_size.x;
        projected_translation.y *= playfield_half_size.y;

        let projected_translation = camera_pivot.rotation * projected_translation;
        let projected_translation = projected_translation
            + Vec3::new(camera_pivot.translation.x, camera_pivot.translation.y, 0.0);

        // Calculate projection scalar
        let scalar = (projection_offset * Vec4::X * (plane_local.translation.x + 1.0)).x
            - (projection_offset * Vec4::X * plane_local.translation.x).x;

        // Calculate intersection
        let intersection = plane_collider.shape.cross_section(CrossSectionContext {
            plane_local,
            projected_translation,
            scalar,
        });

        // Fetch 2D collider
        let collider_2d = query_collider_2d
            .get_mut(plane_collider.collider_2d)
            .is_ok();

        // Add or remove component based on intersection
        match (intersection, collider_2d) {
            (Some((transform, shape)), _) => {
                commands
                    .entity(plane_collider.collider_2d)
                    .insert(transform)
                    .insert(Collider::from(shape));
            }
            (None, false) => (),
            (None, true) => {
                commands
                    .entity(plane_collider.collider_2d)
                    .remove::<Collider>();
            }
        }
    }
}
