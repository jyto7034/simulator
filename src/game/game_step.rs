use uuid::Uuid;

use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    card::{insert::TopInsert, take::TopTake, types::PlayerType},
    exception::GameError,
    selector::TargetCount,
    zone::zone::Zone,
    LogExt,
};

use super::{phase::Phase, Game};

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

impl Game {
    pub fn handle_phase_start(&mut self) -> PhaseResultType {
        match self.get_phase() {
            Phase::Mulligan => Ok(PhaseResult::Mulligan),
            Phase::DrawPhase => Ok(PhaseResult::DrawPhase),
            Phase::StandbyPhase => self.handle_standby_phase(),
            Phase::MainPhaseStart => self.handle_main_phase_start(),
            Phase::MainPhase1 => self.handle_main_phase_1(),
            Phase::BattlePhaseStart => self.handle_battle_phase_start(),
            Phase::BattleStep => self.handle_battle_step(),
            Phase::BattleDamageStepStart => self.handle_damage_step_start(),
            Phase::BattleDamageStepCalculationBefore => self.handle_before_damage_calculation(),
            Phase::BattleDamageStepCalculationStart => self.handle_damage_calculation(),
            Phase::BattleDamageStepCalculationEnd => self.handle_after_damage_calculation(),
            Phase::BattleDamageStepEnd => self.handle_damage_step_end(),
            Phase::BattlePhaseEnd => self.handle_battle_phase_end(),
            Phase::MainPhase2 => self.handle_main_phase_2(),
            Phase::EndPhase => self.handle_end_phase(),
        }
    }

    pub fn get_mulligan_cards<T: Into<PlayerType> + Copy>(
        &mut self,
        player_type: T,
        count: usize,
    ) -> Result<Vec<Uuid>, GameError> {
        Ok(self
            .get_player_by_type(player_type)
            .get()
            .get_deck_mut()
            .take_card(Box::new(TopTake(TargetCount::Exact(count))))?
            .iter()
            .map(|card| return card.get_uuid())
            .collect())
    }

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

        self.phase_state.mark_player_completed(player_type);
        debug!("플레이어 드로우 완료 표시: player={:?}", player_type);

        trace!("드로우 페이즈 효과 발동 중...");
        // self.trigger_draw_phase_effects()?;
        debug!("드로우 페이즈 효과 발동 완료");

        let card = self
            .draw_card(player_type)
            .log_ok(|| debug!("카드 드로우 성공: player={:?}", player_type,))
            .map_err(|e| {
                error!("카드 드로우 실패: player={:?}, error={:?}", player_type, e);
                self.phase_state.reset_player_completed(player_type);
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
        self.trigger_battle_phase_start_effects();
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
