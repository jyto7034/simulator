use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    data::{
        event_pools::EventPhasePool,
        random_event_data::{RandomEventInnerMetadata, RandomEventMetadata},
    },
    enums::{GameOption, RiskLevel},
    events::{EventGenerator, GeneratorContext},
};

/// 랜덤 이벤트로 발생 가능한 이벤트 유형
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RandomEventType {
    Shop,
    Bonus,
    Suppress,
}

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

fn fallback_random_event_uuid(seed: u64) -> Uuid {
    // 재현성을 위해 seed 기반으로 결정적 UUID를 생성 (다른 폴백들과 충돌 방지).
    Uuid::from_u128(0x7a1d_3a09_5b8e_4d7b_8f10_0000_0000_0003u128 ^ ((seed as u128) << 64))
}

fn pick_fallback_target(ctx: &GeneratorContext) -> (RandomEventType, RandomEventInnerMetadata) {
    if let Some(bonus) = ctx.game_data.bonus_data.bonuses.first() {
        return (
            RandomEventType::Bonus,
            RandomEventInnerMetadata::Bonus(bonus.uuid),
        );
    }
    if let Some(shop) = ctx.game_data.shop_data.shops.first() {
        return (
            RandomEventType::Shop,
            RandomEventInnerMetadata::Shop(shop.uuid),
        );
    }
    if let Some(abno) = ctx.game_data.abnormality_data.items.first() {
        return (
            RandomEventType::Suppress,
            RandomEventInnerMetadata::Suppress(abno.uuid),
        );
    }

    (
        RandomEventType::Bonus,
        RandomEventInnerMetadata::Bonus(Uuid::nil()),
    )
}

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
                let (event_type, inner_metadata) = pick_fallback_target(ctx);
                return GameOption::Random {
                    event: RandomEventMetadata {
                        id: "fallback".to_string(),
                        uuid: fallback_random_event_uuid(ctx.random_seed),
                        event_type,
                        name: "임시 이벤트".to_string(),
                        description: "폴백 이벤트".to_string(),
                        image: "default".to_string(),
                        risk_level: RiskLevel::ALEPH,
                        inner_metadata,
                    },
                };
            }
        };

        // 5. GameData에서 RandomEvent 조회
        let event = match ctx.game_data.random_event_data.get_by_uuid(&uuid) {
            Some(event) => event.clone(), // RandomEventMetadata 전체를 clone
            None => {
                // 폴백: UUID에 해당하는 RandomEvent가 없으면 기본값
                let (event_type, inner_metadata) = pick_fallback_target(ctx);
                RandomEventMetadata {
                    id: format!("unknown_{}", uuid),
                    uuid,
                    event_type,
                    name: "알 수 없는 이벤트".to_string(),
                    description: "설명 없음".to_string(),
                    image: "unknown".to_string(),
                    risk_level: RiskLevel::ALEPH,
                    inner_metadata,
                }
            }
        };

        // 6. GameOption 생성 (RandomEventMetadata 전체 데이터 포함)
        GameOption::Random { event }
    }
}
