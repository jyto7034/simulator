use super::movement::{ContinuousPosition, HALF_TILE_UNITS};

pub const DEFAULT_UNIT_HITBOX_RADIUS_UNITS: i64 = (HALF_TILE_UNITS as i64) / 2;

pub fn distance_sq_units(a: ContinuousPosition, b: ContinuousPosition) -> u128 {
    let dx = a.x_units.saturating_sub(b.x_units) as i128;
    let dy = a.y_units.saturating_sub(b.y_units) as i128;
    ((dx * dx) as u128).saturating_add((dy * dy) as u128)
}

fn sqrt_floor_u128(value: u128) -> u128 {
    if value <= 1 {
        return value;
    }

    let mut low = 0_u128;
    let mut high = value;
    while low + 1 < high {
        let mid = low + (high - low) / 2;
        let mid_sq = mid.saturating_mul(mid);
        if mid_sq <= value {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
}

pub fn euclidean_distance_units_ceil(a: ContinuousPosition, b: ContinuousPosition) -> u64 {
    let distance_sq = distance_sq_units(a, b);
    if distance_sq == 0 {
        return 0;
    }

    let floor = sqrt_floor_u128(distance_sq);
    let ceil = if floor.saturating_mul(floor) == distance_sq {
        floor
    } else {
        floor.saturating_add(1)
    };
    ceil.min(u64::MAX as u128) as u64
}

pub fn circle_contains_point(
    center: ContinuousPosition,
    radius_units: i64,
    point: ContinuousPosition,
) -> bool {
    if radius_units <= 0 {
        return center == point;
    }

    let radius_sq = (radius_units as i128).saturating_mul(radius_units as i128) as u128;
    distance_sq_units(center, point) <= radius_sq
}

pub fn box_contains_point_centered(
    center: ContinuousPosition,
    width_units: i64,
    height_units: i64,
    expansion_units: i64,
    point: ContinuousPosition,
) -> bool {
    if width_units <= 0 || height_units <= 0 {
        return point == center;
    }

    let half_width = i128::from(width_units) / 2 + i128::from(expansion_units.max(0));
    let half_height = i128::from(height_units) / 2 + i128::from(expansion_units.max(0));
    let dx = (i128::from(point.x_units) - i128::from(center.x_units)).abs();
    let dy = (i128::from(point.y_units) - i128::from(center.y_units)).abs();
    dx <= half_width && dy <= half_height
}

fn signed_parallel_within_expanded_segment(
    parallel: i128,
    dir_len_sq: i128,
    backward_units: i128,
    forward_units: i128,
) -> bool {
    let parallel_sq = parallel.saturating_mul(parallel);
    let limit_units = if parallel < 0 {
        backward_units
    } else {
        forward_units
    };
    let max_parallel_sq = limit_units
        .saturating_mul(limit_units)
        .saturating_mul(dir_len_sq);
    parallel_sq <= max_parallel_sq
}

pub fn line_contains_point_from_origin(
    origin: ContinuousPosition,
    direction_hint: ContinuousPosition,
    length_units: i64,
    expansion_units: i64,
    point: ContinuousPosition,
) -> bool {
    if length_units <= 0 {
        return point == origin;
    }

    let dir_x = i128::from(direction_hint.x_units) - i128::from(origin.x_units);
    let dir_y = i128::from(direction_hint.y_units) - i128::from(origin.y_units);
    let dir_len_sq = dir_x.saturating_mul(dir_x) + dir_y.saturating_mul(dir_y);
    if dir_len_sq <= 0 {
        return false;
    }

    let rel_x = i128::from(point.x_units) - i128::from(origin.x_units);
    let rel_y = i128::from(point.y_units) - i128::from(origin.y_units);

    let parallel = rel_x
        .saturating_mul(dir_x)
        .saturating_add(rel_y.saturating_mul(dir_y));
    let expansion = i128::from(expansion_units.max(0));
    let max_parallel_distance = i128::from(length_units.max(0)).saturating_add(expansion);
    if !signed_parallel_within_expanded_segment(
        parallel,
        dir_len_sq,
        expansion,
        max_parallel_distance,
    ) {
        return false;
    }

    let perp = rel_x
        .saturating_mul(-dir_y)
        .saturating_add(rel_y.saturating_mul(dir_x));
    let expanded_radius = expansion;
    let max_perp_sq = expanded_radius
        .saturating_mul(expanded_radius)
        .saturating_mul(dir_len_sq);

    perp.saturating_mul(perp) <= max_perp_sq
}

pub fn rectangle_contains_point_from_origin(
    origin: ContinuousPosition,
    direction_hint: ContinuousPosition,
    width_units: i64,
    length_units: i64,
    expansion_units: i64,
    point: ContinuousPosition,
) -> bool {
    if width_units <= 0 || length_units <= 0 {
        return point == origin;
    }

    let dir_x = i128::from(direction_hint.x_units) - i128::from(origin.x_units);
    let dir_y = i128::from(direction_hint.y_units) - i128::from(origin.y_units);
    let dir_len_sq = dir_x.saturating_mul(dir_x) + dir_y.saturating_mul(dir_y);
    if dir_len_sq <= 0 {
        return false;
    }

    let rel_x = i128::from(point.x_units) - i128::from(origin.x_units);
    let rel_y = i128::from(point.y_units) - i128::from(origin.y_units);

    let parallel = rel_x
        .saturating_mul(dir_x)
        .saturating_add(rel_y.saturating_mul(dir_y));
    let half_width = i128::from(width_units) / 2;
    let length = i128::from(length_units);
    let expansion = i128::from(expansion_units.max(0));
    let max_parallel_distance = length.saturating_add(expansion);
    if !signed_parallel_within_expanded_segment(
        parallel,
        dir_len_sq,
        expansion,
        max_parallel_distance,
    ) {
        return false;
    }

    let perp = rel_x
        .saturating_mul(-dir_y)
        .saturating_add(rel_y.saturating_mul(dir_x));
    let expanded_half_width = half_width.saturating_add(expansion);
    let perp_sq = perp.saturating_mul(perp);
    let max_perp_sq = expanded_half_width
        .saturating_mul(expanded_half_width)
        .saturating_mul(dir_len_sq);

    perp_sq <= max_perp_sq
}

pub fn cone_contains_point_from_origin(
    origin: ContinuousPosition,
    direction_hint: ContinuousPosition,
    angle_degrees: u16,
    length_units: i64,
    expansion_units: i64,
    point: ContinuousPosition,
) -> bool {
    if angle_degrees == 0 || length_units <= 0 {
        return point == origin;
    }

    let dir_x = (direction_hint.x_units - origin.x_units) as f64;
    let dir_y = (direction_hint.y_units - origin.y_units) as f64;
    let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt();
    if dir_len <= f64::EPSILON {
        return false;
    }

    let rel_x = (point.x_units - origin.x_units) as f64;
    let rel_y = (point.y_units - origin.y_units) as f64;
    let rel_len = (rel_x * rel_x + rel_y * rel_y).sqrt();
    if rel_len <= f64::EPSILON {
        return true;
    }

    let max_len = (length_units.max(0) + expansion_units.max(0)) as f64;
    if rel_len > max_len {
        return false;
    }

    let dot = rel_x * dir_x + rel_y * dir_y;
    if dot < 0.0 {
        return false;
    }

    let cos_theta = dot / (rel_len * dir_len);
    let half_angle = (f64::from(angle_degrees) / 2.0).to_radians();
    cos_theta >= half_angle.cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclidean_distance_units_ceil_uses_true_distance() {
        let a = ContinuousPosition::new(0, 0);
        let b = ContinuousPosition::new(3, 4);
        assert_eq!(euclidean_distance_units_ceil(a, b), 5);
    }

    #[test]
    fn circle_contains_point_respects_radius() {
        let center = ContinuousPosition::new(10, 10);
        assert!(circle_contains_point(
            center,
            5,
            ContinuousPosition::new(13, 14)
        ));
        assert!(!circle_contains_point(
            center,
            4,
            ContinuousPosition::new(13, 14)
        ));
    }

    #[test]
    fn rectangle_contains_point_from_origin_respects_axis_and_width() {
        let origin = ContinuousPosition::new(0, 0);
        let anchor = ContinuousPosition::new(10, 0);
        assert!(rectangle_contains_point_from_origin(
            origin,
            anchor,
            4,
            10,
            0,
            ContinuousPosition::new(8, 1)
        ));
        assert!(!rectangle_contains_point_from_origin(
            origin,
            anchor,
            4,
            10,
            0,
            ContinuousPosition::new(8, 5)
        ));
        assert!(!rectangle_contains_point_from_origin(
            origin,
            anchor,
            4,
            10,
            0,
            ContinuousPosition::new(12, 0)
        ));
    }

    #[test]
    fn rectangle_contains_point_from_origin_scales_backward_expansion_by_direction_length() {
        let origin = ContinuousPosition::new(0, 0);
        let anchor = ContinuousPosition::new(1_000_000, 0);

        assert!(rectangle_contains_point_from_origin(
            origin,
            anchor,
            100_000,
            1_000_000,
            250_000,
            ContinuousPosition::new(-250_000, 0)
        ));
        assert!(!rectangle_contains_point_from_origin(
            origin,
            anchor,
            100_000,
            1_000_000,
            250_000,
            ContinuousPosition::new(-250_001, 0)
        ));
    }

    #[test]
    fn box_contains_point_centered_respects_width_and_height() {
        let center = ContinuousPosition::new(0, 0);
        assert!(box_contains_point_centered(
            center,
            10,
            8,
            0,
            ContinuousPosition::new(5, 4)
        ));
        assert!(!box_contains_point_centered(
            center,
            10,
            8,
            0,
            ContinuousPosition::new(6, 0)
        ));
        assert!(!box_contains_point_centered(
            center,
            10,
            8,
            0,
            ContinuousPosition::new(0, 5)
        ));
    }

    #[test]
    fn line_contains_point_from_origin_respects_segment_and_radius() {
        let origin = ContinuousPosition::new(0, 0);
        let anchor = ContinuousPosition::new(10, 0);
        assert!(line_contains_point_from_origin(
            origin,
            anchor,
            10,
            1,
            ContinuousPosition::new(8, 1)
        ));
        assert!(!line_contains_point_from_origin(
            origin,
            anchor,
            10,
            1,
            ContinuousPosition::new(8, 2)
        ));
        assert!(!line_contains_point_from_origin(
            origin,
            anchor,
            10,
            1,
            ContinuousPosition::new(12, 0)
        ));
    }

    #[test]
    fn line_contains_point_from_origin_scales_backward_expansion_by_direction_length() {
        let origin = ContinuousPosition::new(0, 0);
        let anchor = ContinuousPosition::new(1_000_000, 0);

        assert!(line_contains_point_from_origin(
            origin,
            anchor,
            1_000_000,
            250_000,
            ContinuousPosition::new(-250_000, 0)
        ));
        assert!(!line_contains_point_from_origin(
            origin,
            anchor,
            1_000_000,
            250_000,
            ContinuousPosition::new(-250_001, 0)
        ));
    }

    #[test]
    fn cone_contains_point_from_origin_respects_angle_and_length() {
        let origin = ContinuousPosition::new(0, 0);
        let anchor = ContinuousPosition::new(10, 0);

        assert!(cone_contains_point_from_origin(
            origin,
            anchor,
            60,
            10,
            0,
            ContinuousPosition::new(8, 2)
        ));
        assert!(!cone_contains_point_from_origin(
            origin,
            anchor,
            60,
            10,
            0,
            ContinuousPosition::new(5, 6)
        ));
        assert!(!cone_contains_point_from_origin(
            origin,
            anchor,
            60,
            10,
            0,
            ContinuousPosition::new(12, 0)
        ));
    }
}
