use crate::enums::constant::{self, CardType, UUID};
use crate::game::Behavior;
use crate::utils::json::CardJson;

/// 카드의 행동, 정보를 정의하는 구조체 입니다.

#[derive(Clone, Debug)]
pub struct Card {
    card_type: constant::CardType,
    uuid: String,
    name: String,
    behavior_table: Vec<Behavior>,
    card_json: CardJson,
}

impl Card {
    pub fn dummy() -> Card {
        Card {
            card_type: CardType::Dummy,
            uuid: "".to_string(),
            name: "dummy".to_string(),
            behavior_table: vec![],
            card_json: CardJson::new(),
        }
    }

    pub fn new(
        card_type: CardType,
        uuid: UUID,
        name: String,
        behavior_table: Vec<Behavior>,
        card_json: CardJson,
    ) -> Card {
        Card {
            card_type,
            uuid,
            name,
            behavior_table,
            card_json,
        }
    }

    pub fn is_dummy(&self) -> bool {
        true
    }

    pub fn get_uuid(&self) -> &String {
        &self.uuid
    }

    pub fn get_card_type(&self) -> &constant::CardType {
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

    // Setter 함수들
    pub fn set_card_type(&mut self, new_card_type: constant::CardType) {
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

    pub fn set_card_json(&mut self, new_card_json: CardJson) {
        self.card_json = new_card_json;
    }

    pub fn execution() {}
}
