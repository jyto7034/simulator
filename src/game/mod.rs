pub mod turn_manager;

use turn_manager::TurnManager;

use crate::{
    card::deck::deckcode_to_cards, enums::{phase::Phase, DeckCode, InsertType, PlayerType, ZoneType, PLAYER_1, PLAYER_2}, exception::Exception, unit::player::{Player, Resoruce}, OptRcRef
};

pub struct GameConfig {
    /// Player's Deckcode
    pub player_1_deckcode: DeckCode,
    pub player_2_deckcode: DeckCode,

    /// 1 : Player 1,
    /// 2 : Player 2
    pub attacker: usize,
}

/// 게임의 상태를 관리/저장 하는 구조체
/// Card 로 인한 모든 변경 사항은 Task 로써 저장되며,
/// 그것을 담은 Tasks 를 Procedure 에게 전달하여 게임 결과를 계산한다.
#[derive(Clone)]
pub struct Game {
    pub player1: OptRcRef<Player>,
    pub player2: OptRcRef<Player>,
    pub current_phase: Phase,
    pub turn: TurnManager,
}

/// initialize 함수에 GameConfig 을 넣음으로써 두 플레이어의 Cards 을 설정한다.
impl Game {
    pub fn initialize(&mut self, _config: GameConfig) -> Result<(), Exception> {
        let cards = deckcode_to_cards(_config.player_1_deckcode, _config.player_2_deckcode)?;

        println!("{}, {}", cards[0].len(), cards[1].len());

        // Player 설정
        self.player1 = OptRcRef::new(Player::new(
            OptRcRef::none(),
            PlayerType::Player1,
            cards[PLAYER_1].clone(),
            Resoruce::new(0, 3),
            Resoruce::new(0, 3),
        ));
        self.player2 = OptRcRef::new(Player::new(
            OptRcRef::none(),
            PlayerType::Player2,
            cards[PLAYER_2].clone(),
            Resoruce::new(0, 3),
            Resoruce::new(0, 3),
        ));

        // 순환 참조이긴 한데, 딱히 문제 없음. 정리만 수동적으로 잘 정리해주면 됨

        self.player1.get_mut().opponent = OptRcRef::clone(&self.player2);
        self.player2.get_mut().opponent = OptRcRef::clone(&self.player1);

        let cards = self.player1.get().get_cards().clone();
        for card in &cards {
            self.player1
                .get_mut()
                .add_card(ZoneType::DeckZone, card.clone(), InsertType::Top)?;
        }

        let cards = self.player2.get().get_cards().clone();
        for card in &cards {
            self.player2
                .get_mut()
                .add_card(ZoneType::DeckZone, card.clone(), InsertType::Top)?;
        }

        self.player1.get_mut().set_cost(0);
        self.player1.get_mut().set_mana(0);

        self.player2.get_mut().set_cost(0);
        self.player2.get_mut().set_mana(0);

        Ok(())
    }

    pub fn get_player(&self, player_type: PlayerType) -> &OptRcRef<Player> {
        match player_type {
            PlayerType::Player1 => &self.player1,
            PlayerType::Player2 => &self.player2,
            PlayerType::None => todo!(),
        }
    }

    pub fn proceed_phase(&mut self) -> Result<(), Exception> {
        let next_phase = self.current_phase.next_phase();
        self.handle_phase_transition(next_phase)
    }

    /// 페이즈 전환 처리
    fn handle_phase_transition(&mut self, next_phase: Phase) -> Result<(), Exception> {
        // 페이즈 전환 전 현재 페이즈의 종료 처리
        self.handle_phase_end()?;
        
        self.current_phase = next_phase;
        
        // 새로운 페이즈의 시작 처리
        self.handle_phase_start()?;
        
        Ok(())
    }

    /// 페이즈 시작 시 처리
    fn handle_phase_start(&mut self) -> Result<(), Exception> {
        match self.current_phase {
            Phase::DrawPhase => self.handle_draw_phase()?,
            Phase::StandbyPhase => self.handle_standby_phase()?,
            Phase::MainPhaseStart => self.handle_main_phase_start()?,
            Phase::MainPhase1 => self.handle_main_phase_1()?,
            Phase::BattlePhaseStart => self.handle_battle_phase_start()?,
            Phase::BattleStep => self.handle_battle_step()?,
            Phase::BattleDamageStepStart => self.handle_damage_step_start()?,
            Phase::BattleDamageStepCalculationBefore => self.handle_before_damage_calculation()?,
            Phase::BattleDamageStepCalculationStart => self.handle_damage_calculation()?,
            Phase::BattleDamageStepCalculationEnd => self.handle_after_damage_calculation()?,
            Phase::BattleDamageStepEnd => self.handle_damage_step_end()?,
            Phase::BattlePhaseEnd => self.handle_battle_phase_end()?,
            Phase::MainPhase2 => self.handle_main_phase_2()?,
            Phase::EndPhase => self.handle_end_phase()?,
        }
        Ok(())
    }

    // 각 페이즈별 구체적인 처리
    fn handle_draw_phase(&mut self) -> Result<(), Exception> {
        Ok(())
    }

    fn handle_standby_phase(&mut self) -> Result<(), Exception> {
        // 스탠바이 페이즈에서 발동하는 효과들 처리
        self.trigger_standby_effects()?;
        Ok(())
    }

    fn handle_main_phase_start(&mut self) -> Result<(), Exception> {
        // 메인 페이즈 1 개시시 효과 처리
        self.trigger_main_phase_start_effects()?;
        Ok(())
    }

    fn handle_main_phase_1(&mut self) -> Result<(), Exception> {
        // 메인 페이즈 1 진입 처리
        Ok(())
    }

    fn handle_battle_phase_start(&mut self) -> Result<(), Exception> {
        // 배틀 페이즈 개시시 효과 처리
        self.trigger_battle_phase_start_effects()?;
        Ok(())
    }

    fn handle_battle_step(&mut self) -> Result<(), Exception> {
        // 배틀 스텝 처리
        Ok(())
    }

    fn handle_damage_step_start(&mut self) -> Result<(), Exception> {
        // 데미지 스텝 시작 처리
        self.trigger_damage_step_start_effects()?;
        Ok(())
    }

    fn handle_before_damage_calculation(&mut self) -> Result<(), Exception> {
        // 데미지 계산 전 효과 처리
        Ok(())
    }

    fn handle_damage_calculation(&mut self) -> Result<(), Exception> {
        // 실제 데미지 계산 처리
        self.calculate_battle_damage()?;
        Ok(())
    }

    fn handle_after_damage_calculation(&mut self) -> Result<(), Exception> {
        // 데미지 계산 후 효과 처리
        Ok(())
    }

    fn handle_damage_step_end(&mut self) -> Result<(), Exception> {
        // 데미지 스텝 종료 처리
        Ok(())
    }

    fn handle_battle_phase_end(&mut self) -> Result<(), Exception> {
        // 배틀 페이즈 종료 처리
        Ok(())
    }

    fn handle_main_phase_2(&mut self) -> Result<(), Exception> {
        // 메인 페이즈 2 처리
        Ok(())
    }

    fn handle_end_phase(&mut self) -> Result<(), Exception> {
        // 턴 종료 처리
        self.handle_turn_end()?;
        Ok(())
    }

    /// 페이즈 종료 시 처리
    fn handle_phase_end(&mut self) -> Result<(), Exception> {
        // 현재 페이즈 종료 시 필요한 처리
        Ok(())
    }

    /// 턴 종료 처리
    fn handle_turn_end(&mut self) -> Result<(), Exception> {
        Ok(())
    }

    // 유틸리티 메서드들
    fn trigger_standby_effects(&mut self) -> Result<(), Exception> {
        // 스탠바이 페이즈 효과 발동
        Ok(())
    }

    fn trigger_main_phase_start_effects(&mut self) -> Result<(), Exception> {
        // 메인 페이즈 개시시 효과 발동
        Ok(())
    }

    fn trigger_battle_phase_start_effects(&mut self) -> Result<(), Exception> {
        // 배틀 페이즈 개시시 효과 발동
        Ok(())
    }

    fn trigger_damage_step_start_effects(&mut self) -> Result<(), Exception> {
        // 데미지 스텝 개시시 효과 발동
        Ok(())
    }

    fn calculate_battle_damage(&mut self) -> Result<(), Exception> {
        // 전투 데미지 계산
        Ok(())
    }

    fn check_hand_limit(&mut self) -> Result<(), Exception> {
        // 손 카드 제한(10장) 체크
        Ok(())
    }

    
}
