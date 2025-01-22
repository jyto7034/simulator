use crate::{card::{effect::Effect, types::CardType, Card}, enums::PlayerType, exception::Exception, utils::{self, json::CardJson}};

pub struct CardBuilder {
    uuid: String,
    name: String,
    card_type: CardType,
    effects: Vec<Box<dyn Effect>>,
    json_data: CardJson,
}

impl CardBuilder {
    pub fn new(card_json: &CardJson) -> Result<Self, Exception> {
        Ok(CardBuilder {
            uuid: utils::generate_uuid()?,
            name: card_json.name.clone().unwrap(),
            card_type: CardType::Unit,
            effects: Vec::new(),
            json_data: card_json.clone(),
        })
    }

    pub fn add_effect<E: Effect + 'static>(mut self, effect: E) -> Self {
        self.effects.push(Box::new(effect));
        self
    }

    // card_type: CardType,
    // uuid: UUID,
    // name: String,
    // card_json: CardJson,
    // player_type: PlayerType,
    pub fn build(self) -> Card {
        Card::new(
            self.card_type,
            self.uuid,
            self.name,
            self.effects,
            self.json_data,
            PlayerType::None,
        )
    }
}