use uuid::Uuid;

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

impl ZoneType {
    pub fn to_string(&self) -> String {
        match self {
            ZoneType::Hand => "Hand".to_string(),
            ZoneType::Deck => "Deck".to_string(),
            ZoneType::Graveyard => "Graveyard".to_string(),
            ZoneType::Effect => "Effect".to_string(),
            ZoneType::Field => "Field".to_string(),
            ZoneType::None => "None".to_string(),
        }
    }
}

pub const MAX_CARD_SIZE: usize = 30;

pub type CardsUuid = Vec<Uuid>;

pub const COUNT_OF_CARDS: usize = 30;
pub const COUNT_OF_MULLIGAN_CARDS: usize = 5;
pub const PLAYER_1: usize = 0;
pub const PLAYER_2: usize = 1;

pub const UNIT_ZONE_SIZE: usize = 12;
pub const DECK_ZONE_SIZE: usize = 30;
pub const HAND_ZONE_SIZE: usize = 10;

pub const HEARTBEAT_INTERVAL: u64 = 5;
pub const CLIENT_TIMEOUT: u64 = 20; // 20초 동안 응답 없으면 연결 끊김
