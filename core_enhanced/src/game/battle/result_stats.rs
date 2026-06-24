use std::{cmp::Ordering, collections::HashMap};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    battle::{
        event_log::{BattleEventLog, BattleLogEvent},
        ids::UnitInstanceId,
        types::{BattleUnitSourceIdentity, BattleWinner, ParticipantBattleResult},
    },
    data::run_policy_data::{
        BattleResultMvpMetricWeight, BattleResultMvpPolicy, BattleResultMvpTieBreaker,
        BattleResultStatsPolicy,
    },
    enums::Side,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleResultStatsDto {
    pub battle: BattleResultBattleStatsDto,
    pub employees: Vec<BattleResultEmployeeStatsDto>,
    pub mvp: Option<BattleResultMvpDto>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleResultBattleStatsDto {
    pub duration_ms: u64,
    pub winner: BattleWinner,
    pub killed_enemy_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleResultEmployeeStatsDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub survived: bool,
    pub became_incapacitated: bool,
    pub final_hp: u32,
    pub max_hp: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub kill_count: u32,
    pub basic_attack_count: u32,
    pub skill_cast_count: u32,
    pub deployed_count: u32,
    pub withdrawn_count: u32,
    pub deployed_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleResultMvpDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub score: i64,
    pub title: String,
    pub highlighted_metrics: Vec<BattleResultMetricHighlightDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleResultMetricHighlightDto {
    pub metric: BattleResultMetricKind,
    pub value: u64,
    pub weighted_score: i64,
    pub title: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BattleResultMetricKind {
    DamageDealt,
    DamageTaken,
    KillCount,
    BasicAttackCount,
    SkillCastCount,
    DeployedCount,
    WithdrawnCount,
    DeployedTimeMs,
    Survived,
}

#[derive(Debug, Clone, Copy)]
struct UnitSourceInfo {
    owner: Side,
    employee_uuid: Option<Uuid>,
}

#[derive(Debug, Clone)]
struct EmployeeAccumulator {
    stats: BattleResultEmployeeStatsDto,
}

impl EmployeeAccumulator {
    fn new(employee_uuid: Uuid, unit_instance_id: UnitInstanceId) -> Self {
        Self {
            stats: BattleResultEmployeeStatsDto {
                employee_uuid,
                unit_instance_id,
                survived: false,
                became_incapacitated: false,
                final_hp: 0,
                max_hp: 0,
                damage_dealt: 0,
                damage_taken: 0,
                kill_count: 0,
                basic_attack_count: 0,
                skill_cast_count: 0,
                deployed_count: 0,
                withdrawn_count: 0,
                deployed_time_ms: 0,
            },
        }
    }
}

pub fn collect_battle_result_stats(
    winner: BattleWinner,
    event_log: &BattleEventLog,
    participant_results: &[ParticipantBattleResult],
    policy: &BattleResultStatsPolicy,
) -> BattleResultStatsDto {
    let mut unit_sources = HashMap::<UnitInstanceId, UnitSourceInfo>::new();
    let mut employees = HashMap::<Uuid, EmployeeAccumulator>::new();
    let mut active_deployments = HashMap::<UnitInstanceId, (Uuid, u64)>::new();
    let mut duration_ms = 0;
    let mut killed_enemy_count = 0_u32;

    for entry in &event_log.entries {
        if let BattleLogEvent::UnitSpawned {
            unit_instance_id,
            owner,
            unit_source,
            ..
        } = &entry.event
        {
            let employee_uuid = match unit_source {
                BattleUnitSourceIdentity::Employee { employee_uuid, .. } => Some(*employee_uuid),
                _ => None,
            };
            unit_sources.insert(
                *unit_instance_id,
                UnitSourceInfo {
                    owner: *owner,
                    employee_uuid,
                },
            );
            if let Some(employee_uuid) = employee_uuid {
                employees
                    .entry(employee_uuid)
                    .or_insert_with(|| EmployeeAccumulator::new(employee_uuid, *unit_instance_id));
            }
        }
    }

    for participant in participant_results
        .iter()
        .filter(|participant| participant.side == Side::Player)
    {
        let Some(employee_uuid) =
            direct_employee_uuid(&unit_sources, &participant.unit_instance_id)
        else {
            continue;
        };
        let employee = employees.entry(employee_uuid).or_insert_with(|| {
            EmployeeAccumulator::new(employee_uuid, participant.unit_instance_id)
        });
        employee.stats.unit_instance_id = participant.unit_instance_id;
        employee.stats.survived = participant.survived;
        employee.stats.became_incapacitated = participant.became_incapacitated;
        employee.stats.final_hp = participant.final_hp;
        employee.stats.max_hp = participant.max_hp;
    }

    for entry in &event_log.entries {
        match &entry.event {
            BattleLogEvent::UnitDeployed {
                employee_uuid,
                unit_instance_id,
                ..
            } => {
                let employee = employees
                    .entry(*employee_uuid)
                    .or_insert_with(|| EmployeeAccumulator::new(*employee_uuid, *unit_instance_id));
                employee.stats.deployed_count = employee.stats.deployed_count.saturating_add(1);
                active_deployments.insert(*unit_instance_id, (*employee_uuid, entry.time_ms));
            }
            BattleLogEvent::UnitWithdrawn {
                unit_instance_id, ..
            } => {
                if let Some(employee_uuid) = unit_sources
                    .get(unit_instance_id)
                    .and_then(|source| source.employee_uuid)
                {
                    if let Some(employee) = employees.get_mut(&employee_uuid) {
                        employee.stats.withdrawn_count =
                            employee.stats.withdrawn_count.saturating_add(1);
                    }
                }
                close_deployment(
                    &mut employees,
                    &mut active_deployments,
                    *unit_instance_id,
                    entry.time_ms,
                );
            }
            BattleLogEvent::AttackResolve {
                attacker_instance_id,
                ..
            } => {
                if let Some(employee_uuid) =
                    direct_employee_uuid(&unit_sources, attacker_instance_id)
                {
                    if let Some(employee) = employees.get_mut(&employee_uuid) {
                        employee.stats.basic_attack_count =
                            employee.stats.basic_attack_count.saturating_add(1);
                    }
                }
            }
            BattleLogEvent::AbilityCast {
                caster_instance_id, ..
            } => {
                if let Some(employee_uuid) = direct_employee_uuid(&unit_sources, caster_instance_id)
                {
                    if let Some(employee) = employees.get_mut(&employee_uuid) {
                        employee.stats.skill_cast_count =
                            employee.stats.skill_cast_count.saturating_add(1);
                    }
                }
            }
            BattleLogEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                hp_before,
                hp_after,
                damage_source,
                ..
            } => {
                let actual_damage = hp_before.saturating_sub(*hp_after);
                if actual_damage == 0 {
                    continue;
                }
                if let Some(target_employee_uuid) =
                    direct_employee_uuid(&unit_sources, target_instance_id)
                {
                    if let Some(employee) = employees.get_mut(&target_employee_uuid) {
                        employee.stats.damage_taken =
                            employee.stats.damage_taken.saturating_add(actual_damage);
                    }
                }
                let Some(source_instance_id) = source_instance_id else {
                    continue;
                };
                if !is_first_slice_damage_source(*damage_source) {
                    continue;
                }
                let Some(source_employee_uuid) =
                    direct_employee_uuid(&unit_sources, source_instance_id)
                else {
                    continue;
                };
                let target_owner = unit_sources
                    .get(target_instance_id)
                    .map(|source| source.owner);
                if target_owner == Some(Side::Player) {
                    continue;
                }
                if let Some(employee) = employees.get_mut(&source_employee_uuid) {
                    employee.stats.damage_dealt =
                        employee.stats.damage_dealt.saturating_add(actual_damage);
                }
            }
            BattleLogEvent::UnitDied {
                unit_instance_id,
                owner,
                killer_instance_id,
                ..
            } => {
                if *owner != Side::Player {
                    killed_enemy_count = killed_enemy_count.saturating_add(1);
                }
                if let Some(killer_instance_id) = killer_instance_id {
                    if *owner != Side::Player {
                        if let Some(employee_uuid) =
                            direct_employee_uuid(&unit_sources, killer_instance_id)
                        {
                            if let Some(employee) = employees.get_mut(&employee_uuid) {
                                employee.stats.kill_count =
                                    employee.stats.kill_count.saturating_add(1);
                            }
                        }
                    }
                }
                close_deployment(
                    &mut employees,
                    &mut active_deployments,
                    *unit_instance_id,
                    entry.time_ms,
                );
            }
            BattleLogEvent::BattleEnd { .. } => {
                duration_ms = entry.time_ms;
                let open_deployments = active_deployments.keys().copied().collect::<Vec<_>>();
                for unit_instance_id in open_deployments {
                    close_deployment(
                        &mut employees,
                        &mut active_deployments,
                        unit_instance_id,
                        entry.time_ms,
                    );
                }
            }
            _ => {}
        }
    }

    let mut employees = employees
        .into_values()
        .map(|accumulator| accumulator.stats)
        .collect::<Vec<_>>();
    employees.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

    let mvp = select_mvp(&employees, &policy.mvp);

    BattleResultStatsDto {
        battle: BattleResultBattleStatsDto {
            duration_ms,
            winner,
            killed_enemy_count,
        },
        employees,
        mvp,
    }
}

fn direct_employee_uuid(
    unit_sources: &HashMap<UnitInstanceId, UnitSourceInfo>,
    unit_instance_id: &UnitInstanceId,
) -> Option<Uuid> {
    unit_sources
        .get(unit_instance_id)
        .and_then(|source| source.employee_uuid)
}

fn close_deployment(
    employees: &mut HashMap<Uuid, EmployeeAccumulator>,
    active_deployments: &mut HashMap<UnitInstanceId, (Uuid, u64)>,
    unit_instance_id: UnitInstanceId,
    end_ms: u64,
) {
    let Some((employee_uuid, start_ms)) = active_deployments.remove(&unit_instance_id) else {
        return;
    };
    if let Some(employee) = employees.get_mut(&employee_uuid) {
        employee.stats.deployed_time_ms = employee
            .stats
            .deployed_time_ms
            .saturating_add(end_ms.saturating_sub(start_ms));
    }
}

fn is_first_slice_damage_source(
    damage_source: Option<crate::game::battle::damage::DamageSource>,
) -> bool {
    matches!(
        damage_source,
        Some(crate::game::battle::damage::DamageSource::BasicAttack)
            | Some(crate::game::battle::damage::DamageSource::Ability)
    )
}

fn select_mvp(
    employees: &[BattleResultEmployeeStatsDto],
    policy: &BattleResultMvpPolicy,
) -> Option<BattleResultMvpDto> {
    employees
        .iter()
        .max_by(|left, right| compare_mvp_candidates(left, right, policy))
        .map(|employee| {
            let (score, highlight) = score_employee(employee, &policy.metric_weights);
            BattleResultMvpDto {
                employee_uuid: employee.employee_uuid,
                unit_instance_id: employee.unit_instance_id,
                score,
                title: highlight
                    .as_ref()
                    .map(|highlight| highlight.title.clone())
                    .unwrap_or_else(|| "MVP".to_string()),
                highlighted_metrics: highlight.into_iter().collect(),
            }
        })
}

fn compare_mvp_candidates(
    left: &BattleResultEmployeeStatsDto,
    right: &BattleResultEmployeeStatsDto,
    policy: &BattleResultMvpPolicy,
) -> Ordering {
    for tiebreaker in &policy.tiebreakers {
        let ordering = match tiebreaker {
            BattleResultMvpTieBreaker::Score => score_total(left, &policy.metric_weights)
                .cmp(&score_total(right, &policy.metric_weights)),
            BattleResultMvpTieBreaker::DamageDealt => left.damage_dealt.cmp(&right.damage_dealt),
            BattleResultMvpTieBreaker::KillCount => left.kill_count.cmp(&right.kill_count),
            BattleResultMvpTieBreaker::SkillCastCount => {
                left.skill_cast_count.cmp(&right.skill_cast_count)
            }
            BattleResultMvpTieBreaker::BasicAttackCount => {
                left.basic_attack_count.cmp(&right.basic_attack_count)
            }
            BattleResultMvpTieBreaker::DeployedTimeMs => {
                left.deployed_time_ms.cmp(&right.deployed_time_ms)
            }
            BattleResultMvpTieBreaker::EmployeeUuid => right.employee_uuid.cmp(&left.employee_uuid),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    right.employee_uuid.cmp(&left.employee_uuid)
}

fn score_total(
    employee: &BattleResultEmployeeStatsDto,
    weights: &[BattleResultMvpMetricWeight],
) -> i64 {
    weights
        .iter()
        .map(|weight| metric_score(employee, weight))
        .sum()
}

fn score_employee(
    employee: &BattleResultEmployeeStatsDto,
    weights: &[BattleResultMvpMetricWeight],
) -> (i64, Option<BattleResultMetricHighlightDto>) {
    let mut total = 0_i64;
    let mut highlight = None;
    for weight in weights {
        let value = metric_value(employee, weight.metric);
        let weighted_score = metric_score(employee, weight);
        total = total.saturating_add(weighted_score);
        let candidate = BattleResultMetricHighlightDto {
            metric: weight.metric,
            value,
            weighted_score,
            title: weight.title.clone(),
        };
        let replace = highlight
            .as_ref()
            .is_none_or(|current: &BattleResultMetricHighlightDto| {
                candidate.weighted_score > current.weighted_score
                    || (candidate.weighted_score == current.weighted_score
                        && candidate.value > current.value)
            });
        if replace {
            highlight = Some(candidate);
        }
    }
    (total, highlight)
}

fn metric_score(
    employee: &BattleResultEmployeeStatsDto,
    weight: &BattleResultMvpMetricWeight,
) -> i64 {
    let value = metric_value(employee, weight.metric);
    let value = i64::try_from(value).unwrap_or(i64::MAX);
    value.saturating_mul(weight.weight)
}

fn metric_value(employee: &BattleResultEmployeeStatsDto, metric: BattleResultMetricKind) -> u64 {
    match metric {
        BattleResultMetricKind::DamageDealt => u64::from(employee.damage_dealt),
        BattleResultMetricKind::DamageTaken => u64::from(employee.damage_taken),
        BattleResultMetricKind::KillCount => u64::from(employee.kill_count),
        BattleResultMetricKind::BasicAttackCount => u64::from(employee.basic_attack_count),
        BattleResultMetricKind::SkillCastCount => u64::from(employee.skill_cast_count),
        BattleResultMetricKind::DeployedCount => u64::from(employee.deployed_count),
        BattleResultMetricKind::WithdrawnCount => u64::from(employee.withdrawn_count),
        BattleResultMetricKind::DeployedTimeMs => employee.deployed_time_ms,
        BattleResultMetricKind::Survived => u64::from(employee.survived),
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{
        battle::damage::{DamageSource, DamageType},
        battle::{
            core::movement::types::EventLogVec2,
            event_log::{AttackDelivery, AttackKind, BattleEventLogEntry, HpChangeReason},
            tile_range::FacingDirection,
        },
        resources::Position,
        stats::UnitStats,
    };

    use super::*;

    fn unit_id(id: u128) -> UnitInstanceId {
        UnitInstanceId::from(Uuid::from_u128(id))
    }

    fn employee_uuid(id: u128) -> Uuid {
        Uuid::from_u128(id)
    }

    fn entry(seq: u64, time_ms: u64, event: BattleLogEvent) -> BattleEventLogEntry {
        BattleEventLogEntry {
            time_ms,
            seq,
            cause: Default::default(),
            source_command_id: None,
            event,
        }
    }

    fn policy() -> BattleResultStatsPolicy {
        BattleResultStatsPolicy {
            mvp: BattleResultMvpPolicy {
                metric_weights: vec![
                    BattleResultMvpMetricWeight {
                        metric: BattleResultMetricKind::DamageDealt,
                        weight: 1,
                        title: "Top Damage".to_string(),
                    },
                    BattleResultMvpMetricWeight {
                        metric: BattleResultMetricKind::KillCount,
                        weight: 100,
                        title: "Most Kills".to_string(),
                    },
                    BattleResultMvpMetricWeight {
                        metric: BattleResultMetricKind::Survived,
                        weight: 50,
                        title: "Survivor".to_string(),
                    },
                ],
                tiebreakers: vec![
                    BattleResultMvpTieBreaker::Score,
                    BattleResultMvpTieBreaker::DamageDealt,
                    BattleResultMvpTieBreaker::KillCount,
                    BattleResultMvpTieBreaker::EmployeeUuid,
                ],
            },
        }
    }

    fn spawn_employee(
        seq: u64,
        time_ms: u64,
        unit: UnitInstanceId,
        employee: Uuid,
    ) -> BattleEventLogEntry {
        entry(
            seq,
            time_ms,
            BattleLogEvent::UnitSpawned {
                unit_instance_id: unit,
                owner: Side::Player,
                role: Default::default(),
                threat_class: Default::default(),
                mobility_kind: Default::default(),
                base_uuid: employee,
                unit_source: BattleUnitSourceIdentity::Employee {
                    employee_uuid: employee,
                    base_uuid: employee,
                },
                world_position: EventLogVec2::default(),
                stats: UnitStats::default(),
            },
        )
    }

    fn spawn_enemy(seq: u64, time_ms: u64, unit: UnitInstanceId) -> BattleEventLogEntry {
        entry(
            seq,
            time_ms,
            BattleLogEvent::UnitSpawned {
                unit_instance_id: unit,
                owner: Side::Opponent,
                role: Default::default(),
                threat_class: Default::default(),
                mobility_kind: Default::default(),
                base_uuid: unit.as_uuid(),
                unit_source: BattleUnitSourceIdentity::Abnormality {
                    abnormality_id: "enemy".to_string(),
                    base_uuid: unit.as_uuid(),
                },
                world_position: EventLogVec2::default(),
                stats: UnitStats::default(),
            },
        )
    }

    fn spawn_defense_object(seq: u64, time_ms: u64, unit: UnitInstanceId) -> BattleEventLogEntry {
        entry(
            seq,
            time_ms,
            BattleLogEvent::UnitSpawned {
                unit_instance_id: unit,
                owner: Side::Player,
                role: Default::default(),
                threat_class: Default::default(),
                mobility_kind: Default::default(),
                base_uuid: unit.as_uuid(),
                unit_source: BattleUnitSourceIdentity::DefenseObject {
                    base_uuid: unit.as_uuid(),
                },
                world_position: EventLogVec2::default(),
                stats: UnitStats::default(),
            },
        )
    }

    #[test]
    fn collector_uses_actual_hp_delta_and_merges_participant_results() {
        let employee = employee_uuid(1);
        let player = unit_id(10);
        let enemy = unit_id(20);
        let mut event_log = BattleEventLog::new();
        event_log.entries = vec![
            spawn_employee(1, 0, player, employee),
            spawn_enemy(2, 0, enemy),
            entry(
                3,
                100,
                BattleLogEvent::UnitDeployed {
                    employee_uuid: employee,
                    unit_instance_id: player,
                    position: Position::new(1, 1),
                    facing: FacingDirection::Right,
                },
            ),
            entry(
                4,
                200,
                BattleLogEvent::AttackResolve {
                    attacker_instance_id: player,
                    target_instance_id: enemy,
                    kind: Some(AttackKind::Auto),
                    delivery: Some(AttackDelivery::Instant),
                },
            ),
            entry(
                5,
                250,
                BattleLogEvent::AbilityCast {
                    skill_id: crate::game::ability::SkillId::new("focused_test_skill"),
                    caster_instance_id: player,
                    target_instance_id: Some(enemy),
                },
            ),
            entry(
                6,
                300,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(enemy),
                    target_instance_id: player,
                    delta: -22,
                    hp_before: 77,
                    hp_after: 55,
                    reason: HpChangeReason::Command,
                    damage_source: Some(DamageSource::Ability),
                    damage_type: Some(DamageType::Physical),
                    raw_damage: Some(22),
                    final_damage: Some(22),
                    damage_breakdown: None,
                    critical: Some(false),
                    feedback_tags: Vec::new(),
                },
            ),
            entry(
                7,
                400,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(player),
                    target_instance_id: enemy,
                    delta: -10,
                    hp_before: 10,
                    hp_after: 0,
                    reason: HpChangeReason::BasicAttack,
                    damage_source: Some(DamageSource::BasicAttack),
                    damage_type: Some(DamageType::Physical),
                    raw_damage: Some(100),
                    final_damage: Some(100),
                    damage_breakdown: None,
                    critical: Some(false),
                    feedback_tags: Vec::new(),
                },
            ),
            entry(
                8,
                400,
                BattleLogEvent::UnitDied {
                    unit_instance_id: enemy,
                    owner: Side::Opponent,
                    killer_instance_id: Some(player),
                    world_position: EventLogVec2::default(),
                    position: Position::new(2, 1),
                },
            ),
            entry(
                9,
                900,
                BattleLogEvent::UnitWithdrawn {
                    unit_instance_id: player,
                    world_position: EventLogVec2::default(),
                    position: Position::new(1, 1),
                },
            ),
            entry(
                10,
                1_000,
                BattleLogEvent::BattleEnd {
                    winner: BattleWinner::Player,
                },
            ),
        ];
        let participant_results = vec![ParticipantBattleResult {
            unit_instance_id: player,
            owned_uuid: employee,
            side: Side::Player,
            survived: true,
            final_hp: 55,
            max_hp: 100,
            became_incapacitated: false,
        }];

        let stats = collect_battle_result_stats(
            BattleWinner::Player,
            &event_log,
            &participant_results,
            &policy(),
        );

        assert_eq!(stats.battle.duration_ms, 1_000);
        assert_eq!(stats.battle.killed_enemy_count, 1);
        assert_eq!(stats.employees.len(), 1);
        let employee_stats = &stats.employees[0];
        assert_eq!(employee_stats.damage_dealt, 10);
        assert_eq!(employee_stats.damage_taken, 22);
        assert_eq!(employee_stats.kill_count, 1);
        assert_eq!(employee_stats.basic_attack_count, 1);
        assert_eq!(employee_stats.skill_cast_count, 1);
        assert_eq!(employee_stats.deployed_count, 1);
        assert_eq!(employee_stats.withdrawn_count, 1);
        assert_eq!(employee_stats.deployed_time_ms, 800);
        assert_eq!(employee_stats.final_hp, 55);
        assert!(employee_stats.survived);
        assert_eq!(stats.mvp.as_ref().unwrap().employee_uuid, employee);
        assert_eq!(stats.mvp.as_ref().unwrap().score, 160);
    }

    #[test]
    fn collector_keeps_incapacitated_employee_mvp_eligible() {
        let first_employee = employee_uuid(1);
        let second_employee = employee_uuid(2);
        let first_unit = unit_id(10);
        let second_unit = unit_id(11);
        let enemy = unit_id(20);
        let mut event_log = BattleEventLog::new();
        event_log.entries = vec![
            spawn_employee(1, 0, first_unit, first_employee),
            spawn_employee(2, 0, second_unit, second_employee),
            spawn_enemy(3, 0, enemy),
            entry(
                4,
                100,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(first_unit),
                    target_instance_id: enemy,
                    delta: -200,
                    hp_before: 300,
                    hp_after: 100,
                    reason: HpChangeReason::Command,
                    damage_source: Some(DamageSource::Ability),
                    damage_type: Some(DamageType::Physical),
                    raw_damage: Some(200),
                    final_damage: Some(200),
                    damage_breakdown: None,
                    critical: Some(false),
                    feedback_tags: Vec::new(),
                },
            ),
            entry(
                5,
                150,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(second_unit),
                    target_instance_id: enemy,
                    delta: -10,
                    hp_before: 100,
                    hp_after: 90,
                    reason: HpChangeReason::Command,
                    damage_source: Some(DamageSource::Ability),
                    damage_type: Some(DamageType::Physical),
                    raw_damage: Some(10),
                    final_damage: Some(10),
                    damage_breakdown: None,
                    critical: Some(false),
                    feedback_tags: Vec::new(),
                },
            ),
            entry(
                6,
                1_000,
                BattleLogEvent::BattleEnd {
                    winner: BattleWinner::Player,
                },
            ),
        ];
        let participant_results = vec![
            ParticipantBattleResult {
                unit_instance_id: first_unit,
                owned_uuid: first_employee,
                side: Side::Player,
                survived: false,
                final_hp: 0,
                max_hp: 100,
                became_incapacitated: true,
            },
            ParticipantBattleResult {
                unit_instance_id: second_unit,
                owned_uuid: second_employee,
                side: Side::Player,
                survived: true,
                final_hp: 100,
                max_hp: 100,
                became_incapacitated: false,
            },
        ];

        let stats = collect_battle_result_stats(
            BattleWinner::Player,
            &event_log,
            &participant_results,
            &policy(),
        );

        assert_eq!(stats.mvp.as_ref().unwrap().employee_uuid, first_employee);
        assert!(
            stats
                .employees
                .iter()
                .find(|employee| employee.employee_uuid == first_employee)
                .unwrap()
                .became_incapacitated
        );
    }

    #[test]
    fn collector_excludes_player_side_defense_objects_from_employee_stats() {
        let employee = employee_uuid(1);
        let player = unit_id(10);
        let defense_object = unit_id(30);
        let mut event_log = BattleEventLog::new();
        event_log.entries = vec![
            spawn_employee(1, 0, player, employee),
            spawn_defense_object(2, 0, defense_object),
            entry(
                3,
                1_000,
                BattleLogEvent::BattleEnd {
                    winner: BattleWinner::Player,
                },
            ),
        ];
        let participant_results = vec![
            ParticipantBattleResult {
                unit_instance_id: player,
                owned_uuid: employee,
                side: Side::Player,
                survived: true,
                final_hp: 100,
                max_hp: 100,
                became_incapacitated: false,
            },
            ParticipantBattleResult {
                unit_instance_id: defense_object,
                owned_uuid: defense_object.as_uuid(),
                side: Side::Player,
                survived: true,
                final_hp: 350,
                max_hp: 350,
                became_incapacitated: false,
            },
        ];

        let stats = collect_battle_result_stats(
            BattleWinner::Player,
            &event_log,
            &participant_results,
            &policy(),
        );

        assert_eq!(stats.employees.len(), 1);
        assert_eq!(stats.employees[0].employee_uuid, employee);
    }
}
