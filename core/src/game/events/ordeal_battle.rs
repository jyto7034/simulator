use serde::{Deserialize, Serialize};

use crate::game::events::{EventError, EventExecutor, EventGenerator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdealSelectionOptions {
    pub options: [OrdealOption; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrdealOption {}

pub struct OrdealBattleGenerator;

impl EventGenerator for OrdealBattleGenerator {
    type Output = OrdealSelectionOptions;

    fn generate(&self, ctx: &super::GeneratorContext) -> Self::Output {
        // ⭐ opponent_data가 있는지 확인
        if let Some(opponent) = &ctx.extras.opponent_data {
            // Ordeal 전투 시: opponent_data 사용
            println!("Generating Ordeal battle against: {}", opponent.name);

            // TODO: opponent 정보 기반으로 전투 생성
            // - opponent.level
            // - opponent.deck
            // - opponent.items
            // 등을 활용
        } else {
            // opponent_data 없으면 에러 (Ordeal인데 opponent 없음)
            panic!("Ordeal battle requires opponent_data!");
        }

        todo!("Implement Ordeal battle generation with opponent_data")
    }
}

pub struct OrdealBattleExecutor;

impl EventExecutor for OrdealBattleExecutor {
    type Input = String; // ordeal_id

    fn execute(&self, ctx: &super::ExecutorContext, input: Self::Input) -> Result<(), EventError> {
        todo!()
    }
}
