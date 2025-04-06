use crate::{
    card::{types::PlayerType, Card},
    effect::Effect,
    exception::GameError,
};
use std::collections::HashSet;
use tracing::info;
use uuid::Uuid;

use super::{game_step::PlayCardResult, Game};

// 체인 처리 단계
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChainPhase {
    Building,  // 체인 구성 중 (효과 추가 가능)
    Resolving, // 체인 해결 중 (효과 실행 단계)
    Waiting,   // 사용자 입력 대기 중
    Completed, // 체인 처리 완료
}

impl Default for ChainPhase {
    fn default() -> Self {
        todo!()
    }
}

// 체인 아이템 구조체 (효과와 소스 카드 연결)
#[derive(Clone)]
pub struct ChainLink {
    effect: Box<dyn Effect>,
    source_card: Card,
}

#[derive(Clone, Default)]
pub struct Chain {
    // 체인 큐 (LIFO 방식으로 처리)
    links: Vec<ChainLink>,

    // 현재 체인 처리 단계
    current_phase: ChainPhase,

    // 이미 처리된 효과 ID
    processed_effect_ids: HashSet<Uuid>,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            current_phase: ChainPhase::Completed, // 초기 상태는 완료
            processed_effect_ids: HashSet::new(),
        }
    }

    /// 카드의 모든 효과를 처리합니다
    pub async fn process_card_effects(
        &mut self,
        game: &mut Game,
        player_type: PlayerType,
        card: Card,
    ) -> Result<PlayCardResult, GameError> {
        info!(
            "카드 효과 처리: player={:?}, card={:?}",
            player_type,
            card.get_uuid()
        );

        // 카드는 효과를 여러개 가질 수 있음.
        // 효과 중 이미 처리된 효과, 현재 상태에는 발동할 수 없는 효과

        // 현재 입력된 카드의 효과를 가져옵니다.
        let effects = card.get_prioritized_effect().clone();

        todo!()
    }
}
// 체인 해결 결과
pub enum ChainResolutionResult {
    Completed,                       // 모든 효과 처리 완료
    WaitingForInput(PlayCardResult), // 사용자 입력 대기
}
