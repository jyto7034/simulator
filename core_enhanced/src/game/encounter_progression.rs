use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::game::{
    boss_omen::BossOmenRunState,
    data::{
        pve_data::{PveEncounterClass, PveEncounterDatabase, PveEncounterRewardSource},
        skill_fragment_data::SkillFragmentId,
    },
    determinism,
    monolith::MonolithRunState,
    reward::RewardEffect,
};

const MASTERED_ELITE_CYCLE_NAMESPACE: u64 = 0x454C_4954_455F_4359;
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
    pub mastered_elite_cycle: EncounterCycleState,
    pub normal_boss_cycle: EncounterCycleState,
    pub monolith: MonolithRunState,
    pub boss_omen: BossOmenRunState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterRewardPlan {
    pub reward_source_id: String,
    pub fragment_id: SkillFragmentId,
    pub first_completion: bool,
    pub effects: Vec<RewardEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasteredEliteSweepRewardBranch {
    ReacquireFragment,
    RepeatReward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MasteredEliteSweepRewardPlan {
    pub reward_source_id: String,
    pub fragment_id: SkillFragmentId,
    pub branch: MasteredEliteSweepRewardBranch,
    pub effects: Vec<RewardEffect>,
}

impl EncounterProgressionState {
    pub fn plan_encounter_reward(
        &self,
        pve_data: &PveEncounterDatabase,
        encounter_id: &str,
    ) -> Result<EncounterRewardPlan, String> {
        let source = pve_data
            .get_reward_source_by_encounter_id(encounter_id)
            .ok_or_else(|| format!("encounter '{encounter_id}' has no authored reward source"))?;
        let first_completion = !self
            .completed_reward_source_ids
            .contains(&source.reward_source_id);
        let effects = if first_completion {
            vec![RewardEffect::GrantSkillFragment {
                fragment_id: source.fragment_id.clone(),
            }]
        } else {
            vec![source.repeat_reward.clone()]
        };
        Ok(EncounterRewardPlan {
            reward_source_id: source.reward_source_id.clone(),
            fragment_id: source.fragment_id.clone(),
            first_completion,
            effects,
        })
    }

    pub fn commit_reward_plan(&mut self, plan: &EncounterRewardPlan) -> Result<(), String> {
        let already_completed = self
            .completed_reward_source_ids
            .contains(&plan.reward_source_id);
        if plan.first_completion == already_completed {
            return Err(format!(
                "reward source '{}' changed completion branch before commit",
                plan.reward_source_id
            ));
        }
        if plan.first_completion {
            self.completed_reward_source_ids
                .insert(plan.reward_source_id.clone());
        }
        Ok(())
    }

    pub fn plan_mastered_elite_sweep_reward(
        &self,
        pve_data: &PveEncounterDatabase,
        encounter_id: &str,
        owned_fragment_count: u32,
    ) -> Result<MasteredEliteSweepRewardPlan, String> {
        let encounter = pve_data
            .get_by_id(encounter_id)
            .ok_or_else(|| format!("unknown sweep encounter '{encounter_id}'"))?;
        if encounter.encounter_class != PveEncounterClass::Elite {
            return Err(format!(
                "encounter '{encounter_id}' is not an Elite sweep source"
            ));
        }
        let abnormality_id = encounter.primary_abnormality_id().ok_or_else(|| {
            format!("Elite sweep encounter '{encounter_id}' has no primary abnormality")
        })?;
        if !self
            .monolith
            .progress(abnormality_id)
            .is_some_and(|progress| progress.victories == 3)
        {
            return Err(format!(
                "Elite sweep encounter '{encounter_id}' is not mastered"
            ));
        }

        let source = pve_data
            .get_reward_source_by_encounter_id(encounter_id)
            .ok_or_else(|| format!("encounter '{encounter_id}' has no authored reward source"))?;
        if !self
            .completed_reward_source_ids
            .contains(&source.reward_source_id)
        {
            return Err(format!(
                "mastered Elite sweep source '{}' is not complete",
                source.reward_source_id
            ));
        }
        if !self
            .mastered_elite_cycle
            .assigned_source_ids
            .iter()
            .any(|assigned| assigned == &source.reward_source_id)
        {
            return Err(format!(
                "mastered Elite sweep source '{}' is not currently assigned",
                source.reward_source_id
            ));
        }

        let (branch, effects) = if owned_fragment_count == 0 {
            (
                MasteredEliteSweepRewardBranch::ReacquireFragment,
                vec![RewardEffect::GrantSkillFragment {
                    fragment_id: source.fragment_id.clone(),
                }],
            )
        } else {
            (
                MasteredEliteSweepRewardBranch::RepeatReward,
                vec![source.repeat_reward.clone()],
            )
        };
        Ok(MasteredEliteSweepRewardPlan {
            reward_source_id: source.reward_source_id.clone(),
            fragment_id: source.fragment_id.clone(),
            branch,
            effects,
        })
    }

    pub fn commit_mastered_elite_sweep_reward(
        &mut self,
        plan: &MasteredEliteSweepRewardPlan,
    ) -> Result<(), String> {
        if !self
            .completed_reward_source_ids
            .contains(&plan.reward_source_id)
        {
            return Err(format!(
                "mastered Elite sweep source '{}' lost completion before commit",
                plan.reward_source_id
            ));
        }
        if !self
            .mastered_elite_cycle
            .consume_assigned(&plan.reward_source_id)
        {
            return Err(format!(
                "mastered Elite sweep source '{}' lost its assignment before commit",
                plan.reward_source_id
            ));
        }
        Ok(())
    }

    pub fn prepare_normal_boss_cycle(&mut self, pve_data: &PveEncounterDatabase, run_seed: u64) {
        self.normal_boss_cycle.prepare_if_exhausted(
            source_ids_for_class(pve_data, PveEncounterClass::NormalBoss),
            run_seed,
            NORMAL_BOSS_CYCLE_NAMESPACE,
        );
    }

    pub fn reserve_normal_boss(
        &mut self,
        pve_data: &PveEncounterDatabase,
        run_seed: u64,
        excluded_source_ids: &HashSet<String>,
    ) -> Option<String> {
        self.prepare_normal_boss_cycle(pve_data, run_seed);
        self.normal_boss_cycle.reserve_next(excluded_source_ids)
    }

    pub fn prepare_mastered_elite_cycle(&mut self, pve_data: &PveEncounterDatabase, run_seed: u64) {
        let mastered = pve_data
            .reward_sources
            .iter()
            .filter(|source| source_class(pve_data, source) == Some(PveEncounterClass::Elite))
            .filter(|source| {
                pve_data
                    .get_by_id(&source.encounter_id)
                    .and_then(|encounter| encounter.primary_abnormality_id())
                    .and_then(|abnormality_id| self.monolith.progress(abnormality_id))
                    .is_some_and(|progress| progress.victories == 3)
            })
            .map(|source| source.reward_source_id.clone())
            .collect::<Vec<_>>();
        self.mastered_elite_cycle.prepare_if_exhausted(
            mastered,
            run_seed,
            MASTERED_ELITE_CYCLE_NAMESPACE,
        );
    }

    pub fn reserve_mastered_elite(
        &mut self,
        pve_data: &PveEncounterDatabase,
        run_seed: u64,
        excluded_source_ids: &HashSet<String>,
    ) -> Option<String> {
        self.prepare_mastered_elite_cycle(pve_data, run_seed);
        self.mastered_elite_cycle.reserve_next(excluded_source_ids)
    }

    pub fn add_mastered_elite(&mut self, reward_source_id: &str) {
        self.mastered_elite_cycle
            .add_current_cycle_member(reward_source_id);
    }

    pub fn handle_gate_advance(&mut self) {
        self.mastered_elite_cycle.release_all_assigned();
        self.normal_boss_cycle.release_all_assigned();
        self.boss_omen.handle_gate_advance();
    }

    pub fn validate_checkpoint_state(&self, pve_data: &PveEncounterDatabase) -> Result<(), String> {
        let all_source_ids = pve_data
            .reward_sources
            .iter()
            .map(|source| source.reward_source_id.as_str())
            .collect::<HashSet<_>>();
        for source_id in &self.completed_reward_source_ids {
            if !all_source_ids.contains(source_id.as_str()) {
                return Err(format!(
                    "completed reward ledger references unknown source '{source_id}'"
                ));
            }
        }
        let elite_source_ids =
            source_ids_for_class(pve_data, PveEncounterClass::Elite).collect::<HashSet<_>>();
        self.mastered_elite_cycle
            .validate_members(&elite_source_ids, "mastered Elite")?;
        let normal_boss_source_ids =
            source_ids_for_class(pve_data, PveEncounterClass::NormalBoss).collect::<HashSet<_>>();
        self.normal_boss_cycle
            .validate_members(&normal_boss_source_ids, "NormalBoss")?;
        Ok(())
    }
}

fn source_ids_for_class(
    pve_data: &PveEncounterDatabase,
    class: PveEncounterClass,
) -> impl Iterator<Item = String> + '_ {
    pve_data
        .reward_sources
        .iter()
        .filter(move |source| source_class(pve_data, source) == Some(class))
        .map(|source| source.reward_source_id.clone())
}

fn source_class(
    pve_data: &PveEncounterDatabase,
    source: &PveEncounterRewardSource,
) -> Option<PveEncounterClass> {
    pve_data
        .get_by_id(&source.encounter_id)
        .map(|encounter| encounter.encounter_class)
}

fn stable_string_hash(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::pve_data::{PveEncounter, PveEncounterRewardSource};
    use crate::game::map::MapNodeId;
    use uuid::Uuid;

    fn encounter(id: &str, class: PveEncounterClass, abnormality_id: &str) -> PveEncounter {
        ron::de::from_str(&format!(
            r#"(
                id: "{id}",
                encounter_class: {class:?},
                primary_abnormality_id: Some("{abnormality_id}"),
                risk_level: ZAYIN,
                waves: [],
            )"#
        ))
        .expect("test encounter should parse")
    }

    fn source(
        id: &str,
        encounter_id: &str,
        fragment_id: &str,
        dust: u32,
    ) -> PveEncounterRewardSource {
        PveEncounterRewardSource {
            reward_source_id: id.to_string(),
            encounter_id: encounter_id.to_string(),
            fragment_id: SkillFragmentId::from(fragment_id),
            repeat_reward: RewardEffect::GrantFragmentDust { amount: dust },
        }
    }

    fn database() -> PveEncounterDatabase {
        PveEncounterDatabase::new_with_reward_sources(
            vec![
                encounter("elite_a", PveEncounterClass::Elite, "abnormality_a"),
                encounter("boss_a", PveEncounterClass::NormalBoss, "abnormality_b"),
                encounter("boss_b", PveEncounterClass::NormalBoss, "abnormality_c"),
            ],
            vec![
                source("source_elite_a", "elite_a", "fragment_a", 5),
                source("source_boss_a", "boss_a", "fragment_b", 10),
                source("source_boss_b", "boss_b", "fragment_c", 10),
            ],
        )
    }

    fn master_elite(state: &mut EncounterProgressionState) {
        state
            .monolith
            .record_natural_victory("abnormality_a")
            .unwrap();
        state.monolith.purchase("abnormality_a", 3).unwrap();
        for value in [101_u128, 102] {
            let node_id = MapNodeId::new(Uuid::from_u128(value));
            state
                .monolith
                .begin_remanifestation(node_id, "abnormality_a", "elite_a")
                .unwrap();
            state
                .monolith
                .resolve_remanifestation_victory(node_id, 4)
                .unwrap();
        }
    }

    #[test]
    fn reward_plan_is_first_then_explicit_repeat_without_implicit_fallback() {
        let data = database();
        let mut state = EncounterProgressionState::default();
        let first = state.plan_encounter_reward(&data, "boss_a").unwrap();
        assert!(first.first_completion);
        assert_eq!(
            first.effects,
            vec![RewardEffect::GrantSkillFragment {
                fragment_id: SkillFragmentId::from("fragment_b")
            }]
        );
        state.commit_reward_plan(&first).unwrap();

        let repeat = state.plan_encounter_reward(&data, "boss_a").unwrap();
        assert!(!repeat.first_completion);
        assert_eq!(
            repeat.effects,
            vec![RewardEffect::GrantFragmentDust { amount: 10 }]
        );
        state.commit_reward_plan(&repeat).unwrap();
        assert_eq!(state.completed_reward_source_ids.len(), 1);
    }

    #[test]
    fn mastered_sweep_exclusively_reacquires_at_zero_and_repeats_when_owned() {
        let data = database();
        let mut state = EncounterProgressionState::default();
        master_elite(&mut state);
        state
            .completed_reward_source_ids
            .insert("source_elite_a".to_string());
        state
            .mastered_elite_cycle
            .assigned_source_ids
            .push("source_elite_a".to_string());

        let reacquire = state
            .plan_mastered_elite_sweep_reward(&data, "elite_a", 0)
            .unwrap();
        assert_eq!(
            reacquire.branch,
            MasteredEliteSweepRewardBranch::ReacquireFragment
        );
        assert_eq!(
            reacquire.effects,
            vec![RewardEffect::GrantSkillFragment {
                fragment_id: SkillFragmentId::from("fragment_a")
            }]
        );

        let repeat = state
            .plan_mastered_elite_sweep_reward(&data, "elite_a", 1)
            .unwrap();
        assert_eq!(repeat.branch, MasteredEliteSweepRewardBranch::RepeatReward);
        assert_eq!(
            repeat.effects,
            vec![RewardEffect::GrantFragmentDust { amount: 5 }]
        );
    }

    #[test]
    fn mastered_sweep_commit_consumes_assignment_without_resetting_progress_or_source() {
        let data = database();
        let mut state = EncounterProgressionState::default();
        master_elite(&mut state);
        state
            .completed_reward_source_ids
            .insert("source_elite_a".to_string());
        state
            .mastered_elite_cycle
            .assigned_source_ids
            .push("source_elite_a".to_string());
        let before_monolith = state.monolith.clone();
        let plan = state
            .plan_mastered_elite_sweep_reward(&data, "elite_a", 0)
            .unwrap();

        state.commit_mastered_elite_sweep_reward(&plan).unwrap();

        assert!(state.completed_reward_source_ids.contains("source_elite_a"));
        assert_eq!(state.monolith, before_monolith);
        assert!(state.mastered_elite_cycle.assigned_source_ids.is_empty());
        assert!(state.commit_mastered_elite_sweep_reward(&plan).is_err());
    }

    #[test]
    fn cycle_distinguishes_assignment_consumption_and_unvisited_release() {
        let data = database();
        let mut state = EncounterProgressionState::default();
        let excluded = HashSet::new();
        let first = state
            .reserve_normal_boss(&data, 42, &excluded)
            .expect("first boss");
        let second = state
            .reserve_normal_boss(&data, 42, &excluded)
            .expect("second boss");
        assert_ne!(first, second);
        assert!(state.normal_boss_cycle.consume_assigned(&first));

        state.handle_gate_advance();
        assert!(state.normal_boss_cycle.assigned_source_ids.is_empty());
        assert_eq!(
            state.normal_boss_cycle.remaining_source_ids,
            vec![second.clone()]
        );
        let returned = state
            .reserve_normal_boss(&data, 42, &excluded)
            .expect("unvisited boss returns");
        assert_eq!(returned, second);
        assert!(state.normal_boss_cycle.consume_assigned(&returned));

        let next_cycle = state
            .reserve_normal_boss(&data, 42, &excluded)
            .expect("complete cycle refills");
        assert!(next_cycle == first || next_cycle == second);
        assert_eq!(state.normal_boss_cycle.cycle_index, 2);
    }

    #[test]
    fn checkpoint_validation_rejects_unknown_duplicate_and_wrong_class_members() {
        let data = database();
        let mut state = EncounterProgressionState::default();
        state
            .normal_boss_cycle
            .remaining_source_ids
            .push("source_boss_a".to_string());
        state
            .normal_boss_cycle
            .assigned_source_ids
            .push("source_boss_a".to_string());
        assert!(state.validate_checkpoint_state(&data).is_err());

        state.normal_boss_cycle.assigned_source_ids.clear();
        state.normal_boss_cycle.remaining_source_ids[0] = "source_elite_a".to_string();
        assert!(state.validate_checkpoint_state(&data).is_err());

        state.normal_boss_cycle.remaining_source_ids[0] = "unknown".to_string();
        assert!(state.validate_checkpoint_state(&data).is_err());
    }
}

crate::authoring::clone_shape::struct_shape!(EncounterProgressionState {
    completed_reward_source_ids,
    mastered_elite_cycle,
    normal_boss_cycle,
    monolith,
    boss_omen
});

crate::authoring::clone_shape::struct_shape!(EncounterCycleState {
    cycle_index,
    remaining_source_ids,
    assigned_source_ids
});
