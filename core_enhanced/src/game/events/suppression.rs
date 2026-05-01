use std::sync::Arc;

use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    ecs::resources::{Field, GameProgression, Inventory, Position},
    game::{
        battle::{
            core::BattleCore,
            types::{BattleResult, OwnedArtifact, OwnedUnit, PlayerDeckInfo},
        },
        behavior::GameError,
        data::{pve_data::PveEncounter, GameDataBase},
        determinism,
        enums::{
            BonusEventOption, GameOption, OrdealType, PhaseType, RewardMode, RiskLevel, Side, Tier,
        },
        events::EventGenerator,
        growth::GrowthStack,
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
    type Output = Vec<GameOption>;

    fn generate(&self, ctx: &super::GeneratorContext) -> Result<Self::Output, GameError> {
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
        if selected.is_empty() {
            return Err(GameError::InvalidStaticData(format!(
                "not enough suppression encounters for ordeal {:?}: need at least 1, got 0",
                current_ordeal
            )));
        }

        let make_option = |encounter: &&PveEncounter, index: u64| -> GameOption {
            const SUPPRESSION_OPTION_NS: u64 = 0x5355_5050_5253; // "SUPPRS"
            GameOption::SuppressAbnormality {
                abnormality_id: encounter.abnormality_id.clone(),
                encounter_id: encounter.id.clone(),
                risk_level: encounter.risk_level,
                uuid: determinism::uuid_v4_from_seed(ctx.random_seed, SUPPRESSION_OPTION_NS, index),
            }
        };

        Ok(selected
            .iter()
            .enumerate()
            .map(|(index, encounter)| make_option(encounter, index as u64))
            .collect())
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
    /// * `encounter_id` - 실제로 선택된 PvE encounter ID
    pub fn start_battle(
        world: &mut World,
        game_data: Arc<GameDataBase>,
        abnormality_id: &str,
        encounter_id: &str,
        movement_seed: u64,
    ) -> Result<BattleResult, GameError> {
        info!(
            "Starting suppression battle for abnormality={} encounter={}",
            abnormality_id, encounter_id
        );

        // 1. Player 덱 정보 구성
        let player_deck = Self::build_player_deck(world)?;

        let field = world
            .get_resource::<Field>()
            .ok_or(GameError::MissingResource("Field"))?;

        // 2. Opponent 덱 정보 구성 (PvE 데이터에서 로드)
        let opponent_deck = Self::build_opponent_deck(&game_data, encounter_id, field.height)?;
        let static_obstacles =
            Self::build_static_obstacles(&game_data, encounter_id, field.height)?;
        Self::validate_static_obstacles_do_not_overlap_units(
            &static_obstacles,
            &player_deck,
            &opponent_deck,
        )?;

        // 3. BattleCore 생성 및 전투 실행
        let field_size = (
            field.width,
            field
                .height
                .checked_mul(2)
                .ok_or(GameError::InvalidAction)?,
        );
        let mut battle = BattleCore::new(
            &player_deck,
            &opponent_deck,
            game_data,
            field_size,
            movement_seed,
        );
        let result = battle.run_battle_with_setup(world, |core| {
            for obstacle in static_obstacles {
                core.battlefield
                    .add_static_obstacle(obstacle)
                    .expect("validated PvE static obstacle should be placeable");
            }
        })?;

        info!("Suppression battle completed");

        Ok(result)
    }

    pub fn resolve_rewards(
        game_data: &GameDataBase,
        encounter_id: &str,
    ) -> Result<(RewardMode, Vec<BonusEventOption>), GameError> {
        let encounter = game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;

        let mut rewards = Vec::with_capacity(encounter.reward_bonus_uuids.len());
        for bonus_uuid in &encounter.reward_bonus_uuids {
            let bonus = game_data
                .bonus_data
                .get_by_uuid(bonus_uuid)
                .map(BonusEventOption::from)
                .ok_or(GameError::EventNotFound)?;
            rewards.push(bonus);
        }

        Ok((encounter.reward_mode, rewards))
    }

    /// Player 덱 정보 구성
    fn build_player_deck(world: &World) -> Result<PlayerDeckInfo, GameError> {
        let field = world
            .get_resource::<Field>()
            .ok_or(GameError::MissingResource("Field"))?;

        let inventory = world
            .get_resource::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        let mut placements: Vec<(Uuid, crate::ecs::resources::Position)> = field
            .get_positions_by_side(Side::Player)
            .into_iter()
            .collect();
        placements.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let mut units: Vec<OwnedUnit> = Vec::new();
        let mut positions = std::collections::HashMap::new();

        for (unit_uuid, pos) in placements {
            let owned = inventory
                .abnormalities
                .get_owned(&unit_uuid)
                .ok_or(GameError::UnitNotFound)?;
            let equipped_items: Vec<Uuid> = owned.item_slot.iter().map(|r| r.base_uuid).collect();

            units.push(OwnedUnit {
                owned_uuid: unit_uuid,
                base_uuid: owned.meta.uuid,
                level: Tier::I,
                growth_stacks: owned.growth_stacks.clone(),
                equipped_items,
            });
            positions.insert(unit_uuid, pos);
        }

        let mut artifact_items = inventory.artifacts.get_all_items();
        artifact_items.sort_by(|a, b| a.uuid.as_bytes().cmp(b.uuid.as_bytes()));
        let artifacts: Vec<OwnedArtifact> = artifact_items
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
        encounter_id: &str,
        opponent_row_offset: u8,
    ) -> Result<PlayerDeckInfo, GameError> {
        const PVE_OWNED_ABNORMALITY_NS: u64 = 0x0050_5645_4f57_4e44_u64; // "PVEOWND"

        let encounter = game_data.pve_data.get_by_id(encounter_id).ok_or_else(|| {
            warn!("PvE encounter not found: {}", encounter_id);
            GameError::MissingResource("PveEncounter")
        })?;

        let mut units = Vec::new();
        let mut positions = std::collections::HashMap::new();

        for (idx, pve_unit) in encounter.units.iter().enumerate() {
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

            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&abnormality_meta.uuid.as_bytes()[..8]);
            let seed = u64::from_be_bytes(bytes);
            let owned_uuid = crate::game::determinism::uuid_v4_from_seed(
                seed,
                PVE_OWNED_ABNORMALITY_NS,
                idx as u64,
            );

            units.push(OwnedUnit {
                owned_uuid,
                base_uuid: abnormality_meta.uuid,
                level: pve_unit.tier,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![],
            });

            let mut position = crate::ecs::resources::Position::from(pve_unit.position);
            position.y += i32::from(opponent_row_offset);
            positions.insert(owned_uuid, position);
        }

        Ok(PlayerDeckInfo {
            units,
            artifacts: vec![],
            positions,
        })
    }

    fn build_static_obstacles(
        game_data: &GameDataBase,
        encounter_id: &str,
        opponent_row_offset: u8,
    ) -> Result<Vec<Position>, GameError> {
        let encounter = game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;

        Ok(encounter
            .static_obstacles
            .iter()
            .map(|obstacle| {
                let mut position = Position::from(obstacle.position);
                position.y += i32::from(opponent_row_offset);
                position
            })
            .collect())
    }

    fn validate_static_obstacles_do_not_overlap_units(
        static_obstacles: &[Position],
        player_deck: &PlayerDeckInfo,
        opponent_deck: &PlayerDeckInfo,
    ) -> Result<(), GameError> {
        for obstacle in static_obstacles {
            if player_deck
                .positions
                .values()
                .any(|position| position == obstacle)
                || opponent_deck
                    .positions
                    .values()
                    .any(|position| position == obstacle)
            {
                return Err(GameError::PositionOccupied);
            }
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::resources::{GameProgression, Position};
    use crate::game::battle::timeline::TimelineEvent;
    use crate::game::data::{
        abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
        artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase,
        equipment_data::EquipmentDatabase,
        event_pools::EventPhasePool,
        event_pools::EventPoolConfig,
        pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        GameDataBase, Item,
    };
    use crate::game::enums::{OrdealType, PhaseType, RewardMode, RiskLevel};
    use crate::game::events::GeneratorContext;
    use std::collections::HashSet;
    use std::sync::Arc;

    fn empty_event_pools() -> EventPoolConfig {
        let pool = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        }
    }

    fn game_data_with_pve(encounters: Vec<PveEncounter>) -> Arc<GameDataBase> {
        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(encounters)),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        }))
    }

    fn game_data_with_abnormalities_and_pve(
        abnormalities: Vec<AbnormalityMetadata>,
        encounters: Vec<PveEncounter>,
    ) -> Arc<GameDataBase> {
        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(abnormalities)),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(encounters)),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        }))
    }

    fn abnormality(id: &str, uuid: Uuid, attack: u32) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: id.to_string(),
            uuid,
            name: id.to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 30,
            attack,
            defense: 0,
            magic_resist: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        }
    }

    fn pve_encounter(id: &str, abnormality_id: &str, risk_level: RiskLevel) -> PveEncounter {
        PveEncounter {
            id: id.to_string(),
            abnormality_id: abnormality_id.to_string(),
            difficulty: 1,
            risk_level,
            reward_mode: RewardMode::ClaimAll,
            reward_bonus_uuids: vec![],
            units: vec![crate::game::data::pve_data::PveUnitData {
                abnormality_id: abnormality_id.to_string(),
                position: crate::game::data::pve_data::PvePosition { x: 0, y: 0 },
                tier: crate::game::enums::Tier::I,
            }],
            static_obstacles: vec![],
        }
    }

    #[test]
    fn suppression_battle_uses_combined_player_and_opponent_field() {
        let player_owned_uuid = Uuid::from_u128(0x100);
        let player_base_uuid = Uuid::from_u128(0x101);
        let enemy_base_uuid = Uuid::from_u128(0x201);
        let player = abnormality("player", player_base_uuid, 30);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player.clone(), enemy],
            vec![PveEncounter {
                id: "encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_bonus_uuids: vec![],
                units: vec![crate::game::data::pve_data::PveUnitData {
                    abnormality_id: "enemy".to_string(),
                    position: crate::game::data::pve_data::PvePosition { x: 3, y: 2 },
                    tier: crate::game::enums::Tier::I,
                }],
                static_obstacles: vec![],
            }],
        );

        let mut inventory = Inventory::new();
        inventory
            .add_item_owned(player_owned_uuid, Item::Abnormality(Arc::new(player)))
            .expect("player unit should be added to inventory");

        let mut field = Field::new(7, 4);
        field
            .place(player_owned_uuid, Side::Player, Position::new(3, 0))
            .expect("player unit should be placed in player field");

        let mut world = World::new();
        world.insert_resource(inventory);
        world.insert_resource(field);

        let result =
            SuppressionExecutor::start_battle(&mut world, game_data, "enemy", "encounter", 7)
                .expect("suppression battle should start");

        let battle_start = result
            .timeline
            .entries
            .iter()
            .find_map(|entry| match entry.event {
                TimelineEvent::BattleStart { width, height } => Some((width, height)),
                _ => None,
            })
            .expect("timeline should contain BattleStart");
        assert_eq!(battle_start, (7, 8));

        let opponent_spawn = result
            .timeline
            .entries
            .iter()
            .find_map(|entry| match entry.event {
                TimelineEvent::UnitSpawned {
                    owner: Side::Opponent,
                    world_position,
                    ..
                } => Some(world_position.to_world()),
                _ => None,
            })
            .expect("timeline should contain opponent spawn");
        assert!(opponent_spawn.x.is_finite() && opponent_spawn.y.is_finite());
        assert!(opponent_spawn.x >= 0.0 && opponent_spawn.y >= 0.0);
    }

    #[test]
    fn suppression_battle_rejects_encounter_static_obstacle_overlapping_unit() {
        let player_owned_uuid = Uuid::from_u128(0x110);
        let player_base_uuid = Uuid::from_u128(0x111);
        let enemy_base_uuid = Uuid::from_u128(0x211);
        let player = abnormality("player", player_base_uuid, 30);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player.clone(), enemy],
            vec![PveEncounter {
                id: "encounter_with_obstacle".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_bonus_uuids: vec![],
                units: vec![crate::game::data::pve_data::PveUnitData {
                    abnormality_id: "enemy".to_string(),
                    position: crate::game::data::pve_data::PvePosition { x: 3, y: 2 },
                    tier: crate::game::enums::Tier::I,
                }],
                static_obstacles: vec![crate::game::data::pve_data::PveStaticObstacleData {
                    position: crate::game::data::pve_data::PvePosition { x: 3, y: 2 },
                }],
            }],
        );

        let mut inventory = Inventory::new();
        inventory
            .add_item_owned(player_owned_uuid, Item::Abnormality(Arc::new(player)))
            .expect("player unit should be added to inventory");

        let mut field = Field::new(7, 4);
        field
            .place(player_owned_uuid, Side::Player, Position::new(3, 0))
            .expect("player unit should be placed in player field");

        let mut world = World::new();
        world.insert_resource(inventory);
        world.insert_resource(field);

        let result = SuppressionExecutor::start_battle(
            &mut world,
            game_data,
            "enemy",
            "encounter_with_obstacle",
            7,
        );
        let err = match result {
            Ok(_) => panic!("overlapping static obstacle should reject battle start"),
            Err(err) => err,
        };

        assert!(matches!(err, GameError::PositionOccupied));
    }

    #[test]
    fn suppression_generator_filters_candidates_by_ordeal_risk_level() {
        // Given: Noon은 TETH/HE만 후보로 허용한다.
        let game_data = game_data_with_pve(vec![
            pve_encounter("zayin_1", "z1", RiskLevel::ZAYIN),
            pve_encounter("teth_1", "t1", RiskLevel::TETH),
            pve_encounter("teth_2", "t2", RiskLevel::TETH),
            pve_encounter("he_1", "h1", RiskLevel::HE),
            pve_encounter("waw_1", "w1", RiskLevel::WAW),
        ]);

        let mut world = World::new();
        world.insert_resource(GameProgression {
            current_ordeal: OrdealType::Noon,
            current_phase: PhaseType::I,
            phase_roll_index: 0,
        });

        // When: 동일 seed로 Suppression 후보 3개를 생성한다.
        let ctx = GeneratorContext::new(&world, game_data.as_ref(), 123);
        let generator = SuppressionGenerator;
        let options = generator
            .generate(&ctx)
            .expect("suppression generation should succeed");

        // Then: 결과는 3개이며, risk_level은 TETH/HE만 포함해야 한다.
        let allowed = [RiskLevel::TETH, RiskLevel::HE];
        let mut abnormality_ids = HashSet::new();
        for opt in options.iter() {
            let (abnormality_id, risk_level) = match opt {
                GameOption::SuppressAbnormality {
                    abnormality_id,
                    risk_level,
                    ..
                } => (abnormality_id.as_str(), *risk_level),
                other => panic!("expected SuppressAbnormality, got {other:?}"),
            };

            assert!(
                allowed.contains(&risk_level),
                "Noon에서는 TETH/HE만 허용되어야 한다 (got={risk_level:?})"
            );
            assert_ne!(
                abnormality_id, "fallback",
                "충분한 후보가 있는 경우 fallback 후보가 생성되면 안 된다"
            );
            abnormality_ids.insert(abnormality_id.to_string());
        }
        assert_eq!(
            abnormality_ids.len(),
            3,
            "후보 3개의 abnormality_id는 중복되면 안 된다"
        );
    }

    #[test]
    fn suppression_generator_is_deterministic_for_same_seed() {
        // Given: Dawn은 ZAYIN/TETH를 허용한다(충분한 후보를 준비).
        let game_data = game_data_with_pve(vec![
            pve_encounter("zayin_1", "z1", RiskLevel::ZAYIN),
            pve_encounter("zayin_2", "z2", RiskLevel::ZAYIN),
            pve_encounter("teth_1", "t1", RiskLevel::TETH),
            pve_encounter("teth_2", "t2", RiskLevel::TETH),
        ]);

        let mut world = World::new();
        world.insert_resource(GameProgression {
            current_ordeal: OrdealType::Dawn,
            current_phase: PhaseType::I,
            phase_roll_index: 0,
        });

        // When: 같은 seed로 두 번 생성한다.
        let generator = SuppressionGenerator;
        let ctx1 = GeneratorContext::new(&world, game_data.as_ref(), 777);
        let ctx2 = GeneratorContext::new(&world, game_data.as_ref(), 777);

        let a = generator
            .generate(&ctx1)
            .expect("suppression generation should succeed");
        let b = generator
            .generate(&ctx2)
            .expect("suppression generation should succeed");

        // Then: uuid/abnormality_id/risk_level 조합이 완전히 동일해야 한다.
        let normalize = |opt: &GameOption| match opt {
            GameOption::SuppressAbnormality {
                abnormality_id,
                encounter_id,
                risk_level,
                uuid,
            } => (
                abnormality_id.clone(),
                encounter_id.clone(),
                *risk_level,
                *uuid,
            ),
            other => panic!("expected SuppressAbnormality, got {other:?}"),
        };

        let a_norm: Vec<_> = a.iter().map(normalize).collect();
        let b_norm: Vec<_> = b.iter().map(normalize).collect();
        assert_eq!(a_norm, b_norm);
    }

    #[test]
    fn suppression_generator_allows_fewer_than_three_candidates_when_data_is_sparse() {
        let game_data = game_data_with_pve(vec![
            pve_encounter("aleph_only", "a1", RiskLevel::ALEPH),
            pve_encounter("waw_only", "w1", RiskLevel::WAW),
        ]);

        let mut world = World::new();
        world.insert_resource(GameProgression {
            current_ordeal: OrdealType::White,
            current_phase: PhaseType::III,
            phase_roll_index: 0,
        });

        let ctx = GeneratorContext::new(&world, game_data.as_ref(), 999);
        let generator = SuppressionGenerator;
        let options = generator
            .generate(&ctx)
            .expect("white ordeal should allow a sparse candidate list");

        assert_eq!(options.len(), 1);
        let GameOption::SuppressAbnormality {
            abnormality_id,
            risk_level,
            ..
        } = &options[0]
        else {
            panic!("expected a suppression option");
        };
        assert_eq!(abnormality_id, "a1");
        assert_eq!(*risk_level, RiskLevel::ALEPH);
    }
}
