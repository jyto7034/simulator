use uuid::Uuid;

use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    card::{insert::TopInsert, take::TopTake, types::PlayerType},
    exception::GameError,
    selector::TargetCount,
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
    use core::panic;

    use crate::unit::player;

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
