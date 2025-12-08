use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::Inventory,
    game::{
        behavior::GameError,
        data::GameDataBase,
        enums::{Side, Tier},
        stats::UnitStats,
    },
};

pub mod enums;

#[derive(Clone)]
pub struct PlayerDeckInfo {
    pub units: Vec<OwnedUnit>,
    pub artifacts: Vec<OwnedArtifact>,
}

pub enum BattleWinner {
    Player,
    Opponent,
    Draw,
}

pub struct Timeline {}

pub struct BattleResult {
    pub winner: BattleWinner,
    pub timeline: Timeline,
}

pub struct Event {}

// 아티팩트는 그럴 일 없겠지만, Runtime 때 수치 변경 기능 확장을 위해 Owned Layer 유지
#[derive(Debug, Clone)]
pub struct OwnedArtifact {
    pub base_uuid: Uuid,
}

// Runtime 때 수치 변경 기능 확장을 위해 Owned Layer 유지
#[derive(Debug, Clone)]
pub struct OwnedItem {
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct OwnedUnit {
    pub base_uuid: Uuid,
    pub level: Tier,
    pub growth_stacks: GrowthStack, // 영구 성장형 스택
    pub equipped_items: Vec<Uuid>,  // 장비 uuid
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrowthId {
    KillStack,
    PveWinStack,
    QuestRewardStack,
}

#[derive(Debug, Clone)]
pub struct GrowthStack {
    pub stacks: HashMap<GrowthId, i32>,
}

impl OwnedUnit {
    pub fn effective_stats(
        &self,
        game_data: &GameDataBase,
        artifacts: &[Uuid],
    ) -> Result<UnitStats, GameError> {
        // 1) base stats (AbnormalityMetadata)
        let origin = game_data
            .abnormality_data
            .get_by_uuid(&self.base_uuid)
            .ok_or(GameError::MissingResource("AbnormalityMetadata"))?;

        // Abnormality 메타데이터에서 기본 전투 스탯 구성
        if origin.attack_interval_ms == 0 {
            return Err(GameError::InvalidUnitStats(
                "attack_interval_ms must be > 0",
            ));
        }

        let mut stats = UnitStats::with_values(
            origin.max_health,
            origin.attack,
            origin.defense,
            origin.attack_interval_ms,
        );

        // TODO: 수치 연산 순서는 매우 중요함.
        // 최종 데미지 증가의 경우 모든 수치가 더해진 마지막에 계산되어야 하는데
        // 중간에 더해지면 안되는 것 처럼.

        // 2) growth/level 스택
        for (stat_id, value) in &self.growth_stacks.stacks {
            match stat_id {
                GrowthId::KillStack => {
                    stats.add_attack(*value);
                }
                GrowthId::PveWinStack => {
                    // TODO: PvE 승리 스택 반영
                }
                GrowthId::QuestRewardStack => {
                    // TODO: 퀘스트 보상 스택 반영
                }
            }
        }

        // 3) 장비 효과
        // 장비 효과에서 퍼센테이지 증가가 존재함.
        // base attack 에서 증가시키는지, 아니면 최종 attack 에서 증가시키는지 명세 작성해야함.
        for item_uuid in &self.equipped_items {
            let origin_item = game_data
                .equipment_data
                .get_by_uuid(item_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_modifiers(origin_item.modifiers.clone());
        }

        // 4) 덱 전체 아티팩트의 상시 패시브
        for artifact_uuid in artifacts {
            let origin_artifact = game_data
                .artifact_data
                .get_by_uuid(artifact_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_modifiers(origin_artifact.modifiers.clone());
        }

        Ok(stats)
    }
}

/// 전투 중 사용되는 아티팩트 런타임 표현
#[derive(Debug, Clone)]
pub struct RuntimeArtifact {
    pub owner: Side,
    pub artifact_uuid: Uuid,
}

/// 전투 중 사용되는 장비 런타임 표현
#[derive(Debug, Clone)]
pub struct RuntimeItem {
    pub owner: Side,
    pub owner_unit: Uuid,
    pub equipment_uuid: Uuid,
}

pub struct RuntimeUnit {
    pub owner: Side,
    pub base_uuid: Uuid,
    pub stats: UnitStats,
}

pub struct BattleCore {
    event_queue: VecDeque<Event>,

    player_info: PlayerDeckInfo,
    opponent_info: PlayerDeckInfo,

    player_units: Vec<RuntimeUnit>,
    opponent_units: Vec<RuntimeUnit>,

    player_artifacts: Vec<RuntimeArtifact>,
    opponent_artifacts: Vec<RuntimeArtifact>,

    player_items: Vec<RuntimeItem>,
    opponent_items: Vec<RuntimeItem>,

    game_data: Arc<GameDataBase>,
}

impl BattleCore {
    pub fn new(
        player: &PlayerDeckInfo,
        opponent: &PlayerDeckInfo,
        game_data: Arc<GameDataBase>,
    ) -> Self {
        Self {
            event_queue: VecDeque::new(),
            player_info: player.clone(),
            opponent_info: opponent.clone(),
            player_units: Vec::new(),
            opponent_units: Vec::new(),
            player_artifacts: Vec::new(),
            opponent_artifacts: Vec::new(),
            player_items: Vec::new(),
            opponent_items: Vec::new(),
            game_data,
        }
    }

    pub fn build_runtime_units_from_decks(
        &mut self,
        side: Side,
        world: &mut World,
    ) -> Result<(), GameError> {
        // 1. 인벤토리에서 아티팩트 런타임 상태 구성
        let inventory = world
            .get_resource::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        let target_artifacts = match side {
            Side::Player => &mut self.player_artifacts,
            Side::Opponent => &mut self.opponent_artifacts,
        };

        target_artifacts.clear();

        for artifact in inventory.artifacts.get_all_items() {
            target_artifacts.push(RuntimeArtifact {
                owner: side,
                artifact_uuid: artifact.uuid,
            });
        }

        let artifact_uuids: Vec<Uuid> = target_artifacts.iter().map(|a| a.artifact_uuid).collect();

        // 2. 빌드 대상 덱 및 RuntimeUnit / RuntimeItem 벡터 선택
        let (deck, runtime_units, runtime_items) = match side {
            Side::Player => (
                &self.player_info,
                &mut self.player_units,
                &mut self.player_items,
            ),
            Side::Opponent => (
                &self.opponent_info,
                &mut self.opponent_units,
                &mut self.opponent_items,
            ),
        };

        runtime_units.clear();
        runtime_items.clear();

        // 3. 각 OwnedUnit을 RuntimeUnit / RuntimeItem으로 변환
        for unit in &deck.units {
            let stats = unit.effective_stats(&self.game_data, artifact_uuids.as_slice())?;

            runtime_units.push(RuntimeUnit {
                owner: side,
                base_uuid: unit.base_uuid,
                stats,
            });

            for equipment_uuid in &unit.equipped_items {
                runtime_items.push(RuntimeItem {
                    owner: side,
                    owner_unit: unit.base_uuid,
                    equipment_uuid: *equipment_uuid,
                });
            }
        }

        Ok(())
    }

    pub fn run_battle(&mut self, world: &mut World) -> Result<BattleResult, GameError> {
        let res = loop {
            // PlayerDeckInfo 무결성은 외부에서 검증

            // 먼저 Deck 을 읽어서 RuntimeUnit 으로 변환.
            // 유닛 뿐만 아니라 아티팩트, 아이템도 Runtime 으로 변환해서 이벤트 큐에 적용해야함.
            self.build_runtime_units_from_decks(Side::Player, world)?;
            self.build_runtime_units_from_decks(Side::Opponent, world)?;

            // 내부적으로 주기적으로 사이클을 돌기 시작함

            // 전멸 판단을 위한 기물 갯수 참조
            let player_unit_count = self.player_units.len();
            let opponent_unit_count = self.opponent_units.len();

            let winner = match (player_unit_count, opponent_unit_count) {
                (0, 0) => BattleWinner::Draw,
                (0, _) => BattleWinner::Player,
                (_, 0) => BattleWinner::Opponent,
                _ => continue,
            };

            break BattleResult {
                winner,
                timeline: Timeline {},
            };
        };

        Ok(res)
    }
}
