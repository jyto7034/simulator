mod build;
mod commands;
mod deaths;
mod ids;
mod recording;
mod sim;
mod triggers;

use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use uuid::Uuid;

use crate::{
    ecs::resources::{Field, Position},
    game::{data::GameDataBase, enums::Side, stats::UnitStats},
};

use super::{
    ability_executor::{AbilityExecutor, UnitSnapshot},
    buffs::BuffId,
    death::DeathHandler,
    enums::BattleEvent,
    timeline::Timeline,
    types::PlayerDeckInfo,
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

    runtime_field: Field,

    game_data: Arc<GameDataBase>,

    timeline: Timeline,
    timeline_seq: u64,

    death_handler: DeathHandler,
    ability_executor: AbilityExecutor,
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
            death_handler: DeathHandler::new(),
            ability_executor: AbilityExecutor::new(),
        }
    }
}

#[cfg(test)]
mod tests;
