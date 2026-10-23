//! Authored continuous defense rounds. Rewards remain pending until success.
use crate::game::{
    behavior::GameError,
    data::pve_data::{PveEncounterDatabase, PveTerminalConditionData},
    reward::RewardEffect,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StressCombatPolicy {
    pub encounter_id: String,
    pub rounds: Vec<StressCombatRound>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StressCombatRound {
    pub wave_ids: Vec<String>,
    pub rewards: Vec<RewardEffect>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressCombatView {
    pub battle_uuid: uuid::Uuid,
    pub current_round: u8,
    pub completed_rounds: u8,
    pub awaiting_choice: bool,
    pub quantity_multiplier: u32,
    pub pending_rewards: Vec<RewardEffect>,
    pub next_rewards: Vec<RewardEffect>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StressCombatEntryView {
    pub total_rounds: u8,
    pub quantity_multiplier: u32,
    pub first_rewards: Vec<RewardEffect>,
}
crate::authoring::clone_shape::struct_shape!(StressCombatRound { wave_ids, rewards });
crate::authoring::clone_shape::struct_shape!(StressCombatPolicy {
    encounter_id,
    rounds
});

impl StressCombatPolicy {
    pub fn validate(&self, database: &PveEncounterDatabase) -> Result<(), String> {
        let fail = || {
            "invalid stress combat policy: expected five nonempty rounds covering a protected DefenseRoute encounter".to_string()
        };
        let encounter = database.get_by_id(&self.encounter_id).ok_or_else(fail)?;
        let Some(crate::game::data::pve_data::PveBattleObjectiveData::ProtectUnit { unit_ref }) =
            encounter
                .tactical_plan
                .as_ref()
                .and_then(|p| p.objective.as_ref())
        else {
            return Err(fail());
        };
        if self.rounds.len() != 5 || encounter.waves.is_empty()
            || encounter.encounter_class != crate::game::data::pve_data::PveEncounterClass::StressCombat
            || encounter.mission_variant != Some(crate::game::combat_preview::CombatMissionVariant::Defense)
            || !encounter.terminal_outcomes.iter().any(|o| o.verdict == crate::game::battle::types::BattleVerdict::Defeat && matches!(&o.condition, PveTerminalConditionData::UnitDefeated { unit_ref: target } if target == unit_ref))
            || encounter.terminal_outcomes.iter().any(|o| o.verdict == crate::game::battle::types::BattleVerdict::Victory && !matches!(o.condition, PveTerminalConditionData::AllRequiredEnemyGroupsDefeated))
            || !encounter.bonus_objectives.is_empty() {
            return Err(fail());
        }
        let mut seen = std::collections::HashSet::new();
        for (index, round) in self.rounds.iter().enumerate() {
            if round.wave_ids.is_empty() {
                return Err(fail());
            }
            for id in &round.wave_ids {
                let wave = encounter
                    .waves
                    .iter()
                    .find(|w| &w.id == id)
                    .ok_or_else(fail)?;
                if !seen.insert(id) || !wave.required_for_victory {
                    return Err(fail());
                }
                // A round cannot wait on a wave gated by a future round.
                use crate::game::data::pve_data::PveWavePrimaryTriggerData as Trigger;
                let dependency = match &wave.activation.primary {
                    Trigger::AtTimeMs { .. } => None,
                    Trigger::GroupCleared { wave_id } => Some(wave_id),
                    Trigger::ActorHealthAtOrBelowPercent { actor_slot_id, .. }
                    | Trigger::ActorDefeated { actor_slot_id }
                    | Trigger::ActorReachedPoint { actor_slot_id, .. } => Some(
                        &encounter
                            .boss_runtime
                            .as_ref()
                            .and_then(|binding| {
                                binding
                                    .actors
                                    .iter()
                                    .find(|actor| &actor.actor_slot_id == actor_slot_id)
                            })
                            .ok_or_else(fail)?
                            .wave_id,
                    ),
                };
                if let Some(wave_id) = dependency {
                    if !self.rounds[..=index]
                        .iter()
                        .any(|r| r.wave_ids.contains(wave_id))
                    {
                        return Err(fail());
                    }
                }
            }
            for effect in &round.rewards {
                if !matches!(
                    effect,
                    RewardEffect::GrantEnkephalin { .. }
                        | RewardEffect::GrantEquipmentDust { .. }
                        | RewardEffect::GrantEquipmentMaterial { .. }
                        | RewardEffect::GrantExperience { .. }
                        | RewardEffect::GrantRelic { .. }
                ) {
                    return Err(fail());
                }
                if matches!(
                    effect,
                    RewardEffect::GrantExperience {
                        target: crate::game::reward::ExperienceTargetPolicy::SelectedEmployee,
                        ..
                    }
                ) {
                    return Err(fail());
                }
                match effect {
                    RewardEffect::GrantEnkephalin { amount }
                    | RewardEffect::GrantEquipmentDust { amount }
                    | RewardEffect::GrantEquipmentMaterial { amount, .. }
                    | RewardEffect::GrantExperience { amount, .. }
                        if amount.checked_mul(2).is_none() =>
                    {
                        return Err(
                            "stress combat reward overflows the fixed unrestrained multiplier"
                                .into(),
                        );
                    }
                    _ => {}
                }
            }
        }
        if seen.len() != encounter.waves.len() {
            return Err(fail());
        }
        Ok(())
    }
    pub fn rewards_through(
        &self,
        completed: usize,
        multiplier: u32,
    ) -> Result<Vec<RewardEffect>, GameError> {
        self.project_rewards(self.rounds.iter().take(completed), multiplier)
    }

    pub(crate) fn rewards_for_round(
        &self,
        index: usize,
        multiplier: u32,
    ) -> Result<Vec<RewardEffect>, GameError> {
        self.project_rewards(self.rounds.get(index).into_iter(), multiplier)
    }

    fn project_rewards<'a>(
        &self,
        rounds: impl Iterator<Item = &'a StressCombatRound>,
        multiplier: u32,
    ) -> Result<Vec<RewardEffect>, GameError> {
        let mut effects = Vec::new();
        for round in rounds {
            crate::game::relic::allocation::admit_owned_projection(
                &round.rewards,
                "stress_reward_owned_projection",
            )?;
            crate::game::battle::runtime_control::admit_retained_growth(
                effects.len() * std::mem::size_of::<RewardEffect>(),
                round
                    .rewards
                    .len()
                    .checked_mul(std::mem::size_of::<RewardEffect>()),
                "stress_reward_projection",
            )?;
            for effect in &round.rewards {
                effects.push(crate::authoring::clone_shape::controlled_clone(effect)?);
            }
        }
        crate::game::combat_setup::rewards::apply_deterrence_quantity_multiplier(
            &mut effects,
            multiplier,
        )?;
        Ok(effects)
    }

    pub(crate) fn battle_round_groups(&self) -> Result<Vec<Vec<String>>, GameError> {
        use crate::game::battle::runtime_control as control;
        let mut bytes = self
            .rounds
            .len()
            .checked_mul(std::mem::size_of::<Vec<String>>());
        for round in &self.rounds {
            control::charge(round.wave_ids.len() as u64 + 1, "stress_round_group_shape")?;
            bytes = bytes.and_then(|n| {
                round
                    .wave_ids
                    .len()
                    .checked_mul(std::mem::size_of::<String>())
                    .and_then(|more| n.checked_add(more))
            });
            for id in &round.wave_ids {
                bytes = bytes
                    .and_then(|n| n.checked_add(id.len()))
                    .and_then(|n| n.checked_add("enemy_".len()));
            }
        }
        control::admit_retained_growth(0, bytes, "stress_round_group_storage")?;
        Ok(self
            .rounds
            .iter()
            .map(|round| {
                round
                    .wave_ids
                    .iter()
                    .map(|id| format!("enemy_{id}"))
                    .collect()
            })
            .collect())
    }
}
