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
        assert!(circle_contains_point(center, 5, ContinuousPosition::new(13, 14)));
        assert!(!circle_contains_point(center, 4, ContinuousPosition::new(13, 14)));
    }
}
