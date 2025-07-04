use uuid::Uuid;

use crate::{
    card::{
        types::{CardSpecs, CardStatus, CardType, OwnerType},
        Card,
    },
    effect::{effects::EffectTiming, types::EffectSpeed, Effect},
    exception::{GameError, GameplayError},
    utils::{self, json::CardJson},
};

/// `CardBuilder`는 카드 생성을 위한 빌더 패턴을 구현합니다.
/// 카드 속성을 설정하고 `Card` 인스턴스를 생성하는 데 사용됩니다.
pub struct CardBuilder {
    uuid: Uuid,
    name: String,
    card_type: CardType,
    effects: Vec<EffectTiming>,
    json_data: CardJson,
    owner: OwnerType,
    /// CardSpecs는 카드의 스펙 정보(공격력, 방어력 등)를 담고 있습니다.  `CardBuilder`를 통해 설정됩니다.
    pub specs: CardSpecs,
    status: CardStatus,
}

impl CardBuilder {
    /// `new`는 `CardBuilder`의 새 인스턴스를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `card_json` - 카드 정보가 담긴 JSON 데이터에 대한 참조.
    ///
    /// # Returns
    ///
    /// * `Result<Self, GameError>` - 성공하면 `CardBuilder` 인스턴스, 실패하면 `GameError`.
    pub fn new(card_json: &CardJson) -> Result<Self, GameError> {
        Ok(Self {
            uuid: utils::generate_uuid().unwrap(),
            name: card_json.name.clone().ok_or(GameError::Gameplay(GameplayError::InvalidAction { reason: "Invalid card data".to_string() }))?,
            card_type: CardType::from_json(card_json)?,
            effects: vec![],
            json_data: card_json.clone(),
            owner: OwnerType::None,
            specs: CardSpecs::new(card_json),
            status: CardStatus::new(),
        })
    }

    pub fn add_effect<E: Effect + 'static>(mut self, effect: E) -> Self {
        // TODO: priority 설정
        self.effects
            .push(EffectTiming::new(1, EffectSpeed::Medium, Box::new(effect)));
        self
    }

    // // UUID 설정
    // pub fn uuid(mut self, uuid: UUID) -> Self {
    //     self.uuid = uuid;
    //     self
    // }

    // // 이름 설정
    // pub fn name(mut self, name: String) -> Self {
    //     self.name = name;
    //     self
    // }

    // // 카드 타입 설정
    // pub fn card_type(mut self, card_type: CardType) -> Self {
    //     self.card_type = card_type;
    //     self
    // }

    // // 효과 목록 한번에 설정
    // pub fn effects(mut self, effects: Vec<Box<dyn Effect>>) -> Self {
    //     self.effects = effects;
    //     self
    // }

    // // JSON 데이터 설정
    // pub fn json_data(mut self, json_data: CardJson) -> Self {
    //     self.json_data = json_data;
    //     self
    // }

    // // 소유자 설정
    // pub fn owner(mut self, owner: OwnerType) -> Self {
    //     self.owner = owner;
    //     self
    // }

    // // 스펙 설정
    // pub fn specs(mut self, specs: CardSpecs) -> Self {
    //     self.specs = specs;
    //     self
    // }

    // pub fn status(mut self, status: CardStatus) -> Self {
    //     self.status = status;
    //     self
    // }

    /// `build`는 설정된 속성을 사용하여 `Card` 인스턴스를 생성합니다.
    ///
    /// # Returns
    ///
    /// * `Card` - 생성된 카드 인스턴스.
    pub fn build(self) -> Card {
        // owner: OwnerType,
        // uuid: UUID,
        // name: String,
        // effects: Vec<Box<dyn Effect>>,
        // r#type: CardType,
        // specs: CardSpecs,
        // status: CardStatus,
        // json_data: CardJson)
        Card::new(
            self.owner,
            self.uuid,
            self.name,
            self.effects,
            self.card_type,
            self.specs,
            self.status,
            self.json_data,
        )
    }
}
