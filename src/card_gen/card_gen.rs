use crate::enums::CardType;
use crate::{deck::Card, utils, utils::json::CardJson};
use once_cell::sync::Lazy;
use std::collections::HashMap;

fn test() -> Card {
    println!("test func");
    let uuid = match utils::utils::generate_uuid() {
        Ok(data) => data,
        Err(err) => {
            panic!("test func failed {err}");
        }
    };
    Card::new(CardType::Unit, uuid, "test".into(), vec![], CardJson::new())
}

type CardGeneratorFn = fn() -> Card;
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

struct Species {
    pub species: Vec<String>,
}

impl Species {
    pub fn new() -> Species {
        Species { species: vec![] }
    }

    pub fn initialize(&mut self) {
        self.species = match utils::utils::load_card_id() {
            Ok(data) => data,
            Err(_) => panic!("Unknown Err fun: species initialize"),
        };
    }
}
pub struct CardGenertor {
    species: Species,
    pub card_generators: Lazy<HashMap<String, CardGeneratorFn>>,
}

impl CardGenertor {
    pub fn new() -> CardGenertor {
        let mut species = Species::new();
        species.initialize();

        CardGenertor {
            species,
            card_generators: Lazy::new(|| {
                let mut map = HashMap::new();
                let mut species = Species::new();
                species.initialize();
                let func_it = FUNCTION_TABLE.iter();
                for (id, func) in species.species.iter().zip(func_it) {
                    map.insert(id.to_string(), *func);
                }
                map
            }),
        }
    }

    pub fn gen_card_by_id(&self, id: String) -> Card {
        if let Some(generator) = self.card_generators.get(&id[..]) {
            generator()
        } else {
            panic!("Unknown ID: {}", id);
        }
    }
}

mod monster {
    use super::*;

    #[allow(non_snake_case)]
    pub fn MT_001() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_002() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_003() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_004() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_005() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_006() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_007() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_008() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_009() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn MT_010() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
}

mod public {
    use super::*;

    #[allow(non_snake_case)]
    pub fn PB_001() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_002() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_003() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_004() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_005() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_006() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_007() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn PB_008() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
}

mod human {
    use super::*;

    #[allow(non_snake_case)]
    pub fn HM_001() -> Card {
        let uuid = match utils::utils::generate_uuid() {
            Ok(data) => data,
            Err(err) => {
                panic!("test func failed {err}");
            }
        };
        Card::new(CardType::Unit, uuid, "test".into(), vec![], CardJson::new())
    }

    #[allow(non_snake_case)]
    pub fn HM_002() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }

    #[allow(non_snake_case)]
    pub fn HM_003() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_004() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_005() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_006() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_007() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
    #[allow(non_snake_case)]
    pub fn HM_008() -> Card {
        // Card::new(card_type, uuid, name, count, behavior_table, card_json)
        todo!()
    }
}
