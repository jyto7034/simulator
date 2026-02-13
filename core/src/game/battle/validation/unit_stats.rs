use crate::game::stats::UnitStats;

pub(super) fn validate_spawn_stats(stats: &UnitStats) -> Option<String> {
    validate_unit_stats(stats, "spawned unit")
}

pub(super) fn validate_unit_stats(stats: &UnitStats, context: &str) -> Option<String> {
    if stats.current_health > stats.max_health {
        return Some(format!(
            "{context} has current_health {} > max_health {}",
            stats.current_health, stats.max_health
        ));
    }
    if stats.attack_interval_ms == 0 {
        return Some(format!("{context} has attack_interval_ms == 0"));
    }
    None
}
