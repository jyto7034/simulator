pub mod phase;

use std::fmt::Display;

use crate::{card::Card, procedure::behavior::Behavior};

pub const CARD_ID_JSON_PATH: &str = "Resource/cards_id.json";
pub const CARD_JSON_PATH: &str = "Resource/cards.json";
pub const DECK_JSON_PATH_P1: &str = "Datas/player1_test.json";
pub const DECK_JSON_PATH_P2: &str = "Datas/player2_test.json";
pub const UUID_GENERATOR_PATH: &str = "Resource/uuidgen";
pub const GAME_CONFIG_JSON_PATH: &str = "Datas/config.json";



pub enum EntityType {
    Player,
    Hero,
    Agent,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlayerType {
    Player1,
    Player2,
    None,
}

#[derive(Debug, PartialEq, Clone, Eq, Copy, Hash)]
pub enum SpellType {
    SlowSpell,
    FastSpell,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum CardType {
    Dummy,
    Unit,
    Spell(SpellType),
    Field,
    Game,
}

impl Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dummy => write!(f, "Dummy"),
            Self::Unit => write!(f, "Unit"),
            Self::Spell(arg0) => f.debug_tuple("Spell").field(arg0).finish(),
            Self::Field => write!(f, "Field"),
            Self::Game => write!(f, "Game"),
        }
    }
}

impl std::fmt::Debug for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dummy => write!(f, "Dummy"),
            Self::Unit => write!(f, "Unit"),
            Self::Spell(arg0) => f.debug_tuple("Spell").field(arg0).finish(),
            Self::Field => write!(f, "Field"),
            Self::Game => write!(f, "Game"),
        }
    }
}

impl Copy for CardType {}

pub enum HeroType {
    Name1,
    Name2,
}

#[derive(Clone, Debug)]
pub enum TaskPriority {
    Immediately,
    RoundEnd,
    RoundStart,
    AttackTurn,
    DefenseTurn,
    None,
}

/// 무슨 카드의 유형을 Draw 할 건지에 대한 enum 입니다.
pub enum CardDrawType {
    Top,
    Random(usize),
    Bottom,
    CardType(CardType, usize),
}

#[derive(Clone)]
pub enum TaskType {
    Card(Card),
    Behavior(Behavior),
    None,
}

impl TaskType {
    pub fn get_data_as_behavior(&self) -> Vec<Behavior> {
        match self {
            TaskType::Card(card) => card.get_behavior_table().clone(),
            TaskType::Behavior(bv) => vec![bv.clone()],
            TaskType::None => todo!(),
        }
    }
    pub fn get_data_as_card(&self) -> Card {
        match self {
            TaskType::Card(card) => card.clone(),
            TaskType::Behavior(bv) => {
                let mut card = Card::dummy();
                card.set_card_type(CardType::Game);
                card.set_behavior_table(vec![bv.clone()]);
                card.clone()
            }
            TaskType::None => todo!(),
        }
    }
}

/// 검색 조건에 대한 enum 입니다.
pub enum FindType {
    FindByUUID(String),
    FindByCardType(CardType),
    FindByName(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ZoneType {
    HandZone,
    DeckZone,
    GraveyardZone,
    EffectZone,
    None,
}

// Mulligun !< 자신의 덱에서 카드를 4장을 꺼내서 선택한 카드들을 덱에 섞어넣고. 넣은 만큼 다시 꺼냅니다.
// Target   !< 미정
pub enum ChoiceType {
    Mulligun,
    Target,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct CardAttribute {
    pub hp: i32,
    pub atk: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TargetCard {
    // 아군 카드에게 적용 ( 덱에 있든, 손패에 있든, 필드에 있든 상관 없이 )
    All,

    // 특정 Zone 에 있는 카드에 대해서 적용
    Zone(ZoneType),

    // 특정 카드에 대해서 적용
    Uuid(UUID), // 추후 추가 예정
}

pub enum TimeType {
    Day,
    Night,
    None,
}

pub enum CardParam {
    Uuid(UUID),
    Card(Card),
}

pub enum InsertType {
    Random,
    Top,
    Bottom,
    // n장 다음에 insert
    Under(i32),
    Card(UUID),
    Slot(i32),
}

pub const MAX_CARD_SIZE: u32 = 30;

// pub type TaskQueue = Vec<Task>;
pub type DeckCode = String;
pub type UUID = String;
pub type CardsUuid = Vec<UUID>;
// pub type Runner = Rc<RefCell<dyn FnMut(&Card, &mut Game) -> Result<(), Exception>>>;
pub const COUNT_OF_CARDS: usize = 30;

pub const PLAYER_1: usize = 0;
pub const PLAYER_2: usize = 1;

pub const UNIT_ZONE_SIZE: usize = 12;
pub const DECK_ZONE_SIZE: usize = 30;
