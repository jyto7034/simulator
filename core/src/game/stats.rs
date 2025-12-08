use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Copy)]
pub struct UnitStats {
    /// 최대 체력 (전투 시작 기준)
    pub max_health: u32,
    /// 기본 공격력
    pub attack: u32,
    /// 기본 방어력 (감쇠 계수에 쓸 값)
    pub defense: u32,
    /// 공격 주기(ms). 작을수록 빠름.
    pub attack_interval_ms: u64,
}

impl UnitStats {
    /// 모든 값을 0으로 초기화한 기본 스탯 생성
    pub fn new() -> Self {
        Self {
            max_health: 0,
            attack: 0,
            defense: 0,
            attack_interval_ms: 0,
        }
    }

    /// 명시적인 값으로 초기화
    pub fn with_values(
        max_health: u32,
        attack: u32,
        defense: u32,
        attack_interval_ms: u64,
    ) -> Self {
        Self {
            max_health,
            attack,
            defense,
            attack_interval_ms,
        }
    }

    /// 체력에 델타를 더한다 (음수면 감소, 0 이하로 떨어지지 않도록 보정).
    pub fn add_max_health(&mut self, delta: i32) {
        if delta >= 0 {
            self.max_health = self.max_health.saturating_add(delta as u32);
        } else {
            let dec = delta.unsigned_abs().min(self.max_health);
            self.max_health = self.max_health.saturating_sub(dec);
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
            self.attack_interval_ms = self
                .attack_interval_ms
                .saturating_add(delta_ms as u64);
        } else {
            let dec = delta_ms.unsigned_abs().min(self.attack_interval_ms);
            self.attack_interval_ms = self.attack_interval_ms.saturating_sub(dec);
        }
    }

    /// 단일 StatModifier를 적용
    pub fn apply_modifier(&mut self, modifier: StatModifier) {
        use StatId::*;
        match modifier.stat {
            MaxHealth => match modifier.kind {
                StatModifierKind::Flat => self.add_max_health(modifier.value),
                StatModifierKind::Percent => {
                    let base = self.max_health as i32;
                    let delta = base * modifier.value / 100;
                    self.add_max_health(delta);
                }
            },
            Attack => match modifier.kind {
                StatModifierKind::Flat => self.add_attack(modifier.value),
                StatModifierKind::Percent => {
                    let base = self.attack as i32;
                    let delta = base * modifier.value / 100;
                    self.add_attack(delta);
                }
            },
            Defense => match modifier.kind {
                StatModifierKind::Flat => self.add_defense(modifier.value),
                StatModifierKind::Percent => {
                    let base = self.defense as i32;
                    let delta = base * modifier.value / 100;
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
}

impl Default for UnitStats {
    fn default() -> Self {
        Self::new()
    }
}

