use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::SkillId,
        battle::{
            buffs::BuffId,
            cooldown::CooldownSource,
            core::movement::{ActionState, MovementState},
            ids::UnitInstanceId,
            timeline::{SkillCastTarget, TimelineCause},
            types::UnitSnapshot,
        },
        enums::Side,
        stats::UnitStats,
    },
};

#[derive(Debug, Clone)]
pub struct ProjectileRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionLocks {
    pub movement_until_ms: u64,
    pub basic_attack_until_ms: u64,
    pub resonance_gain_until_ms: u64,
}

impl ActionLocks {
    pub fn can_move(&self, now_ms: u64) -> bool {
        now_ms >= self.movement_until_ms
    }

    pub fn can_basic_attack(&self, now_ms: u64) -> bool {
        now_ms >= self.basic_attack_until_ms
    }

    pub fn can_gain_resonance(&self, now_ms: u64) -> bool {
        now_ms >= self.resonance_gain_until_ms
    }

    pub fn lock_movement_until(&mut self, until_ms: u64) {
        self.movement_until_ms = self.movement_until_ms.max(until_ms);
    }

    pub fn lock_basic_attack_until(&mut self, until_ms: u64) {
        self.basic_attack_until_ms = self.basic_attack_until_ms.max(until_ms);
    }

    pub fn lock_resonance_gain_until(&mut self, until_ms: u64) {
        self.resonance_gain_until_ms = self.resonance_gain_until_ms.max(until_ms);
    }
}

/// 전투 중 사용되는 아티팩트 런타임 표현
#[derive(Debug, Clone)]
pub(super) struct RuntimeArtifact {
    pub(super) instance_id: Uuid,
    pub(super) owner: Side,
    pub(super) base_uuid: Uuid,
}

/// 전투 중 사용되는 장비 런타임 표현
#[derive(Debug, Clone)]
pub(super) struct RuntimeItem {
    pub(super) instance_id: Uuid,
    pub(super) owner: Side,
    pub(super) owner_unit_instance: UnitInstanceId,
    pub(super) base_uuid: Uuid,
}

/// 트리거 수집 시 소스 구분
#[derive(Debug, Clone, Copy)]
pub(super) enum TriggerSource {
    Artifact { side: Side },
    Item { unit_instance_id: UnitInstanceId },
}

#[derive(Debug, Clone)]
pub(super) struct PendingSkillCast {
    pub(super) skill_id: SkillId,
    pub(super) cast_target: Option<SkillCastTarget>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct SkillStepResult {
    pub(super) resolved_target_count: usize,
    pub(super) damage_target_count: usize,
    pub(super) applied_effect_count: usize,
    pub(super) scheduled_attack_count: usize,
}

impl SkillStepResult {
    pub(super) fn merge(&mut self, other: &Self) {
        self.resolved_target_count = self
            .resolved_target_count
            .saturating_add(other.resolved_target_count);
        self.damage_target_count = self
            .damage_target_count
            .saturating_add(other.damage_target_count);
        self.applied_effect_count = self
            .applied_effect_count
            .saturating_add(other.applied_effect_count);
        self.scheduled_attack_count = self
            .scheduled_attack_count
            .saturating_add(other.scheduled_attack_count);
    }

    pub(super) fn dealt_damage(&self) -> bool {
        self.damage_target_count > 0
    }
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedSkillStep {
    pub(super) step_index: usize,
    pub(super) result: SkillStepResult,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveSkillCast {
    pub(super) caster_owner: Side,
    pub(super) anchor_position: Position,
    pub(super) allow_dead_caster: bool,
    pub(super) last_resolved_step: Option<ResolvedSkillStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct AbilityProcKey {
    pub(super) source: CooldownSource,
    pub(super) ability_id: SkillId,
    pub(super) binding_index: usize,
}

#[derive(Debug, Clone, Default)]
pub(super) struct AbilityProcState {
    pub(super) trigger_count: u32,
    pub(super) next_ready_ms: u64,
}

pub struct RuntimeUnit {
    pub instance_id: UnitInstanceId,
    pub owner: Side,
    pub base_uuid: Uuid,
    pub stats: UnitStats,
    pub pos_x_units: i64,
    pub pos_y_units: i64,
    pub move_epoch: u32,
    pub action_state: ActionState,
    pub action_locks: ActionLocks,
    pub current_target: Option<UnitInstanceId>,
    pub next_basic_attack_ms: u64,
    pub pending_basic_attack: bool,
    pub resonance_current: u32,
    pub resonance_max: u32,
    pub resonance_lock_ms: u64,
    pub next_action_time: u64,
    pub pending_cast: bool,
    pub pending_cast_cause: Option<TimelineCause>,
    pub(super) pending_skill_cast: Option<PendingSkillCast>,
}

impl RuntimeUnit {
    /// UnitSnapshot 생성
    pub(super) fn to_snapshot(&self, position: Position) -> UnitSnapshot {
        UnitSnapshot {
            id: self.instance_id,
            owner: self.owner,
            position,
            stats: self.stats,
        }
    }

    pub fn new_action_state_at(
        position: Position,
        now_ms: u64,
        _speed_units_per_ms: u32,
    ) -> ActionState {
        ActionState::Moving(MovementState::new_at(position, now_ms))
    }

    pub fn is_dead(&self) -> bool {
        self.stats.current_health == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct BuffInstanceKey {
    pub(super) caster_instance_id: UnitInstanceId,
    pub(super) target_instance_id: UnitInstanceId,
    pub(super) buff_id: BuffId,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveBuff {
    pub(super) stacks: u8,
    pub(super) expires_at_ms: u64,
    pub(super) next_tick_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_locks_lock_methods_take_max_and_gate_actions() {
        let mut locks = ActionLocks::default();
        assert!(locks.can_move(0));
        assert!(locks.can_basic_attack(0));
        assert!(locks.can_gain_resonance(0));

        locks.lock_movement_until(10);
        locks.lock_movement_until(5);
        assert!(!locks.can_move(9));
        assert!(locks.can_move(10));

        locks.lock_basic_attack_until(20);
        locks.lock_basic_attack_until(15);
        assert!(!locks.can_basic_attack(19));
        assert!(locks.can_basic_attack(20));

        locks.lock_resonance_gain_until(30);
        locks.lock_resonance_gain_until(25);
        assert!(!locks.can_gain_resonance(29));
        assert!(locks.can_gain_resonance(30));
    }
}
