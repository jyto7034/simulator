use std::fmt::Display;

use crate::{enums::{phase::Phase, PlayerType}, game::Game};

#[derive(Clone)]
pub struct CardSpecs {
    attack: i32,
    defense: i32,
    cost: i32,
}

impl CardSpecs{
    pub fn new() -> CardSpecs{
        todo!()
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
    pub fn is_negated(&self) -> bool {
        self.is_negated
    }

    pub fn is_disabled(&self) -> bool {
        self.is_disabled
    }
}

#[derive(Clone)]
pub struct Modifier {
    modifier_type: ModifierType,
    value: i32,
    duration: Duration,
    source_card: Option<String>,
    applied_turn: usize,     // 효과가 적용된 턴
    applied_phase: Phase,    // 효과가 적용된 페이즈
}

impl Modifier {
    pub fn is_expired(&self, game: &Game) -> bool {
        match &self.duration {
            Duration::Permanent => false,  // 영구 지속
            
            Duration::UntilEndOfTurn => {
                // 효과가 적용된 턴이 지났는지 확인
                game.turn.get_turn_count() > self.applied_turn
            },
            
            Duration::UntilEndOfPhase => {
                // 효과가 적용된 페이즈가 지났는지 확인
                game.turn.get_turn_count() > self.applied_turn || 
                (game.turn.get_turn_count() == self.applied_turn && 
                 game.current_phase > self.applied_phase)
            },
            
            Duration::ForXTurns(turns) => {
                // 지정된 턴 수가 지났는지 확인
                game.turn.get_turn_count() >= self.applied_turn + turns
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ModifierType {
    AttackBoost,
    DefenseBoost,
    CostChange,
    EffectNegation,
    AttributeChange,
}

#[derive(Clone)]
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
        self.modifiers.retain(|m| m.modifier_type != modifier_type);
    }

    // 만료된 수정자 제거
    pub fn cleanup_expired_modifiers(&mut self, game: &Game) {
        self.modifiers.retain(|modifier| !modifier.is_expired(game));
    }

    

    // 특정 타입의 수정자 총합 계산
    pub fn get_total_modifier(&self, modifier_type: ModifierType) -> i32 {
        self.modifiers
            .iter()
            .filter(|m| m.modifier_type == modifier_type)
            .map(|m| m.value)
            .sum()
    }
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

#[derive(Copy, Clone)]
pub enum StatType{
    Attack,
    Defense,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerType {
    Self_,      // 자신 (현재 턴 플레이어)
    Opponent,   // 상대방
    Any,        // 아무나 (자신 또는 상대)
    None,       // 소유자 없음
}

impl OwnerType {
    // 현재 게임 상태에서 실제 PlayerType으로 변환
    pub fn to_player_type(&self, current_player: PlayerType) -> Option<PlayerType> {
        match self {
            OwnerType::Self_ => Some(current_player),
            OwnerType::Opponent => Some(match current_player {
                PlayerType::Player1 => PlayerType::Player2,
                PlayerType::Player2 => PlayerType::Player1,
                PlayerType::None => return None,
            }),
            OwnerType::Any => None,  // 선택 필요
            OwnerType::None => None,
        }
    }

    // 특정 플레이어가 이 OwnerType에 해당하는지 확인
    pub fn matches(&self, current_player: PlayerType, target_player: PlayerType) -> bool {
        match self {
            OwnerType::Self_ => current_player == target_player,
            OwnerType::Opponent => {
                match (current_player, target_player) {
                    (PlayerType::Player1, PlayerType::Player2) |
                    (PlayerType::Player2, PlayerType::Player1) => true,
                    _ => false,
                }
            },
            OwnerType::Any => true,
            OwnerType::None => target_player == PlayerType::None,
        }
    }
}