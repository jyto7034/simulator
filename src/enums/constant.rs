use serde::de;

use crate::deck::Card;
use crate::exception::exception::Exception;
use crate::game::*;
use crate::task::task::Task;
use std::cell::RefCell;
use std::rc::Rc;

pub const CARD_ID_JSON_PATH: &str = "Resource/cards_id.json";
pub const CARD_JSON_PATH: &str = "Resource/cards.json";
pub const DECK_JSON_PATH: &str = "Datas/data.json";
pub const UUID_GENERATOR_PATH: &str = "Resource/uuidgen";

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

#[derive(Clone, Debug)]
pub enum PlayerType {
    Player1,
    Player2,
    None,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum SpellType {
    SlowSpell,
    FastSpell,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CardType {
    Dummy,
    Unit,
    Spell(SpellType),
    Field,
}

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

#[derive(Clone)]
pub enum ZoneType {
    HandZone,
    DeckZone,
    GraveyardZone,
    FieldZone,
    None,
}

pub enum TimeType {
    Day,
    Night,
    None,
}

pub const MAX_CARD_SIZE: u32 = 30;

pub type TaskQueue = Vec<Task>;
pub type DeckCode = String;
pub type UUID = String;
pub type CardsUuid = Vec<UUID>;
pub type Runner = Rc<RefCell<dyn FnMut(&Card, &mut Game) -> Result<(), Exception>>>;
pub const COUNT_OF_CARDS: usize = 30;

pub const PLAYER_1: usize = 0;
pub const PLAYER_2: usize = 1;

pub const UNIT_ZONE_SIZE: usize = 12;
