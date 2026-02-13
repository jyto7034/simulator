use bevy_ecs::world::World;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    ecs::resources::{Enkephalin, Field, Inventory, InventoryDiffDto, InventoryItemDto},
    game::{
        behavior::GameError,
        data::{
            bonus_data::{BonusMetadata, BonusType},
            event_pools::EventPhasePool,
            GameDataBase,
        },
        enums::{GameOption, Side},
        events::{EventGenerator, GeneratorContext},
        managers::uuid_manager::UuidManager,
    },
};

pub struct BonusGenerator;

fn fallback_bonus_uuid(seed: u64) -> Uuid {
    // 재현성을 위해 seed 기반으로 결정적 UUID를 생성 (다른 폴백들과 충돌 방지).
    Uuid::from_u128(0x7a1d_3a09_5b8e_4d7b_8f10_0000_0000_0002u128 ^ ((seed as u128) << 64))
}

impl EventGenerator for BonusGenerator {
    type Output = GameOption;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output {
        use crate::ecs::resources::GameProgression;
        use crate::game::enums::OrdealType;
        use rand::SeedableRng;

        // 1. 현재 Ordeal 가져오기
        let current_ordeal = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_ordeal)
            .unwrap_or(OrdealType::Dawn);

        // 2. Bonus pool 가져오기
        let pool = &ctx.game_data.event_pools.get_pool(current_ordeal).bonuses;

        // 3. RNG 생성
        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        // 4. pool에서 가중치 기반 UUID 선택
        let uuid = match EventPhasePool::choose_weighted_uuid(pool, &mut rng) {
            Some(uuid) => uuid,
            None => {
                // 폴백: pool이 비어있으면 기본 보너스 반환
                warn!(
                    "Bonus pool is empty for ordeal={:?}, using fallback bonus",
                    current_ordeal
                );
                return GameOption::Bonus {
                    bonus: BonusMetadata {
                        bonus_type: BonusType::Enkephalin,
                        uuid: fallback_bonus_uuid(ctx.random_seed),
                        name: "임시 보너스".to_string(),
                        description: "폴백 보너스".to_string(),
                        icon: "default".to_string(),
                        amount: 0,
                        id: String::new(),
                    },
                };
            }
        };

        // 5. GameData에서 Bonus 조회
        let bonus = match ctx.game_data.bonus_data.get_by_uuid(&uuid) {
            Some(bonus) => bonus.clone(), // BonusMetadata 전체를 clone
            None => {
                // 폴백: UUID에 해당하는 Bonus가 없으면 기본값
                warn!(
                    "Bonus uuid {:?} not found in GameData, using fallback",
                    uuid
                );
                BonusMetadata {
                    bonus_type: BonusType::Enkephalin,
                    uuid,
                    name: "알 수 없는 보너스".to_string(),
                    description: "설명 없음".to_string(),
                    icon: "unknown".to_string(),
                    amount: 0,
                    id: String::new(),
                }
            }
        };

        debug!(
            "Generated bonus event: id={}, uuid={}",
            bonus.id, bonus.uuid
        );

        // 6. GameOption 생성 (BonusMetadata 전체 데이터 포함)
        GameOption::Bonus { bonus }
    }
}

/// 보너스 비즈니스 로직 헬퍼
pub struct BonusExecutor;

impl BonusExecutor {
    /// 보너스 지급
    ///
    /// # Arguments
    /// * `world` - ECS World
    /// * `game_data` - 정적 게임 데이터베이스 (랜덤 보상 선택용)
    /// * `bonus` - 보너스 메타데이터
    /// * `seed` - 결정적 랜덤 시드
    pub fn grant_bonus(
        world: &mut World,
        game_data: &GameDataBase,
        bonus: &BonusMetadata,
        seed: u64,
    ) -> Result<InventoryDiffDto, GameError> {
        use rand::{Rng, SeedableRng};

        let amount = bonus.amount;
        let mut inventory_diff = InventoryDiffDto::default();

        match bonus.bonus_type {
            BonusType::Enkephalin => {
                // Enkephalin 추가
                let mut enkephalin = world
                    .get_resource_mut::<Enkephalin>()
                    .ok_or(GameError::MissingResource("Enkephalin"))?;
                enkephalin.amount += amount;

                info!(
                    "Granted Enkephalin bonus: amount={}, new_total={}",
                    amount, enkephalin.amount
                );
            }
            BonusType::Experience => {
                // TODO: 경험치 추가
                // let mut player_stats = world.get_resource_mut::<PlayerStats>()?;
                // player_stats.exp += amount;
            }
            BonusType::Item => {
                // 무작위 E.G.O 선물(=장비) 지급
                if game_data.equipment_data.items.is_empty() {
                    warn!("Item bonus requested but equipment database is empty");
                    return Err(GameError::InvalidAction);
                }

                let mut rng = rand::rngs::StdRng::seed_from_u64(seed ^ 0xB0B0_5001);
                let idx = rng.gen_range(0..game_data.equipment_data.items.len());
                let base_uuid = game_data.equipment_data.items[idx].uuid;

                let item = game_data
                    .item(&base_uuid)
                    .ok_or(GameError::InvalidAction)?
                    .clone_arc();

                // 용량 체크 (실제로 추가하기 전에)
                {
                    let inventory = world
                        .get_resource::<Inventory>()
                        .ok_or(GameError::MissingResource("Inventory"))?;
                    if !inventory.can_add_item(&item) {
                        return Err(GameError::InventoryFull);
                    }
                }

                let owned_uuid = {
                    let mut uuid_manager = world
                        .get_resource_mut::<UuidManager>()
                        .ok_or(GameError::MissingResource("UuidManager"))?;
                    uuid_manager.next_owned_equipment()
                };

                {
                    let mut inventory = world
                        .get_resource_mut::<Inventory>()
                        .ok_or(GameError::MissingResource("Inventory"))?;
                    inventory.add_item_owned(owned_uuid, item.clone_arc())?;
                }

                inventory_diff
                    .added
                    .push(InventoryItemDto::from_item_with_uuid(&item, owned_uuid));
            }
            BonusType::Abnormality => {
                // 무작위 기물 지급
                // 규칙:
                // 1) 인벤토리(벤치)에 우선 추가
                // 2) 인벤토리가 가득 차 있으면, 필드에 즉시 배치(필드도 꽉 차있으면 실패)
                if game_data.abnormality_data.items.is_empty() {
                    warn!("Abnormality bonus requested but abnormality database is empty");
                    return Err(GameError::InvalidAction);
                }

                let (owned_uuid, item) = {
                    let mut rng = rand::rngs::StdRng::seed_from_u64(seed ^ 0xB0B0_ABA0);
                    let idx = rng.gen_range(0..game_data.abnormality_data.items.len());
                    let base_uuid = game_data.abnormality_data.items[idx].uuid;

                    let item = game_data
                        .item(&base_uuid)
                        .ok_or(GameError::InvalidAction)?
                        .clone_arc();

                    let owned_uuid = {
                        let mut uuid_manager = world
                            .get_resource_mut::<UuidManager>()
                            .ok_or(GameError::MissingResource("UuidManager"))?;
                        uuid_manager.next_owned_abnormality()
                    };

                    (owned_uuid, item)
                };

                // 1) 인벤토리 우선 추가
                let inventory_added = {
                    let mut inventory = world
                        .get_resource_mut::<Inventory>()
                        .ok_or(GameError::MissingResource("Inventory"))?;
                    match inventory.add_item_owned(owned_uuid, item.clone_arc()) {
                        Ok(()) => true,
                        Err(GameError::InventoryFull) => false,
                        Err(other) => return Err(other),
                    }
                };

                // 2) 인벤토리가 가득 찼다면, 필드에 즉시 배치 + 소유 목록에는 강제로 추가
                if !inventory_added {
                    let place_pos = {
                        let field = world
                            .get_resource::<Field>()
                            .ok_or(GameError::MissingResource("Field"))?;
                        first_empty_field_position(field).ok_or(GameError::InventoryFull)?
                    };

                    {
                        let mut inventory = world
                            .get_resource_mut::<Inventory>()
                            .ok_or(GameError::MissingResource("Inventory"))?;
                        let meta = match &item {
                            crate::game::data::Item::Abnormality(meta) => meta.clone(),
                            _ => return Err(GameError::InvalidAction),
                        };
                        inventory
                            .abnormalities
                            .add_item_ignore_capacity(owned_uuid, meta)
                            .map_err(|_| GameError::InvalidAction)?;
                    }

                    {
                        let mut field = world
                            .get_resource_mut::<Field>()
                            .ok_or(GameError::MissingResource("Field"))?;
                        field.place(owned_uuid, Side::Player, place_pos)?;
                    }
                }

                inventory_diff
                    .added
                    .push(InventoryItemDto::from_item_with_uuid(&item, owned_uuid));
            }
        }

        Ok(inventory_diff)
    }
}

fn first_empty_field_position(field: &Field) -> Option<crate::ecs::resources::Position> {
    for y in 0..field.height as i32 {
        for x in 0..field.width as i32 {
            let pos = crate::ecs::resources::Position::new(x, y);
            if field.get_unit_at(pos).is_none() {
                return Some(pos);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::resources::inventory::{
        AbnormalityInventory, ArtifactSlots, EquipmentInventory,
    };
    use crate::game::data::{
        abnormality_data::AbnormalityDatabase,
        artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase,
        equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType},
        event_pools::{EventPhasePool, EventPoolConfig},
        pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
    };
    use crate::game::enums::RiskLevel;
    use crate::game::stats::TriggeredEffects;
    use std::collections::HashMap;
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

    fn minimal_game_data() -> GameDataBase {
        let equipment = EquipmentMetadata {
            id: "equip".to_string(),
            uuid: Uuid::from_u128(10),
            name: "Equip".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 1,
            allow_duplicate_equip: true,
            triggered_effects: TriggeredEffects::default(),
        };

        let abnormality = crate::game::data::abnormality_data::AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: Uuid::from_u128(20),
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 1,
            max_health: 10,
            attack: 1,
            defense: 1,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };

        let artifact = crate::game::data::artifact_data::ArtifactMetadata {
            id: "art".to_string(),
            uuid: Uuid::from_u128(30),
            name: "Art".to_string(),
            description: "desc".to_string(),
            rarity: RiskLevel::ZAYIN,
            price: 1,
            triggered_effects: HashMap::new(),
        };

        GameDataBase::new(
            Arc::new(AbnormalityDatabase::new(vec![abnormality])),
            Arc::new(ArtifactDatabase::new(vec![artifact])),
            Arc::new(EquipmentDatabase::new(vec![equipment])),
            Arc::new(ShopDatabase::new(vec![])),
            Arc::new(BonusDatabase::new(vec![])),
            Arc::new(RandomEventDatabase::new(vec![])),
            Arc::new(PveEncounterDatabase::new(vec![])),
            Arc::new(SkillDatabase::new(vec![])),
            empty_event_pools(),
        )
    }

    fn world_with_limits(equip_slots: usize, abno_slots: usize, field_w: u8, field_h: u8) -> World {
        let mut world = World::new();
        world.insert_resource(Enkephalin::new(0));
        world.insert_resource(UuidManager::new(123));
        world.insert_resource(Field::new(field_w, field_h));
        world.insert_resource(Inventory {
            abnormalities: AbnormalityInventory::with_max_slots(abno_slots),
            equipments: EquipmentInventory::with_max_slots(equip_slots),
            artifacts: ArtifactSlots::with_max_slots(10),
        });
        world
    }

    #[test]
    fn item_bonus_adds_equipment_or_errors_when_inventory_full() {
        // Given: 장비 1개가 존재하는 GameData와, 장비 슬롯 1개짜리 인벤토리
        let game_data = minimal_game_data();
        let mut world = world_with_limits(1, 10, 1, 1);
        let bonus = BonusMetadata {
            id: "item_bonus".to_string(),
            bonus_type: BonusType::Item,
            uuid: Uuid::from_u128(100),
            name: "Item".to_string(),
            description: "desc".to_string(),
            icon: "icon".to_string(),
            amount: 1,
        };

        // When: 1회 지급
        let diff = BonusExecutor::grant_bonus(&mut world, &game_data, &bonus, 1).unwrap();
        // Then: 인벤토리에 장비가 1개 추가된다.
        assert_eq!(diff.added.len(), 1);
        assert!(matches!(diff.added[0], InventoryItemDto::Equipment(_)));

        // When: 같은 보너스를 한 번 더 시도(인벤토리 Full)
        let err = BonusExecutor::grant_bonus(&mut world, &game_data, &bonus, 1).unwrap_err();
        // Then: 슬롯 부족으로 실패한다.
        assert!(matches!(err, GameError::InventoryFull));
    }

    #[test]
    fn abnormality_bonus_adds_to_inventory_first_then_overflows_to_field() {
        // Given: 기물 1개가 존재하는 GameData와, 기물 인벤 1칸 + 1x1 필드
        let game_data = minimal_game_data();
        let mut world = world_with_limits(10, 1, 1, 1);
        let bonus = BonusMetadata {
            id: "abno_bonus".to_string(),
            bonus_type: BonusType::Abnormality,
            uuid: Uuid::from_u128(101),
            name: "Abno".to_string(),
            description: "desc".to_string(),
            icon: "icon".to_string(),
            amount: 1,
        };

        // When: 1회 지급 (인벤토리 우선)
        let diff = BonusExecutor::grant_bonus(&mut world, &game_data, &bonus, 1).unwrap();
        // Then: 인벤토리에만 추가되고, 필드는 비어있다.
        assert_eq!(diff.added.len(), 1);
        assert!(matches!(diff.added[0], InventoryItemDto::Abnormality(_)));

        let inventory = world.get_resource::<Inventory>().unwrap();
        assert_eq!(inventory.abnormalities.len(), 1);

        let field = world.get_resource::<Field>().unwrap();
        assert_eq!(field.count_by_side(Side::Player), 0);

        // When: 2회차 지급 (인벤 Full → 필드 overflow)
        let diff = BonusExecutor::grant_bonus(&mut world, &game_data, &bonus, 1).unwrap();

        // Then: 소유 목록에는 추가되고, 필드에 1개가 배치된다.
        assert_eq!(diff.added.len(), 1);
        assert!(matches!(diff.added[0], InventoryItemDto::Abnormality(_)));

        let inventory = world.get_resource::<Inventory>().unwrap();
        assert_eq!(inventory.abnormalities.len(), 2);

        let field = world.get_resource::<Field>().unwrap();
        assert_eq!(field.count_by_side(Side::Player), 1);

        // When: 3회차 지급 (인벤 Full + 필드 Full)
        let err = BonusExecutor::grant_bonus(&mut world, &game_data, &bonus, 1).unwrap_err();
        // Then: 더 이상 둘 곳이 없으므로 실패한다.
        assert!(matches!(err, GameError::InventoryFull));
    }
}
