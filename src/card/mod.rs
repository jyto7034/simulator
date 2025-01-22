use effect::Effect;
pub mod insert;
use types::{CardSpecs, CardStatus, StatType};

use crate::{enums::PlayerType, card::types::CardType, exception::Exception, game::Game, utils::{self, json::CardJson}};
pub mod cards;
pub mod effect;
pub mod types;
pub mod target_selector;

#[derive(Clone)]
pub struct Card {
    uuid: String,
    name: String,
    card_type: CardType,
    effects: Vec<Box<dyn Effect>>,
    specs: CardSpecs,
    status: CardStatus,
    owner: PlayerType,
    json_data: CardJson,
}

impl Clone for Box<dyn Effect> {
    fn clone(&self) -> Self {
        self.clone_effect().unwrap()
    }
}

impl Card {
    // 새로운 생성자
    pub fn new(
        card_type: CardType,
        uuid: String,
        name: String,
        effects: Vec<Box<dyn Effect>>,
        json_data: CardJson,
        owner: PlayerType,
    ) -> Self {
        Card {
            uuid,
            name,
            card_type,
            effects,
            specs: CardSpecs::new(),
            status: CardStatus::default(),
            owner,
            json_data,
        }
    }

    // 효과 활성화
    pub fn activate(&self, game: &mut Game) -> Result<(), Exception> {
        // 카드가 효과를 발동할 수 있는 상태인지 확인
        if !self.can_activate(game) {
            return Err(Exception::CannotActivate);
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
        !self.status.is_negated() && 
        !self.status.is_disabled() && 
        self.meets_activation_conditions(game)
    }

    // effect 효과로 처리
    pub fn can_be_targeted(&self) -> bool{
        todo!()
    }

    // 발동 조건 확인
    fn meets_activation_conditions(&self, game: &Game) -> bool {
        // 카드 타입별, 상황별 발동 조건 체크
        match self.card_type {
            CardType::Dummy => todo!(),
            CardType::Unit => todo!(),
            CardType::Spell(spell_type) => todo!(),
            CardType::Field => todo!(),
            CardType::Game => todo!(),
            // CardType::Unit => self.can_activate_as_unit(game),
            // CardType::Spell => self.can_activate_as_spell(game),
        }
    }

    // Getter/Setter 메서드들
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type(&self) -> &CardType {
        &self.card_type
    }

    pub fn get_owner(&self) -> &PlayerType {
        &self.owner
    }

    pub fn set_owner(&mut self, player: PlayerType) {
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

    pub fn modify_stat(&mut self, stat_type: StatType, amount: i32) -> Result<(), Exception>{
        Ok(())
    }

    // 카드 복사 (새로운 UUID 생성)
    pub fn clone_with_new_uuid(&self) -> Result<Self, Exception> {
        Ok(Card {
            uuid: utils::generate_uuid()?,
            name: self.name.clone(),
            card_type: self.card_type.clone(),
            effects: self.effects.iter()
                .map(|e| e.clone_effect())
                .collect::<Result<Vec<_>, _>>()?,
            specs: self.specs.clone(),
            status: CardStatus::default(),
            owner: self.owner.clone(),
            json_data: self.json_data.clone(),
        })
    }
}