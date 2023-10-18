#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use simulator::{
        deck::Deck,
        exception::exception::Exception,
        game::game::{Game, GameConfig},
        task::Procedure,
        utils::utils,
    };

    fn generate_game() -> Result<Game, Exception> {
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

        let proc = Rc::new(RefCell::new(Procedure::new(None)));
        let game = Game::new(Some(proc));
        if let Ok(mut game) = game {
            match game.initialize(config) {
                Ok(_) => Ok(game),
                Err(err) => {
                    println!("{err}");
                    return Err(err);
                }
            }
        } else {
            Err(Exception::GameInitializeFailed)
        }
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

        let game = generate_game().unwrap();

        assert_eq!(
            *game.player_1.as_ref().unwrap().borrow().get_name(),
            "test1"
        );
        assert_eq!(
            *game.player_2.as_ref().unwrap().borrow().get_name(),
            "test2"
        );

        let name = if let Some(data) = game
            .player_1
            .as_ref()
            .and_then(|player| Some(player.as_ref().borrow().get_name().clone()))
        {
            data
        } else {
            "".to_string()
        };
        assert_eq!(name, "test1");

        let name = if let Some(data) = game
            .player_2
            .as_ref()
            .and_then(|player| Some(player.as_ref().borrow().get_name().clone()))
        {
            data
        } else {
            "".to_string()
        };
        assert_eq!(name, "test2");

        game.player_1
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_name("player2".to_string());
        assert_eq!(
            game.player_1.as_ref().unwrap().borrow().get_name(),
            "player2"
        );
        game.player_2
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_name("player1".to_string());
        assert_eq!(
            game.player_2.as_ref().unwrap().borrow().get_name(),
            "player1"
        );
    }

    mod utils_test {
        use simulator::{card_gen::card_gen::CardGenertor, utils::json::CardJson};

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

        // TODO: assert 문 넣어야함.
        #[test]
        fn test_load_card_id() {
            match utils::load_card_id() {
                Ok(data) => println!("{:#?}", data),
                Err(_) => {}
            }
        }

        #[test]
        fn test_card_genertor() {
            let card_generator = CardGenertor::new();
            let card = card_generator.gen_card_by_id("test".to_string(), &CardJson::new(), 1);
            println!("{:#?}", card);
        }
    }

    mod task_test {

        use std::rc::Weak;

        use super::*;
        use simulator::{enums::*, game::Behavior, task::Task};

        fn create_task_and_push(proc: &Rc<RefCell<Procedure>>) -> Task {
            let task = match Task::new(
                PlayerType::Player1,
                &"".to_string(),
                Behavior::AddCardToDeck,
                TaskPriority::Immediately,
            ) {
                Ok(task) => task,
                Err(err) => {
                    assert!(false, "{err}");
                    Task::dummy()
                }
            };
            proc.as_ref().borrow_mut().add_task(&task);
            task
        }

        #[test]
        fn test_task_remove() {
            // task 삭제하는 기능을 테스트 하는 함수입니다.
            let game = generate_game().unwrap();
            let task = create_task_and_push(&game.procedure.as_ref().unwrap().clone());
            let d = game.procedure.as_ref().unwrap().as_ref().borrow().get_task_list();
            println!("{:#?}", d);
            if let Some(proc) = &game.procedure {
                    let result = proc
                        .borrow_mut()
                        .remove_task_by_uuid(task.get_task_uuid());
                    println!("{:#?}", d);

                    match result {
                        Ok(_) => {
                            let exists = proc
                                .borrow()
                                .task_queue
                                .iter()
                                .any(|item| item.get_task_uuid() == task.get_task_uuid());  
                            assert!(!exists, "Task still exists");
                        }
                        Err(Exception::NothingToRemove) => {}
                        Err(err) => assert!(false, "{}", err),
                    }
            } else {
                assert!(false, "Weak err");
            }
        }

        // TODO: 일부로 오류내는 함수 작성해야함.

        #[test]
        fn test_task_find() {
            let game = generate_game().unwrap();
        }

        #[test]
        fn test_task_excution() {
            // todo!()
        }
    }
}
