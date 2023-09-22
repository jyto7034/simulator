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

        assert_eq!(
            *game.player_1.as_ref().unwrap().borrow().get_name(),
            "test1"
        );
        assert_eq!(
            *game.player_2.as_ref().unwrap().borrow().get_name(),
            "test2"
        );

        assert_eq!(
            game.player_1
                .as_ref()
                .unwrap()
                .borrow()
                .get_opponent()
                .as_ref()
                .unwrap()
                .borrow()
                .get_name(),
            "test2"
        );
        assert_eq!(
            game.player_2
                .as_ref()
                .unwrap()
                .borrow()
                .get_opponent()
                .as_ref()
                .unwrap()
                .borrow()
                .get_name(),
            "test1"
        );

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
            .set_name("player2".to_string());
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
            let card = card_generator.gen_card_by_id("test".to_string(), &CardJson::new());
            println!("{:#?}", card);
        }
    }

    mod task_test {

        use super::*;
        use simulator::{enums::*, game::Behavior, task::Task};

        fn add_task(proc: &mut Procedure) -> Task {
            let task = match Task::new(Behavior::AddCardToDeck, TaskPriority::Immediately) {
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
            // task 삭제하는 기능을 테스트 하는 함수입니다.
            use simulator::exception::exception::Exception;
            let mut proc = Procedure::new();

            let task = add_task(&mut proc);

            match proc.remove_task_by_uuid(task.get_task_uuid()) {
                Ok(_) => {}
                Err(err) => {
                    match err {
                        Exception::NothingToRemove => {}
                        _ => assert!(false, "{err}"),
                    };
                }
            }

            for item in &proc.task_queue {
                if item.get_task_uuid() == task.get_task_uuid() {
                    assert!(false, "Exist");
                }
            }
        }

        // TODO: 일부로 오류내는 함수 작성해야함.

        #[test]
        fn test_task_find() {
            let mut proc = Procedure::new();
            let tasks = vec![
                add_task(&mut proc),
                add_task(&mut proc),
                add_task(&mut proc),
                add_task(&mut proc),
            ];
            for item in &tasks {
                if let Some(task) = proc.find_task_by_ref(item) {
                    if task.get_task_uuid() != item.get_task_uuid() {
                        assert!(false, "Diff");
                    }
                }
            }

            if let Some(task) = proc.find_task_by_ref(&tasks[0]) {
                if task.get_task_uuid() == &"error".to_string() {
                    assert!(false, "diff");
                }
            }
        }

        #[test]
        fn test_task_excution() {
            // todo!()
        }
    }
}
