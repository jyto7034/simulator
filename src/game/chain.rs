use crate::{
    card::{
        effect::{Effect, EffectLevel, EffectResult},
        types::PlayerType,
        Card, PrioritizedEffect,
    },
    exception::GameError,
    server::input_handler::InputHandler,
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

    // 현재 처리 중인 카드 (입력 대기 중일 때 사용)
    pending_card: Option<Card>,

    // 처리 대기 중인 효과들 (입력 후 체인에 추가 예정)
    pending_effects: Vec<PrioritizedEffect>,

    // 이미 처리된 효과 ID
    processed_effect_ids: HashSet<Uuid>,

    // 유저 입력 대기 정보
    waiting_effect_index: Option<usize>,
    waiting_input: Option<Vec<Uuid>>,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            current_phase: ChainPhase::Completed, // 초기 상태는 완료
            pending_card: None,
            pending_effects: Vec::new(),
            processed_effect_ids: HashSet::new(),
            waiting_effect_index: None,
            waiting_input: None,
        }
    }

    /// 카드의 모든 효과를 처리합니다
    pub async fn process_card_effects(
        &mut self,
        game: &mut Game,
        player_type: PlayerType,
        card: Card,
        input_handler: &mut InputHandler,
    ) -> Result<PlayCardResult, GameError> {
        info!(
            "카드 효과 처리: player={:?}, card={:?}",
            player_type,
            card.get_uuid()
        );

        // 효과 처리 준비
        let effects = card.get_prioritized_effect();

        // 1. 즉발 효과 처리
        let result = self
            .process_immediate_effects(game, &card, &effects, input_handler)
            .await?;
        if let Some(input_result) = result {
            // 즉발 효과가 입력을 요청한 경우
            // 체인 효과는 보류
            let chain_effects: Vec<_> = effects
                .iter()
                .filter(|e| e.get_effect().get_timing() == EffectLevel::Chain)
                .map(|e| e.clone())
                .collect();

            if !chain_effects.is_empty() {
                self.pending_chain_effects(card, chain_effects);
            }

            return Ok(input_result);
        }

        // 2. 체인 효과 처리
        self.add_chain_effects(game, &card, &effects)?;

        // 3. 체인 해결
        if self.has_effects() {
            match self.resolve(game)? {
                ChainResolutionResult::Completed => Ok(PlayCardResult::Success),
                ChainResolutionResult::WaitingForInput(result) => Ok(result),
            }
        } else {
            Ok(PlayCardResult::Success)
        }
    }

    /// 즉발 효과 처리
    async fn process_immediate_effects(
        &mut self,
        game: &mut Game,
        card: &Card,
        effects: &[PrioritizedEffect],
        input_handler: &mut InputHandler,
    ) -> Result<Option<PlayCardResult>, GameError> {
        // 즉발 효과 수집
        let immediate_effects: Vec<_> = effects
            .iter()
            .filter(|e| e.get_effect().get_timing() == EffectLevel::Immediate)
            .collect();

        // 효과 처리
        for prioritized_effect in immediate_effects {
            // 이미 처리된 효과는 건너뛰기
            let effect_id = prioritized_effect.get_effect().get_id().into();
            if self.processed_effect_ids.contains(&effect_id) {
                continue;
            }

            if let Ok(effect_clone) = prioritized_effect.get_effect().clone_effect() {
                let result = effect_clone.begin_effect(game, card)?;

                match result {
                    EffectResult::Completed => {
                        // 효과 완료, 처리된 효과로 표시
                        self.processed_effect_ids.insert(effect_id);
                    }
                    EffectResult::NeedsInput { inner } => {
                        let input_cards = input_handler.wait_for_input(inner).await?;
                        effect_clone.handle_input(game, card, input_cards)?;
                    }
                }
            }
        }

        // 모든 즉발 효과가 완료됨
        Ok(None)
    }

    /// 체인 효과 추가
    fn add_chain_effects(
        &mut self,
        game: &mut Game,
        card: &Card,
        effects: &[PrioritizedEffect],
    ) -> Result<(), GameError> {
        // 체인 효과 수집
        let chain_effects: Vec<_> = effects
            .iter()
            .filter(|e| e.get_effect().get_timing() == EffectLevel::Chain)
            .collect();

        // 체인에 효과 추가
        for prioritized_effect in chain_effects {
            // 이미 처리된 효과는 건너뛰기
            let effect_id = prioritized_effect.get_effect().get_id().into();
            if self.processed_effect_ids.contains(&effect_id) {
                continue;
            }

            if let Ok(effect_clone) = prioritized_effect.get_effect().clone_effect() {
                self.add_effect(card.clone(), effect_clone);
            }
        }

        Ok(())
    }

    pub fn has_effects(&self) -> bool {
        !self.links.is_empty()
    }

    /// 체인에 효과 추가
    pub fn add_effect(&mut self, card: Card, effect: Box<dyn Effect>) {
        // 체인 구성 단계가 아니면 효과 추가 불가
        if self.current_phase != ChainPhase::Building {
            self.start_building(); // 새 체인 시작
        }

        // 효과를 체인 링크로 래핑하여 추가
        self.links.push(ChainLink {
            effect,
            source_card: card,
        });
    }

    /// 보류 중인 효과들을 체인에 추가
    pub fn pending_chain_effects(&mut self, card: Card, effects: Vec<PrioritizedEffect>) {
        // 입력 대기 상태로 변경
        self.current_phase = ChainPhase::Waiting;
        self.pending_card = Some(card);
        self.pending_effects = effects;
    }

    /// 사용자 입력 처리 후 보류 중인 효과 추가
    pub fn add_pending_effects_after_input(&mut self) -> Result<(), GameError> {
        if self.current_phase != ChainPhase::Waiting || self.pending_card.is_none() {
            return Err(GameError::InvalidChainState);
        }

        // 준비 단계로 전환
        self.current_phase = ChainPhase::Building;
        let card = self.pending_card.take().unwrap();

        // 벡터 소유권 가져오기
        let pending_effects = std::mem::take(&mut self.pending_effects);

        // 소유권을 가진 벡터 처리
        for prioritized_effect in pending_effects {
            let effect_id = prioritized_effect.get_effect().get_id();
            if !self.processed_effect_ids.contains(&effect_id.into()) {
                if let Ok(effect) = prioritized_effect.get_effect().clone_effect() {
                    self.add_effect(card.clone(), effect);
                }
            }
        }

        Ok(())
    }

    /// 체인 구성 시작
    pub fn start_building(&mut self) {
        // 이전 체인 상태 정리
        if self.current_phase == ChainPhase::Building || self.current_phase == ChainPhase::Resolving
        {
            // 이미 활성화된 체인이 있으면 처리 완료해야 함
            // 여기서는 간단히 초기화만 수행
            self.links.clear();
        }

        self.current_phase = ChainPhase::Building;
    }

    /// 체인 해결 시작
    pub fn start_resolving(&mut self) -> Result<(), GameError> {
        if self.links.is_empty() {
            return Ok(()); // 체인이 비어있으면 아무것도 안 함
        }

        self.current_phase = ChainPhase::Resolving;
        Ok(())
    }

    /// 체인의 모든 효과 해결
    pub fn resolve(&mut self, game: &mut Game) -> Result<ChainResolutionResult, GameError> {
        // 체인이 구성 중이면 해결 단계로 전환
        if self.current_phase == ChainPhase::Building {
            self.start_resolving()?;
        }

        // 해결 단계가 아니면 에러
        if self.current_phase != ChainPhase::Resolving {
            return Err(GameError::InvalidChainState);
        }

        // 대기 중인 입력이 있으면 처리
        if let Some(index) = self.waiting_effect_index {
            if let Some(input) = self.waiting_input.take() {
                let link = &self.links[index];
                let result = link.effect.handle_input(game, &link.source_card, input)?;

                match result {
                    EffectResult::Completed => {
                        // 효과 완료, 처리된 효과로 표시
                        self.processed_effect_ids
                            .insert(link.effect.get_id().into());
                        self.waiting_effect_index = None;
                    }
                    EffectResult::NeedsInput { inner } => {
                        // 여전히 입력 필요
                        // return Ok(ChainResolutionResult::WaitingForInput(inner));
                    }
                }
            } else {
                // 대기 중인데 입력이 없으면 에러
                return Err(GameError::MissingInput);
            }
        }

        // LIFO 방식으로 체인 해결 (뒤에서부터 처리)
        while !self.links.is_empty() {
            let index = self.links.len() - 1;
            let link = &self.links[index];
            let effect_id = link.effect.get_id();

            // 이미 처리된 효과는 건너뛰기
            if self.processed_effect_ids.contains(&effect_id.into()) {
                self.links.pop();
                continue;
            }

            // 효과 실행
            let result = link.effect.begin_effect(game, &link.source_card)?;

            match result {
                EffectResult::Completed => {
                    // 효과 완료, 체인에서 제거
                    self.processed_effect_ids.insert(effect_id.into());
                    self.links.pop();
                }
                EffectResult::NeedsInput { inner } => {
                    // 입력 대기 상태로 전환
                    self.waiting_effect_index = Some(index);
                    self.current_phase = ChainPhase::Waiting;
                    // return Ok(ChainResolutionResult::WaitingForInput(inner));
                }
            }
        }

        // 모든 효과 처리 완료
        self.current_phase = ChainPhase::Completed;
        Ok(ChainResolutionResult::Completed)
    }
}

impl Chain {
    /// 현재 체인 처리 단계 반환
    pub fn phase(&self) -> ChainPhase {
        self.current_phase
    }

    /// 현재 처리 중인 카드 참조 반환
    pub fn pending_card(&self) -> Option<&Card> {
        self.pending_card.as_ref()
    }

    /// 처리 대기 중인 효과들 참조 반환
    pub fn pending_effects(&self) -> &[PrioritizedEffect] {
        &self.pending_effects
    }

    /// 이미 처리된 효과 ID 목록 참조 반환
    // TODO: 사용된 효과는 특정 조건에 따라 다시 복구될 수 있어야함.
    pub fn processed_effect_ids(&self) -> &HashSet<Uuid> {
        &self.processed_effect_ids
    }

    /// 대기 중인 효과 인덱스 반환
    pub fn waiting_effect_index(&self) -> Option<usize> {
        self.waiting_effect_index
    }

    /// 대기 중인 사용자 입력 참조 반환
    pub fn waiting_input(&self) -> Option<&Vec<Uuid>> {
        self.waiting_input.as_ref()
    }

    //
    // Setter 메서드 (필요한 필드만)
    //

    /// 체인 처리 단계 설정
    pub fn set_phase(&mut self, phase: ChainPhase) {
        self.current_phase = phase;
    }

    /// 처리 중인 카드 설정
    pub fn set_pending_card(&mut self, card: Option<Card>) {
        self.pending_card = card;
    }

    /// 대기 중인 효과들 설정
    pub fn set_pending_effects(&mut self, effects: Vec<PrioritizedEffect>) {
        self.pending_effects = effects;
    }

    /// 처리된 효과 ID 추가
    pub fn add_processed_effect_id(&mut self, effect_id: Uuid) {
        self.processed_effect_ids.insert(effect_id);
    }

    /// 처리된 효과 ID 목록 초기화
    pub fn clear_processed_effect_ids(&mut self) {
        self.processed_effect_ids.clear();
    }

    /// 효과가 이미 처리되었는지 확인
    pub fn is_effect_processed(&self, effect_id: &Uuid) -> bool {
        self.processed_effect_ids.contains(effect_id)
    }

    /// 대기 중인 효과 인덱스 설정
    pub fn set_waiting_effect_index(&mut self, index: Option<usize>) {
        self.waiting_effect_index = index;
    }

    /// 대기 중인 사용자 입력 설정
    pub fn set_waiting_input(&mut self, input: Option<Vec<Uuid>>) {
        self.waiting_input = input;
    }

    //
    // 유틸리티 메서드
    //

    /// 대기 중인 효과 및 입력 정보 초기화
    pub fn clear_waiting_state(&mut self) {
        self.waiting_effect_index = None;
        self.waiting_input = None;
    }

    /// 모든 상태 초기화 (새 체인 시작용)
    pub fn reset(&mut self) {
        self.links.clear();
        self.current_phase = ChainPhase::Completed;
        self.pending_card = None;
        self.pending_effects.clear();
        self.processed_effect_ids.clear();
        self.clear_waiting_state();
    }
}

// 체인 해결 결과
pub enum ChainResolutionResult {
    Completed,                       // 모든 효과 처리 완료
    WaitingForInput(PlayCardResult), // 사용자 입력 대기
}
