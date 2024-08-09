use crate::{
    enums::DeckCode,
    exception::exception::Exception,
    game::game::{Game, GameConfig},
    procedure::procedure::Procedure,
    server::schema::{Message, MessageInfo, Respones},
    utils::utils,
};

/// client 로부터 msg 를 받으면 그것을 해석 후
/// Procedure 의 함수를 통해 Game 의 상태를 수정/관리함.
pub struct App {
    pub game: Game,
    pub procedure: Procedure,
    pub debug_flag: bool,
}

impl App {
    pub fn instantiate() -> App {
        App {
            game: Game {
                player1: None,
                player2: None,
            },
            procedure: Procedure {
                tasks: vec![],
                trigger_tasks: vec![],
            },
            debug_flag: true,
        }
    }

    pub fn initialize(
        &mut self,
        _code1: Option<DeckCode>,
        _code2: Option<DeckCode>,
    ) -> Result<(), Exception> {
        let config = if self.debug_flag {
            let deckcodes = utils::parse_json_to_deck_code().unwrap();
            GameConfig {
                player_1: deckcodes.0,
                player_2: deckcodes.1,
                attaker: 1, // to random
                player_name: vec!["player1".to_string(), "player2".to_string()],
            }
        } else {
            GameConfig {
                player_1: _code1.unwrap(),
                player_2: _code2.unwrap(),
                attaker: 1, // to random
                // player_name 도 변경해야함.
                player_name: vec!["player1".to_string(), "player2".to_string()],
            }
        };

        self.game.initialize(config)?;

        Ok(())
    }

    pub fn execute_msg(&mut self, info: MessageInfo) -> Result<(), Exception> {
        match info.msg {
            Message::CreateGame => todo!(),
            Message::EntryGame => todo!(),
            Message::PlayCardWithTarget(played_card, target_cards) => todo!(),
            Message::SelectMulligunCard => todo!(),
            Message::GetMulligunCards(data) => Respones::get_mulligun_cards(self, info, data),
            Message::PlayCard(played_card) => todo!(),
            Message::DrawCard => todo!(),
            Message::AttackTo => todo!(),
            Message::TurnEnd => todo!(),
            Message::None => todo!(),
        }
    }
}

impl App {}
