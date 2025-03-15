use crate::{
    enums::DeckCode,
    exception::GameError,
    game::{
        phase::{Phase, PhaseState},
        turn_manager::Turn,
        Game, GameConfig,
    },
    OptArc,
};

/// client 로부터 msg 를 받으면 그것을 해석 후
/// Procedure 의 함수를 통해 Game 의 상태를 수정/관리함.
pub struct App {
    pub game: Game,
}

impl App {
    pub fn instantiate() -> App {
        App {
            game: Game {
                player1: OptArc::none(),
                player2: OptArc::none(),
                phase_state: PhaseState::new(Phase::Mulligan),
                turn: Turn::new(),
            },
        }
    }

    pub fn initialize_game(
        &mut self,
        _code1: DeckCode,
        _code2: DeckCode,
        attacker: usize,
    ) -> Result<(), GameError> {
        let config = GameConfig {
            player_1_deckcode: _code1,
            player_2_deckcode: _code2,
            attacker,
        };

        self.game.initialize(config)?;

        Ok(())
    }
}

impl App {}
