use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::game::{
    boss_omen::BossOmenRunState,
    data::{
        pve_data::{PveEncounterClass, PveEncounterDatabase, PveEncounterRewardSource},
        skill_fragment_data::SkillFragmentId,
    },
    determinism,
    revival::RevivalRunState,
    reward::RewardEffect,
};

const NORMAL_BOSS_CYCLE_NAMESPACE: u64 = 0x4E42_4F53_535F_4359;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EncounterCycleState {
    pub cycle_index: u64,
    pub remaining_source_ids: Vec<String>,
    pub assigned_source_ids: Vec<String>,
}

impl EncounterCycleState {
    pub fn prepare_if_exhausted(
        &mut self,
        member_source_ids: impl IntoIterator<Item = String>,
        run_seed: u64,
        namespace: u64,
    ) {
        if !self.remaining_source_ids.is_empty() || !self.assigned_source_ids.is_empty() {
            return;
        }
        let mut members = member_source_ids.into_iter().collect::<Vec<_>>();
        members.sort();
        members.dedup();
        let cycle_seed = determinism::seed_with_namespace(
            run_seed ^ self.cycle_index,
            namespace ^ self.cycle_index.rotate_left(17),
        );
        members.sort_by_key(|source_id| {
            determinism::seed_with_namespace(cycle_seed ^ stable_string_hash(source_id), namespace)
        });
        self.remaining_source_ids = members;
        self.cycle_index = self.cycle_index.saturating_add(1);
    }

    pub fn reserve_next(&mut self, excluded_source_ids: &HashSet<String>) -> Option<String> {
        let index = self
            .remaining_source_ids
            .iter()
            .position(|source_id| !excluded_source_ids.contains(source_id))?;
        let source_id = self.remaining_source_ids.remove(index);
        self.assigned_source_ids.push(source_id.clone());
        Some(source_id)
    }

    pub fn consume_assigned(&mut self, source_id: &str) -> bool {
        let Some(index) = self
            .assigned_source_ids
            .iter()
            .position(|assigned| assigned == source_id)
        else {
            return false;
        };
        self.assigned_source_ids.remove(index);
        true
    }

    pub fn release_assigned(&mut self, source_id: &str) -> bool {
        if !self.consume_assigned(source_id) {
            return false;
        }
        if !self.remaining_source_ids.iter().any(|id| id == source_id) {
            self.remaining_source_ids.push(source_id.to_string());
        }
        true
    }

    pub fn release_all_assigned(&mut self) {
        let assigned = std::mem::take(&mut self.assigned_source_ids);
        for source_id in assigned {
            if !self.remaining_source_ids.contains(&source_id) {
                self.remaining_source_ids.push(source_id);
            }
        }
    }

    pub fn add_current_cycle_member(&mut self, source_id: &str) {
        if self.remaining_source_ids.iter().any(|id| id == source_id)
            || self.assigned_source_ids.iter().any(|id| id == source_id)
        {
            return;
        }
        self.remaining_source_ids.push(source_id.to_string());
    }

    pub fn validate_members(
        &self,
        authored_member_source_ids: &HashSet<String>,
        label: &str,
    ) -> Result<(), String> {
        let mut seen = HashSet::new();
        for source_id in self
            .remaining_source_ids
            .iter()
            .chain(self.assigned_source_ids.iter())
        {
            if !authored_member_source_ids.contains(source_id) {
                return Err(format!(
                    "{label} cycle references unknown member source '{source_id}'"
                ));
            }
            if !seen.insert(source_id.as_str()) {
                return Err(format!(
                    "{label} cycle contains duplicate member source '{source_id}'"
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EncounterProgressionState {
    pub completed_reward_source_ids: BTreeSet<String>,
    pub normal_boss_cycle: EncounterCycleState,
    pub revival: RevivalRunState,
    pub boss_omen: BossOmenRunState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterRewardPlan {
    pub reward_source_id: String,
    pub fragment_id: Option<SkillFragmentId>,
    pub first_completion: bool,
    pub effects: Vec<RewardEffect>,
}

impl EncounterProgressionState {
    fn is_first_kind_completion(
        &self,
        data: &PveEncounterDatabase,
        source: &PveEncounterRewardSource,
    ) -> Result<bool, String> {
        let kind = data
            .get_by_id(&source.encounter_id)
            .and_then(|e| e.primary_abnormality_id())
            .ok_or_else(|| {
                format!(
                    "reward source '{}' has no abnormality kind",
                    source.reward_source_id
                )
            })?;
        Ok(!self.completed_reward_source_ids.iter().any(|id| {
            data.reward_sources
                .iter()
                .find(|s| &s.reward_source_id == id)
                .and_then(|s| data.get_by_id(&s.encounter_id))
                .and_then(|e| e.primary_abnormality_id())
                == Some(kind)
        }))
    }

    pub fn plan_encounter_reward(
        &self,
        data: &PveEncounterDatabase,
        encounter_id: &str,
    ) -> Result<EncounterRewardPlan, String> {
        let source = data
            .get_reward_source_by_encounter_id(encounter_id)
            .ok_or_else(|| format!("encounter '{encounter_id}' has no authored reward source"))?;
        let first_completion = self.is_first_kind_completion(data, source)?;
        let effects = if first_completion {
            source.fragment_id.iter().cloned().map(|fragment_id| RewardEffect::GrantSkillFragment {
                fragment_id,
            })
            .chain(
                source
                    .first_equipment_ids
                    .iter()
                    .cloned()
                    .map(|equipment_id| RewardEffect::GrantEquipment { equipment_id }),
            )
            .collect()
        } else {
            source.repeat_rewards.clone()
        };
        Ok(EncounterRewardPlan {
            reward_source_id: source.reward_source_id.clone(),
            fragment_id: source.fragment_id.clone(),
            first_completion,
            effects,
        })
    }

    pub fn commit_reward_plan(
        &mut self,
        data: &PveEncounterDatabase,
        plan: &EncounterRewardPlan,
    ) -> Result<(), String> {
        let source = data
            .reward_sources
            .iter()
            .find(|s| s.reward_source_id == plan.reward_source_id)
            .ok_or_else(|| format!("unknown reward source '{}'", plan.reward_source_id))?;
        if self.is_first_kind_completion(data, source)? != plan.first_completion {
            return Err(format!(
                "reward source '{}' changed first-kind branch before commit",
                plan.reward_source_id
            ));
        }
        if plan.first_completion {
            self.completed_reward_source_ids
                .insert(plan.reward_source_id.clone());
        }
        Ok(())
    }

    pub fn prepare_normal_boss_cycle(&mut self, data: &PveEncounterDatabase, seed: u64) {
        self.normal_boss_cycle.prepare_if_exhausted(
            source_ids_for_class(data, PveEncounterClass::NormalBoss),
            seed,
            NORMAL_BOSS_CYCLE_NAMESPACE,
        );
    }
    pub fn reserve_normal_boss(
        &mut self,
        data: &PveEncounterDatabase,
        seed: u64,
        excluded: &HashSet<String>,
    ) -> Option<String> {
        self.prepare_normal_boss_cycle(data, seed);
        self.normal_boss_cycle.reserve_next(excluded)
    }
    pub fn handle_gate_advance(&mut self) {
        self.revival.leave_floor();
        self.normal_boss_cycle.release_all_assigned();
        self.boss_omen.handle_gate_advance();
    }
    pub fn validate_checkpoint_state(&self, data: &PveEncounterDatabase) -> Result<(), String> {
        let all_source_ids = data
            .reward_sources
            .iter()
            .map(|s| s.reward_source_id.as_str())
            .collect::<HashSet<_>>();
        for id in &self.completed_reward_source_ids {
            if !all_source_ids.contains(id.as_str()) {
                return Err(format!(
                    "completed reward ledger references unknown source '{id}'"
                ));
            }
        }
        self.normal_boss_cycle.validate_members(
            &source_ids_for_class(data, PveEncounterClass::NormalBoss).collect(),
            "NormalBoss",
        )?;
        for id in self
            .revival
            .eggs
            .keys()
            .chain(self.revival.nodes.values().map(|n| &n.reward_source_id))
        {
            if !self.completed_reward_source_ids.contains(id)
                || data
                    .reward_sources
                    .iter()
                    .find(|s| &s.reward_source_id == id)
                    .is_none_or(|s| s.revival.is_none())
            {
                return Err(format!(
                    "revival references unavailable completed source '{id}'"
                ));
            }
        }
        Ok(())
    }
}

fn source_ids_for_class(
    data: &PveEncounterDatabase,
    class: PveEncounterClass,
) -> impl Iterator<Item = String> + '_ {
    data.reward_sources
        .iter()
        .filter(move |s| {
            data.get_by_id(&s.encounter_id)
                .is_some_and(|e| e.encounter_class == class)
        })
        .map(|s| s.reward_source_id.clone())
}
fn stable_string_hash(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}
crate::authoring::clone_shape::struct_shape!(EncounterCycleState {
    cycle_index,
    remaining_source_ids,
    assigned_source_ids
});
crate::authoring::clone_shape::struct_shape!(EncounterProgressionState {
    completed_reward_source_ids,
    normal_boss_cycle,
    revival,
    boss_omen
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn u17_first_kind_ledger_is_independent_of_holdings_and_new_expedition_resets_it() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
        let source = data
            .pve_data
            .reward_sources
            .iter()
            .find(|s| s.reward_source_id == "elite_censored")
            .unwrap();
        let mut ledger = EncounterProgressionState::default();
        let first = ledger
            .plan_encounter_reward(&data.pve_data, &source.encounter_id)
            .unwrap();
        assert!(first.first_completion);
        assert!(first
            .effects
            .iter()
            .any(|e| matches!(e, RewardEffect::GrantSkillFragment { .. })));
        assert!(first
            .effects
            .iter()
            .any(|e| matches!(e, RewardEffect::GrantEquipment { .. })));
        ledger.commit_reward_plan(&data.pve_data, &first).unwrap();
        assert!(ledger.commit_reward_plan(&data.pve_data, &first).is_err());
        let wire = serde_json::to_vec(&ledger).unwrap();
        let restored: EncounterProgressionState = serde_json::from_slice(&wire).unwrap();
        let repeat = restored
            .plan_encounter_reward(&data.pve_data, &source.encounter_id)
            .unwrap();
        assert!(!repeat.first_completion);
        assert_eq!(repeat.effects, source.repeat_rewards);
        assert!(restored.revival.eggs.is_empty());
        assert!(
            EncounterProgressionState::default()
                .plan_encounter_reward(&data.pve_data, &source.encounter_id)
                .unwrap()
                .first_completion
        );
    }

    #[test]
    fn u17_boss_cycle_reservation_release_and_consumption_remain_deterministic() {
        let members = || ["a", "b", "c"].map(str::to_owned);
        let mut cycle = EncounterCycleState::default();
        cycle.prepare_if_exhausted(members(), 17, 8);
        let first = cycle.reserve_next(&HashSet::new()).unwrap();
        cycle.prepare_if_exhausted(members(), 17, 8);
        assert_eq!(cycle.cycle_index, 1);
        assert!(cycle.release_assigned(&first));
        let mut seen = HashSet::new();
        while let Some(id) = cycle.reserve_next(&HashSet::new()) {
            assert!(seen.insert(id.clone()));
            assert!(cycle.consume_assigned(&id));
        }
        assert_eq!(seen.len(), 3);
        cycle.prepare_if_exhausted(members(), 17, 8);
        assert_eq!(cycle.cycle_index, 2);
    }
}

#[cfg(test)]
mod optional_fragment_tests {
    use super::*;

    #[test]
    fn optional_fragment_empty_first_commits_once_and_repeat_survives_restore() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
        let source = data.pve_data.get_reward_source_by_encounter_id("suppress_big_bad_wolf").unwrap();
        assert_eq!(source.fragment_id, None);
        let mut ledger = EncounterProgressionState::default();
        let before = ledger.clone();
        let first = ledger.plan_encounter_reward(&data.pve_data, &source.encounter_id).unwrap();
        assert_eq!(ledger, before, "planning cannot consume the first completion");
        assert!(first.first_completion);
        assert!(first.effects.is_empty(), "no substitute fragment or compensation");
        ledger.commit_reward_plan(&data.pve_data, &first).unwrap();
        let committed = ledger.clone();
        assert!(ledger.commit_reward_plan(&data.pve_data, &first).is_err());
        assert_eq!(ledger, committed, "duplicate commit is unchanged");
        let mut restored: EncounterProgressionState = serde_json::from_slice(&serde_json::to_vec(&ledger).unwrap()).unwrap();
        restored.validate_checkpoint_state(&data.pve_data).unwrap();
        let repeat = restored.plan_encounter_reward(&data.pve_data, &source.encounter_id).unwrap();
        assert!(!repeat.first_completion);
        assert_eq!(repeat.effects, vec![RewardEffect::GrantEquipmentDust { amount: 10 }]);
        restored.commit_reward_plan(&data.pve_data, &repeat).unwrap();
        assert_eq!(restored, committed);
        assert!(EncounterProgressionState::default().plan_encounter_reward(&data.pve_data, &source.encounter_id).unwrap().first_completion);
    }
}

#[cfg(test)]
mod optional_fragment_equipment_tests {
    use super::*;
    #[test]
    fn optional_fragment_absence_preserves_authored_first_equipment() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
        let mut pve = (*data.pve_data).clone();
        let source = pve.reward_sources.iter_mut().find(|s| s.reward_source_id == "normal_boss_big_bad_wolf").unwrap();
        source.first_equipment_ids = vec!["placeholder_big_bad_wolf_accessory".into()];
        let encounter = source.encounter_id.clone();
        let first = EncounterProgressionState::default().plan_encounter_reward(&pve, &encounter).unwrap();
        assert_eq!(first.effects, vec![RewardEffect::GrantEquipment { equipment_id: "placeholder_big_bad_wolf_accessory".into() }]);
    }
}
