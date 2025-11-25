use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::{
        event_pools::EventPhasePool,
        random_event_data::{EventRiskLevel, RandomEventMetadata},
    },
    enums::{GameOption, RiskLevel},
    events::{EventGenerator, GeneratorContext},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RandomEventType {
    // Dawn 시련 이벤트들
    SuspiciousBox,
    PrayerShrine,
    WanderingMerchant,
    CursedFountain,
    MysteriousStranger,
    // Noon 시련 이벤트들
    GamblerNpc,
    CursedChest,
    VoidPortal,
    DemonContract,
}

// 구조가 조금 이상함.
// 나중에 마저 다듬기.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEvent {
    pub id: String,
    pub text: String,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventData {
    pub event_type: RandomEventType,
    pub description: String,
    pub choices: Vec<RandomEvent>,
}

pub struct RandomEventGenerator;

impl EventGenerator for RandomEventGenerator {
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

        // 2. RandomEvent pool 가져오기
        let pool = &ctx
            .game_data
            .event_pools
            .get_pool(current_ordeal)
            .random_events;

        // 3. RNG 생성
        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        // 4. pool에서 가중치 기반 UUID 선택
        let uuid = match EventPhasePool::choose_weighted_uuid(pool, &mut rng) {
            Some(uuid) => uuid,
            None => {
                // 폴백: pool이 비어있으면 기본 랜덤 이벤트 반환
                return GameOption::Random {
                    event: RandomEventMetadata {
                        id: "fallback".to_string(),
                        uuid: Uuid::nil(),
                        event_type: RandomEventType::SuspiciousBox,
                        name: "임시 이벤트".to_string(),
                        description: "폴백 이벤트".to_string(),
                        image: "default".to_string(),
                        risk_level: EventRiskLevel::Low,
                    },
                };
            }
        };

        // 5. GameData에서 RandomEvent 조회
        let event = match ctx.game_data.random_events.get_by_uuid(&uuid) {
            Some(event) => event.clone(), // RandomEventMetadata 전체를 clone
            None => {
                // 폴백: UUID에 해당하는 RandomEvent가 없으면 기본값
                RandomEventMetadata {
                    id: format!("unknown_{}", uuid),
                    uuid,
                    event_type: RandomEventType::SuspiciousBox,
                    name: "알 수 없는 이벤트".to_string(),
                    description: "설명 없음".to_string(),
                    image: "unknown".to_string(),
                    risk_level: EventRiskLevel::Low,
                }
            }
        };

        // 6. GameOption 생성 (RandomEventMetadata 전체 데이터 포함)
        GameOption::Random { event }
    }
}

/// 랜덤 이벤트 비즈니스 로직 헬퍼
pub struct RandomEventExecutor;

impl RandomEventExecutor {
    /// 선택지 결과 처리
    ///
    /// # Arguments
    /// * `world` - ECS World
    /// * `event` - 랜덤 이벤트 메타데이터
    /// * `choice_id` - 플레이어가 선택한 선택지 ID
    pub fn process_choice(
        world: &mut World,
        event: &RandomEventMetadata,
        choice_id: &str,
    ) -> Result<(), GameError> {
        Ok(())
    }
}
