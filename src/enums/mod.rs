use serde::{Deserialize, Serialize};

pub mod phase;

pub const CARD_ID_JSON_PATH: &str = "Resource/cards_id.json";
pub const CARD_JSON_PATH: &str = "Resource/cards.json";
pub const DECK_JSON_PATH_P1: &str = "Datas/player1_test.json";
pub const DECK_JSON_PATH_P2: &str = "Datas/player2_test.json";
pub const UUID_GENERATOR_PATH: &str = "Resource/uuidgen";
pub const GAME_CONFIG_JSON_PATH: &str = "Datas/config.json";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum ZoneType {
    Hand,
    Deck,
    Graveyard,
    Effect,
    Field,
    None,
}

#[derive(Clone, Copy)]
pub struct CardLocation(pub ZoneType);

pub const MAX_CARD_SIZE: u32 = 30;

pub struct DeckCode(pub String);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct UUID(pub String);

pub type CardsUuid = Vec<UUID>;

pub const COUNT_OF_CARDS: usize = 30;
pub const COUNT_OF_MULLIGAN_CARDS: usize = 5;
pub const PLAYER_1: usize = 0;
pub const PLAYER_2: usize = 1;

pub const UNIT_ZONE_SIZE: usize = 12;
pub const DECK_ZONE_SIZE: usize = 30;
