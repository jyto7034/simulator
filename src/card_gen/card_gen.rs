use crate::enums::CardType;
use crate::{deck::Card, utils, utils::json::CardJson};
use crate::{
    exception::exception::Exception,
    game::{Behavior, Game},
};

use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// -------------------------------------------------- FIELD
// [HM_001] Hieda no Akyuu - COST:?? [ATK:??/HP:?]
// - Set: Human, Rarity: C
// --------------------------------------------------------
// Text: 낮동안 인간 카드를 사용할 때 마다 서로 1장 드로우 한다.
// --------------------------------------------------------
// Behaviors:
// - DrawCardFromDeck
// --------------------------------------------------------
fn test(card_json: &CardJson, count: usize) -> Card {
    let uuid = match utils::utils::generate_uuid() {
        Ok(data) => data,
        Err(err) => {
            panic!("test func failed {err}");
        }
    };
    let bvs = vec![Behavior::ListenOtherEvent, Behavior::DrawCardFromDeck];
    let run = Rc::new(RefCell::new(
        |card: &Card, game: &mut Game| -> Result<(), Exception> { Ok(()) },
    ));
    Card::new(
        CardType::Unit,
        uuid,
        "Hieda no Akyuu".into(),
        bvs,
        card_json.clone(),
        count,
        Some(run),
    )
}

type CardGeneratorFn = fn(&CardJson, usize) -> Card;
const FUNCTION_TABLE: [CardGeneratorFn; 27] = [
    test,
    human::HM_001,
    human::HM_002,
    human::HM_003,
    human::HM_004,
    human::HM_005,
    human::HM_006,
    human::HM_007,
    human::HM_008,
    monster::MT_001,
    monster::MT_002,
    monster::MT_003,
    monster::MT_004,
    monster::MT_005,
    monster::MT_006,
    monster::MT_007,
    monster::MT_008,
    monster::MT_009,
    monster::MT_010,
    public::PB_001,
    public::PB_002,
    public::PB_003,
    public::PB_004,
    public::PB_005,
    public::PB_006,
    public::PB_007,
    public::PB_008,
];

type Key = Vec<(String, i32)>;
pub struct CardGenerator {
    keys: Keys,
    pub card_generators: Lazy<HashMap<i32, CardGeneratorFn>>,
}

pub struct Keys {
    keys: Key,
}

impl Keys {
    pub fn new() -> Keys {
        let keys = match utils::utils::load_card_id() {
            Ok(data) => data,
            Err(_) => panic!("Unknown Err fun: Keys initialize"),
        };
        Keys { keys }
    }

    pub fn get_i32_by_string(&self, key: &str) -> Option<i32> {
        self.keys
            .iter()
            .find(|&(item_key, _)| item_key == key)
            .map(|&(_, value)| value)
    }

    pub fn get_string_by_i32(&self, key: i32) -> Option<String> {
        self.keys
            .iter()
            .find(|&(_, item_key)| item_key == &key)
            .map(|(value, _)| value.clone())
    }
}

impl CardGenerator {
    pub fn new() -> CardGenerator {
        let map: Lazy<HashMap<i32, CardGeneratorFn>> = Lazy::new(|| {
            let keys = Keys::new().keys;
            let mut map = HashMap::new();
            let func_it = FUNCTION_TABLE.iter();
            for (key, func) in keys.iter().zip(func_it) {
                map.insert(key.1, *func);
            }
            map
        });

        CardGenerator {
            keys: Keys::new(),
            card_generators: map,
        }
    }

    pub fn gen_card_by_id_i32(&self, id: i32, card_json: &CardJson, count: usize) -> Card {
        if let Some(generator) = self.card_generators.get(&id) {
            generator(card_json, count)
        } else {
            panic!("Unknown ID: {}", id);
        }
    }

    pub fn gen_card_by_id_string(&self, key: String, card_json: &CardJson, count: usize) -> Card {
        println!("key {key}");
        match self.keys.get_i32_by_string(&key[..]) {
            Some(_key) => self.gen_card_by_id_i32(_key, card_json, count),
            None => panic!("Unknown ID: {}", key),
        }
    }
}

mod monster {
    use super::*;

    #[allow(non_snake_case)]
    pub fn MT_001(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_002(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_003(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_004(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_005(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_006(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_007(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_008(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_009(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_010(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
}

mod public {
    use super::*;

    #[allow(non_snake_case)]
    pub fn PB_001(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_002(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_003(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_004(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_005(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_006(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_007(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_008(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
}

mod human {
    use crate::game::TimeManager;

    use super::*;

    // -------------------------------------------------- FIELD
    // [HM_001] Hieda no Akyuu - COST:?? [ATK:??/HP:?]
    // - Set: Human, Rarity: C
    // --------------------------------------------------------
    // Text: 낮동안 인간 카드를 사용할 때 마다 서로 1장 드로우 한다.
    // --------------------------------------------------------
    // Behaviors:
    // - ListenOtherEvent
    // - DrawCardFromDeck
    // --------------------------------------------------------
    #[allow(non_snake_case)]
    pub fn HM_001(card_json: &CardJson, count: usize) -> Card {
        let uuid = match utils::utils::generate_uuid() {
            Ok(data) => data,
            Err(err) => {
                panic!("test func failed {err}");
            }
        };
        let mut bvs = vec![];
        bvs.push(Behavior::ListenOtherEvent);
        bvs.push(Behavior::DrawCardFromDeck);
        let run = Rc::new(RefCell::new(
            |card: &Card, game: &mut Game| -> Result<(), Exception> {
                match game.time.get_state() {
                    crate::enums::TimeType::Day => {}
                    _ => {}
                }
                Ok(())
            },
        ));
        let name = if let Some(name) = &card_json.name{
            name
        }else{
            panic!("Card creating error");
        };
        Card::new(
            CardType::Unit,
            uuid,
            name.clone(),
            bvs,
            card_json.clone(),
            count,
            Some(run),
        )
    }

    #[allow(non_snake_case)]
    pub fn HM_002(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }

    #[allow(non_snake_case)]
    pub fn HM_003(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_004(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_005(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_006(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_007(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_008(card_json: &CardJson, count: usize) -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
}
