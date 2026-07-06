use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game::{
    battle::{
        damage::DamageSource,
        event_log::{BattleEventLog, BattleLogEvent},
        ids::UnitInstanceId,
        result_stats::BattleResultStatsDto,
        types::{BattleUnitSourceIdentity, BattleWinner},
    },
    data::pve_data::{PveBonusObjectiveConditionData, PveBonusObjectiveData, PveEncounter},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PveBonusObjectiveOutcomeDto {
    pub id: String,
    pub condition: PveBonusObjectiveConditionData,
    pub research_bonus: u32,
    pub satisfied: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation: Option<String>,
}

pub fn evaluate_pve_bonus_objectives(
    encounter: &PveEncounter,
    winner: BattleWinner,
    event_log: &BattleEventLog,
    result_stats: &BattleResultStatsDto,
) -> Vec<PveBonusObjectiveOutcomeDto> {
    if encounter.bonus_objectives().is_empty() {
        return Vec::new();
    }

    let Some(primary_abnormality_id) = encounter.primary_abnormality_id() else {
        return Vec::new();
    };
    let primary_targets = primary_abnormality_targets(event_log, primary_abnormality_id);

    encounter
        .bonus_objectives()
        .iter()
        .map(|objective| PveBonusObjectiveOutcomeDto {
            id: objective.id.clone(),
            condition: objective.condition.clone(),
            research_bonus: objective.research_bonus,
            satisfied: winner == BattleWinner::Player
                && objective_satisfied(objective, event_log, result_stats, &primary_targets),
            presentation: objective.presentation.clone(),
        })
        .collect()
}

pub fn satisfied_research_bonus(outcomes: &[PveBonusObjectiveOutcomeDto]) -> u32 {
    outcomes
        .iter()
        .filter(|outcome| outcome.satisfied)
        .fold(0_u32, |acc, outcome| {
            acc.saturating_add(outcome.research_bonus)
        })
}

fn objective_satisfied(
    objective: &PveBonusObjectiveData,
    event_log: &BattleEventLog,
    result_stats: &BattleResultStatsDto,
    primary_targets: &HashMap<UnitInstanceId, u32>,
) -> bool {
    match objective.condition {
        PveBonusObjectiveConditionData::ClearWithin { time_ms } => {
            result_stats.battle.duration_ms <= time_ms
        }
        PveBonusObjectiveConditionData::DecisiveDamage {
            minimum_damage_percent_of_max_hp,
        } => {
            decisive_damage_satisfied(event_log, primary_targets, minimum_damage_percent_of_max_hp)
        }
    }
}

fn primary_abnormality_targets(
    event_log: &BattleEventLog,
    abnormality_id: &str,
) -> HashMap<UnitInstanceId, u32> {
    event_log
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::UnitSpawned {
                unit_instance_id,
                unit_source:
                    BattleUnitSourceIdentity::Abnormality {
                        abnormality_id: spawned_abnormality_id,
                        ..
                    },
                stats,
                ..
            } if spawned_abnormality_id == abnormality_id && stats.max_health > 0 => {
                Some((*unit_instance_id, stats.max_health))
            }
            _ => None,
        })
        .collect()
}

fn decisive_damage_satisfied(
    event_log: &BattleEventLog,
    primary_targets: &HashMap<UnitInstanceId, u32>,
    minimum_percent: u32,
) -> bool {
    if primary_targets.is_empty() {
        return false;
    }

    event_log.entries.iter().any(|entry| {
        let BattleLogEvent::HpChanged {
            target_instance_id,
            hp_before,
            hp_after,
            damage_source,
            final_damage,
            ..
        } = &entry.event
        else {
            return false;
        };

        if !matches!(
            damage_source,
            Some(DamageSource::BasicAttack | DamageSource::Ability)
        ) {
            return false;
        }

        let Some(max_hp) = primary_targets.get(target_instance_id) else {
            return false;
        };
        let effective_damage = final_damage.unwrap_or_else(|| hp_before.saturating_sub(*hp_after));
        u64::from(effective_damage) * 100 >= u64::from(*max_hp) * u64::from(minimum_percent)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        battle::{
            event_log::{BattleEventCause, BattleEventLogEntry, BattleLogEvent},
            types::BattleUnitSourceIdentity,
        },
        combat_preview::CombatNodeType,
        data::pve_data::{PveBattlefieldOverrideData, PveWaveData, PveWaveSource},
        enums::{RewardMode, RiskLevel, Side},
        stats::UnitStats,
    };
    use uuid::Uuid;

    fn encounter_with_objectives() -> PveEncounter {
        PveEncounter {
            id: "bonus_objective_encounter".to_string(),
            encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
            primary_abnormality_id: Some("test_abno".to_string()),
            risk_level: RiskLevel::ZAYIN,
            node_type: Some(CombatNodeType::Defense),
            mission_variant: None,
            survive_timer_ms: None,
            reward_mode: RewardMode::ClaimAll,
            reward_uuids: Vec::new(),
            suppression_research: Some(crate::game::data::pve_data::PveSuppressionResearchData {
                bonus_objectives: vec![
                    PveBonusObjectiveData {
                        id: "fast_clear".to_string(),
                        condition: PveBonusObjectiveConditionData::ClearWithin { time_ms: 10_000 },
                        research_bonus: 20,
                        presentation: Some("abnormality_part_obtained".to_string()),
                    },
                    PveBonusObjectiveData {
                        id: "decisive_damage".to_string(),
                        condition: PveBonusObjectiveConditionData::DecisiveDamage {
                            minimum_damage_percent_of_max_hp: 10,
                        },
                        research_bonus: 30,
                        presentation: None,
                    },
                ],
            }),
            battlefield: Some(PveBattlefieldOverrideData::default()),
            tactical_plan: None,
            win_condition: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: Vec::new(),
                route_id: None,
                required_for_victory: true,
                source: PveWaveSource::Manual(Vec::new()),
            }],
            static_obstacles: Vec::new(),
        }
    }

    fn event_log_with_primary_damage(
        damage_source: DamageSource,
        final_damage: u32,
    ) -> BattleEventLog {
        let target = UnitInstanceId::from(Uuid::from_u128(0xABCD));
        BattleEventLog {
            version: crate::game::battle::event_log::BATTLE_EVENT_LOG_VERSION,
            entries: vec![
                BattleEventLogEntry {
                    time_ms: 0,
                    seq: 1,
                    cause: BattleEventCause::default(),
                    source_command_id: None,
                    event: BattleLogEvent::UnitSpawned {
                        unit_instance_id: target,
                        owner: Side::Opponent,
                        role: crate::game::battle::types::BattleUnitRole::Combatant,
                        threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
                        mobility_kind: crate::game::battle::types::MobilityKind::Ground,
                        base_uuid: Uuid::from_u128(0x1111),
                        unit_source: BattleUnitSourceIdentity::Abnormality {
                            abnormality_id: "test_abno".to_string(),
                            base_uuid: Uuid::from_u128(0x1111),
                        },
                        world_position: crate::game::battle::core::movement::types::EventLogVec2 {
                            x_milli: 0,
                            y_milli: 0,
                        },
                        stats: UnitStats::with_values(1_000, 1_000, 10, 0, 1_000),
                    },
                },
                BattleEventLogEntry {
                    time_ms: 1_000,
                    seq: 2,
                    cause: BattleEventCause::default(),
                    source_command_id: None,
                    event: BattleLogEvent::HpChanged {
                        source_instance_id: None,
                        target_instance_id: target,
                        delta: -(final_damage as i32),
                        hp_before: 1_000,
                        hp_after: 1_000_u32.saturating_sub(final_damage),
                        reason: crate::game::battle::event_log::HpChangeReason::Command,
                        damage_source: Some(damage_source),
                        damage_type: Some(crate::game::battle::damage::DamageType::Physical),
                        raw_damage: Some(final_damage),
                        final_damage: Some(final_damage),
                        damage_breakdown: None,
                        critical: Some(false),
                        feedback_tags: Vec::new(),
                    },
                },
                BattleEventLogEntry {
                    time_ms: 9_000,
                    seq: 3,
                    cause: BattleEventCause::default(),
                    source_command_id: None,
                    event: BattleLogEvent::BattleEnd {
                        winner: BattleWinner::Player,
                    },
                },
            ],
        }
    }

    #[test]
    fn evaluates_clear_within_and_one_non_dot_decisive_damage_source() {
        let encounter = encounter_with_objectives();
        let event_log = event_log_with_primary_damage(DamageSource::Ability, 100);
        let result_stats = crate::game::battle::result_stats::collect_battle_result_stats(
            BattleWinner::Player,
            &event_log,
            &[],
            &crate::game::data::run_policy_data::RunPolicyData::builtin().battle_result_stats,
        );

        let outcomes = evaluate_pve_bonus_objectives(
            &encounter,
            BattleWinner::Player,
            &event_log,
            &result_stats,
        );

        assert_eq!(outcomes.len(), 2);
        assert!(outcomes.iter().all(|outcome| outcome.satisfied));
        assert_eq!(satisfied_research_bonus(&outcomes), 50);
    }

    #[test]
    fn excludes_dot_damage_from_decisive_damage() {
        let encounter = encounter_with_objectives();
        let event_log = event_log_with_primary_damage(DamageSource::BuffTick, 500);
        let result_stats = crate::game::battle::result_stats::collect_battle_result_stats(
            BattleWinner::Player,
            &event_log,
            &[],
            &crate::game::data::run_policy_data::RunPolicyData::builtin().battle_result_stats,
        );

        let outcomes = evaluate_pve_bonus_objectives(
            &encounter,
            BattleWinner::Player,
            &event_log,
            &result_stats,
        );

        assert!(outcomes
            .iter()
            .find(|outcome| outcome.id == "decisive_damage")
            .is_some_and(|outcome| !outcome.satisfied));
    }

    #[test]
    fn defeat_never_satisfies_bonus_objectives() {
        let encounter = encounter_with_objectives();
        let event_log = event_log_with_primary_damage(DamageSource::Ability, 500);
        let result_stats = crate::game::battle::result_stats::collect_battle_result_stats(
            BattleWinner::Opponent,
            &event_log,
            &[],
            &crate::game::data::run_policy_data::RunPolicyData::builtin().battle_result_stats,
        );

        let outcomes = evaluate_pve_bonus_objectives(
            &encounter,
            BattleWinner::Opponent,
            &event_log,
            &result_stats,
        );

        assert!(outcomes.iter().all(|outcome| !outcome.satisfied));
        assert_eq!(satisfied_research_bonus(&outcomes), 0);
    }
}
