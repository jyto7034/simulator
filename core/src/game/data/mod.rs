pub mod event_pools;
pub mod shop_data;

use shop_data::{Shop, ShopDatabase};

use crate::game::data::event_pools::EventPoolConfig;

/// 모든 게임 데이터를 담는 구조체
///
/// NOTE: 이 데이터는 game_server에서 로드되어 GameCore에 전달됩니다.
/// game_server에서 Arc<GameData>로 공유하여 메모리 효율적으로 사용합니다.
pub struct GameData {
    pub shops: ShopDatabase,
    pub event_pools: EventPoolConfig,
}

impl GameData {
    /// 특정 상인 조회 (uuid로)
    pub fn get_shop_by_uuid(&self, uuid: &uuid::Uuid) -> Option<&Shop> {
        self.shops.shops.iter().find(|s| &s.uuid == uuid)
    }

    /// 이름으로 상인 조회
    pub fn get_shop_by_name(&self, name: &str) -> Option<&Shop> {
        self.shops.shops.iter().find(|s| s.name == name)
    }
}
