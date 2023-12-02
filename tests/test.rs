#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use simulator::{
        deck::Deck,
        enums::*,
        exception::exception::Exception,
        game::game::{Game, GameConfig},
        game::Behavior,
        task::Procedure,
        task::Task,
        utils::utils,
    };

    fn generate_game() -> Result<Game, Exception> {
        let config = match utils::read_game_config_json() {
            Ok(data) => GameConfig {
                player_1: Deck {
                    raw_deck_code: data.DeckCodes[0].code1.clone(),
                },
                player_2: Deck {
                    raw_deck_code: data.DeckCodes[0].code2.clone(),
                },
                attaker: data.Attacker as usize,
                name: vec![data.Names[0].name1.clone(), data.Names[0].name2.clone()],
            },
            Err(err) => return Err(err),
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

    mod utils_test {
        use simulator::{card_gen::card_gen::CardGenerator, utils::json::CardJson};

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
            match utils::parse_json_to_deck_code() {
                Ok(deck_code) => match utils::load_card_data(deck_code) {
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
                Err(err) => {
                    assert!(false, "{err}");
                }
            }
        }

        #[test]
        fn test_card_genertor() {
            let card_generator = CardGenerator::new();
            let card =
                card_generator.gen_card_by_id_string("HM_001".to_string(), &CardJson::new(), 1);
            // println!("{:#?}", card);
        }
    }

    mod task_test {

        use super::*;

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

            if let Some(proc) = &game.procedure {
                let result = proc.borrow_mut().remove_task_by_uuid(task.get_task_uuid());

                match result {
                    Ok(_) => {
                        let exists = proc
                            .borrow()
                            .task_queue
                            .iter()
                            .any(|item| item.get_task_uuid() == task.get_task_uuid());
                        assert!(!exists, "Task still exists");
                    }
                    Err(Exception::NothingToRemove) => {
                        assert!(false, "Nothing to remove");
                    }
                    Err(err) => assert!(false, "{}", err),
                }
            } else {
                assert!(false, "Initialize failed");
            }
        }

        #[test]
        #[should_panic]
        fn test_task_remove_failing() {
            // task 삭제하는 기능을 테스트 하는 함수입니다.
            let game = generate_game().unwrap();

            if let Some(proc) = &game.procedure {
                let uuid = "wow".to_string();
                let result = proc.borrow_mut().remove_task_by_uuid(&uuid);

                match result {
                    Ok(_) => {
                        let exists = proc
                            .borrow()
                            .task_queue
                            .iter()
                            .any(|item| item.get_task_uuid() == &uuid);
                        assert!(!exists, "Task still exists");
                    }
                    Err(Exception::NothingToRemove) => {
                        assert!(false, "Nothing to remove");
                    }
                    Err(err) => assert!(false, "{}", err),
                }
            } else {
                assert!(false, "Initialize failed");
            }
        }

        #[test]
        fn test_task_find() {
            let game = generate_game().unwrap();
        }

        #[test]
        fn test_task_excution() {
            // todo!()
        }
    }

    mod game_test {
        use std::collections::HashMap;

        use super::*;
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
            let game = generate_game().unwrap();

            assert_eq!(
                *game.player_1.as_ref().unwrap().borrow().get_name(),
                "player1"
            );
            assert_eq!(
                *game.player_2.as_ref().unwrap().borrow().get_name(),
                "player2"
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
            assert_eq!(name, "player1");

            let name = if let Some(data) = game
                .player_2
                .as_ref()
                .and_then(|player| Some(player.as_ref().borrow().get_name().clone()))
            {
                data
            } else {
                "".to_string()
            };
            assert_eq!(name, "player2");

            if let Some(player) = &game.player_1 {
                player.as_ref().borrow_mut().set_name("player1".to_string());
            }
            // game.player_1
            //     .as_ref()
            //     .unwrap()
            //     .borrow_mut()
            //     .set_name("player2".to_string());
            assert_eq!(
                game.player_2.as_ref().unwrap().borrow().get_name(),
                "player2"
            );
            if let Some(player) = &game.player_2 {
                player.as_ref().borrow_mut().set_name("player2".to_string());
            }
            assert_eq!(
                game.player_2.as_ref().unwrap().borrow().get_name(),
                "player2"
            );
        }

        #[test]
        fn test_game_step_initialize() {
            let mut game = generate_game();

            if let Ok(game) = &mut game {
                match game.game_step_initialize() {
                    Ok(_) => {}
                    Err(err) => {
                        assert!(false, "{err}");
                    }
                }
            }
        }

        #[test]
        fn test_player_exceed_draw() {
            let mut game = generate_game();

            if let Ok(game) = &mut game {
                match game.game_step_initialize() {
                    Ok(_) => {}
                    Err(err) => {
                        assert!(false, "{err}");
                    }
                }

                match (&game.player_1, &game.player_2) {
                    (Some(player1), Some(player2)) => {
                        match player1
                            .as_ref()
                            .borrow_mut()
                            .draw(ZoneType::DeckZone, CardDrawType::Random(1))
                        {
                            Ok(card) => println!("{:#?}", card),
                            Err(_) => panic!("Exceed Draw"),
                        }
                        match player2
                            .as_ref()
                            .borrow_mut()
                            .draw(ZoneType::DeckZone, CardDrawType::Random(1))
                        {
                            Ok(card) => println!("{:#?}", card),
                            Err(_) => panic!("Exceed Draw"),
                        }
                    }
                    _ => {}
                }
            }
        }

        macro_rules! get_state {
            ($game:expr, $player:expr, $zone_type:expr) => {{
                // zone 에 있는 카드의 uuid 와 count 를 map 으로 집계하여 반환합니다.
                
                let mut map = HashMap::new();
                for item in $player
                    .as_ref()
                    .borrow_mut()
                    .get_zone($zone_type)
                    .get_cards()
                    .v_card
                    .iter()
                {
                    if map.contains_key(item.get_uuid()) {
                        if let Some(value) = map.get_mut(item.get_uuid()) {
                            *value += 1;
                        } else {
                            panic!();
                        }
                    } else {
                        map.insert(item.get_uuid().clone(), 1);
                    }
                }
                map
            }};
        }

        #[test]
        fn test_game_step_mulligun() {
            let mut game = generate_game();

            if let Ok(game) = &mut game {
                match game.game_step_initialize() {
                    Ok(_) => {}
                    Err(err) => {
                        assert!(false, "{err}");
                    }
                }
                
                let before1 =
                    get_state!(&game, &game.player_1.as_ref().unwrap(), ZoneType::DeckZone);

                let before2 =
                    get_state!(&game, &game.player_2.as_ref().unwrap(), ZoneType::DeckZone);

                match game.game_step_mulligun() {
                    Ok(_) => {
                        // 멀리건 이전의 상태를 저장 후( before ), 멀리건을 수행하고 멀리건 이후의 hand 와 deck 의 상태를 저장한다.
                        // 이후의 hand 와 deck 을 합쳤을때, before 상태와 같아야한다.
                        let after_hand =
                            get_state!(&game, &game.player_1.as_ref().unwrap(), ZoneType::HandZone);
                        let mut after_deck =
                            get_state!(&game, &game.player_1.as_ref().unwrap(), ZoneType::DeckZone);

                        // hand, deck 의 hashmap 을 합칩니다.
                        for (key, value) in after_hand.iter() {
                            after_deck
                                .entry(key.clone())
                                .and_modify(|exit| *exit += value)
                                .or_insert(*value);
                        }

                        // 멀리건이 성공적으로 잘 되었는지 확인합니다.
                        for (key, value) in before1 {
                            if after_deck[&key] != value {
                                assert!(false);
                            }
                        }
                        // player2 checker
                        {

                            let after_hand =
                            get_state!(&game, &game.player_2.as_ref().unwrap(), ZoneType::HandZone);
                        let mut after_deck =
                            get_state!(&game, &game.player_2.as_ref().unwrap(), ZoneType::DeckZone);

                        for (key, value) in after_hand.iter() {
                            after_deck
                                .entry(key.clone())
                                .and_modify(|exit| *exit += value)
                                .or_insert(*value);
                        }

                        for (key, value) in before2 {
                            if after_deck[&key] != value {
                                assert!(false);
                            }
                        }
                        }
                    }
                    Err(err) => {
                        assert!(false, "{err}");
                    }
                }
            }
        }
    }

    mod player_test {
    }
}
