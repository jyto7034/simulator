#[cfg(test)]
mod tests {
    use simulator::{game::game::{GameConfig, Game}, deck::Deck};

    #[test]
    fn check_entity_type() {
        use simulator::unit::Entity;
        let hero = simulator::unit::hero::Hero::new().get_entity_type();
        assert_eq!(hero, "Hero".to_string());

        let agent = simulator::unit::agent::Agent::new().get_entity_type();
        assert_eq!(agent, "Agent".to_string());
    }

    #[test]
    fn check_set_opponent_player(){
        let config = GameConfig{
            player_1: Deck{raw_deck_code: "".to_string()},
            player_2: Deck{raw_deck_code: "".to_string()},
            attaker: 1,
        };

        let game = Game{

        };
    }
}
