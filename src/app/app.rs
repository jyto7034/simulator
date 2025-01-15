use crate::{
    enums::DeckCode,
    exception::exception::Exception,
    game::game::{Game, GameConfig},
    procedure::procedure::Procedure,
    server::schema::{Message, MessageInfo, Respones},
    OptRcRef,
};

/// client 로부터 msg 를 받으면 그것을 해석 후
/// Procedure 의 함수를 통해 Game 의 상태를 수정/관리함.
pub struct App {
    pub game: Game,
    pub procedure: Procedure,
}

impl App {
    pub fn instantiate() -> App {
        App {
            game: Game {
                player1: OptRcRef::none(),
                player2: OptRcRef::none(),
            },
            procedure: Procedure {
                tasks: vec![],
                trigger_tasks: vec![],
            },
        }
    }

    pub fn initialize_game(
        &mut self,
        _code1: DeckCode,
        _code2: DeckCode,
        attacker: usize,
    ) -> Result<(), Exception> {
        let config = GameConfig {
            player_1_deckcode: _code1,
            player_2_deckcode: _code2,
            attacker,
        };

        self.game.initialize(config)?;

        Ok(())
    }

    pub fn execute_msg(&mut self, info: MessageInfo) -> Result<(), Exception> {
        match info.msg {
            Message::CreateGame => todo!(),
            Message::EntryGame => todo!(),
            Message::PlayCardWithTarget(_played_card, _target_cards) => todo!(),
            Message::SelectMulligunCard => todo!(),
            Message::GetMulligunCards(data) => Respones::get_mulligun_cards(self, info, data),
            Message::PlayCard(_played_card) => todo!(),
            Message::DrawCard => todo!(),
            Message::AttackTo => todo!(),
            Message::TurnEnd => todo!(),
            Message::None => todo!(),
        }
    }
}

impl App {}
