use std::fmt;

use serde::Deserialize;

use crate::{exception::GameError, game::Game, resource::CardSpecsResource, utils::json::CardJson};

use super::modifier::Modifier;

#[derive(Clone)]
pub struct CardSpecs {
    attack: CardSpecsResource,
    defense: CardSpecsResource,
    cost: i32,
}

impl CardSpecs {
    pub fn new(json: &CardJson) -> Self {
        Self {
            attack: CardSpecsResource::new(json.attack.unwrap()),
            defense: CardSpecsResource::new(json.health.unwrap()),
            cost: json.cost.unwrap(),
        }
    }
}

// CardStatus 구조체 (카드의 현재 상태)
#[derive(Clone, Default)]
pub struct CardStatus {
    is_negated: bool,
    is_disabled: bool,
    modifiers: Vec<Modifier>,
}

impl CardStatus {
    pub fn new() -> Self {
        Self {
            is_negated: false,
            is_disabled: false,
            modifiers: vec![],
        }
    }

    pub fn is_negated(&self) -> bool {
        self.is_negated
    }

    pub fn is_disabled(&self) -> bool {
        self.is_disabled
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum ModifierType {
    AttackBoost,
    DefenseBoost,
    CostChange,
    EffectNegation,
    AttributeChange,
}

#[derive(Clone, Copy)]
pub enum Duration {
    Permanent,
    UntilEndOfTurn,
    UntilEndOfPhase,
    ForXTurns(usize),
}

impl CardStatus {
    // 수정자 추가
    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
    }

    // 수정자 제거
    pub fn remove_modifier(&mut self, index: usize) {
        self.modifiers.remove(index);
    }

    // 특정 타입의 수정자 모두 제거
    pub fn remove_modifiers_of_type(&mut self, modifier_type: ModifierType) {
        self.modifiers
            .retain(|m| m.get_modifier_type() != modifier_type);
    }

    // 만료된 수정자 제거
    pub fn cleanup_expired_modifiers(&mut self, game: &Game) {
        self.modifiers.retain(|modifier| {
            !modifier.is_expired(game.get_turn().get_turn_count(), game.get_phase())
        });
    }

    // 특정 타입의 수정자 총합 계산
    pub fn get_total_modifier(&self, modifier_type: ModifierType) -> i32 {
        self.modifiers
            .iter()
            .filter(|m| m.get_modifier_type() == modifier_type)
            .map(|m| m.get_value())
            .sum()
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Copy, Hash)]
pub enum SpellType {
    SlowSpell,
    FastSpell,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum CardType {
    Dummy,
    Unit,
    Spell,
    Field,
    Ace,
    Trap,
    Game,
    Any,
}

impl CardType {
    pub fn from_json(json: &CardJson) -> Result<Self, GameError> {
        match &json.r#type {
            Some(type_str) => match type_str.as_str() {
                "Dummy" => Ok(CardType::Dummy),
                "Unit" => Ok(CardType::Unit),
                "Spell" => Ok(CardType::Spell),
                "Field" => Ok(CardType::Field),
                "Ace" => Ok(CardType::Ace),
                "Trap" => Ok(CardType::Trap),
                "Game" => Ok(CardType::Game),
                _ => Err(GameError::InvalidCardType),
            },
            None => Err(GameError::InvalidCardType),
        }
    }

    // 추가 유틸리티 메서드들
    pub fn to_string(&self) -> &'static str {
        match self {
            CardType::Dummy => "Dummy",
            CardType::Unit => "Unit",
            CardType::Spell => "Spell",
            CardType::Field => "Field",
            CardType::Ace => "Ace",
            CardType::Trap => "Trap",
            CardType::Game => "Game",
            CardType::Any => "Any",
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, CardType::Unit)
    }

    pub fn is_spell(&self) -> bool {
        matches!(self, CardType::Spell)
    }

    pub fn is_field(&self) -> bool {
        matches!(self, CardType::Field)
    }

    pub fn is_trap(&self) -> bool {
        matches!(self, CardType::Trap)
    }
    // 카드 타입별 특성 확인
    pub fn can_be_played_as_action(&self) -> bool {
        matches!(self, CardType::Spell | CardType::Trap)
    }

    pub fn stays_on_field(&self) -> bool {
        matches!(self, CardType::Unit | CardType::Field)
    }

    pub fn is_permanent(&self) -> bool {
        matches!(self, CardType::Field | CardType::Game)
    }

    // 카드 타입별 제한사항
    pub fn max_copies_allowed(&self) -> i32 {
        match self {
            CardType::Ace => 1,
            CardType::Game => 1,
            _ => 3,
        }
    }
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CardType::Dummy => "Dummy",
            CardType::Unit => "Unit",
            CardType::Spell => "Spell",
            CardType::Field => "Field",
            CardType::Ace => "Ace",
            CardType::Trap => "Trap",
            CardType::Game => "Game",
            CardType::Any => "Any",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Debug for CardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug 구현 시 Display 구현을 재사용합니다.
        write!(f, "{}", self)
    }
}

#[derive(Copy, Clone)]
pub enum StatType {
    Attack,
    Defense,
}

///
/// 백엔드을 실행하는건 Host 역할을 부여 받은 플레이어쪽임.
/// 백엔드에서 Host 는 Player1 혹은 Self_ 로 취급되고
/// Client 는 Player2 혹은 Opponent 로 취급함.
///

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerType {
    Self_,    // 자신 (현재 턴 플레이어)
    Opponent, // 상대방
    Any,      // 아무나 (자신 또는 상대)
    None,     // 소유자 없음
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Hash)]
pub enum PlayerType {
    Player1,
    Player2,
}

impl PlayerType {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Player1 => Self::Player1,
            Self::Player2 => Self::Player2,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PlayerType::Player1 => "player1",
            PlayerType::Player2 => "player2",
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PlayerType::Player1 => "player1".to_string(),
            PlayerType::Player2 => "player2".to_string(),
        }

    }
}

impl From<PlayerType> for OwnerType {
    fn from(value: PlayerType) -> Self {
        match value {
            PlayerType::Player1 => Self::Self_,
            PlayerType::Player2 => Self::Opponent,
        }
    }
}

impl From<OwnerType> for PlayerType {
    fn from(value: OwnerType) -> Self {
        match value {
            OwnerType::Self_ => PlayerType::Player1,
            OwnerType::Opponent => PlayerType::Player2,
            _ => panic!("Invalid OwnerType to convert to PlayerType"),
        }
    }
}

impl From<PlayerType> for String {
    fn from(value: PlayerType) -> Self {
        match value {
            PlayerType::Player1 => "player1".to_string(),
            PlayerType::Player2 => "player2".to_string(),
        }
    }
}

impl From<String> for PlayerType {
    fn from(value: String) -> Self {
        match &value[..] {
            "player1" => PlayerType::Player1,
            "player2" => PlayerType::Player2,
            _ => panic!("Invalid string to convert to PlayerType. Got: {}", value),
        }
    }
}

impl From<&str> for PlayerType {
    fn from(value: &str) -> Self {
        match value {
            "player1" => PlayerType::Player1,
            "player2" => PlayerType::Player2,
            _ => panic!("Invalid string to convert to PlayerType. Got: {}", value),
        }
    }
}

pub trait OwnershipComparable {
    fn matches_owner(&self, owner: &OwnerType) -> bool;
}

impl OwnershipComparable for PlayerType {
    fn matches_owner(&self, owner: &OwnerType) -> bool {
        matches!(
            (self, owner),
            (PlayerType::Player1, OwnerType::Self_)
                | (PlayerType::Player2, OwnerType::Opponent)
                | (_, OwnerType::Any)
        )
    }
}
