use super::movement::types::{WorldVec2, DATA_UNITS_PER_WORLD};

pub(in crate::game::battle::core) fn projectile_flight_ms(
    distance_units: u64,
    speed_units_per_ms: u32,
) -> u64 {
    if distance_units == 0 {
        return 0;
    }
    let speed_units_per_ms = speed_units_per_ms as u64;
    if speed_units_per_ms == 0 {
        // Defensive: treat invalid projectile speed as "instant" rather than panicking/overflowing.
        return 0;
    }
    distance_units.saturating_add(speed_units_per_ms.saturating_sub(1)) / speed_units_per_ms
}

pub(in crate::game::battle::core) fn projectile_flight_ms_between_world_points(
    start: WorldVec2,
    aim: WorldVec2,
    speed_units_per_ms: u32,
) -> u64 {
    let distance_units = (start.distance(aim) * DATA_UNITS_PER_WORLD).ceil().max(0.0) as u64;
    projectile_flight_ms(distance_units, speed_units_per_ms)
}
