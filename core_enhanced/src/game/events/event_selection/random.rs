use crate::game::{
    data::event_pools::EventPhasePool,
    enums::{GameOption, RandomEventOption, RiskLevel},
    events::{EventGenerator, GeneratorContext},
};
use serde::{Deserialize, Serialize};

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

impl EventGenerator for RandomEventGenerator {
    type Output = GameOption;

    fn generate(
        &self,
        ctx: &GeneratorContext,
    ) -> Result<Self::Output, crate::game::behavior::GameError> {
        use crate::ecs::resources::GameProgression;
        use crate::game::determinism;
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
        let mut rng = rand::rngs::StdRng::seed_from_u64(determinism::seed_with_namespace(
            ctx.random_seed,
            0x5241_4E44,
        ));

        // 4. pool에서 가중치 기반 UUID 선택
        let uuid = EventPhasePool::choose_weighted_uuid(pool, &mut rng).ok_or_else(|| {
            crate::game::behavior::GameError::InvalidStaticData(format!(
                "random event pool is empty for ordeal={current_ordeal:?}"
            ))
        })?;

        // 5. GameData에서 RandomEvent 조회
        let event = ctx
            .game_data
            .random_event_data
            .get_by_uuid(&uuid)
            .ok_or_else(|| {
                crate::game::behavior::GameError::InvalidStaticData(format!(
                    "random event uuid {uuid} selected from ordeal={current_ordeal:?} pool is missing from GameData"
                ))
            })?
            .clone();

        // 6. GameOption 생성 (RandomEventMetadata 전체 데이터 포함)
        Ok(GameOption::Random {
            event: RandomEventOption::from(event),
        })
    }
}
