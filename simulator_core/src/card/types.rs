use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};

use actix::Addr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    exception::{GameError, GameplayError},
    game::GameActor,
    resource::CardSpecsResource,
    utils::json::CardJson,
};

use super::modifier::Modifier;

#[derive(Clone)]
/// `CardSpecs`는 카드의 공격력, 방어력, 비용과 같은 스펙을 나타내는 구조체입니다.
// TODO: `attack`, `defense` 필드의 타입을 CardSpecsResource가 아닌 구체적인 타입으로 변경할 필요가 있는지 검토
// TODO: 필드에 대한 상세 설명 추가 (공격력 범위, 방어력 계산 방식 등)
pub struct CardSpecs {
    attack: CardSpecsResource,
    defense: CardSpecsResource,
    cost: i32,
}

/// `CardSpecs`의 새로운 인스턴스를 생성합니다.
///
/// # Arguments
///
/// * `json` - `CardJson` 구조체의 참조. 카드 스펙 정보를 담고 있습니다.
///
/// # Returns
///
/// 새로운 `CardSpecs` 인스턴스.
// TODO: JSON 필드가 없을 경우의 예외 처리 추가
// TODO: `unwrap()` 호출 대신 `?` 연산자를 사용하여 오류를 전파하도록 변경
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
/// `CardStatus`는 카드의 현재 상태를 나타내는 구조체입니다.
///
/// 카드가 무효화되었는지, 비활성화되었는지, 그리고 적용된 수정자 목록을 저장합니다.
// TODO: `modifiers` 필드에 대한 상세 설명 추가 (수정자의 종류, 적용 방식 등)
pub struct CardStatus {
    is_negated: bool,
    is_disabled: bool,
    modifiers: Vec<Modifier>,
}

/// `CardStatus`의 새로운 기본 인스턴스를 생성합니다.
///
/// 초기 상태는 무효화되지 않고, 비활성화되지 않았으며, 수정자 목록은 비어 있습니다.
///
/// # Returns
///
/// 기본값을 가진 새로운 `CardStatus` 인스턴스.
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
/// `ModifierType`은 수정자의 유형을 나타내는 열거형입니다.
///
/// 공격력 증가, 방어력 증가, 비용 변경, 효과 무효화, 속성 변경 등의 유형을 정의합니다.
// TODO: 각 ModifierType에 대한 상세 설명 추가 (공격력 증가의 종류, 속성 변경의 범위 등)
pub enum ModifierType {
    AttackBoost,
    DefenseBoost,
    CostChange,
    EffectNegation,
    AttributeChange,
}

#[derive(Clone, Copy)]
/// `Duration`은 수정자의 지속 시간을 나타내는 열거형입니다.
///
/// 영구, 턴 종료까지, 페이즈 종료까지, 특정 턴 수 동안 등의 지속 시간을 정의합니다.
// TODO: 각 Duration에 대한 상세 설명 추가 (턴 종료, 페이즈 종료의 시점 정의)
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
    pub fn cleanup_expired_modifiers(&mut self, game: Addr<GameActor>) {
        todo!()
        // self.modifiers.retain(|modifier| {
        //     !modifier.is_expired(game.get_turn().get_turn_count(), game.get_phase())
        // });
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
/// `SpellType`은 마법 카드의 종류를 나타내는 열거형입니다.
///
/// SlowSpell, FastSpell 등의 유형을 정의합니다.
// TODO: 각 SpellType에 대한 상세 설명 추가 (발동 조건, 효과 발동 시점 등)
pub enum SpellType {
    SlowSpell,
    FastSpell,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
/// `CardType`은 카드의 종류를 나타내는 열거형입니다.
///
/// Dummy, Unit, Spell, Field, Ace, Trap, Game, Any 등의 유형을 정의합니다.
// TODO: 각 CardType에 대한 상세 설명 추가 (카드 사용 조건, 지속 효과 등)
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
                _ => Err(GameError::Gameplay(GameplayError::InvalidAction {
                    reason: "Invalid card type".to_string(),
                })),
            },
            None => Err(GameError::Gameplay(GameplayError::InvalidAction {
                reason: "Invalid card type".to_string(),
            })),
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
/// `StatType`은 스탯의 종류를 나타내는 열거형입니다.
///
/// Attack, Defense 등의 유형을 정의합니다.
// TODO: 각 StatType에 대한 상세 설명 추가 (스탯 계산 방식, 적용 범위 등)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
/// `PlayerKind`는 플레이어의 종류를 나타내는 열거형입니다.
///
/// Player1, Player2 등의 유형을 정의합니다.
pub enum PlayerKind {
    Player1,
    Player2,
}

impl Display for PlayerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PlayerKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlayerKind::Player1 => "Player1",
            PlayerKind::Player2 => "Player2",
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PlayerKind::Player1 => "Player1".to_string(),
            PlayerKind::Player2 => "Player2".to_string(),
        }
    }

    pub fn reverse(&self) -> Self {
        match self {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        }
    }
}

impl From<String> for PlayerKind {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Player1" => PlayerKind::Player1,
            "Player2" => PlayerKind::Player2,
            _ => panic!("Invalid PlayerKind string"),
        }
    }
}

impl From<PlayerKind> for String {
    fn from(player_kind: PlayerKind) -> Self {
        player_kind.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// `PlayerIdentity`는 플레이어의 고유 식별 정보를 나타내는 구조체입니다.
///
/// `id`는 플레이어의 UUID이고, `kind`는 플레이어의 종류(`PlayerKind`)입니다.
pub struct PlayerIdentity {
    pub id: Uuid,
    pub kind: PlayerKind,
}

impl Hash for PlayerIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.kind.hash(state);
    }
}
