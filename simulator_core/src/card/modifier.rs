use crate::game::phase::Phase;

use super::types::{Duration, ModifierType};

/// `Modifier` 구조체는 카드 효과 또는 기타 게임 메커니즘에 의해 발생하는 상태 변경을 나타냅니다.
///
/// 이 구조체는 수정자 유형, 값, 지속 시간, 출처 카드, 적용된 턴 및 페이즈와 같은 다양한 속성을 포함합니다.
/// `Modifier`는 캐릭터의 능력치 변경, 상태 이상 적용, 특정 효과 활성화 등 게임 내 다양한 기능을 구현하는 데 사용됩니다.
///
/// # Examples
///
/// ```
/// use simulator_core::card::modifier::Modifier;
/// use simulator_core::card::types::{Duration, ModifierType};
/// use simulator_core::game::phase::Phase;
///
/// let modifier = Modifier::new(
///     ModifierType::Attack,
///     10,
///     Duration::ForXTurns(2),
///     Some("ExampleCard".to_string()),
///     1,
///     Phase::Start,
/// );
///
/// assert_eq!(modifier.get_value(), 10);
/// ```
#[derive(Clone)]
pub struct Modifier {
    modifier_type: ModifierType,
    value: i32,
    duration: Duration,
    source_card: Option<String>,
    applied_turn: usize,  // 효과가 적용된 턴
    applied_phase: Phase, // 효과가 적용된 페이즈
}

impl Modifier {
    /// 수정자 유형을 반환합니다.
    ///
    /// # Returns
    ///
    /// `ModifierType`: 수정자의 유형.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.get_modifier_type(), ModifierType::Attack);
    /// ```
    pub fn get_modifier_type(&self) -> ModifierType {
        self.modifier_type
    }

    /// 수정자 값을 반환합니다.
    ///
    /// # Returns
    ///
    /// `i32`: 수정자의 값.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.get_value(), 10);
    /// ```
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// 수정자 지속 시간을 반환합니다.
    ///
    /// # Returns
    ///
    /// `Duration`: 수정자의 지속 시간.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.get_duration(), Duration::ForXTurns(2));
    /// ```
    pub fn get_duration(&self) -> Duration {
        self.duration
    }

    /// 수정자를 발생시킨 카드 이름을 반환합니다 (있는 경우).
    ///
    /// # Returns
    ///
    /// `Option<&String>`: 카드 이름에 대한 참조 (`Some`) 또는 수정자가 카드에서 발생하지 않은 경우 `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.get_source_card(), Some(&"ExampleCard".to_string()));
    /// ```
    pub fn get_source_card(&self) -> Option<&String> {
        self.source_card.as_ref()
    }

    /// 수정자가 적용된 턴을 반환합니다.
    ///
    /// # Returns
    ///
    /// `usize`: 수정자가 적용된 턴.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.get_applied_turn(), 1);
    /// ```
    pub fn get_applied_turn(&self) -> usize {
        self.applied_turn
    }

    /// 수정자가 적용된 페이즈를 반환합니다.
    ///
    /// # Returns
    ///
    /// `Phase`: 수정자가 적용된 페이즈.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.get_applied_phase(), Phase::Start);
    /// ```
    pub fn get_applied_phase(&self) -> Phase {
        self.applied_phase
    }

    /// 수정자 유형을 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `modifier_type`: 설정할 수정자 유형.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.set_modifier_type(ModifierType::Defense);
    /// assert_eq!(modifier.get_modifier_type(), ModifierType::Defense);
    /// ```
    pub fn set_modifier_type(&mut self, modifier_type: ModifierType) {
        self.modifier_type = modifier_type;
    }

    /// 수정자 값을 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `value`: 설정할 수정자 값.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.set_value(20);
    /// assert_eq!(modifier.get_value(), 20);
    /// ```
    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    /// 수정자 지속 시간을 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `duration`: 설정할 지속 시간.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.set_duration(Duration::Permanent);
    /// assert_eq!(modifier.get_duration(), Duration::Permanent);
    /// ```
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    /// 수정자를 발생시킨 카드 이름을 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `source_card`: 설정할 카드 이름 (있는 경우).
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.set_source_card(Some("NewCard".to_string()));
    /// assert_eq!(modifier.get_source_card(), Some(&"NewCard".to_string()));
    /// ```
    pub fn set_source_card(&mut self, source_card: Option<String>) {
        self.source_card = source_card;
    }

    /// 수정자가 적용된 턴을 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `turn`: 설정할 턴.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.set_applied_turn(2);
    /// assert_eq!(modifier.get_applied_turn(), 2);
    /// ```
    pub fn set_applied_turn(&mut self, turn: usize) {
        self.applied_turn = turn;
    }

    /// 수정자가 적용된 페이즈를 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `phase`: 설정할 페이즈.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.set_applied_phase(Phase::End);
    /// assert_eq!(modifier.get_applied_phase(), Phase::End);
    /// ```
    pub fn set_applied_phase(&mut self, phase: Phase) {
        self.applied_phase = phase;
    }

    /// 새 `Modifier` 인스턴스를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `modifier_type`: 수정자 유형.
    /// * `value`: 수정자 값.
    /// * `duration`: 지속 시간.
    /// * `source_card`: 출처 카드 (있는 경우).
    /// * `applied_turn`: 적용된 턴.
    /// * `applied_phase`: 적용된 페이즈.
    ///
    /// # Returns
    ///
    /// `Self`: 새 `Modifier` 인스턴스.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.get_modifier_type(), ModifierType::Attack);
    /// assert_eq!(modifier.get_value(), 10);
    /// ```
    pub fn new(
        modifier_type: ModifierType,
        value: i32,
        duration: Duration,
        source_card: Option<String>,
        applied_turn: usize,
        applied_phase: Phase,
    ) -> Self {
        Self {
            modifier_type,
            value,
            duration,
            source_card,
            applied_turn,
            applied_phase,
        }
    }

    /// 수정자가 만료되었는지 확인합니다.
    ///
    /// # Arguments
    ///
    /// * `current_turn`: 현재 턴.
    /// * `current_phase`: 현재 페이즈.
    ///
    /// # Returns
    ///
    /// `bool`: 수정자가 만료되었으면 `true`, 그렇지 않으면 `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.is_expired(3, Phase::Start), false);
    /// assert_eq!(modifier.is_expired(4, Phase::Start), true);
    /// ```
    pub fn is_expired(&self, current_turn: usize, current_phase: Phase) -> bool {
        match self.duration {
            Duration::Permanent => false, // Permanent는 만료되지 않음
            Duration::UntilEndOfTurn => current_turn > self.applied_turn,
            Duration::UntilEndOfPhase => {
                current_turn > self.applied_turn
                    || (current_turn == self.applied_turn && current_phase > self.applied_phase)
            }
            Duration::ForXTurns(turns) => current_turn > self.applied_turn + turns,
        }
    }

    /// 수정자의 남은 지속 시간을 계산합니다.
    ///
    /// # Arguments
    ///
    /// * `current_turn`: 현재 턴.
    ///
    /// # Returns
    ///
    /// `Option<usize>`: 남은 턴 수 (`Some`) 또는 지속 시간이 무제한인 경우 `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// assert_eq!(modifier.remaining_duration(2), Some(1));
    /// assert_eq!(modifier.remaining_duration(3), Some(0));
    /// ```
    pub fn remaining_duration(&self, current_turn: usize) -> Option<usize> {
        match self.duration {
            Duration::Permanent => None,
            Duration::UntilEndOfTurn => {
                if current_turn > self.applied_turn {
                    Some(0)
                } else {
                    Some(1)
                }
            }
            Duration::UntilEndOfPhase => Some(if current_turn > self.applied_turn {
                0
            } else {
                1
            }),
            Duration::ForXTurns(turns) => {
                if current_turn <= self.applied_turn {
                    Some(turns)
                } else {
                    Some(turns.saturating_sub(current_turn - self.applied_turn))
                }
            }
        }
    }

    /// 새 타이밍으로 수정자 복사본을 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `turn`: 새 턴.
    /// * `phase`: 새 페이즈.
    ///
    /// # Returns
    ///
    /// `Self`: 새 타이밍으로 생성된 수정자 복사본.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// let new_modifier = modifier.copy_with_new_timing(2, Phase::End);
    /// assert_eq!(new_modifier.get_applied_turn(), 2);
    /// assert_eq!(new_modifier.get_applied_phase(), Phase::End);
    /// ```
    pub fn copy_with_new_timing(&self, turn: usize, phase: Phase) -> Self {
        let mut new = self.clone();
        new.set_applied_turn(turn);
        new.set_applied_phase(phase);
        new
    }

    /// 수정자 값을 변경합니다.
    ///
    /// # Arguments
    ///
    /// * `delta`: 변경할 값.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.modify_value(5);
    /// assert_eq!(modifier.get_value(), 15);
    /// ```
    pub fn modify_value(&mut self, delta: i32) {
        self.value += delta;
    }

    /// 지속 시간을 연장합니다.
    ///
    /// # Arguments
    ///
    /// * `additional`: 추가할 지속 시간.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::modifier::Modifier;
    /// use simulator_core::card::types::{Duration, ModifierType};
    /// use simulator_core::game::phase::Phase;
    ///
    /// let mut modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::ForXTurns(2),
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    ///
    /// modifier.extend_duration(Duration::ForXTurns(1));
    /// assert_eq!(modifier.get_duration(), Duration::ForXTurns(3));
    ///
    /// let mut permanent_modifier = Modifier::new(
    ///     ModifierType::Attack,
    ///     10,
    ///     Duration::Permanent,
    ///     Some("ExampleCard".to_string()),
    ///     1,
    ///     Phase::Start,
    /// );
    /// permanent_modifier.extend_duration(Duration::ForXTurns(1));
    /// assert_eq!(permanent_modifier.get_duration(), Duration::Permanent);
    /// ```
    pub fn extend_duration(&mut self, additional: Duration) {
        self.duration = match (self.duration, additional) {
            (Duration::ForXTurns(t1), Duration::ForXTurns(t2)) => Duration::ForXTurns(t1 + t2),
            (Duration::Permanent, _) => Duration::Permanent,
            _ => self.duration,
        };
    }
}
