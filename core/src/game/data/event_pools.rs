use crate::game::enums::OrdealType;
use rand::{distributions::WeightedIndex, prelude::Distribution};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub uuid: Uuid, // 이벤트 고유 식별자 (RON에서 지정, GameData 조회용)
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
    /// 가중치 기반 UUID 선택 (유틸리티 메서드)
    ///
    /// Generator들이 pool에서 UUID를 선택할 때 사용
    pub fn choose_weighted_uuid<R: rand::Rng>(pool: &[WeightedEvent], rng: &mut R) -> Option<Uuid> {
        if pool.is_empty() {
            return None;
        }

        let weights: Vec<u32> = pool.iter().map(|e| e.weight).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let idx = dist.sample(rng);

        Some(pool[idx].uuid)
    }
}
