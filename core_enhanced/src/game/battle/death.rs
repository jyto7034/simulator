use std::collections::{HashSet, VecDeque};

use crate::game::battle::ids::UnitInstanceId;
use crate::game::{battle::cooldown::SourcedEffect, enums::Side, stats::Effect};

use super::damage::BattleCommand;

/// 사망한 유닛 정보
#[derive(Debug, Clone)]
pub struct DeadUnit {
    pub unit_id: UnitInstanceId,
    pub killer_id: Option<UnitInstanceId>,
    pub owner: Side,
}

/// 사망 처리 결과
#[derive(Debug, Clone)]
pub struct DeathProcessResult {
    /// 제거해야 할 유닛 ID들
    pub units_to_remove: Vec<UnitInstanceId>,
    /// 발생한 추가 커맨드들
    pub commands: Vec<BattleCommand>,
}

/// 사망 처리기 - 연쇄 사망 및 트리거 처리
pub struct DeathHandler {
    /// 처리 대기 중인 사망 유닛 큐
    pub pending_deaths: VecDeque<DeadUnit>,
    /// 이미 처리된 유닛 (중복 방지)
    processed: HashSet<UnitInstanceId>,
}

impl DeathHandler {
    pub fn new() -> Self {
        Self {
            pending_deaths: VecDeque::new(),
            processed: HashSet::new(),
        }
    }

    /// 사망 유닛 추가
    pub fn enqueue_death(&mut self, dead_unit: DeadUnit) {
        if !self.processed.contains(&dead_unit.unit_id) {
            self.pending_deaths.push_back(dead_unit);
        }
    }

    /// 대기 중인 사망이 있는지 확인
    pub fn has_pending(&self) -> bool {
        !self.pending_deaths.is_empty()
    }

    /// 모든 사망 처리 실행
    ///
    /// `get_on_death_effects`: unit_id -> Vec<Effect>
    /// `get_on_kill_effects`: unit_id -> Vec<Effect>
    /// `get_on_ally_death_effects`: (unit_id) -> Vec<Effect>
    /// `get_allies`: (dead_unit_id, dead_unit_side) -> Vec<UnitInstanceId>
    pub fn process_all_deaths<G, H, I, J>(
        &mut self,
        mut get_on_death_effects: G,
        mut get_on_kill_effects: H,
        mut get_on_ally_death_effects: I,
        mut get_allies: J,
    ) -> DeathProcessResult
    where
        G: FnMut(UnitInstanceId) -> Vec<SourcedEffect>,
        H: FnMut(UnitInstanceId) -> Vec<SourcedEffect>,
        I: FnMut(UnitInstanceId) -> Vec<SourcedEffect>,
        J: FnMut(UnitInstanceId, Side) -> Vec<UnitInstanceId>,
    {
        let mut units_to_remove = Vec::new();
        let mut commands = Vec::new();

        while let Some(dead) = self.pending_deaths.pop_front() {
            if self.processed.contains(&dead.unit_id) {
                continue;
            }
            self.processed.insert(dead.unit_id);

            // 1. OnDeath 트리거 (사망 유닛)
            let on_death_effects = get_on_death_effects(dead.unit_id);
            for sourced in on_death_effects {
                if let Effect::Modifier(_modifier) = sourced.effect {
                    // OnDeath Modifier는 보통 의미 없지만 일단 무시
                }
            }

            // 2. OnKill 트리거 (킬러)
            if let Some(killer_id) = dead.killer_id {
                let on_kill_effects = get_on_kill_effects(killer_id);
                for sourced in on_kill_effects {
                    if let Effect::Modifier(modifier) = sourced.effect {
                        commands.push(BattleCommand::ApplyModifier {
                            target_id: killer_id,
                            modifier,
                        });
                    }
                }
            }

            // 3. OnAllyDeath 트리거 (같은 편 유닛들)
            let mut allies = get_allies(dead.unit_id, dead.owner);
            allies.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            for ally_id in allies {
                if self.processed.contains(&ally_id) {
                    continue;
                }

                let on_ally_death_effects = get_on_ally_death_effects(ally_id);
                for sourced in on_ally_death_effects {
                    if let Effect::Modifier(modifier) = sourced.effect {
                        commands.push(BattleCommand::ApplyModifier {
                            target_id: ally_id,
                            modifier,
                        });
                    }
                }
            }

            units_to_remove.push(dead.unit_id);
        }

        DeathProcessResult {
            units_to_remove,
            commands,
        }
    }

    /// 처리 상태 초기화
    pub fn reset(&mut self) {
        self.pending_deaths.clear();
        self.processed.clear();
    }
}

impl Default for DeathHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::cooldown::CooldownSource;
    use crate::game::stats::{StatId, StatModifier, StatModifierKind, TriggerEffectTarget};
    use uuid::Uuid;

    #[test]
    fn death_handler_deduplicates_deaths_and_emits_effects_in_sorted_ally_order() {
        let dead_id: UnitInstanceId = Uuid::from_u128(1).into();
        let killer_id: UnitInstanceId = Uuid::from_u128(9).into();
        let ally_a: UnitInstanceId = Uuid::from_u128(2).into();
        let ally_b: UnitInstanceId = Uuid::from_u128(3).into();

        let mut handler = DeathHandler::new();
        handler.enqueue_death(DeadUnit {
            unit_id: dead_id,
            killer_id: Some(killer_id),
            owner: Side::Player,
        });
        // Duplicate enqueue should not double-remove.
        handler.enqueue_death(DeadUnit {
            unit_id: dead_id,
            killer_id: Some(killer_id),
            owner: Side::Player,
        });

        let result = handler.process_all_deaths(
            |_unit_id| vec![],
            |unit_id| {
                if unit_id == killer_id {
                    vec![SourcedEffect {
                        source: CooldownSource::Unit {
                            unit_instance_id: unit_id,
                        },
                        target: TriggerEffectTarget::SelfUnit,
                        effect: Effect::Modifier(StatModifier {
                            stat: StatId::Attack,
                            kind: StatModifierKind::Flat,
                            value: 7,
                        }),
                    }]
                } else {
                    vec![]
                }
            },
            |unit_id| {
                let value = if unit_id == ally_a {
                    1
                } else if unit_id == ally_b {
                    2
                } else {
                    0
                };
                if value == 0 {
                    return vec![];
                }
                vec![SourcedEffect {
                    source: CooldownSource::Unit {
                        unit_instance_id: unit_id,
                    },
                    target: TriggerEffectTarget::SelfUnit,
                    effect: Effect::Modifier(StatModifier {
                        stat: StatId::Defense,
                        kind: StatModifierKind::Flat,
                        value,
                    }),
                }]
            },
            |unit_id, side| {
                assert_eq!(unit_id, dead_id);
                assert_eq!(side, Side::Player);
                // Intentionally return in reverse order to ensure sorting is applied.
                vec![ally_b, ally_a]
            },
        );

        assert_eq!(result.units_to_remove, vec![dead_id]);

        // Killer commands should include modifier.
        assert!(
            result
                .commands
                .iter()
                .any(|c| matches!(c, BattleCommand::ApplyModifier { target_id, modifier } if *target_id == killer_id && modifier.stat == StatId::Attack))
        );

        // Ally on-ally-death effects should be ordered by ally id (ascending bytes).
        let ally_modifier_targets: Vec<UnitInstanceId> = result
            .commands
            .iter()
            .filter_map(|c| match c {
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier,
                } if modifier.stat == StatId::Defense => Some(*target_id),
                _ => None,
            })
            .collect();
        assert_eq!(ally_modifier_targets, vec![ally_a, ally_b]);
    }
}
