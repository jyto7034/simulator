pub mod cards;
pub mod effect;
pub mod insert;
pub mod modifier;
pub mod take;
pub mod types;

use effect::Effect;
use types::{CardSpecs, CardStatus, OwnerType, StatType};

use crate::{
    card::types::CardType,
    enums::UUID,
    exception::GameError,
    game::Game,
    utils::{self, json::CardJson},
};

#[derive(Clone)]
pub struct Card {
    uuid: UUID,
    name: String,
    card_type: CardType,
    effects: Vec<Box<dyn Effect>>,
    specs: CardSpecs,
    status: CardStatus,
    owner: OwnerType,
    json_data: CardJson,
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
        uuid: UUID,
        name: String,
        effects: Vec<Box<dyn Effect>>,
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
        // 카드가 효과를 발동할 수 있는 상태인지 확인
        if !self.can_activate(game) {
            return Err(GameError::CannotActivate);
        }

        // 새 시스템의 효과들 처리
        for effect in &self.effects {
            if effect.can_activate(game, self) {
                effect.apply(game, self)?;
            }
        }

        Ok(())
    }

    // 카드가 효과를 발동할 수 있는 상태인지 확인
    pub fn can_activate(&self, game: &Game) -> bool {
        !self.status.is_negated()
            && !self.status.is_disabled()
            && self.meets_activation_conditions(game)
    }

    // effect 효과로 처리
    pub fn can_be_targeted(&self) -> bool {
        todo!()
    }

    // 발동 조건 확인
    fn meets_activation_conditions(&self, game: &Game) -> bool {
        // 카드 타입별, 상황별 발동 조건 체크
        match self.card_type {
            CardType::Dummy => todo!(),
            CardType::Unit => todo!(),
            CardType::Field => todo!(),
            CardType::Game => todo!(),
            CardType::Spell => todo!(),
            CardType::Trap => todo!(),
            CardType::Ace => todo!(),
            CardType::Any => todo!(),
        }
    }

    // Getter/Setter 메서드들
    pub fn get_uuid(&self) -> UUID {
        self.uuid.clone()
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

    // 효과 추가
    pub fn add_effect<E: Effect + 'static>(&mut self, effect: E) {
        self.effects.push(Box::new(effect));
    }

    pub fn modify_stat(&mut self, stat_type: StatType, amount: i32) -> Result<(), GameError> {
        Ok(())
    }

    // 카드 복사 (새로운 UUID 생성)
    pub fn clone_with_new_uuid(&self) -> Result<Self, GameError> {
        Ok(Card {
            uuid: utils::generate_uuid()?,
            name: self.name.clone(),
            card_type: self.card_type.clone(),
            effects: self
                .effects
                .iter()
                .map(|e| e.clone_effect())
                .collect::<Result<Vec<_>, _>>()?,
            specs: self.specs.clone(),
            status: CardStatus::default(),
            owner: self.owner.clone(),
            json_data: self.json_data.clone(),
        })
    }
}
