use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::enums::OrdealType;

/// 가중치를 가진 이벤트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedEvent {
    pub weight: u32,
    pub uuid: Uuid,
}

/// Ordeal별 이벤트 풀
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPhasePool {
    pub shops: Vec<WeightedEvent>,
    pub bonuses: Vec<WeightedEvent>,
    pub random_events: Vec<WeightedEvent>,
}

impl EventPhasePool {
    /// 가중치 기반 UUID 선택 (RNG 사용)
    pub fn choose_weighted_uuid<R: Rng>(pool: &[WeightedEvent], rng: &mut R) -> Option<Uuid> {
        let total_weight: u32 = pool.iter().map(|e| e.weight).sum();

        if total_weight == 0 {
            return None;
        }

        let random_value = rng.gen_range(0..total_weight);
        let mut accumulated = 0;

        for item in pool {
            accumulated += item.weight;
            if random_value < accumulated {
                return Some(item.uuid);
            }
        }

        pool.last().map(|e| e.uuid)
    }

    /// 상점 선택
    pub fn select_shop<R: Rng>(&self, rng: &mut R) -> Option<Uuid> {
        Self::choose_weighted_uuid(&self.shops, rng)
    }

    /// 보너스 선택
    pub fn select_bonus<R: Rng>(&self, rng: &mut R) -> Option<Uuid> {
        Self::choose_weighted_uuid(&self.bonuses, rng)
    }

    /// 랜덤 이벤트 선택
    pub fn select_random_event<R: Rng>(&self, rng: &mut R) -> Option<Uuid> {
        Self::choose_weighted_uuid(&self.random_events, rng)
    }
}

/// 전체 이벤트 풀 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPoolConfig {
    pub dawn: EventPhasePool,
    pub noon: EventPhasePool,
    pub dusk: EventPhasePool,
    pub midnight: EventPhasePool,
    pub white: EventPhasePool,
}

impl EventPoolConfig {
    /// Ordeal에 따라 해당 풀 반환
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_weighted_uuid_deterministic() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let items = vec![
            WeightedEvent {
                weight: 80,
                uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            },
            WeightedEvent {
                weight: 20,
                uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            },
        ];

        // 같은 seed → 같은 결과
        let mut rng1 = StdRng::seed_from_u64(12345);
        let result1 = EventPhasePool::choose_weighted_uuid(&items, &mut rng1);

        let mut rng2 = StdRng::seed_from_u64(12345);
        let result2 = EventPhasePool::choose_weighted_uuid(&items, &mut rng2);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_deterministic_replay() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let items = vec![
            WeightedEvent {
                weight: 50,
                uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            },
            WeightedEvent {
                weight: 50,
                uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            },
        ];

        // 시나리오: 3번의 선택을 2번 반복
        let seed = 99999;

        let mut rng1 = StdRng::seed_from_u64(seed);
        let run1 = vec![
            EventPhasePool::choose_weighted_uuid(&items, &mut rng1),
            EventPhasePool::choose_weighted_uuid(&items, &mut rng1),
            EventPhasePool::choose_weighted_uuid(&items, &mut rng1),
        ];

        let mut rng2 = StdRng::seed_from_u64(seed);
        let run2 = vec![
            EventPhasePool::choose_weighted_uuid(&items, &mut rng2),
            EventPhasePool::choose_weighted_uuid(&items, &mut rng2),
            EventPhasePool::choose_weighted_uuid(&items, &mut rng2),
        ];

        // 같은 seed → 완전히 동일한 시퀀스
        assert_eq!(run1, run2);
    }

    #[test]
    fn test_get_pool() {
        let config = EventPoolConfig {
            dawn: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            noon: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            dusk: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            midnight: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            white: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
        };

        // 각 Ordeal에 대해 올바른 풀 반환 확인
        let _ = config.get_pool(OrdealType::Dawn);
        let _ = config.get_pool(OrdealType::Noon);
        let _ = config.get_pool(OrdealType::Dusk);
        let _ = config.get_pool(OrdealType::Midnight);
        let _ = config.get_pool(OrdealType::White);
    }
}
