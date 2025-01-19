#[cfg(test)]
mod tests {
    use card_game::{
        app::app::App,
        enums::{PLAYER_1, *},
        game::game::Game,
        procedure::procedure::Procedure,
        test::card_json::*,
        utils::utils::parse_json_to_deck_code,
        OptRcRef,
    };

    fn initialize_app(p1_deck: String, p2_deck: String, attacker: usize) -> App {
        let mut app = App {
            game: Game {
                player1: OptRcRef::none(),
                player2: OptRcRef::none(),
            },
            procedure: Procedure {
                tasks: vec![],
                trigger_tasks: vec![],
            },
        };

        app.initialize_game(p1_deck, p2_deck, attacker)
            .expect("app initialize failed");
        app
    }

    #[test]
    fn init_cards() {
        let (p1_deck, p2_deck) = init_cards_json();

        let (p1_deck, p2_deck) = parse_json_to_deck_code(Some(p1_deck), Some(p2_deck))
            .expect("parse_json_to_deck_code failed");

        let app = initialize_app(p1_deck, p2_deck, PLAYER_1);

        let cards = app
            .game
            .get_player(PlayerType::Player2)
            .get()
            .get_cards()
            .clone();
        for card in &cards {
            println!("{:#?}", card.get_name());
        }
    }
}
