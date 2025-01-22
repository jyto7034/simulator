mod monster;
mod human;
mod public;
mod builder;

use crate::card::Card;
use crate::{utils, utils::json::CardJson};

use once_cell::sync::Lazy;
use std::collections::HashMap;

type CardGeneratorFn = fn(&CardJson, usize) -> Card;

macro_rules! generate_card_map {
    ($($module:ident :: $func:ident),* $(,)?) => {
        static CARD_GENERATORS: Lazy<HashMap<String, CardGeneratorFn>> = Lazy::new(|| {
            let mut m = HashMap::new();
            $(
                m.insert(stringify!($func).to_string(), $module::$func as CardGeneratorFn);
            )*
            m
        });
    };
}

include!(concat!(env!("OUT_DIR"), "/card_registry.rs"));

type Key = Vec<(String, usize)>;
pub struct CardGenerator {
    keys: Keys,
    card_generators: HashMap<usize, CardGeneratorFn>,
}

pub struct Keys {
    pub keys: Key,
}

impl Keys {
    pub fn new() -> Keys {
        let keys = match utils::load_card_id() {
            Ok(data) => data,
            Err(_) => panic!("Unknown Err fun: Keys initialize"),
        };
        Keys { keys }
    }

    pub fn get_usize_by_string(&self, key: &str) -> Option<usize> {
        self.keys
            .iter()
            .find(|&(item_key, _)| item_key == key)
            .map(|&(_, value)| value)
    }

    pub fn get_string_by_usize(&self, key: usize) -> Option<String> {
        self.keys
            .iter()
            .find(|&(_, item_key)| item_key == &key)
            .map(|(value, _)| value.clone())
    }
}

impl CardGenerator {
    pub fn new() -> CardGenerator {
        let keys = Keys::new();
        let mut card_generators = HashMap::new();
        
        for (str_id, func) in CARD_GENERATORS.iter() {
            if let Some(id) = keys.get_usize_by_string(str_id) {
                card_generators.insert(id, *func);
            }
        }

        CardGenerator {
            keys,
            card_generators,
        }
    }

    pub fn gen_card_by_id_usize(&self, id: usize, card_json: &CardJson, count: usize) -> Card {
        if let Some(generator) = self.card_generators.get(&id) {
            generator(card_json, count)
        } else {
            panic!("Unknown ID: {}", id);
        }
    }

    pub fn gen_card_by_id_string(&self, key: String, card_json: &CardJson, count: usize) -> Card {
        match self.keys.get_usize_by_string(&key[..]) {
            Some(id) => self.gen_card_by_id_usize(id, card_json, count),
            None => panic!("Unknown ID: {}", key),
        }
    }
}
