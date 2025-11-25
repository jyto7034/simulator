use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    enums::{GameOption, OrdealType, PhaseType, RiskLevel},
    events::EventGenerator,
};

/// 작업 타입
///
/// 기물에 대해 수행할 수 있는 작업의 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkType {
    Instinct,   // 본능 작업
    Insight,    // 통찰 작업
    Attachment, // 애착 작업
    Repression, // 억압 작업
}

pub struct SuppressionGenerator;

impl EventGenerator for SuppressionGenerator {
    type Output = [GameOption; 3];

    fn generate(&self, ctx: &super::GeneratorContext) -> Self::Output {
        use crate::ecs::resources::GameProgression;
        use rand::SeedableRng;

        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        // TODO: map_or 변경
        let current_ordeal = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_ordeal)
            .unwrap_or(OrdealType::Dawn);

        let current_phase = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_phase)
            .unwrap_or(PhaseType::I);

        // TODO: 실제 abnormality 메타데이터에서 uuid 조회
        // 지금은 임시로 고정 uuid 사용
        let options = [
            GameOption::SuppressAbnormality {
                abnormality_id: "abnormality_1".to_string(),
                risk_level: RiskLevel::TETH,
                uuid: Uuid::parse_str("750e8400-e29b-41d4-a716-446655440001").unwrap(),
            },
            GameOption::SuppressAbnormality {
                abnormality_id: "abnormality_2".to_string(),
                risk_level: RiskLevel::HE,
                uuid: Uuid::parse_str("750e8400-e29b-41d4-a716-446655440002").unwrap(),
            },
            GameOption::SuppressAbnormality {
                abnormality_id: "abnormality_3".to_string(),
                risk_level: RiskLevel::WAW,
                uuid: Uuid::parse_str("750e8400-e29b-41d4-a716-446655440003").unwrap(),
            },
        ];

        options
    }
}

/// 진압 작업 비즈니스 로직 헬퍼
pub struct SuppressionExecutor;

impl SuppressionExecutor {
    /// 진압 작업 수행
    ///
    /// # Arguments
    /// * `world` - ECS World
    /// * `abnormality_uuid` - 기물 UUID
    /// * `work_type` - 수행할 작업 타입
    pub fn perform_work(
        world: &mut World,
        abnormality_uuid: Uuid,
        work_type: WorkType,
    ) -> Result<(), GameError> {
        // TODO: GameData에서 기물 메타데이터 조회 (abnormality_uuid로)
        // TODO: work_type과 기물의 선호 작업 타입 비교
        // TODO: 성공/실패 판정
        //   성공: Enkephalin 획득, E.G.O 획득 가능성
        //   실패: 기물 탈출 또는 직원 사상, 페널티
        // TODO: 결과를 World에 반영 (Resource, Component 업데이트)

        Ok(())
    }
}
