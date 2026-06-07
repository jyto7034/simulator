use super::movement::{
    types::{UnitBody, WorldVec2, DATA_UNITS_PER_WORLD},
    HALF_TILE_UNITS,
};

pub const DEFAULT_UNIT_HITBOX_RADIUS_UNITS: i64 = (HALF_TILE_UNITS as i64) / 2;

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
