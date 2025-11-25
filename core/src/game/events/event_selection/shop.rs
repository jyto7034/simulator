use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::{Enkephalin, Inventory, SelectedEvent},
    game::{
        behavior::{BehaviorResult, GameError},
        data::{
            event_pools::EventPhasePool,
            shop_data::{ShopMetadata, ShopProduct, ShopType},
            GameData,
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
// TODO: 각 상인마다 아이템 목록이 다르니 상인-아이템 목록은 외부 config 로 관리.
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
                let mut shop = ShopMetadata {
                    name: "임시 상점".to_string(),
                    uuid: Uuid::nil(),
                    shop_type: ShopType::Shop,
                    items_raw: vec![ShopProduct::Abnormality(String::new())],
                    visible_items: Vec::new(),
                    hidden_items: Vec::new(),
                    can_reroll: false,
                };
                let _ = shop.hydrate(ctx.game_data, &mut rng);
                return GameOption::Shop { shop };
            }
        };

        // 5. GameData에서 Shop 조회
        let mut shop = match ctx.game_data.shops_db.get_by_uuid(&uuid) {
            Some(shop) => shop.clone(), // Shop 전체를 clone
            None => {
                // 폴백: UUID에 해당하는 Shop이 없으면 기본값
                ShopMetadata {
                    name: "알 수 없는 상점".to_string(),
                    uuid,
                    shop_type: ShopType::Shop,
                    items_raw: vec![],
                    visible_items: Vec::new(),
                    hidden_items: Vec::new(),
                    can_reroll: false,
                }
            }
        };

        if let Err(_) = shop.hydrate(ctx.game_data, &mut rng) {
            // 해석 실패 시 빈 상점 반환
            let fallback = ShopMetadata {
                name: "해석 실패 상점".to_string(),
                uuid,
                shop_type: ShopType::Shop,
                items_raw: vec![],
                visible_items: Vec::new(),
                hidden_items: Vec::new(),
                can_reroll: false,
            };
            return GameOption::Shop { shop: fallback };
        }

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
            .ok_or(GameError::InvalidAction)?;

        // 2. shop 정보 가져오기
        let shop = selected.as_shop_mut()?;

        if !shop.can_reroll {
            return Err(GameError::InvalidAction);
        }

        shop.reroll_items();

        let new_items = shop.visible_items.clone();

        Ok(BehaviorResult::RerollShop { new_items })
    }

    /// 아이템 구매
    ///
    /// # Arguments
    /// * `world` - ECS World (Enkephalin, Inventory 등 접근)
    /// * `game_data` - 게임 데이터 (Shop, Item 조회)
    /// * `item_uuid` - 구매할 아이템 UUID
    pub fn purchase_item(
        world: &mut World,
        _game_data: &GameData,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        // 1. 현재 상점에서 해당 Item 이 구매 가능한지 확인
        let (item_ref, price) = {
            let mut selected_event = world
                .get_resource_mut::<SelectedEvent>()
                .ok_or(GameError::InvalidEvent)?;

            let shop = selected_event.as_shop_mut()?;

            // 해당 아이템을 상인이 가지고 있지 않은 경우
            if shop.find_item(item_uuid).is_err() {
                return Err(GameError::InvalidEvent);
            }

            // 실제 상점 목록에서 제거 (가시 아이템만 구매 가능)
            let purchased = shop.remove_visible_item(item_uuid)?;
            let price = purchased.price();
            let item_ref = purchased.into_item_reference();

            (item_ref, price)
        };

        // 2. Enkephalin 잔액 확인 및 차감
        let amount = {
            let mut enkephalin = world
                .get_resource_mut::<Enkephalin>()
                .ok_or(GameError::InsufficientResources)?;

            if enkephalin.amount < price {
                return Err(GameError::InsufficientResources);
            }
            enkephalin.amount -= price;

            enkephalin.amount
        };

        // 3. 인벤토리에 아이템 추가
        let mut inventory = world
            .get_resource_mut::<Inventory>()
            .ok_or(GameError::InvalidAction)?;
        inventory.add_item(item_ref)?;

        Ok(BehaviorResult::PurchaseItem {
            enkephalin: amount,
            inventory_metadata: todo!(),
            shop_metadata: todo!(),
        })
    }

    /// 아이템 구매
    ///
    /// # Arguments
    /// * `world` - ECS World (Enkephalin, Inventory 등 접근)
    /// * `game_data` - 게임 데이터 (Shop, Item 조회)
    /// * `item_uuid` - 구매할 아이템 UUID
    pub fn sell_tiem(
        world: &mut World,
        game_data: &GameData,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        // 팔고자 하는 아이템이 플레이어 소유가 맞는가?

        // Option: 상인이 충분한 돈을 소유하고 있는가?

        // Option: 상인이 충분한 인벤토리 공간을 소유하고 있는가?

        // 아이템을 플레이어 인벤토리로부터 제거

        // 제거된 아이템의 가치만큼 플레이어의 자원 증가

        Ok(BehaviorResult::PurchaseItem {
            enkephalin: todo!(),
            inventory_metadata: todo!(),
            shop_metadata: todo!(),
        })
    }
}
