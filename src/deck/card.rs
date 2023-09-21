use std::fmt;
use std::sync::Arc;

use crate::enums::constant::{self, CardType, UUID};
use crate::exception::exception::Exception;
use crate::game::Behavior;
use crate::unit::Entity;
use crate::utils::json::CardJson;

/// 카드의 행동, 정보를 정의하는 구조체 입니다.

#[derive(Clone)]
pub struct Card {
    card_type: constant::CardType,
    uuid: String,
    name: String,
    behavior_table: Vec<Behavior>,
    card_json: CardJson,
    runner: Option<Arc<dyn Fn(&mut Card) -> Result<(), Exception>>>,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 원하는 형식으로 클로저를 출력
        write!(f, "MyFn {{ /* 클로저 내용 출력 */ }}")
    }
}

impl Entity for Card {
    fn run(&self) -> Result<(), Exception> {
        todo!()
    }

    fn get_entity_type(&self) -> String {
        "Card".to_string()
    }
}

impl Card {
    pub fn dummy() -> Card {
        Card {
            card_type: CardType::Dummy,
            uuid: "".to_string(),
            name: "dummy".to_string(),
            behavior_table: vec![],
            card_json: CardJson::new(),
            runner: None,
        }
    }

    pub fn new(
        card_type: CardType,
        uuid: UUID,
        name: String,
        behavior_table: Vec<Behavior>,
        card_json: CardJson,
        runner: Option<Arc<dyn Fn(&mut Card) -> Result<(), Exception>>>,
    ) -> Card {
        Card {
            card_type,
            uuid,
            name,
            behavior_table,
            card_json,
            runner,
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

    pub fn set_card_json(&mut self, new_card_json: CardJson) {}
}
