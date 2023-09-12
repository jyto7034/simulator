#[cfg(test)]
mod tests {
    use simulator::{
        deck::Deck,
        game::game::{Game, GameConfig},
        task::Procedure,
        utils::utils,
    };

    fn generate_game() -> Game {
        let config = GameConfig {
            player_1: Deck {
                raw_deck_code: "".to_string(),
            },
            player_2: Deck {
                raw_deck_code: "".to_string(),
            },
            attaker: 1,
            name: vec!["test1".to_string(), "test2".to_string()],
        };

        let task_proc = Procedure { task_queue: vec![] };

        let mut game = Game {
            player_1: None,
            player_2: None,
            task: task_proc,
        };

        match game.initialize(config) {
            Ok(_) => {}
            Err(err) => {
                println!("{err}");
            }
        }

        game
    }

    #[test]
    fn check_entity_type() {
        use simulator::unit::Entity;
        let hero = simulator::unit::hero::Hero::new().get_entity_type();
        assert_eq!(hero, "Hero".to_string());

        let agent = simulator::unit::agent::Agent::new().get_entity_type();
        assert_eq!(agent, "Agent".to_string());
    }

    #[test]
    fn check_set_opponent_player() {
        let config = GameConfig {
            player_1: Deck {
                raw_deck_code: "".to_string(),
            },
            player_2: Deck {
                raw_deck_code: "".to_string(),
            },
            attaker: 1,
            name: vec!["test1".to_string(), "test2".to_string()],
        };

        let task_proc = Procedure { task_queue: vec![] };

        let mut game = Game {
            player_1: None,
            player_2: None,
            task: task_proc,
        };

        match game.initialize(config) {
            Ok(_) => {}
            Err(err) => {
                println!("{err}");
            }
        }

        assert_eq!(game.player_1.as_ref().unwrap().borrow().name, "test1");
        assert_eq!(game.player_2.as_ref().unwrap().borrow().name, "test2");

        assert_eq!(
            game.player_1
                .as_ref()
                .unwrap()
                .borrow()
                .opponent
                .as_ref()
                .unwrap()
                .borrow()
                .name,
            "test2"
        );
        assert_eq!(
            game.player_2
                .as_ref()
                .unwrap()
                .borrow()
                .opponent
                .as_ref()
                .unwrap()
                .borrow()
                .name,
            "test1"
        );

        game.player_1.as_ref().unwrap().borrow_mut().name = "player2".to_string();
        assert_eq!(game.player_1.as_ref().unwrap().borrow().name, "player2");
        game.player_2.as_ref().unwrap().borrow_mut().name = "player1".to_string();
        assert_eq!(game.player_2.as_ref().unwrap().borrow().name, "player1");
    }

    mod utils_test {
        use super::*;

        #[test]
        fn check_generate_uuid() {
            match utils::generate_uuid() {
                Ok(_) => {}
                Err(err) => {
                    assert!(false, "{err}");
                }
            }
        }

        #[test]
        fn test_load_card_data() {
            match utils::parse_json() {
                Ok(json) => match utils::load_card_data(&json) {
                    Ok(data) => {
                        println!("{:#?}", data);
                    }
                    Err(err) => {
                        assert!(false, "{err}");
                    }
                },
                Err(err) => {
                    assert!(false, "{err}");
                }
            }
        }
    }

    mod task_test {
        use super::*;
        use simulator::{game::Behavior, task::Task, enums::*};

        fn add_task(proc: &mut Procedure) -> Task{
            let task = match Task::new(
                Behavior::AddCardToDeck,
                TaskPriority::Immediately,
            ) {
                Ok(task) => task,
                Err(err) => {
                    assert!(false, "{err}");
                    Task::dummy()
                }
            };
            proc.add_task(&task);
            task
        }
            
        #[test]
        fn test_task_remove() {
            let mut proc = Procedure::new();
            let task = add_task(&mut proc);
            proc.find_task(task.get_task_uuid());

        }

        #[test]
        fn test_task_find() {
            todo!()
        }

        #[test]
        fn test_task_excution() {
            todo!()
        }
    }
}
