use crate::game::phase::Phase;

use super::types::{Duration, ModifierType};

///
/// 카드의 상태를 수정하는 구조체
///
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
    // Getters
    pub fn get_modifier_type(&self) -> ModifierType {
        self.modifier_type
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn get_duration(&self) -> Duration {
        self.duration
    }

    pub fn get_source_card(&self) -> Option<&String> {
        self.source_card.as_ref()
    }

    pub fn get_applied_turn(&self) -> usize {
        self.applied_turn
    }

    pub fn get_applied_phase(&self) -> Phase {
        self.applied_phase
    }

    // Setters
    pub fn set_modifier_type(&mut self, modifier_type: ModifierType) {
        self.modifier_type = modifier_type;
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    pub fn set_source_card(&mut self, source_card: Option<String>) {
        self.source_card = source_card;
    }

    pub fn set_applied_turn(&mut self, turn: usize) {
        self.applied_turn = turn;
    }

    pub fn set_applied_phase(&mut self, phase: Phase) {
        self.applied_phase = phase;
    }

    // 편의 메서드들
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

    // 수정자가 아직 유효한지 확인
    pub fn is_expired(&self, current_turn: usize, current_phase: Phase) -> bool {
        match self.duration {
            Duration::Permanent => true,
            Duration::UntilEndOfTurn => current_turn == self.applied_turn,
            Duration::UntilEndOfPhase => {
                current_turn == self.applied_turn && current_phase == self.applied_phase
            }
            Duration::ForXTurns(turns) => current_turn <= self.applied_turn + turns,
        }
    }

    // 수정자의 남은 지속 시간 계산
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
                Some(turns.saturating_sub(current_turn - self.applied_turn))
            }
        }
    }

    // 수정자 복사본 생성 (다른 턴/페이즈에 적용)
    pub fn copy_with_new_timing(&self, turn: usize, phase: Phase) -> Self {
        let mut new = self.clone();
        new.set_applied_turn(turn);
        new.set_applied_phase(phase);
        new
    }

    // 수정자 값 변경
    pub fn modify_value(&mut self, delta: i32) {
        self.value += delta;
    }

    // 지속 시간 연장
    pub fn extend_duration(&mut self, additional: Duration) {
        self.duration = match (self.duration, additional) {
            (Duration::ForXTurns(t1), Duration::ForXTurns(t2)) => Duration::ForXTurns(t1 + t2),
            (Duration::Permanent, _) => Duration::Permanent,
            _ => self.duration,
        };
    }
}
