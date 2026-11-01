use super::{Record, SustainedState};
use crate::game::{
    ability::sustained::TargetStateProfile,
    battle::ids::UnitInstanceId,
    stats::{StatId, StatModifier, StatModifierKind},
};

impl Record {
    pub(super) fn ends_on_source_exit(&self) -> bool {
        if let Some(state) = &self.state { return state.program.end_on_source_exit; }
        !self.independent_shield && self.hit_status.is_none() && self.reactive_heal.is_none()
            && self
                .target_state
                .as_ref()
                .is_none_or(|p| p.end_on_source_exit)
    }
}
impl SustainedState {
    pub(super) fn composition_work(&self) -> u64 {
        let n = self.records.len() as u64;
        let states = self
            .records
            .values()
            .filter(|r| r.target_state.is_some())
            .count() as u64;
        n.saturating_mul(states.saturating_mul(3).saturating_add(1))
            .saturating_add(1)
    }
    // Each property selects one strongest contribution per definition. The
    // ordinal breaks ties without deleting weaker records or their deadlines.
    fn state_winner(
        &self,
        id: u64,
        record: &Record,
        now: Option<u64>,
        value: impl Fn(&TargetStateProfile) -> u32,
    ) -> bool {
        let Some(profile) = record.target_state.as_ref() else {
            return false;
        };
        let amount = value(profile);
        amount > 0
            && now.is_none_or(|now| now < record.expires_at)
            && !self.records.iter().any(|(other_id, other)| {
                other.target == record.target
                    && now.is_none_or(|now| now < other.expires_at)
                    && other.target_state.as_ref().is_some_and(|p| {
                        p.definition == profile.definition && (value(p), *other_id) > (amount, id)
                    })
            })
    }
    pub(super) fn target_state_modifiers(
        &self,
        target: UnitInstanceId,
    ) -> impl Iterator<Item = StatModifier> + '_ {
        self.records
            .iter()
            .filter(move |(_, r)| r.target == target)
            .flat_map(move |(id, r)| {
                let attack = self.state_winner(*id, r, None, |p| p.attack_percent);
                let defense = self.state_winner(*id, r, None, |p| p.defense_percent);
                [
                    attack.then(|| StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Percent,
                        value: r.target_state.as_ref().unwrap().attack_percent as i32,
                    }),
                    defense.then(|| StatModifier {
                        stat: StatId::Defense,
                        kind: StatModifierKind::Percent,
                        value: r.target_state.as_ref().unwrap().defense_percent as i32,
                    }),
                ]
                .into_iter()
                .flatten()
            })
    }
    pub(super) fn target_state_speed(&self, target: UnitInstanceId) -> u32 {
        self.records
            .iter()
            .filter(|(id, r)| {
                r.target == target && self.state_winner(**id, r, None, |p| p.attack_speed_points)
            })
            .fold(0u32, |sum, (_, r)| {
                sum.saturating_add(r.target_state.as_ref().unwrap().attack_speed_points)
            })
    }
}

#[cfg(test)]
mod tests;

impl super::BattleCore {
    pub(in crate::game::battle::core) fn target_state_action_locked(
        &self, target: UnitInstanceId, now: Option<u64>, basic: bool,
    ) -> bool {
        if crate::authoring::control::charge(self.sustained.records.len() as u64, "target_state_action_lock").is_err() { return true; }
        self.sustained.records.values().any(|r| r.target == target && now.is_none_or(|n| n < r.expires_at)
            && r.target_state.as_ref().is_some_and(|p| if basic { p.basic_attack_locked } else { p.active_skill_locked }))
    }
    // Healing may execute inside an Expired handler while that handler's source
    // delivery is still live. Query deadlines without recursively reconciling
    // lifecycle events, including the stronger contributions used for fallback.
    pub(in crate::game::battle::core) fn target_state_received_heal_percent(&self, target: UnitInstanceId, now: u64) -> Result<u32, crate::game::behavior::GameError> {
        crate::authoring::control::charge(self.sustained.composition_work(), "target_state_received_heal")?;
        Ok(self.sustained.records.iter().filter(|(id, r)| r.target == target && self.sustained.state_winner(**id, r, Some(now), |p| p.received_heal_percent))
            .fold(0u32, |sum, (_, r)| sum.saturating_add(r.target_state.as_ref().unwrap().received_heal_percent)))
    }
}
