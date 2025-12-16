use bevy_ecs::world::World;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    ecs::resources::{Enkephalin, Inventory, InventoryDiffDto, InventoryItemDto, SelectedEvent},
    game::{
        behavior::{BehaviorResult, GameError},
        data::{
            event_pools::EventPhasePool,
            shop_data::{ShopMetadata, ShopType},
            GameDataBase,
        },
        enums::GameOption,
        events::{EventGenerator, GeneratorContext},
    },
};

// 상인에도 등급이 존재함.
// 레벨에 비례하여 등급이 높은 상인이 등장할 수 있음.
// 아이템 종류마다 상인이 존재함.
// 아티팩트, 기물, 무기, 방어구 등등
// 각 종류마다 고유한 Npc 를 가짐. ( 상인의 갯수가 너무 많으면 리소스가 더 많이 발생하니 중복 허용함. )
// TODO: 상인에게 소지금 개념을 추가하여 플레이어가 마음껏 아이템을 팔 수 없게 해도 좋음.
pub struct ShopGenerator;

impl EventGenerator for ShopGenerator {
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

        // 2. Shop pool 가져오기
        let pool = &ctx.game_data.event_pools.get_pool(current_ordeal).shops;

        // 3. RNG 생성
        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        // 4. pool에서 가중치 기반 UUID 선택
        let uuid = match EventPhasePool::choose_weighted_uuid(pool, &mut rng) {
            Some(uuid) => uuid,
            None => {
                // 폴백: pool이 비어있으면 임시 Shop 반환
                warn!(
                    "Shop pool is empty for ordeal={:?}, using fallback shop",
                    current_ordeal
                );
                let shop = ShopMetadata {
                    id: String::new(),
                    name: "임시 상점".to_string(),
                    uuid: Uuid::nil(),
                    shop_type: ShopType::Shop,
                    can_reroll: false,
                    visible_items: Vec::new(),
                    hidden_items: Vec::new(),
                };
                return GameOption::Shop { shop };
            }
        };

        // 5. GameData에서 Shop 조회
        let shop = match ctx.game_data.shop_data.get_by_uuid(&uuid) {
            Some(shop) => shop.clone(), // Shop 전체를 clone
            None => {
                // 폴백: UUID에 해당하는 Shop이 없으면 기본값
                warn!("Shop uuid {:?} not found in GameData, using fallback", uuid);
                ShopMetadata {
                    id: String::new(),
                    name: "임시 상점".to_string(),
                    uuid: Uuid::nil(),
                    shop_type: ShopType::Shop,
                    can_reroll: false,
                    visible_items: Vec::new(),
                    hidden_items: Vec::new(),
                }
            }
        };

        info!("Generated shop event: id={}, uuid={}", shop.id, shop.uuid);

        // 7. GameOption 생성 (Shop 전체 데이터 포함)
        GameOption::Shop { shop }
    }
}

/// 상점 비즈니스 로직 헬퍼
pub struct ShopExecutor;

impl ShopExecutor {
    /// 상점 새로고침
    ///
    /// # Arguments
    /// * `world` - ECS World (Enkephalin, Inventory 등 접근)
    pub fn reroll(world: &mut World) -> Result<BehaviorResult, GameError> {
        // 1. Selected_Event 에서 현재 상점 정보를 가져옴.
        let mut selected = world
            .get_resource_mut::<SelectedEvent>()
            .ok_or(GameError::NotInShopState)?;

        // 2. shop 정보 가져오기
        let shop = selected.as_shop_mut()?;

        if !shop.can_reroll {
            warn!("Reroll requested but current shop does not allow reroll");
            return Err(GameError::ShopRerollNotAllowed);
        }

        shop.reroll_items();
        debug!("Shop items rerolled (shop_uuid={})", shop.uuid);

        let new_items = shop.visible_items.clone();

        Ok(BehaviorResult::RerollShop { new_items })
    }

    /// 아이템 구매
    ///
    /// # Arguments
    /// * `world` - ECS World (Enkephalin, Inventory 등 접근)
    /// * `game_data` - 정적 게임 데이터베이스 (아이템 메타데이터 조회용)
    /// * `item_uuid` - 구매할 아이템 UUID
    pub fn purchase_item(
        world: &mut World,
        game_data: &GameDataBase,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        // ============================================================
        // 1단계: 검증
        // ============================================================

        // 1-1. 상점에서 아이템 조회 (UUID가 현재 상점에 노출되어 있는지 확인)
        let (item, price) = {
            let selected_event = world
                .get_resource::<SelectedEvent>()
                .ok_or(GameError::NotInShopState)?;

            let shop = selected_event.as_shop()?;

            // 치팅 방지: 해당 상점의 visible_items 에 존재하는지 확인
            if !shop.visible_items.iter().any(|id| *id == item_uuid) {
                warn!(
                    "Item uuid {} not found in visible_items of shop '{}'",
                    item_uuid, shop.id
                );
                return Err(GameError::ShopItemNotFound);
            }

            // 전역 ItemRegistry 를 통해 실제 아이템 메타데이터 조회
            let item = game_data
                .item(&item_uuid)
                .cloned()
                .ok_or(GameError::ShopItemNotFound)?;
            let price = item.price();

            debug!(
                "Found item in shop: item_uuid={}, price={}",
                item_uuid, price
            );

            (item, price)
        };

        // 1-2. Enkephalin 잔액 확인
        {
            let enkephalin = world
                .get_resource::<Enkephalin>()
                .ok_or(GameError::MissingResource("Enkephalin"))?;

            if enkephalin.amount < price {
                warn!(
                    "Insufficient Enkephalin: have={}, price={} (item_uuid={})",
                    enkephalin.amount, price, item_uuid
                );
                return Err(GameError::InsufficientResources);
            }
        }

        // 1-3. 인벤토리 공간 확인
        {
            let inventory = world
                .get_resource::<Inventory>()
                .ok_or(GameError::MissingResource("Inventory"))?;

            if !inventory.can_add_item(&item) {
                warn!("Inventory full: cannot add item (item_uuid={})", item_uuid);
                return Err(GameError::InventoryFull);
            }
        }

        // ============================================================
        // 2단계: 실행 (모든 검증 통과 후)
        // ============================================================

        // 2-1. 상점에서 아이템 제거 (visible_items 에서만 제거)
        {
            let mut selected_event = world
                .get_resource_mut::<SelectedEvent>()
                .ok_or(GameError::NotInShopState)?;

            let shop = selected_event.as_shop_mut()?;
            shop.remove_visible_item(item_uuid)?;

            info!(
                "Removed purchased item from shop: item_uuid={}, price={}",
                item_uuid, price
            );
        }

        // 2-2. Enkephalin 차감
        let remaining_enkephalin = {
            let mut enkephalin = world
                .get_resource_mut::<Enkephalin>()
                .ok_or(GameError::MissingResource("Enkephalin"))?;

            enkephalin.amount -= price;
            enkephalin.amount
        };

        // 2-3. 인벤토리에 아이템 추가
        let mut inventory = world
            .get_resource_mut::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        inventory.add_item(item.clone())?;

        // 2-4. 인벤토리 변화 DTO 생성
        let item_dto = InventoryItemDto::from_item(&item);

        info!(
            "Item purchased successfully: item_uuid={}, remaining_enkephalin={}",
            item_uuid, remaining_enkephalin
        );

        Ok(BehaviorResult::PurchaseItem {
            enkephalin: remaining_enkephalin,
            inventory_diff: InventoryDiffDto {
                added: vec![item_dto],
                updated: Vec::new(),
                removed: Vec::new(),
            },
        })
    }

    /// 아이템 판매
    ///
    /// # Arguments
    /// * `world` - ECS World (Enkephalin, Inventory 등 접근)
    /// * `item_uuid` - 판매할 아이템 UUID
    ///
    /// # 판매 가격
    /// 아이템 원가의 50%로 판매됩니다.
    pub fn sell_tiem(world: &mut World, item_uuid: Uuid) -> Result<BehaviorResult, GameError> {
        // ============================================================
        // 1단계: 검증
        // ============================================================

        // 1-1. 인벤토리에서 아이템 조회 (제거하지 않음)
        let sell_price = {
            let inventory = world
                .get_resource::<Inventory>()
                .ok_or(GameError::MissingResource("Inventory"))?;

            let item = inventory
                .find_item(item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;

            // 판매 가격 = 원가의 50%
            let original_price = item.price();
            let sell_price = original_price / 2;

            debug!(
                "Found item in inventory: item_uuid={}, original_price={}, sell_price={}",
                item_uuid, original_price, sell_price
            );

            sell_price
        };

        // 1-2. 상점 상태 확인 (상점 안에 있는지)
        {
            let _selected_event = world
                .get_resource::<SelectedEvent>()
                .ok_or(GameError::NotInShopState)?;

            // 상점이 맞는지 확인
            let _shop = _selected_event.as_shop()?;
        }

        // ============================================================
        // 2단계: 실행 (모든 검증 통과 후)
        // ============================================================

        // 2-1. 인벤토리에서 아이템 제거
        {
            let mut inventory = world
                .get_resource_mut::<Inventory>()
                .ok_or(GameError::MissingResource("Inventory"))?;

            inventory
                .remove_item(item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;

            info!("Removed item from inventory: item_uuid={}", item_uuid);
        }

        // 2-2. Enkephalin 증가
        let remaining_enkephalin = {
            let mut enkephalin = world
                .get_resource_mut::<Enkephalin>()
                .ok_or(GameError::MissingResource("Enkephalin"))?;

            enkephalin.amount += sell_price;
            enkephalin.amount
        };

        info!(
            "Item sold successfully: item_uuid={}, sell_price={}, remaining_enkephalin={}",
            item_uuid, sell_price, remaining_enkephalin
        );

        Ok(BehaviorResult::SellItem {
            enkephalin: remaining_enkephalin,
            inventory_diff: InventoryDiffDto {
                added: Vec::new(),
                updated: Vec::new(),
                removed: vec![item_uuid],
            },
        })
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        equipment_data::{EquipmentMetadata, EquipmentType},
        Item,
    };
    use crate::game::enums::RiskLevel;
    use std::sync::Arc;

    /// 테스트용 World 생성 헬퍼
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Enkephalin::new(100));
        world.insert_resource(Inventory::new());
        world
    }

    /// 테스트용 Equipment 생성 헬퍼
    fn create_test_equipment(price: u32) -> (Uuid, Arc<EquipmentMetadata>) {
        let uuid = Uuid::new_v4();
        let equipment = Arc::new(EquipmentMetadata {
            id: "test_weapon".to_string(),
            uuid,
            name: "Test Weapon".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::HE,
            price,
            triggered_effects: Default::default(),
        });
        (uuid, equipment)
    }

    /// 테스트용 Abnormality 생성 헬퍼
    fn create_test_abnormality(price: u32) -> (Uuid, Arc<AbnormalityMetadata>) {
        let uuid = Uuid::new_v4();
        let abnormality = Arc::new(AbnormalityMetadata {
            id: "test_abnormality".to_string(),
            uuid,
            name: "Test Abnormality".to_string(),
            risk_level: RiskLevel::WAW,
            price,
            max_health: 100,
            attack: 30,
            defense: 5,
            attack_interval_ms: 1500,
            abilities: Vec::new(),
        });
        (uuid, abnormality)
    }

    /// 테스트용 상점 설정 헬퍼
    fn setup_shop(world: &mut World) {
        let shop = ShopMetadata {
            id: "test_shop".to_string(),
            name: "Test Shop".to_string(),
            uuid: Uuid::new_v4(),
            shop_type: ShopType::Shop,
            can_reroll: false,
            visible_items: Vec::new(),
            hidden_items: Vec::new(),
        };

        world.insert_resource(SelectedEvent::new(GameOption::Shop { shop }));
    }

    // ============================================================
    // sell_tiem 테스트
    // ============================================================

    #[test]
    fn test_sell_equipment_success() {
        // Given: 인벤토리에 장비가 있고, 상점 안에 있음
        let mut world = setup_world();
        setup_shop(&mut world);

        let (item_uuid, equipment) = create_test_equipment(100);

        // 인벤토리에 아이템 추가
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        let initial_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;

        // When: 아이템 판매
        let result = ShopExecutor::sell_tiem(&mut world, item_uuid);

        // Then: 성공
        assert!(result.is_ok());

        let sell_result = result.unwrap();
        match sell_result {
            BehaviorResult::SellItem {
                enkephalin,
                inventory_diff,
            } => {
                // 판매 가격 = 원가의 50%
                assert_eq!(enkephalin, initial_enkephalin + 50);

                // 인벤토리에서 제거됨
                assert_eq!(inventory_diff.removed.len(), 1);
                assert_eq!(inventory_diff.removed[0], item_uuid);

                // 추가/변경 없음
                assert_eq!(inventory_diff.added.len(), 0);
                assert_eq!(inventory_diff.updated.len(), 0);
            }
            _ => panic!("Expected SellItem result"),
        }

        // Enkephalin 증가 확인
        let final_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;
        assert_eq!(final_enkephalin, initial_enkephalin + 50);

        // 인벤토리에서 아이템 제거 확인
        let inventory = world.get_resource::<Inventory>().unwrap();
        assert!(inventory.find_item(item_uuid).is_none());
    }

    #[test]
    fn test_sell_abnormality_success() {
        // Given: 인벤토리에 환상체가 있고, 상점 안에 있음
        let mut world = setup_world();
        setup_shop(&mut world);

        let (item_uuid, abnormality) = create_test_abnormality(200);

        // 인벤토리에 아이템 추가
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Abnormality(abnormality.clone()))
                .unwrap();
        }

        let initial_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;

        // When: 아이템 판매
        let result = ShopExecutor::sell_tiem(&mut world, item_uuid);

        // Then: 성공, 판매 가격 = 200 * 50% = 100
        assert!(result.is_ok());

        let final_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;
        assert_eq!(final_enkephalin, initial_enkephalin + 100);

        // 인벤토리에서 제거 확인
        let inventory = world.get_resource::<Inventory>().unwrap();
        assert!(inventory.find_item(item_uuid).is_none());
    }

    #[test]
    fn test_sell_item_not_in_inventory() {
        // Given: 인벤토리에 아이템이 없음
        let mut world = setup_world();
        setup_shop(&mut world);

        let non_existent_uuid = Uuid::new_v4();

        // When: 존재하지 않는 아이템 판매 시도
        let result = ShopExecutor::sell_tiem(&mut world, non_existent_uuid);

        // Then: InventoryItemNotFound 에러
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GameError::InventoryItemNotFound
        ));
    }

    #[test]
    fn test_sell_item_not_in_shop() {
        // Given: 상점 안에 있지 않음 (SelectedEvent 없음)
        let mut world = setup_world();

        let (item_uuid, equipment) = create_test_equipment(100);

        // 인벤토리에 아이템 추가
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        // When: 상점 밖에서 판매 시도
        let result = ShopExecutor::sell_tiem(&mut world, item_uuid);

        // Then: NotInShopState 에러
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GameError::NotInShopState));
    }

    #[test]
    fn test_sell_price_calculation() {
        // Given: 다양한 가격의 아이템들
        let test_cases = vec![
            (100, 50),   // 100 → 50
            (200, 100),  // 200 → 100
            (75, 37),    // 75 → 37 (정수 나눗셈)
            (1, 0),      // 1 → 0 (정수 나눗셈)
            (1000, 500), // 1000 → 500
        ];

        for (original_price, expected_sell_price) in test_cases {
            let mut world = setup_world();
            setup_shop(&mut world);

            let (item_uuid, equipment) = create_test_equipment(original_price);

            // 인벤토리에 아이템 추가
            {
                let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
                inventory
                    .add_item(Item::Equipment(equipment.clone()))
                    .unwrap();
            }

            let initial_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;

            // When: 판매
            let result = ShopExecutor::sell_tiem(&mut world, item_uuid);

            // Then: 판매 가격이 원가의 50%인지 확인
            assert!(result.is_ok());

            let final_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;
            assert_eq!(
                final_enkephalin,
                initial_enkephalin + expected_sell_price,
                "Price {}: expected sell price {}, but got {}",
                original_price,
                expected_sell_price,
                final_enkephalin - initial_enkephalin
            );
        }
    }

    #[test]
    fn test_sell_multiple_items_sequentially() {
        // Given: 여러 아이템이 인벤토리에 있음
        let mut world = setup_world();
        setup_shop(&mut world);

        let (uuid1, equipment1) = create_test_equipment(100);
        let (uuid2, equipment2) = create_test_equipment(200);
        let (uuid3, abnormality) = create_test_abnormality(300);

        // 인벤토리에 아이템들 추가
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory.add_item(Item::Equipment(equipment1)).unwrap();
            inventory.add_item(Item::Equipment(equipment2)).unwrap();
            inventory.add_item(Item::Abnormality(abnormality)).unwrap();
        }

        let initial_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;

        // When: 아이템들을 순차적으로 판매
        ShopExecutor::sell_tiem(&mut world, uuid1).unwrap();
        ShopExecutor::sell_tiem(&mut world, uuid2).unwrap();
        ShopExecutor::sell_tiem(&mut world, uuid3).unwrap();

        // Then: 총 판매 가격 = 50 + 100 + 150 = 300
        let final_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;
        assert_eq!(final_enkephalin, initial_enkephalin + 300);

        // 인벤토리가 비어있는지 확인
        let inventory = world.get_resource::<Inventory>().unwrap();
        assert!(inventory.find_item(uuid1).is_none());
        assert!(inventory.find_item(uuid2).is_none());
        assert!(inventory.find_item(uuid3).is_none());
    }

    #[test]
    fn test_sell_same_item_twice() {
        // Given: 인벤토리에 아이템이 하나 있음
        let mut world = setup_world();
        setup_shop(&mut world);

        let (item_uuid, equipment) = create_test_equipment(100);

        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        // When: 첫 번째 판매 성공
        let result1 = ShopExecutor::sell_tiem(&mut world, item_uuid);
        assert!(result1.is_ok());

        // When: 두 번째 판매 시도 (이미 제거됨)
        let result2 = ShopExecutor::sell_tiem(&mut world, item_uuid);

        // Then: InventoryItemNotFound 에러
        assert!(result2.is_err());
        assert!(matches!(
            result2.unwrap_err(),
            GameError::InventoryItemNotFound
        ));
    }

    // ============================================================
    // Panic 테스트 (should_panic)
    // ============================================================

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    #[cfg(debug_assertions)] // debug 모드에서만 오버플로우 패닉 발생
    fn test_sell_enkephalin_overflow_panic() {
        // Given: Enkephalin이 거의 u32::MAX에 가까움
        let mut world = setup_world();
        setup_shop(&mut world);

        // Enkephalin을 u32::MAX - 10으로 설정
        {
            let mut enkephalin = world.get_resource_mut::<Enkephalin>().unwrap();
            enkephalin.amount = u32::MAX - 10;
        }

        // 100 가격의 아이템 추가 (판매 시 50 획득)
        let (item_uuid, equipment) = create_test_equipment(100);
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        // When: 판매 시도 (u32::MAX - 10 + 50 = 오버플로우)
        // Then: Debug 모드에서 패닉 발생 (expected)
        let _ = ShopExecutor::sell_tiem(&mut world, item_uuid);
    }

    #[test]
    #[should_panic(expected = "MissingResource")]
    fn test_sell_without_enkephalin_resource_panic() {
        // Given: Enkephalin 리소스가 없는 World
        let mut world = World::new();
        // Enkephalin 리소스를 추가하지 않음!
        world.insert_resource(Inventory::new());
        setup_shop(&mut world);

        let (item_uuid, equipment) = create_test_equipment(100);
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        // When: Enkephalin 리소스 없이 판매 시도
        // Then: unwrap()으로 인한 패닉 발생 (expected)
        ShopExecutor::sell_tiem(&mut world, item_uuid).unwrap();
    }

    #[test]
    #[should_panic(expected = "MissingResource")]
    fn test_sell_without_inventory_resource_panic() {
        // Given: Inventory 리소스가 없는 World
        let mut world = World::new();
        world.insert_resource(Enkephalin::new(100));
        // Inventory 리소스를 추가하지 않음!
        setup_shop(&mut world);

        let item_uuid = Uuid::new_v4();

        // When: Inventory 리소스 없이 판매 시도
        // Then: unwrap()으로 인한 패닉 발생 (expected)
        ShopExecutor::sell_tiem(&mut world, item_uuid).unwrap();
    }

    #[test]
    #[should_panic(expected = "NotInShopState")]
    fn test_sell_without_shop_state_panic() {
        // Given: SelectedEvent가 없는 World
        let mut world = setup_world();
        // setup_shop()을 호출하지 않음!

        let (item_uuid, equipment) = create_test_equipment(100);
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        // When: 상점 상태 없이 판매 시도
        // Then: unwrap()으로 인한 패닉 발생 (expected)
        ShopExecutor::sell_tiem(&mut world, item_uuid).unwrap();
    }

    #[test]
    #[should_panic(expected = "InventoryItemNotFound")]
    fn test_sell_nonexistent_item_panic() {
        // Given: 존재하지 않는 아이템 UUID
        let mut world = setup_world();
        setup_shop(&mut world);

        let nonexistent_uuid = Uuid::new_v4();

        // When: 존재하지 않는 아이템 판매 시도
        // Then: unwrap()으로 인한 패닉 발생 (expected)
        ShopExecutor::sell_tiem(&mut world, nonexistent_uuid).unwrap();
    }

    #[test]
    #[should_panic(expected = "EventTypeMismatch")]
    fn test_sell_with_wrong_event_type_panic() {
        // Given: SelectedEvent가 Shop이 아닌 다른 타입
        let mut world = setup_world();

        // Bonus 이벤트로 설정 (Shop이 아님!)
        use crate::game::data::bonus_data::{BonusMetadata, BonusType};

        let bonus = BonusMetadata {
            id: "test_bonus".to_string(),
            uuid: Uuid::new_v4(),
            bonus_type: BonusType::Enkephalin,
            name: "Test Bonus".to_string(),
            description: "Test bonus description".to_string(),
            icon: "test_icon.png".to_string(),
            amount: 30,
        };
        world.insert_resource(SelectedEvent::new(GameOption::Bonus { bonus }));

        let (item_uuid, equipment) = create_test_equipment(100);
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        // When: Bonus 이벤트 상태에서 판매 시도
        // Then: as_shop() 실패로 unwrap() 패닉 발생 (expected)
        ShopExecutor::sell_tiem(&mut world, item_uuid).unwrap();
    }

    // ============================================================
    // 경계값 테스트 (Boundary Testing)
    // ============================================================

    #[test]
    fn test_sell_price_zero() {
        // Given: 가격이 1인 아이템 (판매 시 0)
        let mut world = setup_world();
        setup_shop(&mut world);

        let (item_uuid, equipment) = create_test_equipment(1);

        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        let initial_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;

        // When: 판매 (1 / 2 = 0)
        let result = ShopExecutor::sell_tiem(&mut world, item_uuid);

        // Then: 성공하지만 Enkephalin 변화 없음
        assert!(result.is_ok());

        let final_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;
        assert_eq!(final_enkephalin, initial_enkephalin);
    }

    #[test]
    fn test_sell_price_max_safe() {
        // Given: 매우 큰 가격의 아이템 (하지만 오버플로우는 발생하지 않음)
        let mut world = setup_world();
        setup_shop(&mut world);

        // u32::MAX / 2 보다 작은 가격 설정
        let max_safe_price = 1_000_000_000u32; // 10억
        let (item_uuid, equipment) = create_test_equipment(max_safe_price);

        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        let initial_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;

        // When: 판매
        let result = ShopExecutor::sell_tiem(&mut world, item_uuid);

        // Then: 성공
        assert!(result.is_ok());

        let final_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;
        assert_eq!(final_enkephalin, initial_enkephalin + max_safe_price / 2);
    }

    #[test]
    fn test_sell_with_zero_enkephalin() {
        // Given: Enkephalin이 0인 상태
        let mut world = setup_world();
        setup_shop(&mut world);

        // Enkephalin을 0으로 설정
        {
            let mut enkephalin = world.get_resource_mut::<Enkephalin>().unwrap();
            enkephalin.amount = 0;
        }

        let (item_uuid, equipment) = create_test_equipment(100);
        {
            let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
            inventory
                .add_item(Item::Equipment(equipment.clone()))
                .unwrap();
        }

        // When: 판매
        let result = ShopExecutor::sell_tiem(&mut world, item_uuid);

        // Then: 성공 (판매는 Enkephalin 체크 없음)
        assert!(result.is_ok());

        let final_enkephalin = world.get_resource::<Enkephalin>().unwrap().amount;
        assert_eq!(final_enkephalin, 50); // 0 + 50
    }
}
