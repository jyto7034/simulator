use crate::task::task::Task;

pub const CARD_JSON_PATH: &str = "C:/WORK/simulator/Resource/cards.json";
pub const DECK_JSON_PATH: &str = "C:/WORK/simulator/Datas/data.json";

pub enum GameStep {
    GameStart,
    GameEnd,
    RoundStart,
    RoundEnd,
    AttackTurn,
    DefenseTurn,
    Execution,
    Mulligun,
}

pub enum GameState {
    GameStart,
    GameEnd,
    RoundStart,
    RoundEnd,
    AttackTurn,
    DefenseTurn,
    Execution,
    Mulligun,
}

pub enum EntityType {
    Player,
    Hero,
    Agent,
}

pub enum PlayerType {}

#[derive(Debug, PartialEq, Clone)]
pub enum SpellType {
    SlowSpell,
    FastSpell,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CardType {
    Dummy,
    Agent,
    Spell(SpellType),
    Field,
}

pub enum HeroType {
    Name1,
    Name2,
}

#[derive(Clone)]
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
    Random,
    Bottom,
    CardType(CardType),
}

/// 검색 조건에 대한 enum 입니다.
pub enum FindType {
    FindByUUID(String),
    FindByCardType(CardType),
    FindByName(String),
}

pub const MAX_CARD_SIZE: u32 = 30;

pub type TaskQueue = Vec<Task>;

pub type UUID = String;

pub const COUNT_OF_CARDS: usize = 30;

pub const PLAYER_1: usize = 0;
pub const PLAYER_2: usize = 1;
