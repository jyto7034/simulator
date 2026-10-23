use super::{Record, SustainedState};
use crate::game::{
    ability::sustained::TargetStateProfile,
    battle::ids::UnitInstanceId,
    stats::{StatId, StatModifier, StatModifierKind},
};

impl Record {
    pub(super) fn ends_on_source_exit(&self) -> bool {
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
        value: impl Fn(&TargetStateProfile) -> u32,
    ) -> bool {
        let Some(profile) = record.target_state.as_ref() else {
            return false;
        };
        let amount = value(profile);
        amount > 0
            && !self.records.iter().any(|(other_id, other)| {
                other.target == record.target
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
                let attack = self.state_winner(*id, r, |p| p.attack_percent);
                let defense = self.state_winner(*id, r, |p| p.defense_percent);
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
                r.target == target && self.state_winner(**id, r, |p| p.attack_speed_points)
            })
            .fold(0u32, |sum, (_, r)| {
                sum.saturating_add(r.target_state.as_ref().unwrap().attack_speed_points)
            })
    }
}

#[cfg(test)]
mod tests;
