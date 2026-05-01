use std::cmp::Ordering;

use uuid::Uuid;

use crate::game::ability::SkillId;

use super::buffs::BuffId;
use super::core::movement::types::WorldVec2;
use super::ids::UnitInstanceId;
use super::timeline::{AttackKind, SkillCastTarget, TimelineCause};

/// 전투 이벤트
///
/// 모든 unit/caster/target ID는 `instance_id`를 참조합니다.
/// `base_uuid`(메타데이터 참조용)와 혼동하지 않도록 주의하세요.
#[derive(Debug, Clone, PartialEq)]
pub enum BattleEvent {
    AttackStart {
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        /// Attack 이벤트에 타겟 힌트를 줄 때 사용 (예: ExtraAttack).
        target_instance_id: Option<UnitInstanceId>,
        /// 자동 공격(반복 스케줄) 여부. false면 1회성 공격으로 처리.
        schedule_next: bool,
        cause: TimelineCause,
    },
    AttackResolve {
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        kind: AttackKind,
        cause: TimelineCause,
    },
    BasicAttackProjectileAdvance {
        time_ms: u64,
        projectile_id: Uuid,
        cause: TimelineCause,
    },
    SkillProjectileImpact {
        time_ms: u64,
        delivery_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        impact_position: WorldVec2,
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_vfx_id: Option<String>,
        terminal: bool,
        cause: TimelineCause,
    },
    SkillProjectileAdvance {
        time_ms: u64,
        delivery_id: Uuid,
        cause: TimelineCause,
    },
    SkillAreaTick {
        time_ms: u64,
        area_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        center: WorldVec2,
        cause: TimelineCause,
    },
    SkillAreaExpire {
        time_ms: u64,
        area_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        cause: TimelineCause,
    },
    /// 공명(=마나) 만땅 시 자동 시전 시작
    AutoCastStart {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        cause: TimelineCause,
    },
    /// 자동 시전 종료 훅 (공명 리셋/락 적용)
    AutoCastEnd {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        cause: TimelineCause,
    },
    SkillStep {
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        skill_id: SkillId,
        step_id: String,
        cast_target: Option<SkillCastTarget>,
        cause: TimelineCause,
    },
    ApplyBuff {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        duration_ms: u64,
        cause: TimelineCause,
    },
    BuffTick {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        cause: TimelineCause,
    },
    BuffExpire {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        cause: TimelineCause,
    },
    /// Advance the continuous movement engine by one fixed tick.
    ContinuousMovementTick { time_ms: u64 },
}

// `BattleEvent` needs `Eq` for `Ord`/`BinaryHeap`. Runtime world positions are
// produced by deterministic finite simulation values; NaN positions are invalid.
impl Eq for BattleEvent {}

impl BattleEvent {
    /// 이벤트 발생 시간(ms)
    pub fn time_ms(&self) -> u64 {
        match self {
            BattleEvent::AttackStart { time_ms, .. }
            | BattleEvent::AttackResolve { time_ms, .. }
            | BattleEvent::BasicAttackProjectileAdvance { time_ms, .. }
            | BattleEvent::SkillProjectileAdvance { time_ms, .. }
            | BattleEvent::SkillProjectileImpact { time_ms, .. }
            | BattleEvent::SkillAreaTick { time_ms, .. }
            | BattleEvent::SkillAreaExpire { time_ms, .. }
            | BattleEvent::AutoCastStart { time_ms, .. }
            | BattleEvent::AutoCastEnd { time_ms, .. }
            | BattleEvent::SkillStep { time_ms, .. }
            | BattleEvent::ApplyBuff { time_ms, .. }
            | BattleEvent::BuffTick { time_ms, .. }
            | BattleEvent::BuffExpire { time_ms, .. }
            | BattleEvent::ContinuousMovementTick { time_ms } => *time_ms,
        }
    }

    /// 같은 시각에 여러 이벤트가 있을 때 우선순위
    fn priority(&self) -> u8 {
        match self {
            // Projectile advances first so collision impacts are resolved before new actions.
            BattleEvent::BasicAttackProjectileAdvance { .. } => 0,
            BattleEvent::SkillProjectileAdvance { .. } => 1,
            BattleEvent::SkillProjectileImpact { .. } => 2,
            BattleEvent::SkillAreaTick { .. } => 3,
            // 버프 틱/적용을 먼저 처리하고, 시전 종료, 공격, 시전 시작, 만료 순으로 처리
            BattleEvent::ApplyBuff { .. } => 4,
            BattleEvent::BuffTick { .. } => 5,
            BattleEvent::AutoCastEnd { .. } => 6,
            BattleEvent::SkillStep { .. } => 7,
            BattleEvent::AttackStart { .. } => 8,
            BattleEvent::AttackResolve { .. } => 9,
            BattleEvent::AutoCastStart { .. } => 10,
            BattleEvent::BuffExpire { .. } => 11,
            BattleEvent::SkillAreaExpire { .. } => 12,
            BattleEvent::ContinuousMovementTick { .. } => 13,
        }
    }
}

/// BinaryHeap에서 가장 이른 시간의 이벤트가 먼저 나오도록
/// Ord/PartialOrd를 커스텀 구현한다.
impl Ord for BattleEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap은 기본이 max-heap이므로, 더 작은 time_ms가
        // 먼저 나오게 하려면 순서를 뒤집어서 비교한다.
        other
            .time_ms()
            .cmp(&self.time_ms())
            .then_with(|| other.priority().cmp(&self.priority()))
            .then_with(|| match (self, other) {
                (
                    BattleEvent::SkillProjectileAdvance {
                        delivery_id: a_d, ..
                    },
                    BattleEvent::SkillProjectileAdvance {
                        delivery_id: b_d, ..
                    },
                ) => b_d.as_bytes().cmp(a_d.as_bytes()),
                (
                    BattleEvent::SkillProjectileImpact {
                        delivery_id: a_d,
                        first_hit_unit_id: a_t,
                        ..
                    },
                    BattleEvent::SkillProjectileImpact {
                        delivery_id: b_d,
                        first_hit_unit_id: b_t,
                        ..
                    },
                ) => b_d.as_bytes().cmp(a_d.as_bytes()).then_with(|| {
                    b_t.map(|id| *id.as_bytes())
                        .cmp(&a_t.map(|id| *id.as_bytes()))
                }),
                (
                    BattleEvent::SkillAreaTick {
                        area_id: a_area,
                        step_index: a_step,
                        ..
                    },
                    BattleEvent::SkillAreaTick {
                        area_id: b_area,
                        step_index: b_step,
                        ..
                    },
                ) => b_area
                    .as_bytes()
                    .cmp(a_area.as_bytes())
                    .then_with(|| b_step.cmp(a_step)),
                (
                    BattleEvent::SkillAreaExpire {
                        area_id: a_area,
                        step_index: a_step,
                        ..
                    },
                    BattleEvent::SkillAreaExpire {
                        area_id: b_area,
                        step_index: b_step,
                        ..
                    },
                ) => b_area
                    .as_bytes()
                    .cmp(a_area.as_bytes())
                    .then_with(|| b_step.cmp(a_step)),
                (
                    BattleEvent::AutoCastStart {
                        caster_instance_id: a,
                        ..
                    },
                    BattleEvent::AutoCastStart {
                        caster_instance_id: b,
                        ..
                    },
                )
                | (
                    BattleEvent::AutoCastEnd {
                        caster_instance_id: a,
                        ..
                    },
                    BattleEvent::AutoCastEnd {
                        caster_instance_id: b,
                        ..
                    },
                ) => b.as_bytes().cmp(a.as_bytes()),
                (
                    BattleEvent::SkillStep {
                        cast_seq: a_cast,
                        step_index: a_step,
                        caster_instance_id: a,
                        ..
                    },
                    BattleEvent::SkillStep {
                        cast_seq: b_cast,
                        step_index: b_step,
                        caster_instance_id: b,
                        ..
                    },
                ) => b
                    .as_bytes()
                    .cmp(a.as_bytes())
                    .then_with(|| b_cast.cmp(a_cast))
                    .then_with(|| b_step.cmp(a_step)),
                (
                    BattleEvent::AttackStart {
                        attacker_instance_id: a,
                        ..
                    },
                    BattleEvent::AttackStart {
                        attacker_instance_id: b,
                        ..
                    },
                ) => {
                    let a_t = match self {
                        BattleEvent::AttackStart {
                            target_instance_id, ..
                        } => target_instance_id.map(|id| *id.as_bytes()),
                        _ => None,
                    };
                    let b_t = match other {
                        BattleEvent::AttackStart {
                            target_instance_id, ..
                        } => target_instance_id.map(|id| *id.as_bytes()),
                        _ => None,
                    };
                    b.as_bytes().cmp(a.as_bytes()).then_with(|| b_t.cmp(&a_t))
                }
                (
                    BattleEvent::AttackResolve {
                        attacker_instance_id: a,
                        target_instance_id: a_t,
                        ..
                    },
                    BattleEvent::AttackResolve {
                        attacker_instance_id: b,
                        target_instance_id: b_t,
                        ..
                    },
                ) => b
                    .as_bytes()
                    .cmp(a.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes())),
                (
                    BattleEvent::BasicAttackProjectileAdvance {
                        projectile_id: a_p, ..
                    },
                    BattleEvent::BasicAttackProjectileAdvance {
                        projectile_id: b_p, ..
                    },
                ) => b_p.as_bytes().cmp(a_p.as_bytes()),
                (
                    BattleEvent::ApplyBuff {
                        caster_instance_id: a_c,
                        target_instance_id: a_t,
                        buff_id: a_b,
                        duration_ms: a_d,
                        ..
                    },
                    BattleEvent::ApplyBuff {
                        caster_instance_id: b_c,
                        target_instance_id: b_t,
                        buff_id: b_b,
                        duration_ms: b_d,
                        ..
                    },
                ) => b_c
                    .as_bytes()
                    .cmp(a_c.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes()))
                    .then_with(|| b_b.as_u64().cmp(&a_b.as_u64()))
                    .then_with(|| b_d.cmp(a_d)),
                (
                    BattleEvent::BuffTick {
                        caster_instance_id: a_c,
                        target_instance_id: a_t,
                        buff_id: a_b,
                        ..
                    },
                    BattleEvent::BuffTick {
                        caster_instance_id: b_c,
                        target_instance_id: b_t,
                        buff_id: b_b,
                        ..
                    },
                ) => b_c
                    .as_bytes()
                    .cmp(a_c.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes()))
                    .then_with(|| b_b.as_u64().cmp(&a_b.as_u64())),
                (
                    BattleEvent::BuffExpire {
                        caster_instance_id: a_c,
                        target_instance_id: a_t,
                        buff_id: a_b,
                        ..
                    },
                    BattleEvent::BuffExpire {
                        caster_instance_id: b_c,
                        target_instance_id: b_t,
                        buff_id: b_b,
                        ..
                    },
                ) => b_c
                    .as_bytes()
                    .cmp(a_c.as_bytes())
                    .then_with(|| b_t.as_bytes().cmp(a_t.as_bytes()))
                    .then_with(|| b_b.as_u64().cmp(&a_b.as_u64())),
                _ => Ordering::Equal,
            })
    }
}

impl PartialOrd for BattleEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::timeline::{TimelineCause, TimelineRootCause};
    use std::collections::BinaryHeap;
    use uuid::Uuid;

    #[test]
    fn battle_event_heap_orders_by_time_then_priority_then_ids() {
        let cause = TimelineCause::Root {
            kind: TimelineRootCause::System,
        };

        let attacker_early: UnitInstanceId = Uuid::from_u128(1).into();
        let attacker_late: UnitInstanceId = Uuid::from_u128(2).into();

        let caster_small: UnitInstanceId = Uuid::from_u128(3).into();
        let caster_large: UnitInstanceId = Uuid::from_u128(4).into();
        assert!(caster_small.as_bytes() < caster_large.as_bytes());

        let mut heap = BinaryHeap::new();

        heap.push(BattleEvent::AttackStart {
            time_ms: 9,
            attacker_instance_id: attacker_early,
            target_instance_id: None,
            schedule_next: false,
            cause,
        });

        heap.push(BattleEvent::AttackStart {
            time_ms: 10,
            attacker_instance_id: attacker_late,
            target_instance_id: None,
            schedule_next: false,
            cause,
        });

        heap.push(BattleEvent::BasicAttackProjectileAdvance {
            time_ms: 10,
            projectile_id: Uuid::from_u128(10),
            cause,
        });
        heap.push(BattleEvent::SkillProjectileAdvance {
            time_ms: 10,
            delivery_id: Uuid::from_u128(11),
            cause,
        });

        heap.push(BattleEvent::AutoCastStart {
            time_ms: 10,
            caster_instance_id: caster_large,
            cause,
        });
        heap.push(BattleEvent::AutoCastStart {
            time_ms: 10,
            caster_instance_id: caster_small,
            cause,
        });

        let mut popped = Vec::new();
        while let Some(ev) = heap.pop() {
            popped.push(ev);
        }

        assert!(matches!(
            popped[0],
            BattleEvent::AttackStart { time_ms: 9, .. }
        ));
        assert!(matches!(
            popped[1],
            BattleEvent::BasicAttackProjectileAdvance { .. }
        ));
        assert!(matches!(
            popped[2],
            BattleEvent::SkillProjectileAdvance { .. }
        ));
        assert!(matches!(
            popped[3],
            BattleEvent::AttackStart { time_ms: 10, .. }
        ));
        assert!(matches!(
            popped[4],
            BattleEvent::AutoCastStart {
                caster_instance_id,
                ..
            } if caster_instance_id == caster_small
        ));
        assert!(matches!(
            popped[5],
            BattleEvent::AutoCastStart {
                caster_instance_id,
                ..
            } if caster_instance_id == caster_large
        ));
    }
}
