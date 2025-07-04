mod builder;
mod human;
mod monster;
mod public;

use crate::card::Card;
use crate::{utils, utils::json::CardJson};

use once_cell::sync::Lazy;
use std::collections::HashMap;

type CardGeneratorFn = fn(&CardJson, i32) -> Card;

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

type Key = Vec<(String, i32)>;
/// `CardGenerator`는 카드 생성을 관리하는 구조체입니다.
/// `keys`는 카드 ID와 문자열 ID 간의 매핑을 저장하고, `card_generators`는 ID별 카드 생성 함수를 저장합니다.
// TODO: 필드에 대한 더 자세한 설명 추가
pub struct CardGenerator {
    keys: Keys,
    card_generators: HashMap<i32, CardGeneratorFn>,
}

/// `Keys`는 카드 ID와 문자열 ID 간의 매핑 정보를 담는 구조체입니다.
/// `keys` 필드는 벡터 형태로 매핑 정보를 저장합니다.
// TODO: `keys` 필드의 구체적인 데이터 형태 (예: `Vec<(String, i32)>`) 명시
pub struct Keys {
    pub keys: Key,
}

/// `Keys` 구조체에 대한 구현 블록입니다.
impl Keys {
    /// `new`는 `CardGenerator` 구조체의 생성자 함수입니다.
    /// `Keys`를 초기화하고, `CARD_GENERATORS`에 등록된 함수들을 기반으로 카드 생성 함수 매핑을 수행합니다.
    // TODO: 초기화 과정에 대한 더 자세한 설명 추가
    pub fn new() -> Keys {
        let keys = match utils::load_card_id() {
            Ok(data) => data,
            Err(_) => panic!("Unknown Err fun: Keys initialize"),
        };
        Keys { keys }
    }

    /// `get_usize_by_string`은 문자열 ID를 사용하여 해당 ID에 매핑된 숫자 ID를 가져오는 함수입니다.
    ///
    /// # Arguments
    ///
    /// * `key` - 찾고자 하는 문자열 ID입니다.
    ///
    /// # Returns
    ///
    /// 매핑된 숫자 ID가 존재하면 `Some(i32)`를 반환하고, 존재하지 않으면 `None`을 반환합니다.
    pub fn get_usize_by_string(&self, key: &str) -> Option<i32> {
        self.keys
            .iter()
            .find(|&(item_key, _)| item_key == key)
            .map(|&(_, value)| value)
    }

    /// `get_string_by_usize`는 숫자 ID를 사용하여 해당 ID에 매핑된 문자열 ID를 가져오는 함수입니다.
    ///
    /// # Arguments
    ///
    /// * `key` - 찾고자 하는 숫자 ID입니다.
    ///
    /// # Returns
    ///
    /// 매핑된 문자열 ID가 존재하면 `Some(String)`을 반환하고, 존재하지 않으면 `None`을 반환합니다.
    pub fn get_string_by_usize(&self, key: i32) -> Option<String> {
        self.keys
            .iter()
            .find(|&(_, item_key)| item_key == &key)
            .map(|(value, _)| value.clone())
    }
}

/// `CardGenerator` 구조체에 대한 구현 블록입니다.
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

    /// `gen_card_by_id_i32`는 숫자 ID를 사용하여 카드를 생성하는 함수입니다.
    ///
    /// # Arguments
    ///
    /// * `id` - 카드 ID입니다.
    /// * `card_json` - 카드 생성에 필요한 JSON 데이터입니다.
    /// * `count` - 생성할 카드 갯수입니다.
    ///
    /// # Returns
    ///
    /// 생성된 `Card` 인스턴스를 반환합니다.
    ///
    /// # Panics
    /// 알 수 없는 ID를 받으면 패닉을 발생시킵니다.
    // TODO: 패닉 발생 조건에 대한 더 자세한 설명 추가
    pub fn gen_card_by_id_i32(&self, id: i32, card_json: &CardJson, count: i32) -> Card {
        if let Some(generator) = self.card_generators.get(&id) {
            generator(card_json, count)
        } else {
            panic!("Unknown ID: {}", id);
        }
    }

    /// `gen_card_by_id_string`은 문자열 ID를 사용하여 카드를 생성하는 함수입니다.
    ///
    /// # Arguments
    ///
    /// * `key` - 카드 문자열 ID입니다.
    /// * `card_json` - 카드 생성에 필요한 JSON 데이터입니다.
    /// * `count` - 생성할 카드 갯수입니다.
    ///
    /// # Returns
    ///
    /// 생성된 `Card` 인스턴스를 반환합니다.
    ///
    /// # Panics
    /// 알 수 없는 ID를 받으면 패닉을 발생시킵니다.
    // TODO: 패닉 발생 조건에 대한 더 자세한 설명 추가
    pub fn gen_card_by_id_string(&self, key: String, card_json: &CardJson, count: i32) -> Card {
        match self.keys.get_usize_by_string(&key[..]) {
            Some(id) => self.gen_card_by_id_i32(id, card_json, count),
            None => panic!("Unknown ID: {}", key),
        }
    }
}
