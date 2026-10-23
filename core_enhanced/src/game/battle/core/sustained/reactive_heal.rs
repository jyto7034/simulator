use super::*;
use crate::game::{
    ability::sustained::{ReactiveHealProfile, ReactiveHealView},
    battle::{
        damage::{BattleCommand, DamageSourceSnapshot},
        runtime_control,
    },
};

#[derive(Debug, Clone)]
pub(super) struct ReactiveHealRecord {
    pub profile: ReactiveHealProfile,
    pub remaining: u32,
    // Observation keys only; these ordinals cannot authorize execution.
    pub seen_actions: Vec<u64>,
}
crate::authoring::clone_shape::struct_shape!(ReactiveHealRecord {
    profile,
    remaining,
    seen_actions
});
impl ReactiveHealRecord {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.profile.definition.capacity().checked_add(
            self.seen_actions
                .capacity()
                .checked_mul(std::mem::size_of::<u64>())?,
        )
    }
    pub(super) fn view(&self) -> ReactiveHealView {
        ReactiveHealView {
            definition: self.profile.definition.clone(),
            initial_charges: self.profile.charges,
            remaining_charges: self.remaining,
            max_health_percent: self.profile.max_health_percent,
            consumed_actions: self.seen_actions.len() as u32,
        }
    }
}

impl BattleCore {
    pub(super) fn replace_reactive_heal(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        definition: &str,
        source: &RuntimeFactSource,
        now: u64,
    ) -> Result<(), GameError> {
        loop {
            control::charge(
                self.sustained.records.len() as u64,
                "reactive_heal_replace_scan",
            )?;
            let old = self
                .sustained
                .records
                .iter()
                .find(|(_, r)| {
                    r.caster == caster
                        && r.target == target
                        && r.reactive_heal
                            .as_ref()
                            .is_some_and(|h| h.profile.definition == definition)
                        && r.source.fragment.as_ref().map(|f| f.copy_uuid)
                            == source.fragment.as_ref().map(|f| f.copy_uuid)
                        && r.source.equipment == source.equipment
                })
                .map(|(id, _)| *id);
            let Some(id) = old else {
                return Ok(());
            };
            self.end_sustained(id, now, SustainedEndReason::Replaced)?;
        }
    }

    pub(in crate::game::battle::core) fn reactive_heal_after_damage(
        &mut self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
        now: u64,
    ) -> Result<(), GameError> {
        runtime_control::poll()?;
        let Some(action) = incoming.fact_source.action.as_ref() else {
            return Ok(());
        };
        let ordinal = action.ordinal().0;
        if !self.normal_player_employee(target)
            || self.units[&target].owner == incoming.source_side
            || self.units[&target].stats.current_health == 0
        {
            return Ok(());
        }
        let mut cursor = 0u64;
        loop {
            control::charge(self.sustained.records.len() as u64, "reactive_heal_scan")?;
            let mut selected = None;
            for (id, record) in self.sustained.records.range((
                std::ops::Bound::Excluded(cursor),
                std::ops::Bound::Unbounded,
            )) {
                let Some(heal) = record.reactive_heal.as_ref() else {
                    continue;
                };
                control::charge(
                    heal.seen_actions.len() as u64 + 1,
                    "reactive_heal_action_lookup",
                )?;
                if record.target == target
                    && record.expires_at > now
                    && heal.remaining > 0
                    && !heal.seen_actions.contains(&ordinal)
                {
                    selected = Some(*id);
                    break;
                }
            }
            let Some(id) = selected else {
                return Ok(());
            };
            cursor = id;
            let record = &self.sustained.records[&id];
            record.source.admit_owner("reactive_heal_source")?;
            let source = record.source.observation();
            let caster = record.caster;
            let heal = record.reactive_heal.as_ref().unwrap();
            let percent = heal.profile.max_health_percent as i32;
            runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<u64>() + std::mem::size_of::<BattleCommand>()),
                "reactive_heal_consume",
            )?;
            self.admit_authoring_runtime_growth(1, "reactive_heal_action")?;
            let heal = self
                .sustained
                .records
                .get_mut(&id)
                .unwrap()
                .reactive_heal
                .as_mut()
                .unwrap();
            // Exact growth keeps the retained allocation admission above accurate.
            heal.seen_actions.reserve_exact(1);
            heal.seen_actions.push(ordinal);
            heal.remaining -= 1;
            let depleted = heal.remaining == 0;
            control::poll("reactive_heal_consumed")?;
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            self.process_commands(
                vec![BattleCommand::ApplyHeal {
                    target_id: target,
                    flat: 0,
                    percent,
                    source_id: Some(caster),
                }],
                now,
            );
            self.runtime_fact_source = prior;
            control::poll("reactive_heal_applied")?;
            runtime_control::poll()?;
            if depleted {
                self.end_sustained(id, now, SustainedEndReason::Depleted)?;
            }
        }
    }
}

#[cfg(test)]
mod tests;
