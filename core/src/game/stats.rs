use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game::ability::AbilityId;

/// 트리거 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerType {
    /// 상시 적용
    Permanent,
    /// 공격 시
    OnAttack,
    /// 피격 시
    OnHit,
    /// 처치 시
    OnKill,
    /// 사망 시
    OnDeath,
    /// 전투 시작 시
    OnBattleStart,
    /// 아군 사망 시
    OnAllyDeath,
}

/// 트리거 발동 시 적용되는 효과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    /// 스탯 변경
    Modifier(StatModifier),
    /// 추가 데미지 (현재 공격에)
    BonusDamage { flat: i32, percent: i32 },
    /// 체력 회복
    Heal { flat: i32, percent: i32 },
    /// 버프 적용
    ApplyBuff { buff_id: String, duration_ms: u64 },
    /// 어빌리티 실행 (복잡한 로직)
    Ability(AbilityId),
}

/// 트리거 기반 효과 맵
pub type TriggeredEffects = HashMap<TriggerType, Vec<Effect>>;

/// 전역 전투 스탯 ID
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum StatId {
    /// 최대 체력
    MaxHealth,
    /// 기본 공격력
    Attack,
    /// 기본 방어력
    Defense,
    /// 공격 주기(ms). 작을수록 빠름.
    AttackIntervalMs,
}

/// 스탯 변경 타입
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum StatModifierKind {
    /// 고정 값 증감 (예: +50 HP)
    Flat,
    /// 퍼센트 증감 (예: +20% 공격력)
    Percent,
}

/// 하나의 스탯 변경을 표현
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatModifier {
    pub stat: StatId,
    pub kind: StatModifierKind,
    /// Flat이면 그대로 값, Percent이면 퍼센트 (예: +20% → 20, -10% → -10)
    pub value: i32,
}

/// 전투에 사용되는 최종 스탯
/// 아이템, 아티팩트 등 적용된 수치
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnitStats {
    /// 최대 체력 (전투 시작 기준)
    pub max_health: u32,
    /// 현재 체력
    pub current_health: u32,
    /// 기본 공격력
    pub attack: u32,
    /// 기본 방어력 (감쇠 계수에 쓸 값)
    pub defense: u32,
    /// 공격 주기(ms). 작을수록 빠름.
    pub attack_interval_ms: u64,
}

impl UnitStats {
    const MAX_HEALTH: u32 = i32::MAX as u32;

    /// 모든 값을 0으로 초기화한 기본 스탯 생성
    pub fn new() -> Self {
        Self {
            max_health: 0,
            attack: 0,
            defense: 0,
            attack_interval_ms: 0,
            current_health: 0,
        }
    }

    /// 명시적인 값으로 초기화
    pub fn with_values(
        max_health: u32,
        current_health: u32,
        attack: u32,
        defense: u32,
        attack_interval_ms: u64,
    ) -> Self {
        let max_health = max_health.min(Self::MAX_HEALTH);
        Self {
            max_health,
            attack,
            defense,
            attack_interval_ms,
            current_health: current_health.min(max_health),
        }
    }

    /// 체력에 델타를 더한다 (음수면 감소, 0 이하로 떨어지지 않도록 보정).
    pub fn add_max_health(&mut self, delta: i32) {
        if delta >= 0 {
            self.max_health = self
                .max_health
                .saturating_add(delta as u32)
                .min(Self::MAX_HEALTH);
        } else {
            let dec = delta.unsigned_abs().min(self.max_health);
            self.max_health = self.max_health.saturating_sub(dec);
        }
        if self.current_health > self.max_health {
            self.current_health = self.max_health;
        }
    }

    /// 공격력에 델타를 더한다 (음수면 감소, 0 이하로 떨어지지 않도록 보정).
    pub fn add_attack(&mut self, delta: i32) {
        if delta >= 0 {
            self.attack = self.attack.saturating_add(delta as u32);
        } else {
            let dec = delta.unsigned_abs().min(self.attack);
            self.attack = self.attack.saturating_sub(dec);
        }
    }

    /// 방어력에 델타를 더한다 (음수면 감소, 0 이하로 떨어지지 않도록 보정).
    pub fn add_defense(&mut self, delta: i32) {
        if delta >= 0 {
            self.defense = self.defense.saturating_add(delta as u32);
        } else {
            let dec = delta.unsigned_abs().min(self.defense);
            self.defense = self.defense.saturating_sub(dec);
        }
    }

    /// 공격 주기(ms)에 델타를 더한다.
    /// 양수면 느려지고, 음수면 빨라지며 0 이하로는 내려가지 않는다.
    pub fn add_attack_interval_ms(&mut self, delta_ms: i64) {
        if delta_ms >= 0 {
            self.attack_interval_ms = self.attack_interval_ms.saturating_add(delta_ms as u64);
        } else {
            let dec = delta_ms.unsigned_abs().min(self.attack_interval_ms);
            self.attack_interval_ms = self.attack_interval_ms.saturating_sub(dec);
        }
        // 전투 이벤트 스케줄링이 멈추지 않도록 1ms 이상 유지
        if self.attack_interval_ms == 0 {
            self.attack_interval_ms = 1;
        }
    }

    /// 단일 StatModifier를 적용
    pub fn apply_modifier(&mut self, modifier: StatModifier) {
        use StatId::*;
        match modifier.stat {
            MaxHealth => match modifier.kind {
                StatModifierKind::Flat => self.add_max_health(modifier.value),
                StatModifierKind::Percent => {
                    let base = i64::from(self.max_health);
                    let delta = base.saturating_mul(i64::from(modifier.value)) / 100;
                    let delta = delta.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                    self.add_max_health(delta);
                }
            },
            Attack => match modifier.kind {
                StatModifierKind::Flat => self.add_attack(modifier.value),
                StatModifierKind::Percent => {
                    let base = i64::from(self.attack);
                    let delta = base.saturating_mul(i64::from(modifier.value)) / 100;
                    let delta = delta.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                    self.add_attack(delta);
                }
            },
            Defense => match modifier.kind {
                StatModifierKind::Flat => self.add_defense(modifier.value),
                StatModifierKind::Percent => {
                    let base = i64::from(self.defense);
                    let delta = base.saturating_mul(i64::from(modifier.value)) / 100;
                    let delta = delta.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                    self.add_defense(delta);
                }
            },
            AttackIntervalMs => match modifier.kind {
                StatModifierKind::Flat => self.add_attack_interval_ms(modifier.value as i64),
                StatModifierKind::Percent => {
                    let base = self.attack_interval_ms as i64;
                    let delta = base * (modifier.value as i64) / 100;
                    self.add_attack_interval_ms(delta);
                }
            },
        }
    }

    /// 여러 StatModifier를 순서대로 적용
    pub fn apply_modifiers<I>(&mut self, modifiers: I)
    where
        I: IntoIterator<Item = StatModifier>,
    {
        for m in modifiers {
            self.apply_modifier(m);
        }
    }

    /// TriggeredEffects에서 Permanent 트리거의 Modifier 효과만 적용
    pub fn apply_permanent_effects(&mut self, effects: &TriggeredEffects) {
        if let Some(permanent_effects) = effects.get(&TriggerType::Permanent) {
            for effect in permanent_effects {
                if let Effect::Modifier(modifier) = effect {
                    self.apply_modifier(*modifier);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_stats_percent_modifiers_do_not_wrap_on_large_values() {
        let mut stats = UnitStats::with_values(u32::MAX, u32::MAX, u32::MAX, u32::MAX, 1);
        assert_eq!(stats.max_health, i32::MAX as u32);
        assert_eq!(stats.current_health, i32::MAX as u32);

        stats.apply_modifier(StatModifier {
            stat: StatId::MaxHealth,
            kind: StatModifierKind::Percent,
            value: 100,
        });
        assert_eq!(stats.max_health, i32::MAX as u32);
        assert_eq!(stats.current_health, i32::MAX as u32);

        stats.apply_modifier(StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Percent,
            value: 100,
        });
        assert_eq!(stats.attack, u32::MAX);
    }
}

impl Default for UnitStats {
    fn default() -> Self {
        Self::new()
    }
}
