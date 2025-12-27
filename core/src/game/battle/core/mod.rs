use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use uuid::Uuid;

pub mod sim;

use crate::{
    ecs::resources::{Field, Position},
    game::{
        battle::{
            buffs::BuffId,
            enums::BattleEvent,
            timeline::{Timeline, TimelineCause},
            types::{PlayerDeckInfo, UnitSnapshot},
        },
        data::GameDataBase,
        enums::Side,
        stats::UnitStats,
    },
};

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

    runtime_field: Field,

    game_data: Arc<GameDataBase>,

    pub timeline: Timeline,
    pub timeline_seq: u64,
    pub recording_cause_stack: Vec<TimelineCause>,
}

impl BattleCore {
    pub fn new(
        player: &PlayerDeckInfo,
        opponent: &PlayerDeckInfo,
        game_data: Arc<GameDataBase>,
        field_size: (u8, u8),
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
            runtime_field: Field::new(field_size.0, field_size.1),
            game_data,
            timeline: Timeline::new(),
            timeline_seq: 0,
            recording_cause_stack: Vec::new(),
        }
    }

    fn can_gain_resonance(unit: &RuntimeUnit, now_ms: u64) -> bool {
        now_ms >= unit.resonance_gain_locked_until_ms && now_ms >= unit.next_action_time
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

        for caster_instance_id in casters {
            if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                unit.pending_cast = false;
            }
            self.event_queue.push(BattleEvent::AutoCastStart {
                time_ms: now_ms,
                caster_instance_id,
                cause: self.recording_cause().unwrap_or_default(),
            });
        }
    }
}
