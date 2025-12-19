use std::collections::{HashSet, VecDeque};

use uuid::Uuid;

use crate::game::{enums::Side, stats::Effect};

use super::damage::BattleCommand;

/// 사망한 유닛 정보
#[derive(Debug, Clone)]
pub struct DeadUnit {
    pub unit_id: Uuid,
    pub killer_id: Option<Uuid>,
    pub owner: Side,
}

/// 사망 처리 결과
#[derive(Debug, Clone)]
pub struct DeathProcessResult {
    /// 제거해야 할 유닛 ID들
    pub units_to_remove: Vec<Uuid>,
    /// 발생한 추가 커맨드들
    pub commands: Vec<BattleCommand>,
}

/// 사망 처리기 - 연쇄 사망 및 트리거 처리
pub struct DeathHandler {
    /// 처리 대기 중인 사망 유닛 큐
    pub pending_deaths: VecDeque<DeadUnit>,
    /// 이미 처리된 유닛 (중복 방지)
    processed: HashSet<Uuid>,
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
    /// `get_allies`: (dead_unit_id, dead_unit_side) -> Vec<Uuid>
    pub fn process_all_deaths<G, H, I, J>(
        &mut self,
        mut get_on_death_effects: G,
        mut get_on_kill_effects: H,
        mut get_on_ally_death_effects: I,
        mut get_allies: J,
    ) -> DeathProcessResult
    where
        G: FnMut(Uuid) -> Vec<Effect>,
        H: FnMut(Uuid) -> Vec<Effect>,
        I: FnMut(Uuid) -> Vec<Effect>,
        J: FnMut(Uuid, Side) -> Vec<Uuid>,
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
            for effect in on_death_effects {
                match effect {
                    // 죽은 유닛은 행동(스킬 시전)을 하지 않는다.
                    Effect::Ability(_) => {}
                    Effect::Modifier(_modifier) => {
                        // OnDeath Modifier는 보통 의미 없지만 일단 무시
                    }
                    _ => {}
                }
            }

            // 2. OnKill 트리거 (킬러)
            if let Some(killer_id) = dead.killer_id {
                let on_kill_effects = get_on_kill_effects(killer_id);
                for effect in on_kill_effects {
                    match effect {
                        Effect::Modifier(modifier) => {
                            commands.push(BattleCommand::ApplyModifier {
                                target_id: killer_id,
                                modifier,
                            });
                        }
                        Effect::Ability(ability_id) => {
                            commands.push(BattleCommand::ExecuteAbility {
                                ability_id,
                                caster_id: killer_id,
                                target_id: Some(dead.unit_id),
                            });
                        }
                        _ => {}
                    }
                }
            }

            // 3. OnAllyDeath 트리거 (같은 편 유닛들)
            let allies = get_allies(dead.unit_id, dead.owner);
            for ally_id in allies {
                if self.processed.contains(&ally_id) {
                    continue;
                }

                let on_ally_death_effects = get_on_ally_death_effects(ally_id);
                for effect in on_ally_death_effects {
                    match effect {
                        Effect::Modifier(modifier) => {
                            commands.push(BattleCommand::ApplyModifier {
                                target_id: ally_id,
                                modifier,
                            });
                        }
                        Effect::Ability(ability_id) => {
                            commands.push(BattleCommand::ExecuteAbility {
                                ability_id,
                                caster_id: ally_id,
                                target_id: None,
                            });
                        }
                        _ => {}
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
