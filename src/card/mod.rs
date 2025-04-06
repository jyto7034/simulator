pub mod cards;
pub mod insert;
pub mod modifier;
pub mod take;
pub mod types;

use std::fmt;

use types::{CardSpecs, CardStatus, OwnerType, StatType};
use uuid::Uuid;

use crate::{
    card::types::CardType, effect::Effect, exception::GameError, game::Game, utils::json::CardJson,
};

#[derive(Clone)]
pub struct PrioritizedEffect {
    priority: u8, // 낮을수록 높은 우선순위
    effect: Box<dyn Effect>,
}

impl PrioritizedEffect {
    pub fn new(priority: u8, effect: Box<dyn Effect>) -> Self {
        Self { priority, effect }
    }

    pub fn get_priority(&self) -> u8 {
        self.priority
    }

    pub fn get_effect(&self) -> &Box<dyn Effect> {
        &self.effect
    }

    pub fn get_effect_mut(&mut self) -> &mut Box<dyn Effect> {
        &mut self.effect
    }
}

#[derive(Clone)]
pub struct Card {
    uuid: Uuid,
    name: String,
    card_type: CardType,
    effects: Vec<PrioritizedEffect>,
    specs: CardSpecs,
    status: CardStatus,
    owner: OwnerType,
    json_data: CardJson,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Card")
            .field("uuid", &self.uuid)
            .field("name", &self.name)
            .field("card_type", &self.card_type)
            .field("owner", &self.owner)
            // .field("effects", &self.effects)
            // .field("specs", &self.specs)
            // .field("status", &self.status)
            // .field("json_data", &self.json_data)
            .finish()
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Card {}

impl Clone for Box<dyn Effect> {
    fn clone(&self) -> Self {
        self.clone_effect().unwrap()
    }
}

impl Card {
    pub fn new(
        owner: OwnerType,
        uuid: Uuid,
        name: String,
        effects: Vec<PrioritizedEffect>,
        r#type: CardType,
        specs: CardSpecs,
        status: CardStatus,
        json_data: CardJson,
    ) -> Self {
        Self {
            uuid,
            name,
            card_type: r#type,
            effects,
            specs,
            status,
            owner,
            json_data,
        }
    }

    pub fn activate(&self, game: &mut Game) -> Result<(), GameError> {
        todo!()
    }

    // effect 효과로 처리
    pub fn can_be_targeted(&self) -> bool {
        todo!()
    }

    // Getter/Setter 메서드들
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type(&self) -> &CardType {
        &self.card_type
    }

    pub fn get_owner(&self) -> OwnerType {
        self.owner
    }

    pub fn set_owner(&mut self, player: OwnerType) {
        self.owner = player;
    }

    pub fn get_specs(&self) -> &CardSpecs {
        &self.specs
    }

    pub fn get_status(&self) -> &CardStatus {
        &self.status
    }

    pub fn get_status_mut(&mut self) -> &mut CardStatus {
        &mut self.status
    }

    pub fn get_prioritized_effect(&self) -> &Vec<PrioritizedEffect> {
        &self.effects
    }

    pub fn get_prioritized_effect_mut(&mut self) -> &mut Vec<PrioritizedEffect> {
        &mut self.effects
    }

    // 효과 추가
    pub fn add_effect<E: Effect + 'static>(&mut self, effect: E) {
        todo!()
        // self.effects.push(Box::new(effect));
    }

    pub fn modify_stat(&mut self, stat_type: StatType, amount: i32) -> Result<(), GameError> {
        Ok(())
    }

    // 카드 복사 (새로운 UUID 생성)
    pub fn clone_with_new_uuid(&self) -> Result<Self, GameError> {
        todo!()
        // Ok(Card {
        //     uuid: utils::generate_uuid()?,
        //     name: self.name.clone(),
        //     card_type: self.card_type.clone(),
        //     effects: self
        //         .effects
        //         .iter()
        //         .map(|e| e.clone_effect())
        //         .collect::<Result<Vec<_>, _>>()?,
        //     specs: self.specs.clone(),
        //     status: CardStatus::default(),
        //     owner: self.owner.clone(),
        //     json_data: self.json_data.clone(),
        // })
    }
}
