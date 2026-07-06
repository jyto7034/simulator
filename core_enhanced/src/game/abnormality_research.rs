use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::{
    battle::types::BattleUnitThreatClass,
    data::{run_policy_data::AbnormalityResearchPolicy, GameDataBase},
    reward::RewardEffect,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunAbnormalityResearchState {
    pub entries: BTreeMap<String, RunAbnormalityResearchEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunAbnormalityEncounterHistory {
    pub last_appeared_floor_by_abnormality: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunAbnormalityResearchEntry {
    pub research_points: u32,
    pub research_required: u32,
    pub response_complete: bool,
    pub suppression_wins: u32,
    pub last_encountered_floor: Option<u32>,
    pub completed_at_floor: Option<u32>,
    pub unique_fragment_granted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbnormalityResearchVictoryOutcome {
    pub abnormality_id: String,
    pub research_points_before: u32,
    pub research_points_after: u32,
    pub response_completed_now: bool,
    pub reward_effects: Vec<RewardEffect>,
}

impl RunAbnormalityResearchState {
    pub fn initialize(game_data: &GameDataBase) -> Self {
        let required = game_data
            .run_policy
            .abnormality_research
            .default_research_required;
        let entries = game_data
            .abnormality_data
            .items
            .iter()
            .map(|abnormality| {
                (
                    abnormality.id.clone(),
                    RunAbnormalityResearchEntry {
                        research_points: 0,
                        research_required: required,
                        response_complete: false,
                        suppression_wins: 0,
                        last_encountered_floor: None,
                        completed_at_floor: None,
                        unique_fragment_granted: false,
                    },
                )
            })
            .collect();

        Self { entries }
    }

    pub fn apply_suppression_victory(
        &mut self,
        game_data: &GameDataBase,
        abnormality_id: &str,
        floor_index: u32,
        is_final_boss: bool,
    ) -> Result<AbnormalityResearchVictoryOutcome, String> {
        self.apply_suppression_victory_with_bonus(
            game_data,
            abnormality_id,
            floor_index,
            is_final_boss,
            0,
        )
    }

    pub fn apply_suppression_victory_with_bonus(
        &mut self,
        game_data: &GameDataBase,
        abnormality_id: &str,
        floor_index: u32,
        is_final_boss: bool,
        bonus_research_gain: u32,
    ) -> Result<AbnormalityResearchVictoryOutcome, String> {
        let abnormality = game_data
            .abnormality_data
            .get_by_id(abnormality_id)
            .ok_or_else(|| format!("unknown abnormality '{abnormality_id}'"))?;
        let entry = self
            .entries
            .get_mut(abnormality_id)
            .ok_or_else(|| format!("missing abnormality research entry '{abnormality_id}'"))?;
        let research_points_before = entry.research_points;
        entry.suppression_wins = entry.suppression_wins.saturating_add(1);
        entry.last_encountered_floor = Some(floor_index);

        let mut reward_effects = Vec::new();
        let mut response_completed_now = false;
        if entry.response_complete {
            let amount = repeat_complete_dust_amount(
                game_data.run_policy.abnormality_research,
                abnormality.threat_class,
                is_final_boss,
            );
            if amount > 0 {
                reward_effects.push(RewardEffect::GrantFragmentDust { amount });
            }
        } else {
            let research_gain = game_data
                .run_policy
                .abnormality_research
                .victory_research_gain
                .saturating_add(bonus_research_gain);
            entry.research_points = entry
                .research_required
                .min(entry.research_points.saturating_add(research_gain));
            if entry.research_points >= entry.research_required {
                entry.response_complete = true;
                entry.completed_at_floor = Some(floor_index);
                response_completed_now = true;
                if !entry.unique_fragment_granted {
                    let fragment_id = abnormality
                        .response_complete_skill_fragment_id
                        .clone()
                        .ok_or_else(|| {
                            format!(
                                "abnormality '{}' is missing response_complete_skill_fragment_id",
                                abnormality.id
                            )
                        })?;
                    entry.unique_fragment_granted = true;
                    reward_effects.push(RewardEffect::GrantSkillFragment { fragment_id });
                }
            }
        }

        Ok(AbnormalityResearchVictoryOutcome {
            abnormality_id: abnormality_id.to_string(),
            research_points_before,
            research_points_after: entry.research_points,
            response_completed_now,
            reward_effects,
        })
    }

    pub fn all_response_complete(&self) -> bool {
        !self.entries.is_empty() && self.entries.values().all(|entry| entry.response_complete)
    }
}

impl RunAbnormalityEncounterHistory {
    pub fn record_appearance(&mut self, abnormality_id: &str, floor_index: u32) {
        self.last_appeared_floor_by_abnormality
            .insert(abnormality_id.to_string(), floor_index);
    }

    pub fn last_appeared_floor(&self, abnormality_id: &str) -> Option<u32> {
        self.last_appeared_floor_by_abnormality
            .get(abnormality_id)
            .copied()
    }
}

fn repeat_complete_dust_amount(
    policy: AbnormalityResearchPolicy,
    threat_class: BattleUnitThreatClass,
    is_final_boss: bool,
) -> u32 {
    if is_final_boss {
        return policy.repeat_complete_fragment_dust.final_boss;
    }
    match threat_class {
        BattleUnitThreatClass::Normal => 0,
        BattleUnitThreatClass::Elite => policy.repeat_complete_fragment_dust.elite,
        BattleUnitThreatClass::Boss => policy.repeat_complete_fragment_dust.boss,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::*;
    use crate::game::{
        battle::types::BattleUnitThreatClass,
        data::{
            abnormality_data::AbnormalityMetadata,
            skill_fragment_data::{
                SkillFragmentDatabase, SkillFragmentEffectDef, SkillFragmentEquipLimit,
                SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
            },
            GameDataBuilder,
        },
        enums::RiskLevel,
    };

    fn test_data() -> Arc<GameDataBase> {
        let fragment = SkillFragmentMetadata {
            id: SkillFragmentId::from("fragment_test_abnormality"),
            uuid: Uuid::from_u128(0xF00D),
            name: "Test Fragment".to_string(),
            description: "test".to_string(),
            rarity: SkillFragmentRarity::Rare,
            equip_limit: SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: Vec::new(),
            dependencies: Vec::new(),
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::BasicAttackModifier {
                attack_bonus: 1,
                attack_interval_ms_reduction: 0,
            },
        };
        let abnormality = AbnormalityMetadata {
            id: "test_abnormality".to_string(),
            uuid: Uuid::from_u128(0xABCD),
            name: "Test Abnormality".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 10,
            max_health: 10,
            attack: 1,
            defense: 1,
            magic_resist: 0,
            target_traits: Vec::new(),
            mobility_kind: Default::default(),
            threat_class: BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: Some(SkillFragmentId::from(
                "fragment_test_abnormality",
            )),
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };
        GameDataBuilder::live_defaults()
            .with_skill_fragments(SkillFragmentDatabase::new(vec![fragment]))
            .with_abnormalities(vec![abnormality])
            .build_arc()
    }

    #[test]
    fn victory_caps_research_and_grants_unique_fragment_once() {
        let game_data = test_data();
        let mut state = RunAbnormalityResearchState::initialize(&game_data);
        state
            .entries
            .get_mut("test_abnormality")
            .unwrap()
            .research_points = 90;

        let outcome = state
            .apply_suppression_victory(&game_data, "test_abnormality", 2, false)
            .unwrap();

        let entry = state.entries.get("test_abnormality").unwrap();
        assert_eq!(entry.research_points, 100);
        assert!(entry.response_complete);
        assert_eq!(entry.completed_at_floor, Some(2));
        assert!(entry.unique_fragment_granted);
        assert!(outcome.response_completed_now);
        assert!(matches!(
            outcome.reward_effects.as_slice(),
            [RewardEffect::GrantSkillFragment { fragment_id }]
            if fragment_id.as_str() == "fragment_test_abnormality"
        ));
    }

    #[test]
    fn repeat_complete_victory_grants_dust_without_duplicate_fragment() {
        let game_data = test_data();
        let mut state = RunAbnormalityResearchState::initialize(&game_data);
        let entry = state.entries.get_mut("test_abnormality").unwrap();
        entry.research_points = 100;
        entry.response_complete = true;
        entry.unique_fragment_granted = true;

        let outcome = state
            .apply_suppression_victory(&game_data, "test_abnormality", 3, false)
            .unwrap();

        assert_eq!(outcome.research_points_before, 100);
        assert_eq!(outcome.research_points_after, 100);
        assert!(!outcome.response_completed_now);
        assert!(matches!(
            outcome.reward_effects.as_slice(),
            [RewardEffect::GrantFragmentDust { amount }] if *amount == 5
        ));
    }
}
