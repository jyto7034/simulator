use std::cmp::Ordering;

use uuid::Uuid;

/// 전투 이벤트
///
/// 모든 unit/caster/target ID는 `instance_id`를 참조합니다.
/// `base_uuid`(메타데이터 참조용)와 혼동하지 않도록 주의하세요.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BattleEvent {
    Attack {
        time_ms: u64,
        attacker_instance_id: Uuid,
    },
    ApplyBuff {
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: Uuid,
    },
    BuffTick {
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: Uuid,
    },
    BuffExpire {
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: Uuid,
    },
}

impl BattleEvent {
    /// 이벤트 발생 시간(ms)
    pub fn time_ms(&self) -> u64 {
        match self {
            BattleEvent::Attack { time_ms, .. }
            | BattleEvent::ApplyBuff { time_ms, .. }
            | BattleEvent::BuffTick { time_ms, .. }
            | BattleEvent::BuffExpire { time_ms, .. } => *time_ms,
        }
    }

    /// 같은 시각에 여러 이벤트가 있을 때 우선순위
    fn priority(&self) -> u8 {
        match self {
            // 버프 틱/적용을 먼저 처리하고, 공격, 만료 순으로 처리
            BattleEvent::ApplyBuff { .. } => 1,
            BattleEvent::BuffTick { .. } => 2,
            BattleEvent::Attack { .. } => 3,
            BattleEvent::BuffExpire { .. } => 4,
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
    }
}

impl PartialOrd for BattleEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
