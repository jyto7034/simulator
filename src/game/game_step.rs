use std::fmt;

use tokio::sync::oneshot::Receiver;
use uuid::Uuid;

use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    card::{insert::TopInsert, take::TopTake, types::PlayerType},
    effect::types::HandlerType,
    exception::GameError,
    selector::TargetCount,
    server::input_handler::InputAnswer,
    zone::zone::Zone,
    LogExt,
};

use super::Game;

pub enum PhaseResult {
    Mulligan,
    DrawPhase,
    StandbyPhase,
    MainPhaseStart,
    MainPhase1,
    BattlePhaseStart,
    BattleStep,
    BattleDamageStepStart,
    BattleDamageStepCalculationBefore,
    BattleDamageStepCalculationStart,
    BattleDamageStepCalculationEnd,
    BattleDamageStepEnd,
    BattlePhaseEnd,
    MainPhase2,
    EndPhase,
}

type PhaseResultType = Result<PhaseResult, GameError>;

// mulligan 에 필요한 게임 함수들.
pub mod mulligan {
    use super::*;
    impl Game {
        #[instrument(skip(self), fields(player_type = ?player_type.into()))]
        pub fn get_mulligan_cards<T: Into<PlayerType> + Copy>(
            &mut self,
            player_type: T,
            count: usize,
        ) -> Result<Vec<Uuid>, GameError> {
            let player_type = player_type.into();
            debug!(
                "멀리건 카드 뽑기 시도: player={:?}, count={}",
                player_type, count
            );

            let result = self
                .get_player_by_type(player_type)
                .get()
                .get_deck_mut()
                .take_card(Box::new(TopTake(TargetCount::Exact(count))));

            match &result {
                Ok(cards) => {
                    debug!(
                        "덱에서 카드 추출 성공: player={:?}, count={}",
                        player_type,
                        cards.len()
                    );
                }
                Err(e) => {
                    // 에러 로깅 추가
                    error!(
                        "덱에서 카드 추출 실패: player={:?}, error={:?}",
                        player_type, e
                    );
                }
            }

            // 성공 시 UUID 변환
            let cards = result?
                .iter()
                .map(|card| card.get_uuid())
                .collect::<Vec<_>>();
            debug!(
                "멀리건 카드 뽑기 완료: player={:?}, card_count={}",
                player_type,
                cards.len()
            );

            Ok(cards)
        }

        #[instrument(skip(self), fields(player_type = ?player_type.into()))]
        pub fn add_select_cards<T: Into<PlayerType> + Copy>(
            &mut self,
            cards: Vec<Uuid>,
            player_type: T,
        ) {
            let player_type = player_type.into();
            debug!(
                "멀리건 상태에 카드 추가 시작: player={:?}, cards={:?}",
                player_type, cards
            );

            let mut player = self.get_player_by_type(player_type).get();

            player
                .get_mulligan_state_mut()
                .add_select_cards(cards.clone());
            debug!("멀리건 상태에 카드 추가 완료: player={:?}", player_type);
        }

        pub fn add_reroll_cards<T: Into<PlayerType> + Copy>(
            &mut self,
            player_type: T,
            payload_cards: Vec<Uuid>,
            rerolled_cards: Vec<Uuid>,
        ) {
            let player_type = player_type.into();
            debug!("선택 카드 제거: player={:?}", player_type);
            self.get_player_by_type(player_type)
                .get()
                .get_mulligan_state_mut()
                .remove_select_cards(payload_cards);

            debug!("리롤된 카드 추가: player={:?}", player_type);
            self.get_player_by_type(player_type)
                .get()
                .get_mulligan_state_mut()
                .add_select_cards(rerolled_cards);
        }

        pub fn reroll_request<T: Into<PlayerType> + Copy>(
            &mut self,
            player_type: T,
            cards: Vec<Uuid>,
        ) -> Result<Vec<Uuid>, GameError> {
            let player_type = player_type.into();
            // 플레이어가 이미 준비 상태인 경우
            if self
                .get_player_by_type(player_type)
                .get()
                .get_mulligan_state_mut()
                .is_ready()
            {
                warn!("플레이어가 이미 준비 상태: player={:?}", player_type);
                return Err(GameError::AlreadyReady);
                // try_send_error!(session, GameError::AlreadyReady, retry 3);
            }

            // 플레이어가 선택한 카드가 유효한지 확인합니다.
            debug!("선택한 카드 유효성 검사: player={:?}", player_type);
            if let Err(e) = self.get_cards_by_uuids(cards.clone()) {
                error!("유효하지 않은 카드 선택: player={:?}", player_type);
                return Err(e);
            }

            // 기존 카드를 덱의 최하단에 위치 시킨 뒤, 새로운 카드를 뽑아서 player 의 mulligan cards 에 저장하고 json 으로 변환하여 전송합니다.
            info!("카드 리롤 시작: player={:?}", player_type);
            let rerolled_card = match self.restore_then_reroll_mulligan_cards(player_type, cards) {
                Ok(cards) => {
                    debug!("카드 리롤 성공: card_count={}", cards.len());
                    cards
                }
                Err(e) => {
                    error!("카드 리롤 실패: player={:?}, error={:?}", player_type, e);
                    panic!("카드 리롤 실패: player={:?}, error={:?}", player_type, e);
                }
            };

            Ok(rerolled_card)
        }

        /// 멀리건 완료 처리 함수
        /// - 게임 객체를 받아서, 플레이어의 멀리건 상태를 완료로 변경하고, 선택한 카드들을 손으로 이동시킵니다.
        /// - 선택한 카드들의 UUID를 반환합니다.
        /// # Arguments
        /// * `game` - 게임 객체
        /// * `player_type` - 플레이어 타입
        /// # Returns
        /// * `Vec<Uuid>` - 선택한 카드들의 UUID

        pub fn process_mulligan_completion<T: Into<PlayerType> + Copy>(
            &mut self,
            player_type: T,
        ) -> Result<Vec<Uuid>, GameError> {
            let player_type = player_type.into();

            // 선택된 멀리건 카드들의 UUID 를 얻습니다.
            let selected_cards = self
                .get_player_by_type(player_type)
                .get()
                .get_mulligan_state_mut()
                .get_select_cards();

            // UUID -> Card 객체로 변환하는 과정입니다.
            let cards = self.get_cards_by_uuids(selected_cards.clone())?;

            // add_card 함수를 통해 선택된 카드들을 손으로 이동시킵니다.
            self.get_player_by_type(player_type)
                .get()
                .get_hand_mut()
                .add_card(cards, Box::new(TopInsert))
                .map_err(|_| GameError::InternalServerError)?;

            // 멀리건 상태를 "완료" 상태로 변경합니다.
            self.get_player_by_type(player_type)
                .get()
                .get_mulligan_state_mut()
                .confirm_selection();

            // 그런 뒤, 선택한 카드들을 반환합니다.
            Ok(selected_cards)
        }

        pub fn check_player_ready_state<T: Into<PlayerType> + Copy>(&self, player_type: T) -> bool {
            let player_type = player_type.into();
            self.get_player_by_type(player_type.reverse())
                .get()
                .get_mulligan_state_mut()
                .is_ready()
        }
    }
}

pub enum PlayCardResult {
    Success,
    NeedInput(Receiver<InputAnswer>, HandlerType),
    Fail(GameError),
}

impl fmt::Debug for PlayCardResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayCardResult::Success => write!(f, "PlayCardResult::Success"),
            PlayCardResult::NeedInput(rx, _) => {
                write!(f, "PlayCardResult::NeedInput({:?}, <function>)", rx)
            }
            PlayCardResult::Fail(err) => write!(f, "PlayCardResult::Fail({:?})", err),
        }
    }
}

pub mod main_phase1 {

    use super::*;
    impl Game {
        // 카드를 처리하는 함수인데
        // 외부 ( end point ) 에서 사용하는 함수라서 카드 처리 함수는 이 함수로 유일해야함.
        pub async fn proceed_card<T: Into<PlayerType> + Copy>(
            &mut self,
            player_type: T,
            card_uuid: Uuid,
        ) -> Result<PlayCardResult, GameError> {
            let player_type = player_type.into();
            info!(
                "카드 처리 시작: player={:?}, card_uuid={:?}",
                player_type, card_uuid
            );

            // 카드 조회 및 활성화 가능 여부 확인
            let card = self.get_cards_by_uuid(card_uuid)?;
            // card.can_activate(&self)?;

            // 효과 처리 다시 작업해야함.
            // 카드의 종료는 다음과 같음.
            // 1 . 프리체인 ( 체인에 안걸리고 바로 발동하는 효과들 )
            // 2 . 체인에 걸리는 효과들 ( 카드 효과들 )

            // 체인 형성 중 일반 카드 ( 일반 소환 등 )는 사용할 수 없음.
            // 카드 효과 유발에 레벨을 개념을 적용시켜야함.
            // 스펠 스피드 1, 2, 3 등으로 나눔.
            // 스펠 스피드 1 은 가장 느린 스피드를 가지는 효과로써, 체인을 이어갈 수 없음.
            // 스펠 스피드 2 는 스피드 1, 2에 효과에 대해 체인을 이어갈 수 있음.
            // 스펠 스피드 3 은 스피드 1, 2, 3에 효과에 대해 체인을 이어갈 수 있음.

            // Chain에 카드 효과 처리 위임
            let mut chain = std::mem::take(self.get_chain_mut());
            let result = chain.process_card_effects(self, player_type, card).await;
            *self.get_chain_mut() = chain;

            result
        }
    }
}

pub mod gmae_effects_funcs {

    use crate::{card::Card, effect::DigEffect};

    use super::*;

    impl Game {
        /// 카드 탐색(Digging) 기능을 수행합니다.
        ///
        /// # Arguments
        /// * `player_type` - 카드를 탐색하는 플레이어
        /// * `effect_id` - 사용할 탐색 효과의 ID
        /// * `card_uuid` - 탐색 효과가 있는 소스 카드의 UUID
        ///
        /// # Returns
        /// * `Result<Vec<Uuid>, GameError>` - 탐색 가능한 카드들의 UUID 목록
        pub fn digging_cards<T: Into<PlayerType> + Copy>(
            &mut self,
            player_type: T,
            effect_id: Uuid,
            card_uuid: Uuid,
        ) -> Result<Vec<Uuid>, GameError> {
            let player_type = player_type.into();
            info!(
                "카드 탐색 시작: player={:?}, effect_id={:?}, card={}",
                player_type, effect_id, card_uuid
            );

            // 1. 소스 카드 찾기
            let source_card = self.get_cards_by_uuid(card_uuid)?;

            // 2. 카드에서 해당 Dig 효과 찾기
            let dig_effect = self.find_dig_effect(&source_card, effect_id)?;

            // 3. 선택 가능한 카드들 찾기
            let selectable_cards = dig_effect
                .get_selector()
                .select_targets(self, &source_card)
                .map_err(|e| {
                    error!("대상 선택 실패: {:?}", e);
                    GameError::InvalidTarget
                })?;

            // 4. 선택 가능한 카드가 없는 경우 처리
            if selectable_cards.is_empty() {
                warn!("선택 가능한 카드가 없음: player={:?}", player_type);
                return Err(GameError::NoValidTargets);
            }

            // 5. UUID 목록 생성
            let card_uuids: Vec<Uuid> = selectable_cards
                .iter()
                .map(|card| card.get_uuid())
                .collect();

            debug!(
                "탐색 가능한 카드: count={}, uuids={:?}",
                card_uuids.len(),
                card_uuids
            );

            Ok(card_uuids)
        }

        /// 카드에서 특정 ID를 가진 DigEffect를 찾습니다.
        fn find_dig_effect<'a>(
            &mut self,
            card: &'a Card,
            effect_id: Uuid,
        ) -> Result<&'a DigEffect, GameError> {
            // 효과 찾기
            let effect = card
                .get_prioritized_effect()
                .iter()
                .find(|e| e.get_effect().get_id() == effect_id)
                .ok_or_else(|| {
                    error!("효과를 찾을 수 없음: effect_id={:?}", effect_id);
                    GameError::EffectNotFound
                })?;

            // DigEffect로 다운캐스팅
            effect
                .get_effect()
                .as_any()
                .downcast_ref::<DigEffect>()
                .ok_or_else(|| {
                    error!(
                        "잘못된 효과 타입: expected=DigEffect, effect_id={:?}",
                        effect_id
                    );
                    GameError::InvalidEffectType
                })
        }
    }
}

impl Game {
    // pub fn handle_phase_start(&mut self) -> PhaseResultType {
    //     match self.get_phase() {
    //         Phase::Mulligan => Ok(PhaseResult::Mulligan),
    //         Phase::DrawPhase => Ok(PhaseResult::DrawPhase),
    //         Phase::StandbyPhase => self.handle_standby_phase(),
    //         Phase::MainPhaseStart => self.handle_main_phase_start(),
    //         Phase::MainPhase1 => self.handle_main_phase_1(),
    //         Phase::BattlePhaseStart => self.handle_battle_phase_start(),
    //         Phase::BattleStep => self.handle_battle_step(),
    //         Phase::BattleDamageStepStart => self.handle_damage_step_start(),
    //         Phase::BattleDamageStepCalculationBefore => self.handle_before_damage_calculation(),
    //         Phase::BattleDamageStepCalculationStart => self.handle_damage_calculation(),
    //         Phase::BattleDamageStepCalculationEnd => self.handle_after_damage_calculation(),
    //         Phase::BattleDamageStepEnd => self.handle_damage_step_end(),
    //         Phase::BattlePhaseEnd => self.handle_battle_phase_end(),
    //         Phase::MainPhase2 => self.handle_main_phase_2(),
    //         Phase::EndPhase => self.handle_end_phase(),
    //     }
    // }

    pub fn handle_muliigan_phase(&mut self) {
        // 멀리건 페이즈 시작
        info!("멀리건 페이즈 시작");

        // 멀리건 페이즈 종료
        info!("멀리건 페이즈 종료");
    }

    #[instrument(skip(self), fields(player_type = ?player_type.into()))]
    pub fn handle_draw_phase<T: Into<PlayerType> + Copy>(
        &mut self,
        player_type: T,
    ) -> Result<Uuid, GameError> {
        let player_type = player_type.into();
        info!("드로우 페이즈 시작: player={:?}", player_type);

        trace!("드로우 페이즈 효과 발동 중...");
        // self.trigger_draw_phase_effects()?;
        debug!("드로우 페이즈 효과 발동 완료");

        let card = self
            .draw_card(player_type)
            .log_ok(|| debug!("카드 드로우 성공: player={:?}", player_type,))
            .map_err(|e| {
                error!("카드 드로우 실패: player={:?}, error={:?}", player_type, e);
                e
            })?;

        debug!(
            "카드 드로우 성공: player={:?}, card_uuid={}",
            player_type,
            card.get_uuid()
        );

        trace!("핸드에 카드 추가 시작: player={:?}", player_type);
        let mut player = self.get_player_by_type(player_type).get();

        player
            .get_hand_mut()
            .add_card(vec![card.clone()], Box::new(TopInsert))
            .log_ok(|| debug!("핸드에 카드 추가 성공: player={:?}", player_type))
            .log_err(|e| {
                warn!(
                    "핸드에 카드 추가 중 문제 발생: player={:?}, error={:?}",
                    player_type, e
                )
            })?;

        info!(
            "드로우 페이즈 완료: player={:?}, card_uuid={}",
            player_type,
            card.get_uuid()
        );
        Ok(card.get_uuid())
    }

    pub fn handle_standby_phase(&mut self) -> PhaseResultType {
        // 스탠바이 페이즈에서 발동하는 효과들 처리
        self.trigger_standby_effects()?;
        todo!()
    }

    pub fn handle_main_phase_start(&mut self) -> PhaseResultType {
        // 메인 페이즈 1 개시시 효과 처리
        self.trigger_main_phase_start_effects()?;
        todo!()
    }

    fn handle_main_phase_1(&mut self) -> PhaseResultType {
        // 메인 페이즈 1 진입 처리
        todo!()
    }

    fn handle_battle_phase_start(&mut self) -> PhaseResultType {
        // 배틀 페이즈 개시시 효과 처리
        self.trigger_battle_phase_start_effects()?;
        todo!()
    }

    fn handle_battle_step(&mut self) -> PhaseResultType {
        // 배틀 스텝 처리
        todo!()
    }

    fn handle_damage_step_start(&mut self) -> PhaseResultType {
        // 데미지 스텝 시작 처리
        self.trigger_damage_step_start_effects()?;
        todo!()
    }

    fn handle_before_damage_calculation(&mut self) -> PhaseResultType {
        // 데미지 계산 전 효과 처리
        todo!()
    }

    fn handle_damage_calculation(&mut self) -> PhaseResultType {
        // 실제 데미지 계산 처리
        self.calculate_battle_damage()?;
        todo!()
    }

    fn handle_after_damage_calculation(&mut self) -> PhaseResultType {
        // 데미지 계산 후 효과 처리
        todo!()
    }

    fn handle_damage_step_end(&mut self) -> PhaseResultType {
        // 데미지 스텝 종료 처리
        todo!()
    }

    fn handle_battle_phase_end(&mut self) -> PhaseResultType {
        // 배틀 페이즈 종료 처리
        todo!()
    }

    fn handle_main_phase_2(&mut self) -> PhaseResultType {
        // 메인 페이즈 2 처리
        todo!()
    }

    fn handle_end_phase(&mut self) -> PhaseResultType {
        // 턴 종료 처리
        self.handle_turn_end()?;
        todo!()
    }

    /// 페이즈 종료 시 처리
    fn handle_phase_end(&mut self) -> PhaseResultType {
        // 현재 페이즈 종료 시 필요한 처리
        todo!()
    }

    /// 턴 종료 처리
    fn handle_turn_end(&mut self) -> PhaseResultType {
        todo!()
    }

    //
    fn trigger_draw_phase_effects(&mut self) -> PhaseResultType {
        // 스탠바이 페이즈 효과 발동
        todo!()
    }

    // 유틸리티 메서드들
    fn trigger_standby_effects(&mut self) -> PhaseResultType {
        // 스탠바이 페이즈 효과 발동
        todo!()
    }

    fn trigger_main_phase_start_effects(&mut self) -> PhaseResultType {
        // 메인 페이즈 개시시 효과 발동
        todo!()
    }

    fn trigger_battle_phase_start_effects(&mut self) -> PhaseResultType {
        // 배틀 페이즈 개시시 효과 발동
        todo!()
    }

    fn trigger_damage_step_start_effects(&mut self) -> PhaseResultType {
        // 데미지 스텝 개시시 효과 발동
        todo!()
    }

    fn calculate_battle_damage(&mut self) -> PhaseResultType {
        // 전투 데미지 계산
        todo!()
    }

    fn check_hand_limit(&mut self) -> PhaseResultType {
        // 손 카드 제한(10장) 체크
        todo!()
    }
}
