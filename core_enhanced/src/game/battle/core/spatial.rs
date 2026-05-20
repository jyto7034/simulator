use crate::game::ability::SkillAreaShapeDef;
use rapier2d::{
    glamx::Vec2,
    parry::math::Pose,
    parry::query::{cast_shapes, intersection_test, ShapeCastOptions},
    prelude::{Ball, Capsule, Cuboid, Triangle},
};

use super::movement::{
    types::{UnitBody, WorldVec2, DATA_UNITS_PER_WORLD},
    HALF_TILE_UNITS,
};

pub const DEFAULT_UNIT_HITBOX_RADIUS_UNITS: i64 = (HALF_TILE_UNITS as i64) / 2;
const MIN_CONE_WEDGE_COS: f32 = 0.01;

pub fn distance_world(a: WorldVec2, b: WorldVec2) -> f32 {
    a.distance(b)
}

pub fn body_distance(a: &UnitBody, b: &UnitBody) -> f32 {
    a.distance_to(b)
}

pub fn bodies_in_range(a: &UnitBody, b: &UnitBody, range_units: f32) -> bool {
    a.can_reach(b, range_units)
}

pub fn data_units_to_world(units: i64) -> f32 {
    units as f32 / DATA_UNITS_PER_WORLD
}

pub fn circle_contains_world_point(center: WorldVec2, radius: f32, point: WorldVec2) -> bool {
    if radius <= 0.0 {
        return center == point;
    }

    center.distance_squared(point) <= radius * radius
}

pub fn box_contains_world_point_centered(
    center: WorldVec2,
    width: f32,
    height: f32,
    expansion: f32,
    point: WorldVec2,
) -> bool {
    if width <= 0.0 || height <= 0.0 {
        return point == center;
    }

    let half_width = width / 2.0 + expansion.max(0.0);
    let half_height = height / 2.0 + expansion.max(0.0);
    let dx = (point.x - center.x).abs();
    let dy = (point.y - center.y).abs();
    dx <= half_width && dy <= half_height
}

pub fn line_contains_world_point_from_origin(
    origin: WorldVec2,
    direction_hint: WorldVec2,
    length: f32,
    expansion: f32,
    point: WorldVec2,
) -> bool {
    if length <= 0.0 {
        return point == origin;
    }

    let direction = direction_hint - origin;
    let dir_len_sq = direction.length_squared();
    if dir_len_sq <= f32::EPSILON {
        return false;
    }

    let rel = point - origin;
    let parallel = rel.x * direction.x + rel.y * direction.y;
    let expansion = expansion.max(0.0);
    let min_parallel = -expansion * direction.length();
    let max_parallel = (length + expansion) * direction.length();
    if parallel < min_parallel || parallel > max_parallel {
        return false;
    }

    let perp = rel.x * -direction.y + rel.y * direction.x;
    perp.abs() <= expansion * direction.length()
}

pub fn rectangle_contains_world_point_from_origin(
    origin: WorldVec2,
    direction_hint: WorldVec2,
    width: f32,
    length: f32,
    expansion: f32,
    point: WorldVec2,
) -> bool {
    if width <= 0.0 || length <= 0.0 {
        return point == origin;
    }

    let direction = direction_hint - origin;
    let dir_len_sq = direction.length_squared();
    if dir_len_sq <= f32::EPSILON {
        return false;
    }

    let dir_len = direction.length();
    let rel = point - origin;
    let parallel = rel.x * direction.x + rel.y * direction.y;
    let expansion = expansion.max(0.0);
    let min_parallel = -expansion * dir_len;
    let max_parallel = (length + expansion) * dir_len;
    if parallel < min_parallel || parallel > max_parallel {
        return false;
    }

    let perp = rel.x * -direction.y + rel.y * direction.x;
    perp.abs() <= (width / 2.0 + expansion) * dir_len
}

pub fn cone_contains_world_point_from_origin(
    origin: WorldVec2,
    direction_hint: WorldVec2,
    angle_degrees: u16,
    length: f32,
    expansion: f32,
    point: WorldVec2,
) -> bool {
    if angle_degrees == 0 || length <= 0.0 {
        return point == origin;
    }

    let direction = direction_hint - origin;
    let dir_len = direction.length();
    if dir_len <= f32::EPSILON {
        return false;
    }

    // Cone skills intentionally use an expanded triangle wedge. This is the
    // gameplay contract, not an exact Minkowski sum of a circular sector.
    let half_angle = (f32::from(angle_degrees) / 2.0).to_radians();
    let direction_unit = direction * (1.0 / dir_len);
    let reach = length + expansion.max(0.0);
    let edge_reach = cone_wedge_edge_reach(reach, half_angle);
    let left = origin + rotate_world_vec2(direction_unit, half_angle) * edge_reach;
    let right = origin + rotate_world_vec2(direction_unit, -half_angle) * edge_reach;

    triangle_contains_world_point(origin, left, right, point)
}

fn cone_wedge_edge_reach(centerline_reach: f32, half_angle_radians: f32) -> f32 {
    centerline_reach / half_angle_radians.cos().max(MIN_CONE_WEDGE_COS)
}

fn triangle_contains_world_point(
    a: WorldVec2,
    b: WorldVec2,
    c: WorldVec2,
    point: WorldVec2,
) -> bool {
    let ab = cross_world_vec2(b - a, point - a);
    let bc = cross_world_vec2(c - b, point - b);
    let ca = cross_world_vec2(a - c, point - c);
    let has_negative = ab < 0.0 || bc < 0.0 || ca < 0.0;
    let has_positive = ab > 0.0 || bc > 0.0 || ca > 0.0;

    !(has_negative && has_positive)
}

fn cross_world_vec2(a: WorldVec2, b: WorldVec2) -> f32 {
    a.x * b.y - a.y * b.x
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AreaQueryShape {
    pub origin: WorldVec2,
    pub center: WorldVec2,
    pub direction_hint: WorldVec2,
    pub shape: SkillAreaShapeDef,
    pub hitbox_expansion: f32,
}

pub fn moving_circle_sweep_hit_fraction(
    projectile_start: WorldVec2,
    projectile_end: WorldVec2,
    reach: f32,
    target_start: WorldVec2,
    target_end: WorldVec2,
) -> Option<f32> {
    if reach < 0.0 {
        return None;
    }

    let initial_delta = projectile_start - target_start;
    let relative_motion = (projectile_end - projectile_start) - (target_end - target_start);
    let c = initial_delta.length_squared() - reach * reach;
    if c <= 0.0 {
        return Some(0.0);
    }

    let a = relative_motion.length_squared();
    if a <= f32::EPSILON {
        return None;
    }

    let b = 2.0 * (initial_delta.x * relative_motion.x + initial_delta.y * relative_motion.y);
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let inv_2a = 1.0 / (2.0 * a);
    let enter = (-b - sqrt_discriminant) * inv_2a;
    let exit = (-b + sqrt_discriminant) * inv_2a;
    if exit < 0.0 || enter > 1.0 {
        return None;
    }

    Some(enter.max(0.0).clamp(0.0, 1.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialQueryBackend {
    Direct,
    Rapier(RapierSpatialQuery),
}

impl Default for SpatialQueryBackend {
    fn default() -> Self {
        Self::Rapier(RapierSpatialQuery)
    }
}

impl SpatialQueryBackend {
    pub fn contains_area_point(&self, query: &AreaQueryShape, point: WorldVec2) -> bool {
        match self {
            SpatialQueryBackend::Direct => DirectSpatialQuery::contains_area_point(query, point),
            SpatialQueryBackend::Rapier(rapier) => rapier.contains_area_point(query, point),
        }
    }

    pub fn projectile_hits_point(
        &self,
        projectile_position: WorldVec2,
        reach: f32,
        point: WorldVec2,
    ) -> bool {
        match self {
            SpatialQueryBackend::Direct => {
                DirectSpatialQuery::projectile_hits_point(projectile_position, reach, point)
            }
            SpatialQueryBackend::Rapier(rapier) => {
                rapier.projectile_hits_point(projectile_position, reach, point)
            }
        }
    }

    pub fn projectile_sweep_hit_fraction(
        &self,
        projectile_start: WorldVec2,
        projectile_end: WorldVec2,
        reach: f32,
        point: WorldVec2,
    ) -> Option<f32> {
        match self {
            SpatialQueryBackend::Direct => DirectSpatialQuery::projectile_sweep_hit_fraction(
                projectile_start,
                projectile_end,
                reach,
                point,
            ),
            SpatialQueryBackend::Rapier(rapier) => {
                rapier.projectile_sweep_hit_fraction(projectile_start, projectile_end, reach, point)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirectSpatialQuery;

impl DirectSpatialQuery {
    pub fn contains_area_point(query: &AreaQueryShape, point: WorldVec2) -> bool {
        match query.shape {
            SkillAreaShapeDef::Circle { radius_units } => circle_contains_world_point(
                query.center,
                data_units_to_world(i64::from(radius_units)) + query.hitbox_expansion,
                point,
            ),
            SkillAreaShapeDef::Line { length_units } => line_contains_world_point_from_origin(
                query.origin,
                query.direction_hint,
                data_units_to_world(i64::from(length_units)),
                query.hitbox_expansion,
                point,
            ),
            SkillAreaShapeDef::Box {
                width_units,
                height_units,
            } => box_contains_world_point_centered(
                query.center,
                data_units_to_world(i64::from(width_units)),
                data_units_to_world(i64::from(height_units)),
                query.hitbox_expansion,
                point,
            ),
            SkillAreaShapeDef::Rectangle {
                width_units,
                length_units,
            } => rectangle_contains_world_point_from_origin(
                query.origin,
                query.direction_hint,
                data_units_to_world(i64::from(width_units)),
                data_units_to_world(i64::from(length_units)),
                query.hitbox_expansion,
                point,
            ),
            SkillAreaShapeDef::Cone {
                angle_degrees,
                length_units,
            } => cone_contains_world_point_from_origin(
                query.origin,
                query.direction_hint,
                angle_degrees,
                data_units_to_world(i64::from(length_units)),
                query.hitbox_expansion,
                point,
            ),
        }
    }

    pub fn projectile_hits_point(
        projectile_position: WorldVec2,
        reach: f32,
        point: WorldVec2,
    ) -> bool {
        circle_contains_world_point(projectile_position, reach, point)
    }

    pub fn projectile_sweep_hit_fraction(
        projectile_start: WorldVec2,
        projectile_end: WorldVec2,
        reach: f32,
        point: WorldVec2,
    ) -> Option<f32> {
        if reach < 0.0 {
            return None;
        }

        let motion = projectile_end - projectile_start;
        let relative = projectile_start - point;
        let a = motion.length_squared();
        if a <= f32::EPSILON {
            return Self::projectile_hits_point(projectile_start, reach, point).then_some(0.0);
        }

        let c = relative.length_squared() - reach * reach;
        if c <= 0.0 {
            return Some(0.0);
        }

        let b = 2.0 * (relative.x * motion.x + relative.y * motion.y);
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        (0.0..=1.0).contains(&t).then_some(t)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RapierSpatialQuery;

impl RapierSpatialQuery {
    pub fn contains_area_point(&self, query: &AreaQueryShape, point: WorldVec2) -> bool {
        match query.shape {
            SkillAreaShapeDef::Circle { radius_units } => self.circle_intersects_point(
                query.center,
                data_units_to_world(i64::from(radius_units)) + query.hitbox_expansion,
                point,
            ),
            SkillAreaShapeDef::Box {
                width_units,
                height_units,
            } => self.cuboid_intersects_point(
                query.center,
                data_units_to_world(i64::from(width_units)) * 0.5 + query.hitbox_expansion,
                data_units_to_world(i64::from(height_units)) * 0.5 + query.hitbox_expansion,
                0.0,
                point,
            ),
            SkillAreaShapeDef::Rectangle {
                width_units,
                length_units,
            } => {
                let direction = query.direction_hint - query.origin;
                let direction_len = direction.length();
                if direction_len <= f32::EPSILON {
                    return false;
                }
                let length = data_units_to_world(i64::from(length_units));
                let expansion = query.hitbox_expansion.max(0.0);
                let direction_unit = direction * (1.0 / direction_len);
                let center = query.origin + direction_unit * (length * 0.5);
                self.cuboid_intersects_point(
                    center,
                    length * 0.5 + expansion,
                    data_units_to_world(i64::from(width_units)) * 0.5 + expansion,
                    direction.y.atan2(direction.x),
                    point,
                )
            }
            SkillAreaShapeDef::Line { length_units } => {
                let length = data_units_to_world(i64::from(length_units));
                if length <= 0.0 {
                    return point == query.origin;
                }

                let direction = query.direction_hint - query.origin;
                let direction_len = direction.length();
                if direction_len <= f32::EPSILON {
                    return false;
                }

                let expansion = query.hitbox_expansion.max(0.0);
                let direction_unit = direction * (1.0 / direction_len);
                let center = query.origin + direction_unit * (length * 0.5);
                self.capsule_intersects_point(
                    center,
                    length * 0.5,
                    expansion,
                    direction.y.atan2(direction.x),
                    point,
                )
            }
            SkillAreaShapeDef::Cone {
                angle_degrees,
                length_units,
            } => {
                let length = data_units_to_world(i64::from(length_units));
                if angle_degrees == 0 || length <= 0.0 {
                    return point == query.origin;
                }

                let direction = query.direction_hint - query.origin;
                let direction_len = direction.length();
                if direction_len <= f32::EPSILON {
                    return false;
                }

                let reach = length + query.hitbox_expansion.max(0.0);
                let half_angle = (f32::from(angle_degrees) * 0.5).to_radians();
                let direction_unit = direction * (1.0 / direction_len);
                // Cone is intentionally a triangle wedge, not a circular sector.
                // `length_units` is the centerline reach; edge vertices extend to preserve it.
                let edge_reach = cone_wedge_edge_reach(reach, half_angle);
                let left =
                    query.origin + rotate_world_vec2(direction_unit, half_angle) * edge_reach;
                let right =
                    query.origin + rotate_world_vec2(direction_unit, -half_angle) * edge_reach;
                self.triangle_intersects_point(query.origin, left, right, point)
            }
        }
    }

    pub fn projectile_hits_point(
        &self,
        projectile_position: WorldVec2,
        reach: f32,
        point: WorldVec2,
    ) -> bool {
        self.circle_intersects_point(projectile_position, reach, point)
    }

    pub fn projectile_sweep_hit_fraction(
        &self,
        projectile_start: WorldVec2,
        projectile_end: WorldVec2,
        reach: f32,
        point: WorldVec2,
    ) -> Option<f32> {
        if reach < 0.0 {
            return None;
        }

        let motion = projectile_end - projectile_start;
        if motion.length_squared() <= f32::EPSILON {
            return self
                .projectile_hits_point(projectile_start, reach, point)
                .then_some(0.0);
        }

        let projectile = Ball::new(reach);
        let point_shape = Ball::new(0.0);
        let options = ShapeCastOptions::with_max_time_of_impact(1.0);
        let hit = cast_shapes(
            &pose_at(projectile_start, 0.0),
            Vec2::new(motion.x, motion.y),
            &projectile,
            &pose_at(point, 0.0),
            Vec2::ZERO,
            &point_shape,
            options,
        )
        .ok()
        .flatten()?;
        Some(hit.time_of_impact.clamp(0.0, 1.0))
    }

    fn circle_intersects_point(&self, center: WorldVec2, radius: f32, point: WorldVec2) -> bool {
        if radius <= 0.0 {
            return center == point;
        }
        let circle = Ball::new(radius);
        let point_shape = Ball::new(0.0);
        intersects(
            pose_at(center, 0.0),
            &circle,
            pose_at(point, 0.0),
            &point_shape,
        )
    }

    fn cuboid_intersects_point(
        &self,
        center: WorldVec2,
        half_width: f32,
        half_height: f32,
        rotation_radians: f32,
        point: WorldVec2,
    ) -> bool {
        if half_width <= 0.0 || half_height <= 0.0 {
            return center == point;
        }
        let cuboid = Cuboid::new(Vec2::new(half_width, half_height));
        let point_shape = Ball::new(0.0);
        intersects(
            pose_at(center, rotation_radians),
            &cuboid,
            pose_at(point, 0.0),
            &point_shape,
        )
    }

    fn capsule_intersects_point(
        &self,
        center: WorldVec2,
        half_height: f32,
        radius: f32,
        rotation_radians: f32,
        point: WorldVec2,
    ) -> bool {
        if half_height <= 0.0 || radius < 0.0 {
            return center == point;
        }

        let capsule = Capsule::new_x(half_height, radius);
        let point_shape = Ball::new(0.0);
        intersects(
            pose_at(center, rotation_radians),
            &capsule,
            pose_at(point, 0.0),
            &point_shape,
        )
    }

    fn triangle_intersects_point(
        &self,
        a: WorldVec2,
        b: WorldVec2,
        c: WorldVec2,
        point: WorldVec2,
    ) -> bool {
        let triangle = Triangle::new(
            Vec2::new(a.x, a.y),
            Vec2::new(b.x, b.y),
            Vec2::new(c.x, c.y),
        );
        let point_shape = Ball::new(0.0);
        intersects(
            pose_at(WorldVec2::ZERO, 0.0),
            &triangle,
            pose_at(point, 0.0),
            &point_shape,
        )
    }
}

fn pose_at(position: WorldVec2, rotation_radians: f32) -> Pose {
    Pose::new(Vec2::new(position.x, position.y), rotation_radians)
}

fn rotate_world_vec2(v: WorldVec2, radians: f32) -> WorldVec2 {
    let (sin, cos) = radians.sin_cos();
    WorldVec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn intersects(
    pose_a: Pose,
    shape_a: &dyn rapier2d::parry::shape::Shape,
    pose_b: Pose,
    shape_b: &dyn rapier2d::parry::shape::Shape,
) -> bool {
    intersection_test(&pose_a, shape_a, &pose_b, shape_b).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bodies_in_range_uses_world_distance_and_radii() {
        let a = UnitBody::new_at(WorldVec2::new(0.0, 0.0), 0.35, 1.0);
        let b = UnitBody::new_at(WorldVec2::new(2.0, 0.0), 0.35, 1.0);

        assert!(bodies_in_range(&a, &b, 1.3));
        assert!(!bodies_in_range(&a, &b, 1.29));
        assert_eq!(body_distance(&a, &b), 2.0);
    }

    #[test]
    fn rapier_spatial_query_matches_direct_for_circle_box_and_rectangle() {
        let direct = SpatialQueryBackend::Direct;
        let rapier = SpatialQueryBackend::Rapier(RapierSpatialQuery);

        let circle = AreaQueryShape {
            origin: WorldVec2::ZERO,
            center: WorldVec2::new(2.0, 2.0),
            direction_hint: WorldVec2::new(3.0, 2.0),
            shape: SkillAreaShapeDef::Circle {
                radius_units: DATA_UNITS_PER_WORLD as u32,
            },
            hitbox_expansion: 0.25,
        };
        for point in [
            WorldVec2::new(2.0, 2.0),
            WorldVec2::new(3.2, 2.0),
            WorldVec2::new(3.4, 2.0),
        ] {
            assert_eq!(
                direct.contains_area_point(&circle, point),
                rapier.contains_area_point(&circle, point)
            );
        }

        let box_query = AreaQueryShape {
            origin: WorldVec2::ZERO,
            center: WorldVec2::new(2.0, 2.0),
            direction_hint: WorldVec2::new(3.0, 2.0),
            shape: SkillAreaShapeDef::Box {
                width_units: (DATA_UNITS_PER_WORLD * 2.0) as u32,
                height_units: DATA_UNITS_PER_WORLD as u32,
            },
            hitbox_expansion: 0.25,
        };
        for point in [
            WorldVec2::new(2.0, 2.0),
            WorldVec2::new(3.2, 2.0),
            WorldVec2::new(3.3, 2.0),
        ] {
            assert_eq!(
                direct.contains_area_point(&box_query, point),
                rapier.contains_area_point(&box_query, point)
            );
        }

        let rectangle = AreaQueryShape {
            origin: WorldVec2::new(1.0, 1.0),
            center: WorldVec2::new(4.0, 1.0),
            direction_hint: WorldVec2::new(4.0, 1.0),
            shape: SkillAreaShapeDef::Rectangle {
                width_units: DATA_UNITS_PER_WORLD as u32,
                length_units: (DATA_UNITS_PER_WORLD * 3.0) as u32,
            },
            hitbox_expansion: 0.25,
        };
        for point in [
            WorldVec2::new(1.0, 1.0),
            WorldVec2::new(4.2, 1.0),
            WorldVec2::new(4.4, 1.0),
        ] {
            assert_eq!(
                direct.contains_area_point(&rectangle, point),
                rapier.contains_area_point(&rectangle, point)
            );
        }
    }

    #[test]
    fn rapier_spatial_query_matches_direct_for_line_representative_points() {
        let direct = SpatialQueryBackend::Direct;
        let rapier = SpatialQueryBackend::Rapier(RapierSpatialQuery);
        let line = AreaQueryShape {
            origin: WorldVec2::new(1.0, 1.0),
            center: WorldVec2::new(1.0, 1.0),
            direction_hint: WorldVec2::new(4.0, 1.0),
            shape: SkillAreaShapeDef::Line {
                length_units: (DATA_UNITS_PER_WORLD * 3.0) as u32,
            },
            hitbox_expansion: 0.25,
        };

        for point in [
            WorldVec2::new(0.75, 1.0),
            WorldVec2::new(1.0, 1.0),
            WorldVec2::new(3.5, 1.2),
            WorldVec2::new(3.5, 1.3),
            WorldVec2::new(4.26, 1.0),
        ] {
            assert_eq!(
                direct.contains_area_point(&line, point),
                rapier.contains_area_point(&line, point),
                "point: {point:?}"
            );
        }
    }

    #[test]
    fn rapier_spatial_query_matches_direct_for_cone_representative_points() {
        let direct = SpatialQueryBackend::Direct;
        let rapier = SpatialQueryBackend::Rapier(RapierSpatialQuery);
        let cone = AreaQueryShape {
            origin: WorldVec2::new(1.0, 1.0),
            center: WorldVec2::new(1.0, 1.0),
            direction_hint: WorldVec2::new(4.0, 1.0),
            shape: SkillAreaShapeDef::Cone {
                angle_degrees: 60,
                length_units: (DATA_UNITS_PER_WORLD * 3.0) as u32,
            },
            hitbox_expansion: 0.25,
        };

        for point in [
            WorldVec2::new(1.0, 1.0),
            WorldVec2::new(3.0, 1.5),
            WorldVec2::new(4.2, 1.0),
            WorldVec2::new(3.0, 2.4),
            WorldVec2::new(4.4, 1.0),
        ] {
            assert_eq!(
                direct.contains_area_point(&cone, point),
                rapier.contains_area_point(&cone, point),
                "point: {point:?}"
            );
        }
    }

    #[test]
    fn cone_area_uses_triangle_wedge_not_circular_sector() {
        let direct = SpatialQueryBackend::Direct;
        let rapier = SpatialQueryBackend::Rapier(RapierSpatialQuery);
        let cone = AreaQueryShape {
            origin: WorldVec2::ZERO,
            center: WorldVec2::ZERO,
            direction_hint: WorldVec2::new(1.0, 0.0),
            shape: SkillAreaShapeDef::Cone {
                angle_degrees: 60,
                length_units: (DATA_UNITS_PER_WORLD * 3.0) as u32,
            },
            hitbox_expansion: 0.0,
        };

        let far_wedge_corner = WorldVec2::new(2.7, 1.5);

        assert!(far_wedge_corner.length() > 3.0);
        assert!(direct.contains_area_point(&cone, far_wedge_corner));
        assert!(rapier.contains_area_point(&cone, far_wedge_corner));
    }

    #[test]
    fn rapier_projectile_query_matches_direct_circle_overlap() {
        let direct = SpatialQueryBackend::Direct;
        let rapier = SpatialQueryBackend::Rapier(RapierSpatialQuery);
        let projectile_position = WorldVec2::new(1.0, 1.0);

        for point in [
            WorldVec2::new(1.0, 1.0),
            WorldVec2::new(1.5, 1.0),
            WorldVec2::new(1.6, 1.0),
        ] {
            assert_eq!(
                direct.projectile_hits_point(projectile_position, 0.5, point),
                rapier.projectile_hits_point(projectile_position, 0.5, point)
            );
        }
    }

    #[test]
    fn rapier_projectile_sweep_matches_direct_circle_cast() {
        let direct = SpatialQueryBackend::Direct;
        let rapier = SpatialQueryBackend::Rapier(RapierSpatialQuery);
        let start = WorldVec2::new(0.0, 0.0);
        let end = WorldVec2::new(4.0, 0.0);

        for point in [
            WorldVec2::new(0.0, 0.0),
            WorldVec2::new(2.0, 0.0),
            WorldVec2::new(4.6, 0.0),
            WorldVec2::new(2.0, 0.7),
        ] {
            let direct_hit = direct.projectile_sweep_hit_fraction(start, end, 0.5, point);
            let rapier_hit = rapier.projectile_sweep_hit_fraction(start, end, 0.5, point);
            match (direct_hit, rapier_hit) {
                (Some(direct_hit), Some(rapier_hit)) => {
                    assert!((direct_hit - rapier_hit).abs() <= 0.0001);
                }
                (None, None) => {}
                other => panic!("sweep mismatch: {other:?}"),
            }
        }
    }

    #[test]
    fn moving_circle_sweep_detects_crossing_paths_between_window_endpoints() {
        let hit_fraction = moving_circle_sweep_hit_fraction(
            WorldVec2::new(0.0, 0.0),
            WorldVec2::new(10.0, 0.0),
            0.25,
            WorldVec2::new(5.0, 2.0),
            WorldVec2::new(5.0, -2.0),
        )
        .expect("projectile and target paths should cross");

        assert!((hit_fraction - 0.5).abs() <= 0.025);
    }

    #[test]
    fn moving_circle_sweep_returns_none_when_paths_do_not_overlap() {
        assert_eq!(
            moving_circle_sweep_hit_fraction(
                WorldVec2::new(0.0, 0.0),
                WorldVec2::new(10.0, 0.0),
                0.25,
                WorldVec2::new(5.0, 2.0),
                WorldVec2::new(5.0, 1.0),
            ),
            None
        );
    }

    #[test]
    fn moving_circle_sweep_returns_zero_when_starting_overlapped() {
        assert_eq!(
            moving_circle_sweep_hit_fraction(
                WorldVec2::new(0.0, 0.0),
                WorldVec2::new(10.0, 0.0),
                0.25,
                WorldVec2::new(0.1, 0.0),
                WorldVec2::new(5.0, 0.0),
            ),
            Some(0.0)
        );
    }
}
