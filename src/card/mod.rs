pub mod cards;
pub mod deck;

use crate::{
    enums::*,
    procedure::behavior::Behavior,
    utils::json::CardJson,
};
use std::fmt;

/// 단일 카드 정보를 담은 구조체 입니다.
#[derive(Clone, Eq, Hash)]
pub struct Card {
    card_type: CardType,
    uuid: String,
    name: String,
    behavior_table: Vec<Behavior>,
    card_json: CardJson,
    player_type: PlayerType,
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(CardParam::Card(other.clone()))
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Card {{\n")?;
        write!(f, "    card_type: {:#?}\n", self.card_type)?;
        write!(f, "    uuid: {}", self.uuid)?;
        write!(f, "    name: {}\n", self.name)?;
        write!(f, "    behavior_table: {:?}\n", self.behavior_table)?;
        write!(f, "    card_json: {:#?}\n", self.card_json)?;
        write!(f, "    player_type: {:?}\n", self.player_type)?;
        // if let Some(_runner) = &self.runner {
        //     write!(f, "    runner: Ok\n")?;
        // } else {
        //     write!(f, "    runner: None\n")?;
        // }
        write!(f, "}}")
    }
}

impl Card {
    pub fn new(
        card_type: CardType,
        uuid: UUID,
        name: String,
        behavior_table: Vec<Behavior>,
        card_json: CardJson,
        player_type: PlayerType,
    ) -> Card {
        Card {
            card_type,
            uuid,
            name,
            behavior_table,
            card_json,
            player_type,
        }
    }

    pub fn dummy() -> Card {
        Card {
            card_type: CardType::Dummy,
            uuid: "".to_string(),
            name: "dummy".to_string(),
            behavior_table: vec![],
            card_json: CardJson::new(),
            player_type: PlayerType::None,
        }
    }

    pub fn is_dummy(&self) -> bool {
        true
    }

    pub fn get_uuid(&self) -> &String {
        &self.uuid
    }

    pub fn get_card_type(&self) -> &CardType {
        &self.card_type
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_behavior_table(&self) -> &Vec<Behavior> {
        &self.behavior_table
    }

    pub fn get_card_json(&self) -> &CardJson {
        &self.card_json
    }

    pub fn get_player_type(&self) -> PlayerType {
        self.player_type.clone()
    }

    // Setter 함수들
    pub fn set_card_type(&mut self, new_card_type: CardType) {
        self.card_type = new_card_type;
    }

    pub fn set_uuid(&mut self, new_uuid: String) {
        self.uuid = new_uuid;
    }

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn set_behavior_table(&mut self, new_behavior_table: Vec<Behavior>) {
        self.behavior_table = new_behavior_table;
    }

    pub fn set_card_json(&mut self, _new_card_json: CardJson) {}

    // uuid 를 대조합니다.
    // 동일하다면, true 를.
    // 그렇지않다면, false 를 반환합니다.
    pub fn cmp(&self, cmp_type: CardParam) -> bool {
        match cmp_type {
            CardParam::Uuid(uuid) => self.get_uuid().cmp(&uuid) == std::cmp::Ordering::Equal,
            CardParam::Card(card) => {
                self.get_name().cmp(card.get_name()) == std::cmp::Ordering::Equal
            }
        }
    }
}
