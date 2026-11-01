use super::BattleCore;
use crate::game::{battle::{damage::DamageSource, types::BattleVerdict}, data::pve_data::PveBonusCondition, reward::RewardEffect};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PerformanceBonusState {
    pub primary: Option<uuid::Uuid>,
    pub largest_loss: u32,
    pub target_max_health: u32,
}
crate::authoring::clone_shape::struct_shape!(PerformanceBonusState { primary, largest_loss, target_max_health });
impl PerformanceBonusState {
    pub fn observe(&mut self, primary: bool, source: DamageSource, actual_loss: u32, max_health: u32) {
        if !primary || !matches!(source, DamageSource::BasicAttack | DamageSource::Ability) || max_health == 0 { return; }
        // Compare exact fractions; never sum separate strikes/ticks or count absorbed/overkill HP.
        if u64::from(actual_loss) * u64::from(self.target_max_health)
            > u64::from(self.largest_loss) * u64::from(max_health) || self.target_max_health == 0 {
            self.largest_loss = actual_loss;
            self.target_max_health = max_health;
        }
    }
}
impl BattleCore {
    pub(crate) fn configure_performance_bonus(&mut self, primary_id: Option<&str>) {
        self.performance_bonus = PerformanceBonusState {
            primary: primary_id.and_then(|id| self.game_data.abnormality_data.get_by_id(id)).map(|a|a.uuid),
            ..Default::default()
        };
    }
    pub(crate) fn performance_bonus_effects(&self, encounter_id: &str, verdict: BattleVerdict, duration_ms: u64) -> Vec<RewardEffect> {
        if verdict != BattleVerdict::Victory { return Vec::new(); }
        let Some(encounter) = self.game_data.pve_data.get_by_id(encounter_id) else { return Vec::new(); };
        encounter.bonus_objectives.iter().filter(|objective| match objective.condition {
            PveBonusCondition::ClearWithin { time_ms } => duration_ms <= time_ms,
            PveBonusCondition::DecisiveDamage { max_health_percent } => self.performance_bonus.target_max_health > 0
                && u64::from(self.performance_bonus.largest_loss) * 100
                    >= u64::from(self.performance_bonus.target_max_health) * u64::from(max_health_percent),
        }).flat_map(|objective| objective.effects.iter().cloned()).collect()
    }
}
