//! mod.rs
//! 
//! 게임 시뮬레이터의 핵심 모듈
//! 이 모듈은 card와 관련된 기능을 제공합니다.

pub mod cards;
pub mod insert;
pub mod modifier;
pub mod take;
pub mod types;

use std::fmt;

use actix::Addr;
use types::{CardSpecs, CardStatus, OwnerType, StatType};
use uuid::Uuid;

use crate::{
    card::types::CardType,
    effect::{effects::EffectTiming, Effect},
    exception::GameError,
    game::GameActor,
    utils::json::CardJson,
};

/// `Card` 구조체는 게임 내의 카드 정보를 담고 있습니다.
///
/// 각 카드는 고유한 UUID, 이름, 타입, 효과, 스펙, 상태, 소유자, 그리고 JSON 데이터를 가집니다.
/// 이 구조체를 사용하여 게임 내 카드 객체를 표현하고 관리합니다.
#[derive(Clone)]
pub struct Card {
    uuid: Uuid,
    name: String,
    card_type: CardType,
    effects: Vec<EffectTiming>,
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
    /// 새로운 `Card` 인스턴스를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `owner` - 카드의 소유자 타입 (`OwnerType`).
    /// * `uuid` - 카드의 고유 UUID (`Uuid`).
    /// * `name` - 카드의 이름 (`String`).
    /// * `effects` - 카드에 적용될 효과들의 벡터 (`Vec<EffectTiming>`).
    /// * `r#type` - 카드의 타입 (`CardType`).
    /// * `specs` - 카드의 스펙 (`CardSpecs`).
    /// * `status` - 카드의 상태 (`CardStatus`).
    /// * `json_data` - 카드의 JSON 데이터 (`CardJson`).
    ///
    /// # Returns
    ///
    /// 새로운 `Card` 인스턴스.
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use simulator_core::card::{Card, types::{CardType, CardSpecs, CardStatus, OwnerType}};
    /// use simulator_core::effect::effects::EffectTiming;
    /// use simulator_core::utils::json::CardJson;
    ///
    /// let owner = OwnerType::Player(1);
    /// let uuid = Uuid::new_v4();
    /// let name = "Test Card".to_string();
    /// let effects = Vec::new();
    /// let card_type = CardType::Normal;
    /// let specs = CardSpecs::default();
    /// let status = CardStatus::default();
    /// let json_data = CardJson::default();
    ///
    /// let card = Card::new(owner, uuid, name, effects, card_type, specs, status, json_data);
    /// ```
    pub fn new(
        owner: OwnerType,
        uuid: Uuid,
        name: String,
        effects: Vec<EffectTiming>,
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

    /// 카드를 활성화합니다.
    ///
    /// # Arguments
    ///
    /// * `game` - 게임 액터의 주소 (`Addr<GameActor>`).
    ///
    /// # Returns
    ///
    /// 성공하면 `Ok(())`, 실패하면 `Err(GameError)`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn activate(&self, game: Addr<GameActor>) -> Result<(), GameError> {
        todo!()
    }

    /// 카드가 타겟팅될 수 있는지 여부를 반환합니다.
    ///
    /// # Returns
    ///
    /// 타겟팅 가능하면 `true`, 불가능하면 `false`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn can_be_targeted(&self) -> bool {
        todo!()
    }

    /// 카드의 UUID를 반환합니다.
    ///
    /// # Returns
    ///
    /// 카드의 UUID (`Uuid`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    /// 카드의 이름을 반환합니다.
    ///
    /// # Returns
    ///
    /// 카드의 이름 (`&str`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// 카드의 타입을 반환합니다.
    ///
    /// # Returns
    ///
    /// 카드의 타입 (`&CardType`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_type(&self) -> &CardType {
        &self.card_type
    }

    /// 카드의 소유자를 반환합니다.
    ///
    /// # Returns
    ///
    /// 카드의 소유자 (`OwnerType`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_owner(&self) -> OwnerType {
        self.owner
    }

    /// 카드의 소유자를 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `player` - 새로운 소유자 (`OwnerType`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn set_owner(&mut self, player: OwnerType) {
        self.owner = player;
    }

    /// 카드의 스펙을 반환합니다.
    ///
    /// # Returns
    ///
    /// 카드의 스펙 (`&CardSpecs`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_specs(&self) -> &CardSpecs {
        &self.specs
    }

    /// 카드의 상태를 반환합니다.
    ///
    /// # Returns
    ///
    /// 카드의 상태 (`&CardStatus`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_status(&self) -> &CardStatus {
        &self.status
    }

    /// 카드의 가변 상태를 반환합니다.
    ///
    /// # Returns
    ///
    /// 카드의 가변 상태 (`&mut CardStatus`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_status_mut(&mut self) -> &mut CardStatus {
        &mut self.status
    }

    /// 우선순위가 지정된 효과 벡터를 반환합니다.
    ///
    /// # Returns
    ///
    /// 우선순위가 지정된 효과 벡터 (`&Vec<EffectTiming>`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_prioritized_effect(&self) -> &Vec<EffectTiming> {
        &self.effects
    }

    /// 우선순위가 지정된 가변 효과 벡터를 반환합니다.
    ///
    /// # Returns
    ///
    /// 우선순위가 지정된 가변 효과 벡터 (`&mut Vec<EffectTiming>`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn get_prioritized_effect_mut(&mut self) -> &mut Vec<EffectTiming> {
        &mut self.effects
    }

    /// 카드에 효과를 추가합니다.
    ///
    /// # Arguments
    ///
    /// * `effect` - 추가할 효과 (`E`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn add_effect<E: Effect + 'static>(&mut self, effect: E) {
        todo!()
        // self.effects.push(Box::new(effect));
    }

    /// 카드의 스탯을 변경합니다.
    ///
    /// # Arguments
    ///
    /// * `stat_type` - 변경할 스탯의 타입 (`StatType`).
    /// * `amount` - 변경할 양 (`i32`).
    ///
    /// # Returns
    ///
    /// 성공하면 `Ok(())`, 실패하면 `Err(GameError)`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
    pub fn modify_stat(&mut self, stat_type: StatType, amount: i32) -> Result<(), GameError> {
        Ok(())
    }

    /// 새로운 UUID를 사용하여 카드를 복사합니다.
    ///
    /// # Returns
    ///
    /// 성공하면 복사된 카드 (`Ok(Self)`), 실패하면 `Err(GameError)`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 예시 코드를 추가해야 합니다.
    /// ```
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