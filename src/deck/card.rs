use std::fmt;

use crate::enums::constant::{self, CardType, Runner, UUID};
use crate::exception::exception::Exception;
use crate::game::{Behavior, Count, Game};
use crate::unit::Entity;
use crate::utils::json::CardJson;

/// 카드의 행동, 정보를 정의하는 구조체 입니다.

// Card 는 자신의 효과를 실행하는 runner 함수를 가진다.
// 이 runner 함수는 Card 의 필드 멤버인 behavior_table 의 요소를 하나씩 task 로 만들어 procedure 에 밀어넣는다.
// 해당 task 를 취소하기 위해선, 연속적으로 있는 task 에서 일정 부분을 삭제해야 하는데, 이는 task 의 id 를 하나로 통일 시킴으로써 해결한다.
#[derive(Clone)]
pub struct Card {
    card_type: constant::CardType,
    uuid: String,
    name: String,
    behavior_table: Vec<Behavior>,
    card_json: CardJson,
    count: Count,
    runner: Option<Runner>,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 원하는 형식으로 클로저를 출력
        write!(f, "MyFn {{ /* 클로저 내용 출력 */ }}")
    }
}

impl Entity for Card {
    fn run(&self, game: &mut Game) -> Result<(), Exception> {
        if let Some(runner) = &self.runner {
            match runner.as_ref().borrow_mut()(self, game) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }
        } else {
            // 임의로 리턴함.
            Err(Exception::NothingToRemove)
        }
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
            count: Count::new(0, 3),
            runner: None,
        }
    }

    pub fn new(
        card_type: CardType,
        uuid: UUID,
        name: String,
        behavior_table: Vec<Behavior>,
        card_json: CardJson,
        count: usize,
        runner: Option<Runner>,
    ) -> Card {
        Card {
            card_type,
            uuid,
            name,
            behavior_table,
            card_json,
            count: Count::new(count, 3),
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

    pub fn get_count(&mut self) -> &mut Count {
        &mut self.count
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
