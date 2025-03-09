use crate::{
    card::{take::TopTake, types::PlayerType},
    enums::UUID,
    exception::GameError,
    selector::TargetCount,
    zone::zone::Zone,
};

use super::Game;

impl Game {
    // pub async fn proceed_phase(&mut self) -> Result<(), GameError> {
    //     let next_phase = self.phase.next_phase();
    //     self.handle_phase_transition(next_phase).await
    // }

    // /// 페이즈 전환 처리
    // pub async fn handle_phase_transition(&mut self, next_phase: Phase) -> Result<(), GameError> {
    //     // 페이즈 전환 전 현재 페이즈의 종료 처리
    //     self.handle_phase_end()?;

    //     self.phase = next_phase;

    //     // 새로운 페이즈의 시작 처리
    //     self.handle_phase_start().await?;

    //     Ok(())
    // }

    // /// 페이즈 시작 시 처리
    // pub async fn handle_phase_start(&mut self) -> Result<(), GameError> {
    //     match self.phase {
    //         Phase::GameStart => self.handle_game_start().await?,
    //         Phase::DrawPhase => self.handle_draw_phase()?,
    //         Phase::StandbyPhase => self.handle_standby_phase()?,
    //         Phase::MainPhaseStart => self.handle_main_phase_start()?,
    //         Phase::MainPhase1 => self.handle_main_phase_1()?,
    //         Phase::BattlePhaseStart => self.handle_battle_phase_start()?,
    //         Phase::BattleStep => self.handle_battle_step()?,
    //         Phase::BattleDamageStepStart => self.handle_damage_step_start()?,
    //         Phase::BattleDamageStepCalculationBefore => self.handle_before_damage_calculation()?,
    //         Phase::BattleDamageStepCalculationStart => self.handle_damage_calculation()?,
    //         Phase::BattleDamageStepCalculationEnd => self.handle_after_damage_calculation()?,
    //         Phase::BattleDamageStepEnd => self.handle_damage_step_end()?,
    //         Phase::BattlePhaseEnd => self.handle_battle_phase_end()?,
    //         Phase::MainPhase2 => self.handle_main_phase_2()?,
    //         Phase::EndPhase => self.handle_end_phase()?,
    //     }
    //     Ok(())
    // }

    pub fn get_mulligan_cards<T: Into<PlayerType> + Copy>(
        &mut self,
        player_type: T,
        count: usize,
    ) -> Result<Vec<UUID>, GameError> {
        Ok(self
            .get_player_by_type(player_type)
            .get()
            .get_deck_mut()
            .take_card(Box::new(TopTake(TargetCount::Exact(count))))
            .iter()
            .map(|card| return card.get_uuid())
            .collect())
    }

    // 각 페이즈별 구체적인 처리
    pub fn handle_draw_phase(&mut self) -> Result<(), GameError> {
        self.trigger_draw_phase_effects()?;

        // 카드 드로우
        Ok(())
    }

    pub fn handle_standby_phase(&mut self) -> Result<(), GameError> {
        // 스탠바이 페이즈에서 발동하는 효과들 처리
        self.trigger_standby_effects()?;
        Ok(())
    }

    pub fn handle_main_phase_start(&mut self) -> Result<(), GameError> {
        // 메인 페이즈 1 개시시 효과 처리
        self.trigger_main_phase_start_effects()?;
        Ok(())
    }

    fn handle_main_phase_1(&mut self) -> Result<(), GameError> {
        // 메인 페이즈 1 진입 처리
        Ok(())
    }

    fn handle_battle_phase_start(&mut self) -> Result<(), GameError> {
        // 배틀 페이즈 개시시 효과 처리
        self.trigger_battle_phase_start_effects()?;
        Ok(())
    }

    fn handle_battle_step(&mut self) -> Result<(), GameError> {
        // 배틀 스텝 처리
        Ok(())
    }

    fn handle_damage_step_start(&mut self) -> Result<(), GameError> {
        // 데미지 스텝 시작 처리
        self.trigger_damage_step_start_effects()?;
        Ok(())
    }

    fn handle_before_damage_calculation(&mut self) -> Result<(), GameError> {
        // 데미지 계산 전 효과 처리
        Ok(())
    }

    fn handle_damage_calculation(&mut self) -> Result<(), GameError> {
        // 실제 데미지 계산 처리
        self.calculate_battle_damage()?;
        Ok(())
    }

    fn handle_after_damage_calculation(&mut self) -> Result<(), GameError> {
        // 데미지 계산 후 효과 처리
        Ok(())
    }

    fn handle_damage_step_end(&mut self) -> Result<(), GameError> {
        // 데미지 스텝 종료 처리
        Ok(())
    }

    fn handle_battle_phase_end(&mut self) -> Result<(), GameError> {
        // 배틀 페이즈 종료 처리
        Ok(())
    }

    fn handle_main_phase_2(&mut self) -> Result<(), GameError> {
        // 메인 페이즈 2 처리
        Ok(())
    }

    fn handle_end_phase(&mut self) -> Result<(), GameError> {
        // 턴 종료 처리
        self.handle_turn_end()?;
        Ok(())
    }

    /// 페이즈 종료 시 처리
    fn handle_phase_end(&mut self) -> Result<(), GameError> {
        // 현재 페이즈 종료 시 필요한 처리
        Ok(())
    }

    /// 턴 종료 처리
    fn handle_turn_end(&mut self) -> Result<(), GameError> {
        Ok(())
    }

    //
    fn trigger_draw_phase_effects(&mut self) -> Result<(), GameError> {
        // 스탠바이 페이즈 효과 발동
        Ok(())
    }

    // 유틸리티 메서드들
    fn trigger_standby_effects(&mut self) -> Result<(), GameError> {
        // 스탠바이 페이즈 효과 발동
        Ok(())
    }

    fn trigger_main_phase_start_effects(&mut self) -> Result<(), GameError> {
        // 메인 페이즈 개시시 효과 발동
        Ok(())
    }

    fn trigger_battle_phase_start_effects(&mut self) -> Result<(), GameError> {
        // 배틀 페이즈 개시시 효과 발동
        Ok(())
    }

    fn trigger_damage_step_start_effects(&mut self) -> Result<(), GameError> {
        // 데미지 스텝 개시시 효과 발동
        Ok(())
    }

    fn calculate_battle_damage(&mut self) -> Result<(), GameError> {
        // 전투 데미지 계산
        Ok(())
    }

    fn check_hand_limit(&mut self) -> Result<(), GameError> {
        // 손 카드 제한(10장) 체크
        Ok(())
    }
}
