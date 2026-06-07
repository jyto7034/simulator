/// Shared AD/AP balance policy used by damage feedback, combat preview, and
/// live-data validation. Keep these values small and explicit: this module is
/// not an auto-balance solver.

pub const HIGH_DEFENSE_WARNING_THRESHOLD: i32 = 50;
pub const HIGH_MAGIC_RESIST_WARNING_THRESHOLD: i32 = 50;
pub const FAST_BREAKTHROUGH_WARNING_SPEED_THRESHOLD: u32 = 3000;
pub const MITIGATED_DAMAGE_FEEDBACK_PERCENT_THRESHOLD: u32 = 60;

pub fn is_high_defense(defense: i32) -> bool {
    defense >= HIGH_DEFENSE_WARNING_THRESHOLD
}

pub fn is_high_magic_resist(magic_resist: i32) -> bool {
    magic_resist >= HIGH_MAGIC_RESIST_WARNING_THRESHOLD
}

pub fn is_fast_breakthrough_speed(speed_units_per_ms: u32) -> bool {
    speed_units_per_ms >= FAST_BREAKTHROUGH_WARNING_SPEED_THRESHOLD
}

pub fn is_damage_mitigated_for_feedback(raw_damage: u32, final_damage: u32) -> bool {
    raw_damage > 0
        && final_damage > 0
        && u128::from(final_damage).saturating_mul(100)
            <= u128::from(raw_damage)
                .saturating_mul(u128::from(MITIGATED_DAMAGE_FEEDBACK_PERCENT_THRESHOLD))
}
