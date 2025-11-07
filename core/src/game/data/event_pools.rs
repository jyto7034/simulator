use crate::game::enums::OrdealType;
use crate::game::events::event_selection::EventOption;
use rand::{distributions::WeightedIndex, prelude::Distribution};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPoolConfig {
    pub dawn: EventPhasePool,
    pub noon: EventPhasePool,
    pub dusk: EventPhasePool,
    pub midnight: EventPhasePool,
    pub white: EventPhasePool,
}

/// Phase별 이벤트 풀 (카테고리 분리)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPhasePool {
    pub shops: Vec<WeightedEvent>,
    pub bonuses: Vec<WeightedEvent>,
    pub random_events: Vec<WeightedEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedEvent {
    pub weight: u32,
    pub event: EventOption,
}

impl EventPoolConfig {
    /// 시련 타입에 맞는 이벤트 풀 가져오기
    pub fn get_pool(&self, ordeal: OrdealType) -> &EventPhasePool {
        match ordeal {
            OrdealType::Dawn => &self.dawn,
            OrdealType::Noon => &self.noon,
            OrdealType::Dusk => &self.dusk,
            OrdealType::Midnight => &self.midnight,
            OrdealType::White => &self.white,
        }
    }
}

impl EventPhasePool {
    /// 각 카테고리에서 1개씩 선택 (총 3개)
    pub fn choose_one_from_each<R: rand::Rng>(&self, rng: &mut R) -> [EventOption; 3] {
        // 상점 1개 선택
        let shop = Self::choose_from_pool(&self.shops, rng);

        // 보너스 1개 선택
        let bonus = Self::choose_from_pool(&self.bonuses, rng);

        // 랜덤 인카운터 1개 선택
        let random = Self::choose_from_pool(&self.random_events, rng);

        [shop, bonus, random]
    }

    /// 가중치 기반 1개 선택
    fn choose_from_pool<R: rand::Rng>(pool: &[WeightedEvent], rng: &mut R) -> EventOption {
        if pool.is_empty() {
            // 폴백: 기본 이벤트
            use crate::game::data::shop_data::ShopType;
            return EventOption::Shop(ShopType::Shop);
        }

        let weights: Vec<u32> = pool.iter().map(|e| e.weight).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let idx = dist.sample(rng);

        pool[idx].event.clone()
    }
}
