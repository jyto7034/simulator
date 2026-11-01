use crate::owned_collections::OwnedHashMap;
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

#[derive(Debug, Clone)]
pub(crate) struct BattleResultStatsAccumulator {
    employee_participants_possible: bool,
    unit_sources: OwnedHashMap<UnitInstanceId, UnitSourceInfo>,
    employees: OwnedHashMap<Uuid, EmployeeAccumulator>,
    active_deployments: OwnedHashMap<UnitInstanceId, (Uuid, u64)>,
    duration_ms: u64,
    killed_enemy_count: u32,
}

crate::authoring::clone_shape::struct_shape!(BattleResultStatsAccumulator {
    employee_participants_possible,
    unit_sources,
    employees,
    active_deployments,
    duration_ms,
    killed_enemy_count,
});
crate::authoring::clone_shape::copy_shape!(UnitSourceInfo);
crate::authoring::clone_shape::struct_shape!(EmployeeAccumulator { stats });
crate::authoring::clone_shape::struct_shape!(BattleResultEmployeeStatsDto {
    employee_uuid,
    unit_instance_id,
    survived,
    became_incapacitated,
    final_hp,
    max_hp,
    damage_dealt,
    damage_taken,
    kill_count,
    basic_attack_count,
    skill_cast_count,
    deployed_count,
    withdrawn_count,
    deployed_time_ms,
});

impl Default for BattleResultStatsAccumulator {
    fn default() -> Self {
        Self::new(true)
    }
}

impl BattleResultStatsAccumulator {
    pub(crate) fn new(employee_participants_possible: bool) -> Self {
        Self {
            employee_participants_possible,
            unit_sources: OwnedHashMap::new(),
            employees: OwnedHashMap::new(),
            active_deployments: OwnedHashMap::new(),
            duration_ms: 0,
            killed_enemy_count: 0,
        }
    }

    #[inline]
    pub(crate) fn should_observe(&self, event: &BattleLogEvent) -> bool {
        self.employee_participants_possible
            || matches!(
                event,
                BattleLogEvent::UnitDied { .. } | BattleLogEvent::BattleEnd { .. }
            )
    }

    pub(crate) fn observe(&mut self, entry: &crate::game::battle::event_log::BattleEventLogEntry) {
        debug_assert!(self.should_observe(&entry.event));
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
            self.unit_sources.insert(
                *unit_instance_id,
                UnitSourceInfo {
                    owner: *owner,
                    employee_uuid,
                },
            );
            if let Some(employee_uuid) = employee_uuid {
                self.employees
                    .entry(employee_uuid)
                    .and_modify(|employee| employee.stats.unit_instance_id = *unit_instance_id)
                    .or_insert_with(|| EmployeeAccumulator::new(employee_uuid, *unit_instance_id));
            }
        }
        if self.employees.is_empty()
            && matches!(
                &entry.event,
                BattleLogEvent::AttackResolve { .. }
                    | BattleLogEvent::AbilityCast { .. }
                    | BattleLogEvent::HpChanged { .. }
            )
        {
            return;
        }
        match &entry.event {
            BattleLogEvent::UnitDeployed {
                employee_uuid,
                unit_instance_id,
                ..
            } => {
                let employee = self
                    .employees
                    .entry(*employee_uuid)
                    .or_insert_with(|| EmployeeAccumulator::new(*employee_uuid, *unit_instance_id));
                employee.stats.unit_instance_id = *unit_instance_id;
                employee.stats.deployed_count = employee.stats.deployed_count.saturating_add(1);
                self.active_deployments
                    .insert(*unit_instance_id, (*employee_uuid, entry.time_ms));
            }
            BattleLogEvent::UnitWithdrawn {
                unit_instance_id, ..
            } => {
                if let Some(employee_uuid) = self
                    .unit_sources
                    .get(unit_instance_id)
                    .and_then(|source| source.employee_uuid)
                {
                    if let Some(employee) = self.employees.get_mut(&employee_uuid) {
                        employee.stats.withdrawn_count =
                            employee.stats.withdrawn_count.saturating_add(1);
                    }
                }
                close_deployment(
                    &mut self.employees,
                    &mut self.active_deployments,
                    *unit_instance_id,
                    entry.time_ms,
                );
            }
            BattleLogEvent::AttackResolve {
                attacker_instance_id,
                ..
            } => {
                if let Some(employee_uuid) =
                    direct_employee_uuid(&self.unit_sources, attacker_instance_id)
                {
                    if let Some(employee) = self.employees.get_mut(&employee_uuid) {
                        employee.stats.basic_attack_count =
                            employee.stats.basic_attack_count.saturating_add(1);
                    }
                }
            }
            BattleLogEvent::AbilityCast {
                caster_instance_id, ..
            } => {
                if let Some(employee_uuid) =
                    direct_employee_uuid(&self.unit_sources, caster_instance_id)
                {
                    if let Some(employee) = self.employees.get_mut(&employee_uuid) {
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
                reason,
                ..
            } => {
                if *reason == crate::game::battle::event_log::HpChangeReason::HealthCost {
                    return;
                }
                let actual_damage = hp_before.saturating_sub(*hp_after);
                if actual_damage == 0 {
                    return;
                }
                if let Some(target_employee_uuid) =
                    direct_employee_uuid(&self.unit_sources, target_instance_id)
                {
                    if let Some(employee) = self.employees.get_mut(&target_employee_uuid) {
                        employee.stats.damage_taken =
                            employee.stats.damage_taken.saturating_add(actual_damage);
                    }
                }
                let Some(source_instance_id) = source_instance_id else {
                    return;
                };
                if !is_first_slice_damage_source(*damage_source) {
                    return;
                }
                let Some(source_employee_uuid) =
                    direct_employee_uuid(&self.unit_sources, source_instance_id)
                else {
                    return;
                };
                let target_owner = self
                    .unit_sources
                    .get(target_instance_id)
                    .map(|source| source.owner);
                if target_owner == Some(Side::Player) {
                    return;
                }
                if let Some(employee) = self.employees.get_mut(&source_employee_uuid) {
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
                    self.killed_enemy_count = self.killed_enemy_count.saturating_add(1);
                }
                if let Some(killer_instance_id) = killer_instance_id {
                    if *owner != Side::Player {
                        if let Some(employee_uuid) =
                            direct_employee_uuid(&self.unit_sources, killer_instance_id)
                        {
                            if let Some(employee) = self.employees.get_mut(&employee_uuid) {
                                employee.stats.kill_count =
                                    employee.stats.kill_count.saturating_add(1);
                            }
                        }
                    }
                }
                close_deployment(
                    &mut self.employees,
                    &mut self.active_deployments,
                    *unit_instance_id,
                    entry.time_ms,
                );
            }
            BattleLogEvent::BattleEnd { .. } => {
                self.duration_ms = entry.time_ms;
                let open_deployments = self.active_deployments.keys().copied().collect::<Vec<_>>();
                for unit_instance_id in open_deployments {
                    close_deployment(
                        &mut self.employees,
                        &mut self.active_deployments,
                        unit_instance_id,
                        entry.time_ms,
                    );
                }
            }
            _ => {}
        }
    }

    pub(crate) fn finish(
        mut self,
        winner: BattleWinner,
        participant_results: &[ParticipantBattleResult],
        policy: &BattleResultStatsPolicy,
    ) -> BattleResultStatsDto {
        let participant_results_by_unit = participant_results
            .iter()
            .map(|participant| (participant.unit_instance_id, participant))
            .collect::<HashMap<_, _>>();
        for participant in participant_results
            .iter()
            .filter(|participant| participant.side == Side::Player)
        {
            let Some(employee_uuid) =
                direct_employee_uuid(&self.unit_sources, &participant.unit_instance_id)
            else {
                continue;
            };
            let employee = self.employees.entry(employee_uuid).or_insert_with(|| {
                EmployeeAccumulator::new(employee_uuid, participant.unit_instance_id)
            });
            employee.stats.became_incapacitated |= participant.became_incapacitated;
        }

        for employee in self.employees.values_mut() {
            let Some(participant) =
                participant_results_by_unit.get(&employee.stats.unit_instance_id)
            else {
                continue;
            };
            employee.stats.survived = participant.survived;
            employee.stats.final_hp = participant.final_hp;
            employee.stats.max_hp = participant.max_hp;
        }

        let mut employees = self
            .employees
            .into_values()
            .map(|accumulator| accumulator.stats)
            .collect::<Vec<_>>();
        employees.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        let mvp = select_mvp(&employees, &policy.mvp);

        BattleResultStatsDto {
            battle: BattleResultBattleStatsDto {
                duration_ms: self.duration_ms,
                winner,
                killed_enemy_count: self.killed_enemy_count,
            },
            employees,
            mvp,
        }
    }
}

pub fn collect_battle_result_stats(
    winner: BattleWinner,
    event_log: &BattleEventLog,
    participant_results: &[ParticipantBattleResult],
    policy: &BattleResultStatsPolicy,
) -> BattleResultStatsDto {
    let mut accumulator = BattleResultStatsAccumulator::default();
    for entry in &event_log.entries {
        accumulator.observe(entry);
    }
    accumulator.finish(winner, participant_results, policy)
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
    employees: &mut OwnedHashMap<Uuid, EmployeeAccumulator>,
    active_deployments: &mut OwnedHashMap<UnitInstanceId, (Uuid, u64)>,
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

struct MvpScoreEvaluation {
    total: i64,
    highlight: Option<BattleResultMetricHighlightDto>,
}

struct EvaluatedMvpCandidate<'a> {
    employee: &'a BattleResultEmployeeStatsDto,
    score: MvpScoreEvaluation,
}

fn select_mvp(
    employees: &[BattleResultEmployeeStatsDto],
    policy: &BattleResultMvpPolicy,
) -> Option<BattleResultMvpDto> {
    employees
        .iter()
        .map(|employee| EvaluatedMvpCandidate {
            employee,
            score: evaluate_mvp_score(employee, &policy.metric_weights),
        })
        .max_by(|left, right| {
            compare_mvp_candidates(
                left.employee,
                left.score.total,
                right.employee,
                right.score.total,
                policy,
            )
        })
        .map(|candidate| {
            let employee = candidate.employee;
            let MvpScoreEvaluation { total, highlight } = candidate.score;
            BattleResultMvpDto {
                employee_uuid: employee.employee_uuid,
                unit_instance_id: employee.unit_instance_id,
                score: total,
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
    left_score: i64,
    right: &BattleResultEmployeeStatsDto,
    right_score: i64,
    policy: &BattleResultMvpPolicy,
) -> Ordering {
    for tiebreaker in &policy.tiebreakers {
        let ordering = match tiebreaker {
            BattleResultMvpTieBreaker::Score => left_score.cmp(&right_score),
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

fn evaluate_mvp_score(
    employee: &BattleResultEmployeeStatsDto,
    weights: &[BattleResultMvpMetricWeight],
) -> MvpScoreEvaluation {
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
    MvpScoreEvaluation { total, highlight }
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
            skill_effect_source: None,
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

    fn employee_stats(
        id: u128,
        damage_dealt: u32,
        kill_count: u32,
    ) -> BattleResultEmployeeStatsDto {
        BattleResultEmployeeStatsDto {
            employee_uuid: employee_uuid(id),
            unit_instance_id: unit_id(100 + id),
            survived: true,
            became_incapacitated: false,
            final_hp: 100,
            max_hp: 100,
            damage_dealt,
            damage_taken: 0,
            kill_count,
            basic_attack_count: 0,
            skill_cast_count: 0,
            deployed_count: 1,
            withdrawn_count: 0,
            deployed_time_ms: 1_000,
        }
    }

    #[test]
    fn mvp_selection_and_publication_share_positive_saturated_score() {
        let policy = BattleResultMvpPolicy {
            metric_weights: vec![
                BattleResultMvpMetricWeight {
                    metric: BattleResultMetricKind::DamageDealt,
                    weight: i64::MAX,
                    title: "Extreme Damage".to_string(),
                },
                BattleResultMvpMetricWeight {
                    metric: BattleResultMetricKind::KillCount,
                    weight: i64::MAX,
                    title: "Extreme Kills".to_string(),
                },
            ],
            tiebreakers: vec![
                BattleResultMvpTieBreaker::Score,
                BattleResultMvpTieBreaker::DamageDealt,
                BattleResultMvpTieBreaker::EmployeeUuid,
            ],
        };
        let employees = [employee_stats(1, 1, 1), employee_stats(2, 0, 1)];

        let mvp = select_mvp(&employees, &policy).expect("MVP candidate");

        assert_eq!(mvp.employee_uuid, employee_uuid(1));
        assert_eq!(mvp.score, i64::MAX);
        assert_eq!(mvp.title, "Extreme Damage");
        assert_eq!(mvp.highlighted_metrics.len(), 1);
        assert_eq!(
            mvp.highlighted_metrics[0],
            BattleResultMetricHighlightDto {
                metric: BattleResultMetricKind::DamageDealt,
                value: 1,
                weighted_score: i64::MAX,
                title: "Extreme Damage".to_string(),
            }
        );
    }

    #[test]
    fn mvp_selection_and_publication_share_negative_saturated_score() {
        let policy = BattleResultMvpPolicy {
            metric_weights: vec![
                BattleResultMvpMetricWeight {
                    metric: BattleResultMetricKind::DamageDealt,
                    weight: i64::MIN,
                    title: "Negative Damage".to_string(),
                },
                BattleResultMvpMetricWeight {
                    metric: BattleResultMetricKind::KillCount,
                    weight: i64::MIN,
                    title: "Negative Kills".to_string(),
                },
            ],
            tiebreakers: vec![
                BattleResultMvpTieBreaker::Score,
                BattleResultMvpTieBreaker::EmployeeUuid,
            ],
        };
        let employees = [employee_stats(1, 1, 1), employee_stats(2, 0, 0)];

        let mvp = select_mvp(&employees, &policy).expect("MVP candidate");

        assert_eq!(mvp.employee_uuid, employee_uuid(2));
        assert_eq!(mvp.score, 0);
        assert_eq!(mvp.title, "Negative Damage");
        assert_eq!(mvp.highlighted_metrics.len(), 1);
        assert_eq!(
            mvp.highlighted_metrics[0],
            BattleResultMetricHighlightDto {
                metric: BattleResultMetricKind::DamageDealt,
                value: 0,
                weighted_score: 0,
                title: "Negative Damage".to_string(),
            }
        );
    }

    #[test]
    fn mvp_uuid_fallback_is_deterministic_for_equal_candidates() {
        let first = employee_stats(1, 0, 0);
        let second = employee_stats(2, 0, 0);
        let policy = policy().mvp;

        let forward =
            select_mvp(&[first.clone(), second.clone()], &policy).expect("forward MVP candidate");
        let reverse = select_mvp(&[second, first], &policy).expect("reverse MVP candidate");

        assert_eq!(forward.employee_uuid, employee_uuid(1));
        assert_eq!(reverse.employee_uuid, employee_uuid(1));
        assert_eq!(forward.score, 50);
        assert_eq!(reverse.score, 50);
        assert_eq!(forward.title, "Survivor");
        assert_eq!(reverse.title, "Survivor");
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
        ]
        .into();
        let participant_results = vec![ParticipantBattleResult {
            unit_instance_id: player,
            owned_uuid: employee,
            side: Side::Player,
            survived: true,
            final_hp: 55,
            max_hp: 100,
            became_incapacitated: false,
        }];

        let collected_stats = collect_battle_result_stats(
            BattleWinner::Player,
            &event_log,
            &participant_results,
            &policy(),
        );
        let mut accumulator = BattleResultStatsAccumulator::default();
        for event in &event_log.entries {
            accumulator.observe(event);
        }
        event_log.entries.clear();
        let stats = accumulator.finish(BattleWinner::Player, &participant_results, &policy());

        assert!(event_log.entries.is_empty());
        assert_eq!(stats, collected_stats);

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
    fn collector_aggregates_redeploy_instances_into_one_chronological_employee_result() {
        let employee = employee_uuid(1);
        let first_unit = unit_id(10);
        let second_unit = unit_id(11);
        let mut event_log = BattleEventLog::new();
        event_log.entries = vec![
            spawn_employee(1, 0, first_unit, employee),
            entry(
                2,
                0,
                BattleLogEvent::UnitDeployed {
                    employee_uuid: employee,
                    unit_instance_id: first_unit,
                    position: Position::new(1, 1),
                    facing: FacingDirection::Right,
                },
            ),
            entry(
                3,
                100,
                BattleLogEvent::UnitDied {
                    unit_instance_id: first_unit,
                    owner: Side::Player,
                    killer_instance_id: None,
                    world_position: EventLogVec2::default(),
                    position: Position::new(1, 1),
                },
            ),
            spawn_employee(4, 200, second_unit, employee),
            entry(
                5,
                200,
                BattleLogEvent::UnitDeployed {
                    employee_uuid: employee,
                    unit_instance_id: second_unit,
                    position: Position::new(1, 1),
                    facing: FacingDirection::Right,
                },
            ),
            entry(
                6,
                500,
                BattleLogEvent::BattleEnd {
                    winner: BattleWinner::Player,
                },
            ),
        ]
        .into();
        let participant_results = vec![
            ParticipantBattleResult {
                unit_instance_id: second_unit,
                owned_uuid: employee,
                side: Side::Player,
                survived: true,
                final_hp: 37,
                max_hp: 80,
                became_incapacitated: false,
            },
            ParticipantBattleResult {
                unit_instance_id: first_unit,
                owned_uuid: employee,
                side: Side::Player,
                survived: false,
                final_hp: 0,
                max_hp: 100,
                became_incapacitated: true,
            },
        ];

        let stats = collect_battle_result_stats(
            BattleWinner::Player,
            &event_log,
            &participant_results,
            &policy(),
        );

        assert_eq!(stats.employees.len(), 1);
        let employee_stats = &stats.employees[0];
        assert_eq!(employee_stats.employee_uuid, employee);
        assert_eq!(employee_stats.unit_instance_id, second_unit);
        assert_eq!(employee_stats.deployed_count, 2);
        assert_eq!(employee_stats.final_hp, 37);
        assert_eq!(employee_stats.max_hp, 80);
        assert!(employee_stats.survived);
        assert!(employee_stats.became_incapacitated);
        assert_eq!(stats.mvp.as_ref().unwrap().unit_instance_id, second_unit);
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
        ]
        .into();
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
        ]
        .into();
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
