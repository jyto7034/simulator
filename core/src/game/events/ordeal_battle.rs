use bevy_ecs::world::World;
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    enums::{GameOption, OrdealType},
    events::EventGenerator,
};

pub struct OrdealBattleGenerator;

impl EventGenerator for OrdealBattleGenerator {
    type Output = [GameOption; 3];

    fn generate(&self, ctx: &super::GeneratorContext) -> Self::Output {
        if let Some(opponent) = &ctx.extras.opponent_data {
            println!("Generating Ordeal battle against: {}", opponent.name);

            // TODO: opponent 정보 기반으로 전투 생성
            // - opponent.level
            // - opponent.deck
            // - opponent.items
            // 등을 활용
        } else {
            panic!("Ordeal battle requires opponent_data!");
        }

        // TODO: 실제 ordeal 메타데이터에서 uuid 조회
        // 지금은 임시로 고정 uuid 사용
        let options = [
            GameOption::OrdealBattle {
                ordeal_type: OrdealType::Dawn,
                difficulty: 1,
                uuid: Uuid::parse_str("850e8400-e29b-41d4-a716-446655440001").unwrap(),
            },
            GameOption::OrdealBattle {
                ordeal_type: OrdealType::Noon,
                difficulty: 2,
                uuid: Uuid::parse_str("850e8400-e29b-41d4-a716-446655440002").unwrap(),
            },
            GameOption::OrdealBattle {
                ordeal_type: OrdealType::Dusk,
                difficulty: 3,
                uuid: Uuid::parse_str("850e8400-e29b-41d4-a716-446655440003").unwrap(),
            },
        ];

        options
    }
}

/// 시련 전투 비즈니스 로직 헬퍼
pub struct OrdealBattleExecutor;

impl OrdealBattleExecutor {
    /// 전투 시작
    ///
    /// # Arguments
    /// * `world` - ECS World
    /// * `ordeal_battle_uuid` - 시련 전투 UUID
    /// * `deck_card_ids` - 플레이어가 선택한 덱 구성
    pub fn start_battle(
        world: &mut World,
        ordeal_battle_uuid: Uuid,
        deck_card_ids: Vec<Uuid>,
    ) -> Result<(), GameError> {
        // TODO: GameData에서 시련 전투 메타데이터 조회 (ordeal_battle_uuid로)
        // TODO: 전투 초기화
        //   - 플레이어 덱 로드 (deck_card_ids)
        //   - 적 데이터 로드 (PvE: 몬스터, PvP: Ghost 데이터)
        //   - 전투 시스템 초기화
        // TODO: 전투 진행 (별도의 BattleSystem 호출)
        // TODO: 전투 결과 처리
        //   승리: 보상 지급, 다음 Phase 진행
        //   패배: 게임 오버 또는 페널티

        Ok(())
    }
}
