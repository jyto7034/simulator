use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use uuid::Uuid;

pub mod build;
pub mod commands;
pub mod ids;
pub mod movement;
pub mod sim;
pub mod triggers;

use crate::{
    ecs::resources::Position,
    game::{
        battle::{
            buffs::BuffId,
            enums::BattleEvent,
            runtime_field::RuntimeField,
            timeline::{Timeline, TimelineCause},
            types::{PlayerDeckInfo, UnitSnapshot},
        },
        data::GameDataBase,
        enums::Side,
        stats::UnitStats,
    },
};

#[derive(Debug, Clone)]
struct ProjectileRecord {
    fired_at_ms: u64,
}

/// 전투 중 사용되는 아티팩트 런타임 표현
#[derive(Debug, Clone)]
struct RuntimeArtifact {
    instance_id: Uuid,
    owner: Side,
    base_uuid: Uuid,
}

/// 전투 중 사용되는 장비 런타임 표현
#[derive(Debug, Clone)]
struct RuntimeItem {
    instance_id: Uuid,
    owner: Side,
    owner_unit_instance: Uuid,
    base_uuid: Uuid,
}

/// 트리거 수집 시 소스 구분
#[derive(Debug, Clone, Copy)]
enum TriggerSource {
    Artifact { side: Side },
    Item { unit_instance_id: Uuid },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveState {
    /// 공격 가능 여부 확인 및 이동 계획 수립
    Acquire,
    /// 목적지 예약 + 경로를 따라 이동 중
    Moving,
    /// 목적지/경로를 찾지 못해 재탐색 대기
    WaitRepath {
        until_ms: u64,
    },
    /// 빙결/기절 등 하드 CC로 모든 행동 정지 (until_ms까지)
    CCLocked {
        until_ms: u64,
    },
    /// 하드 CC 해제 시점에 주변이 막혀 이동이 불가능할 때,
    /// 모든 유닛(아군/적군)과 예약을 통과할 수 있는 임시 탈출 상태.
    Ghost,
    Dead,
}

struct RuntimeUnit {
    instance_id: Uuid,
    owner: Side,
    base_uuid: Uuid,
    stats: UnitStats,
    position: Position,
    current_target: Option<Uuid>,
    resonance_current: u32,
    resonance_max: u32,
    resonance_lock_ms: u64,
    resonance_gain_locked_until_ms: u64,
    next_action_time: u64,
    pending_cast: bool,
    pending_cast_cause: Option<TimelineCause>,

    /// 이동 상태
    move_state: MoveState,
    /// 같은 편 통과/겹침 허용(현재 타일) 만료 시각(ms)
    ///
    /// `position`이 원천이므로 타일 상태는 저장하지 않고, 이 값으로 "soft 점유" 규칙을 계산한다.
    soft_until_ms: Option<u64>,
    /// 누적 이동 게이지 (tile_units)
    move_progress_units: u64,
    /// 마지막으로 이동 게이지를 갱신한 시각(ms)
    move_last_update_ms: u64,
    /// 이동 속도 (tile_units_per_ms)
    move_speed_units_per_ms: u32,
    /// 다음 이동 스텝 이벤트가 예약된 시각(ms). 예약이 없으면 None.
    move_next_step_ms: Option<u64>,
    /// 현재 예약한 목적지(타일). None이면 예약 없음.
    reserved_destination: Option<Position>,
    /// BFS로 계산된 경로(타일 시퀀스). 구현 단계에서 채워짐.
    move_path: Vec<Position>,
    /// 다음 목적지 계산 실패 횟수(결정론적 jitter 등에 사용)
    repath_counter: u32,
}

impl RuntimeUnit {
    /// UnitSnapshot 생성
    fn to_snapshot(&self) -> UnitSnapshot {
        UnitSnapshot {
            id: self.instance_id,
            owner: self.owner,
            position: self.position,
            stats: self.stats,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BuffInstanceKey {
    caster_instance_id: Uuid,
    target_instance_id: Uuid,
    buff_id: BuffId,
}

#[derive(Debug, Clone)]
struct ActiveBuff {
    stacks: u8,
    expires_at_ms: u64,
    next_tick_ms: Option<u64>,
}

pub struct BattleCore {
    event_queue: BinaryHeap<BattleEvent>,

    player_info: PlayerDeckInfo,
    opponent_info: PlayerDeckInfo,

    units: HashMap<Uuid, RuntimeUnit>,
    artifacts: HashMap<Uuid, RuntimeArtifact>,
    items: HashMap<Uuid, RuntimeItem>,
    graveyard: HashMap<Uuid, UnitSnapshot>,

    buffs: HashMap<BuffInstanceKey, ActiveBuff>,

    projectiles: HashMap<Uuid, ProjectileRecord>,

    pub movement_field: RuntimeField,

    game_data: Arc<GameDataBase>,

    pub timeline: Timeline,
    pub timeline_seq: u64,
    pub projectile_seq: u64,
    pub movement_seed: u64,
    pub recording_cause_stack: Vec<TimelineCause>,
}

impl BattleCore {
    pub fn new(
        player: &PlayerDeckInfo,
        opponent: &PlayerDeckInfo,
        game_data: Arc<GameDataBase>,
        field_size: (u8, u8),
        movement_seed: u64,
    ) -> Self {
        Self {
            event_queue: BinaryHeap::new(),
            player_info: player.clone(),
            opponent_info: opponent.clone(),
            units: HashMap::new(),
            artifacts: HashMap::new(),
            items: HashMap::new(),
            graveyard: HashMap::new(),
            buffs: HashMap::new(),
            projectiles: HashMap::new(),
            movement_field: RuntimeField::new(field_size.0, field_size.1),
            game_data,
            timeline: Timeline::new(),
            timeline_seq: 0,
            projectile_seq: 0,
            movement_seed,
            recording_cause_stack: Vec::new(),
        }
    }

    fn can_gain_resonance(unit: &RuntimeUnit, now_ms: u64) -> bool {
        now_ms >= unit.resonance_gain_locked_until_ms && now_ms >= unit.next_action_time
    }

    fn can_start_autocast(unit: &RuntimeUnit, now_ms: u64) -> bool {
        if now_ms < unit.next_action_time {
            return false;
        }
        if unit.stats.current_health == 0 {
            return false;
        }
        match unit.move_state {
            MoveState::Moving | MoveState::Ghost | MoveState::CCLocked { .. } | MoveState::Dead => {
                false
            }
            MoveState::Acquire | MoveState::WaitRepath { .. } => true,
        }
    }

    fn add_resonance(
        &mut self,
        unit_instance_id: Uuid,
        amount: u32,
        now_ms: u64,
        allow_autocast_when_full: bool,
    ) {
        if amount == 0 {
            return;
        }

        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return;
        };

        if unit.stats.current_health == 0 {
            return;
        }

        if !Self::can_gain_resonance(unit, now_ms) {
            return;
        }

        let max = unit.resonance_max.max(1);
        let before = unit.resonance_current.min(max);
        let after = before.saturating_add(amount).min(max);
        unit.resonance_current = after;

        if allow_autocast_when_full && before < max && after == max {
            unit.pending_cast = true;
        }
    }

    fn schedule_pending_autocast_for(&mut self, caster_instance_id: Uuid, now_ms: u64) {
        let fallback = self.recording_cause().unwrap_or_default();

        let Some((cause, should_schedule)) = self.units.get_mut(&caster_instance_id).map(|unit| {
            if !unit.pending_cast {
                return (fallback, false);
            }
            if !Self::can_start_autocast(unit, now_ms) {
                return (fallback, false);
            }

            unit.pending_cast = false;
            let cause = unit.pending_cast_cause.take().unwrap_or(fallback);
            (cause, true)
        }) else {
            return;
        };

        if !should_schedule {
            return;
        }

        self.event_queue.push(BattleEvent::AutoCastStart {
            time_ms: now_ms,
            caster_instance_id,
            cause,
        });
    }

    fn schedule_pending_autocasts(&mut self, now_ms: u64) {
        let mut casters: Vec<Uuid> = self
            .units
            .iter()
            .filter_map(|(id, unit)| {
                if unit.pending_cast && unit.stats.current_health > 0 {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        casters.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        // Keep a stable root cause for all autocasts triggered by the same context.
        let cause = self.recording_cause().unwrap_or_default();

        for caster_instance_id in casters {
            if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                if unit.pending_cast && unit.pending_cast_cause.is_none() {
                    unit.pending_cast_cause = Some(cause);
                }
            }
            self.schedule_pending_autocast_for(caster_instance_id, now_ms);
        }
    }
}
