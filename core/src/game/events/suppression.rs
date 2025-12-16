use std::{collections::HashMap, sync::Arc};

use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    ecs::resources::{Field, GameProgression, Inventory, Position},
    game::{
        battle::{BattleCore, BattleResult, GrowthStack, OwnedArtifact, OwnedUnit, PlayerDeckInfo},
        behavior::GameError,
        data::{pve_data::PveEncounter, GameDataBase},
        enums::{GameOption, OrdealType, PhaseType, RiskLevel, Tier},
        events::EventGenerator,
    },
};

/// 작업 타입
///
/// 기물에 대해 수행할 수 있는 작업의 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkType {
    Instinct,   // 본능 작업
    Insight,    // 통찰 작업
    Attachment, // 애착 작업
    Repression, // 억압 작업
}

pub struct SuppressionGenerator;

impl EventGenerator for SuppressionGenerator {
    type Output = [GameOption; 3];

    fn generate(&self, ctx: &super::GeneratorContext) -> Self::Output {
        use rand::seq::SliceRandom;
        use rand::SeedableRng;

        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        let current_ordeal = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_ordeal)
            .unwrap_or(OrdealType::Dawn);

        let _current_phase = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_phase)
            .unwrap_or(PhaseType::I);

        let target_risk_levels = match current_ordeal {
            OrdealType::Dawn => vec![RiskLevel::ZAYIN, RiskLevel::TETH],
            OrdealType::Noon => vec![RiskLevel::TETH, RiskLevel::HE],
            OrdealType::Dusk => vec![RiskLevel::HE, RiskLevel::WAW],
            OrdealType::Midnight => vec![RiskLevel::WAW, RiskLevel::ALEPH],
            OrdealType::White => vec![RiskLevel::ALEPH],
        };

        let mut candidates: Vec<&PveEncounter> = ctx
            .game_data
            .pve_data
            .encounters
            .iter()
            .filter(|e| target_risk_levels.contains(&e.risk_level))
            .collect();

        candidates.shuffle(&mut rng);

        let selected: Vec<&PveEncounter> = candidates.into_iter().take(3).collect();

        let make_option = |encounter: Option<&&PveEncounter>| -> GameOption {
            match encounter {
                Some(e) => GameOption::SuppressAbnormality {
                    abnormality_id: e.abnormality_id.clone(),
                    risk_level: e.risk_level,
                    uuid: Uuid::new_v4(),
                },
                None => GameOption::SuppressAbnormality {
                    abnormality_id: "fallback".to_string(),
                    risk_level: RiskLevel::TETH,
                    uuid: Uuid::new_v4(),
                },
            }
        };

        [
            make_option(selected.get(0)),
            make_option(selected.get(1)),
            make_option(selected.get(2)),
        ]
    }
}

/// 진압 작업 비즈니스 로직 헬퍼
pub struct SuppressionExecutor;

impl SuppressionExecutor {
    /// PvE 전투 시작
    ///
    /// # Arguments
    /// * `world` - ECS World
    /// * `game_data` - 게임 데이터베이스
    /// * `abnormality_id` - 진압 대상 환상체 ID
    pub fn start_battle(
        world: &mut World,
        game_data: Arc<GameDataBase>,
        abnormality_id: &str,
    ) -> Result<BattleResult, GameError> {
        info!(
            "Starting suppression battle for abnormality: {}",
            abnormality_id
        );

        // 1. Player 덱 정보 구성
        let player_deck = Self::build_player_deck(world)?;
        debug!(
            "Player deck built: {} units, {} artifacts",
            player_deck.units.len(),
            player_deck.artifacts.len()
        );

        // 2. Opponent 덱 정보 구성 (PvE 데이터에서 로드)
        let opponent_deck = Self::build_opponent_deck(&game_data, abnormality_id)?;
        debug!(
            "Opponent deck built: {} units from encounter '{}'",
            opponent_deck.units.len(),
            abnormality_id
        );

        // 3. BattleCore 생성 및 전투 실행
        let field_size = (3, 3);
        let mut battle = BattleCore::new(&player_deck, &opponent_deck, game_data, field_size);

        let result = battle.run_battle(world)?;

        info!("Suppression battle completed");

        Ok(result)
    }

    /// Player 덱 정보 구성
    fn build_player_deck(world: &World) -> Result<PlayerDeckInfo, GameError> {
        let field = world
            .get_resource::<Field>()
            .ok_or(GameError::MissingResource("Field"))?;

        let inventory = world
            .get_resource::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        let mut units = Vec::new();
        let mut positions = HashMap::new();

        for abnormality in inventory.abnormalities.iter() {
            if let Some(pos) = field.get_position(abnormality.uuid) {
                units.push(OwnedUnit {
                    base_uuid: abnormality.uuid,
                    level: Tier::I,
                    growth_stacks: GrowthStack::new(),
                    equipped_items: vec![],
                });
                positions.insert(abnormality.uuid, pos);
            }
        }

        let artifacts: Vec<OwnedArtifact> = inventory
            .artifacts
            .get_all_items()
            .into_iter()
            .map(|a| OwnedArtifact { base_uuid: a.uuid })
            .collect();

        Ok(PlayerDeckInfo {
            units,
            artifacts,
            positions,
        })
    }

    /// Opponent 덱 정보 구성 (PvE 데이터에서)
    fn build_opponent_deck(
        game_data: &GameDataBase,
        abnormality_id: &str,
    ) -> Result<PlayerDeckInfo, GameError> {
        let encounter = game_data
            .pve_data
            .get_by_abnormality_id(abnormality_id)
            .ok_or_else(|| {
                warn!(
                    "PvE encounter not found for abnormality: {}",
                    abnormality_id
                );
                GameError::MissingResource("PveEncounter")
            })?;

        let mut units = Vec::new();
        let mut positions = HashMap::new();

        for pve_unit in &encounter.units {
            let abnormality_meta = game_data
                .abnormality_data
                .get_by_id(&pve_unit.abnormality_id)
                .ok_or_else(|| {
                    warn!(
                        "Abnormality metadata not found: {}",
                        pve_unit.abnormality_id
                    );
                    GameError::MissingResource("AbnormalityMetadata")
                })?;

            units.push(OwnedUnit {
                base_uuid: abnormality_meta.uuid,
                level: pve_unit.tier,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![],
            });

            positions.insert(abnormality_meta.uuid, Position::from(pve_unit.position));
        }

        Ok(PlayerDeckInfo {
            units,
            artifacts: vec![],
            positions,
        })
    }

    /// 진압 작업 수행 (기존 메서드 - 향후 확장용)
    ///
    /// # Arguments
    /// * `world` - ECS World
    /// * `abnormality_uuid` - 기물 UUID
    /// * `work_type` - 수행할 작업 타입
    pub fn perform_work(
        _world: &mut World,
        _abnormality_uuid: Uuid,
        _work_type: WorkType,
    ) -> Result<(), GameError> {
        // TODO: GameData에서 기물 메타데이터 조회 (abnormality_uuid로)
        // TODO: work_type과 기물의 선호 작업 타입 비교
        // TODO: 성공/실패 판정
        //   성공: Enkephalin 획득, E.G.O 획득 가능성
        //   실패: 기물 탈출 또는 직원 사상, 페널티
        // TODO: 결과를 World에 반영 (Resource, Component 업데이트)

        Ok(())
    }
}
