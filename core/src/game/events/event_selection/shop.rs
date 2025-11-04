use uuid::Uuid;

use crate::{
    game::{
        enums::Tier,
        events::{EventError, EventExecutor, EventGenerator},
    },
    ExecutorContext, GeneratorContext,
};

// 상인에도 등급이 존재함.
// 레벨에 비례하여 등급이 높은 상인이 등장할 수 있음.
// 아이템 종류마다 상인이 존재함.
// 아티팩트, 기물, 무기, 방어구 등등
// 각 종류마다 고유한 Npc 를 가짐. ( 상인의 갯수가 너무 많으면 리소스가 더 많이 발생하니 중복 허용함. )
// TODO: 각 상인마다 아이템 목록이 다르니 상인-아이템 목록은 외부 config 로 관리.

pub struct Item {
    pub uuid: Uuid,
    pub name: String,
    pub price: u32,
    pub tier: Tier,
}

impl Item {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "test".to_string(),
            price: 12,
            tier: Tier::I,
        }
    }
}

pub struct Shop {
    pub npc_id: String,
    pub npc_uuid: Uuid,
    pub items: Vec<Item>,
}

pub struct ShopGenerator;

impl EventGenerator for ShopGenerator {
    type Output = Shop;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output {
        Shop {
            npc_id: "test".to_string(),
            npc_uuid: Uuid::new_v4(),
            items: vec![Item::new()],
        }
    }
}

pub struct ShopExecutor;

impl EventExecutor for ShopExecutor {
    type Input = String;

    fn execute(&self, ctx: &GeneratorContext, input: Self::Input) -> Result<(), EventError> {
        // TODO: 가방에 추가 (또는 티어 업)
        todo!()
    }
}
