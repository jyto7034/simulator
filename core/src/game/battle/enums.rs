use std::cmp::Ordering;

use uuid::Uuid;

use super::buffs::BuffId;

/// 전투 이벤트
///
/// 모든 unit/caster/target ID는 `instance_id`를 참조합니다.
/// `base_uuid`(메타데이터 참조용)와 혼동하지 않도록 주의하세요.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BattleEvent {
    Attack {
        time_ms: u64,
        attacker_instance_id: Uuid,
        /// Attack 이벤트에 타겟 힌트를 줄 때 사용 (예: ExtraAttack).
        target_instance_id: Option<Uuid>,
        /// 자동 공격(반복 스케줄) 여부. false면 1회성 공격으로 처리.
        schedule_next: bool,
        cause_seq: Option<u64>,
    },
    /// 공명(=마나) 만땅 시 자동 시전 시작
    AutoCastStart {
        time_ms: u64,
        caster_instance_id: Uuid,
        cause_seq: Option<u64>,
    },
    /// 자동 시전 종료 훅 (공명 리셋/락 적용)
    AutoCastEnd {
        time_ms: u64,
        caster_instance_id: Uuid,
        cause_seq: Option<u64>,
    },
    ApplyBuff {
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        duration_ms: u64,
        cause_seq: Option<u64>,
    },
    BuffTick {
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        cause_seq: Option<u64>,
    },
    BuffExpire {
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        cause_seq: Option<u64>,
    },
}

impl BattleEvent {
    /// 이벤트 발생 시간(ms)
    pub fn time_ms(&self) -> u64 {
        match self {
            BattleEvent::Attack { time_ms, .. }
            | BattleEvent::AutoCastStart { time_ms, .. }
            | BattleEvent::AutoCastEnd { time_ms, .. }
            | BattleEvent::ApplyBuff { time_ms, .. }
            | BattleEvent::BuffTick { time_ms, .. }
            | BattleEvent::BuffExpire { time_ms, .. } => *time_ms,
        }
    }

    /// 같은 시각에 여러 이벤트가 있을 때 우선순위
    fn priority(&self) -> u8 {
        match self {
            // 버프 틱/적용을 먼저 처리하고, 시전 종료, 공격, 시전 시작, 만료 순으로 처리
            BattleEvent::ApplyBuff { .. } => 1,
            BattleEvent::BuffTick { .. } => 2,
            BattleEvent::AutoCastEnd { .. } => 3,
            BattleEvent::Attack { .. } => 4,
            BattleEvent::AutoCastStart { .. } => 5,
            BattleEvent::BuffExpire { .. } => 6,
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
                    BattleEvent::Attack {
                        attacker_instance_id: a,
                        ..
                    },
                    BattleEvent::Attack {
                        attacker_instance_id: b,
                        ..
                    },
                ) => {
                    let a_t = match self {
                        BattleEvent::Attack {
                            target_instance_id, ..
                        } => target_instance_id.map(|id| *id.as_bytes()),
                        _ => None,
                    };
                    let b_t = match other {
                        BattleEvent::Attack {
                            target_instance_id, ..
                        } => target_instance_id.map(|id| *id.as_bytes()),
                        _ => None,
                    };
                    b.as_bytes().cmp(a.as_bytes()).then_with(|| b_t.cmp(&a_t))
                }
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
