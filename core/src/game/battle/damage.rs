use crate::game::battle::ids::UnitInstanceId;

use crate::game::ability::SkillId;
#[cfg(test)]
use crate::game::stats::TriggerEffectTarget;
use crate::game::{enums::Side, stats::Effect};
use serde::{Deserialize, Serialize};

use super::buffs::BuffId;
use super::cooldown::{CooldownSource, SourcedEffect};

/// 데미지 요청 - 데미지 계산에 필요한 모든 정보
#[derive(Debug, Clone)]
pub struct DamageRequest {
    pub source: DamageSource,
    pub damage_type: DamageType,
    pub modifiers: DamageModifiers,
    pub crit_roll_percent: Option<u8>,
    pub attacker_id: UnitInstanceId,
    pub target_id: UnitInstanceId,
    pub base_damage: u32,
    pub minimum_damage: u32,
    pub time_ms: u64,
}

/// 출처별 2차 데미지 계산 보정값.
///
/// 저항 조정은 감쇠 전에 적용하고, 증폭/감소는 감쇠 후에 적용한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DamageModifiers {
    #[serde(default)]
    pub armor_penetration_flat: i32,
    #[serde(default)]
    pub magic_resist_penetration_flat: i32,
    #[serde(default)]
    pub armor_penetration_percent: i32,
    #[serde(default)]
    pub magic_resist_penetration_percent: i32,
    #[serde(default)]
    pub damage_amp_percent: i32,
    #[serde(default)]
    pub damage_reduction_percent: i32,
    #[serde(default)]
    pub physical_damage_amp_percent: i32,
    #[serde(default)]
    pub magic_damage_amp_percent: i32,
    #[serde(default)]
    pub true_damage_amp_percent: i32,
    #[serde(default)]
    pub physical_damage_reduction_percent: i32,
    #[serde(default)]
    pub magic_damage_reduction_percent: i32,
    #[serde(default)]
    pub true_damage_reduction_percent: i32,
    #[serde(default)]
    pub crit_chance_percent: i32,
    /// Critical hit bonus. 50 means a critical hit deals 150% raw damage.
    #[serde(default)]
    pub crit_damage_percent: i32,
}

impl DamageModifiers {
    pub fn merge(self, other: Self) -> Self {
        Self {
            armor_penetration_flat: self
                .armor_penetration_flat
                .saturating_add(other.armor_penetration_flat),
            magic_resist_penetration_flat: self
                .magic_resist_penetration_flat
                .saturating_add(other.magic_resist_penetration_flat),
            armor_penetration_percent: self
                .armor_penetration_percent
                .saturating_add(other.armor_penetration_percent),
            magic_resist_penetration_percent: self
                .magic_resist_penetration_percent
                .saturating_add(other.magic_resist_penetration_percent),
            damage_amp_percent: self
                .damage_amp_percent
                .saturating_add(other.damage_amp_percent),
            damage_reduction_percent: self
                .damage_reduction_percent
                .saturating_add(other.damage_reduction_percent),
            physical_damage_amp_percent: self
                .physical_damage_amp_percent
                .saturating_add(other.physical_damage_amp_percent),
            magic_damage_amp_percent: self
                .magic_damage_amp_percent
                .saturating_add(other.magic_damage_amp_percent),
            true_damage_amp_percent: self
                .true_damage_amp_percent
                .saturating_add(other.true_damage_amp_percent),
            physical_damage_reduction_percent: self
                .physical_damage_reduction_percent
                .saturating_add(other.physical_damage_reduction_percent),
            magic_damage_reduction_percent: self
                .magic_damage_reduction_percent
                .saturating_add(other.magic_damage_reduction_percent),
            true_damage_reduction_percent: self
                .true_damage_reduction_percent
                .saturating_add(other.true_damage_reduction_percent),
            crit_chance_percent: self
                .crit_chance_percent
                .saturating_add(other.crit_chance_percent),
            crit_damage_percent: self
                .crit_damage_percent
                .saturating_add(other.crit_damage_percent),
        }
    }
}

/// 데미지 출처 구분
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageSource {
    /// 기본 공격
    BasicAttack,
    /// 어빌리티/스킬
    Ability,
    /// 버프/디버프 틱
    BuffTick,
    /// 환경 효과
    Environment,
}

/// 데미지 저항 계산 타입.
///
/// 모든 피해는 Physical, Magic, True 중 하나로 고정한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    #[serde(alias = "physical", alias = "ad", alias = "Ad")]
    Physical,
    #[serde(alias = "magic", alias = "ap", alias = "Ap")]
    Magic,
    #[serde(alias = "true")]
    True,
}

fn default_damage_type() -> DamageType {
    DamageType::Magic
}

impl Default for DamageType {
    fn default() -> Self {
        default_damage_type()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageBreakdown {
    pub damage_type: DamageType,
    pub raw_damage: u32,
    pub final_damage: u32,
    #[serde(default)]
    pub critical: bool,
}

/// 데미지 계산 결과
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub attacker_id: UnitInstanceId,
    pub target_id: UnitInstanceId,
    pub damage_source: DamageSource,
    pub damage_type: DamageType,
    pub raw_damage: u32,
    pub final_damage: u32,
    pub breakdown: Vec<DamageBreakdown>,
    pub critical: bool,
    pub target_killed: bool,
    pub target_remaining_hp: u32,
    /// 발동된 트리거 이벤트들
    pub triggered_commands: Vec<BattleCommand>,
}

/// 전투 중 발생하는 커맨드 (상태 변경 요청)
#[derive(Debug, Clone)]
pub enum BattleCommand {
    /// 유닛 사망 처리 요청
    UnitDied {
        unit_id: UnitInstanceId,
        killer_id: Option<UnitInstanceId>,
    },
    TriggerAbility {
        skill_id: SkillId,
        caster_id: UnitInstanceId,
        target_id: Option<UnitInstanceId>,
        activation_source: CooldownSource,
        binding_index: usize,
        proc_chance_percent: u8,
        internal_cooldown_ms: u64,
        max_triggers_per_battle: Option<u32>,
        allow_dead_caster: bool,
    },
    /// 스탯 변경 요청
    ApplyModifier {
        target_id: UnitInstanceId,
        modifier: crate::game::stats::StatModifier,
    },
    /// 힐 적용 요청
    ApplyHeal {
        target_id: UnitInstanceId,
        flat: i32,
        percent: i32,
        /// 커맨드를 유발한 주체 (킬 크레딧/트리거용). 없으면 환경/미상.
        source_id: Option<UnitInstanceId>,
    },
    /// 피해 적용 요청
    ApplyDamage {
        source_id: UnitInstanceId,
        target_id: UnitInstanceId,
        amount: u32,
        damage_type: DamageType,
        modifiers: DamageModifiers,
        source: DamageSource,
        minimum_damage: u32,
    },
    /// 공명 변경 요청
    ModifyResonance {
        target_id: UnitInstanceId,
        amount: i32,
        allow_autocast_when_full: bool,
    },
    /// 다음 공격 예약
    ScheduleAttack {
        attacker_id: UnitInstanceId,
        target_id: Option<UnitInstanceId>,
        /// 현재 시각 기준 딜레이(ms)
        time_ms: u64,
    },
    /// 버프 적용 요청
    ApplyBuff {
        caster_id: UnitInstanceId,
        target_id: UnitInstanceId,
        buff_id: BuffId,
        duration_ms: u64,
    },
}

/// 데미지 계산 컨텍스트 - 트리거 수집에 필요한 정보
pub struct DamageContext<'a> {
    pub attacker_side: Side,
    pub target_side: Side,
    pub attacker_attack: u32,
    pub target_armor: i32,
    pub target_magic_resist: i32,
    pub target_current_hp: u32,
    pub target_max_hp: u32,
    pub on_attack_effects: &'a [SourcedEffect],
    pub on_hit_effects: &'a [SourcedEffect],
}

pub fn mitigate_damage(raw_damage: u32, damage_type: DamageType, resistance: i32) -> u32 {
    if raw_damage == 0 || matches!(damage_type, DamageType::True) {
        return raw_damage;
    }

    let raw = i128::from(raw_damage);
    let resisted = if resistance >= 0 {
        let denominator = 100_i128.saturating_add(i128::from(resistance));
        raw.saturating_mul(100) / denominator.max(1)
    } else {
        let denominator = 100_i128.saturating_sub(i128::from(resistance));
        let multiplier_basis = 200_i128.saturating_sub(10_000_i128 / denominator.max(1));
        raw.saturating_mul(multiplier_basis.max(0)) / 100
    };

    resisted.clamp(0, i128::from(u32::MAX)) as u32
}

fn resistance_for_type(
    damage_type: DamageType,
    ctx: &DamageContext,
    modifiers: DamageModifiers,
) -> i32 {
    match damage_type {
        DamageType::Physical => adjusted_resistance(
            ctx.target_armor,
            modifiers.armor_penetration_flat,
            modifiers.armor_penetration_percent,
        ),
        DamageType::Magic => adjusted_resistance(
            ctx.target_magic_resist,
            modifiers.magic_resist_penetration_flat,
            modifiers.magic_resist_penetration_percent,
        ),
        DamageType::True => 0,
    }
}

fn adjusted_resistance(resistance: i32, flat_penetration: i32, percent_penetration: i32) -> i32 {
    let percent_penetration = i128::from(percent_penetration.clamp(0, 100));
    let resistance = i128::from(resistance);
    let after_percent = resistance.saturating_mul(100 - percent_penetration) / 100;
    after_percent
        .saturating_sub(i128::from(flat_penetration))
        .clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
}

fn raw_damage_component(raw_damage: i128) -> u32 {
    raw_damage.clamp(0, u32::MAX as i128) as u32
}

fn modifiers_from_effects(effects: &[SourcedEffect]) -> DamageModifiers {
    effects
        .iter()
        .filter_map(|sourced| match sourced.effect {
            Effect::ModifyDamage(modifiers) => Some(modifiers),
            _ => None,
        })
        .fold(DamageModifiers::default(), DamageModifiers::merge)
}

fn type_damage_amp_percent(damage_type: DamageType, modifiers: DamageModifiers) -> i32 {
    match damage_type {
        DamageType::Physical => modifiers.physical_damage_amp_percent,
        DamageType::Magic => modifiers.magic_damage_amp_percent,
        DamageType::True => modifiers.true_damage_amp_percent,
    }
}

fn type_damage_reduction_percent(damage_type: DamageType, modifiers: DamageModifiers) -> i32 {
    match damage_type {
        DamageType::Physical => modifiers.physical_damage_reduction_percent,
        DamageType::Magic => modifiers.magic_damage_reduction_percent,
        DamageType::True => modifiers.true_damage_reduction_percent,
    }
}

fn is_critical_hit(modifiers: DamageModifiers, roll_percent: Option<u8>) -> bool {
    let chance = modifiers.crit_chance_percent.clamp(0, 100) as u8;
    chance > 0 && roll_percent.is_some_and(|roll| roll < chance)
}

fn apply_critical_multiplier(raw_damage: u32, modifiers: DamageModifiers, critical: bool) -> u32 {
    if raw_damage == 0 || !critical {
        return raw_damage;
    }

    let bonus_percent = i128::from(modifiers.crit_damage_percent.max(-100));
    i128::from(raw_damage)
        .saturating_mul(100 + bonus_percent)
        .saturating_div(100)
        .clamp(0, i128::from(u32::MAX)) as u32
}

fn apply_post_mitigation_modifiers(
    damage: u32,
    damage_type: DamageType,
    modifiers: DamageModifiers,
) -> u32 {
    if damage == 0 {
        return 0;
    }

    let damage = i128::from(damage);
    let amp_percent = i128::from(
        modifiers
            .damage_amp_percent
            .saturating_add(type_damage_amp_percent(damage_type, modifiers))
            .max(-100),
    );
    let amplified = damage.saturating_mul(100 + amp_percent) / 100;
    let reduction_percent = i128::from(
        modifiers
            .damage_reduction_percent
            .saturating_add(type_damage_reduction_percent(damage_type, modifiers))
            .clamp(0, 100),
    );
    let reduced = amplified.saturating_mul(100 - reduction_percent) / 100;
    reduced.clamp(0, i128::from(u32::MAX)) as u32
}

/// 데미지 계산 및 결과 생성
pub fn calculate_damage(request: &DamageRequest, ctx: &DamageContext) -> DamageResult {
    let mut commands = Vec::new();
    let modifiers = request
        .modifiers
        .merge(modifiers_from_effects(ctx.on_attack_effects))
        .merge(modifiers_from_effects(ctx.on_hit_effects));
    let critical = is_critical_hit(modifiers, request.crit_roll_percent);

    let mut raw_components = vec![(
        request.damage_type,
        raw_damage_component(request.base_damage.into()),
    )];
    let mut running_raw_damage = i128::from(request.base_damage);

    // 2. OnAttack 효과 적용
    for sourced in ctx.on_attack_effects {
        match &sourced.effect {
            Effect::BonusDamage {
                flat,
                percent,
                damage_type,
            } => {
                let flat_bonus = i128::from(*flat);
                let percent_basis = running_raw_damage.saturating_add(flat_bonus);
                let bonus_raw =
                    flat_bonus + percent_basis.saturating_mul(i128::from(*percent)) / 100;
                let bonus_raw = raw_damage_component(bonus_raw);
                if bonus_raw > 0 {
                    raw_components.push((*damage_type, bonus_raw));
                    running_raw_damage = running_raw_damage.saturating_add(i128::from(bonus_raw));
                }
            }
            _ => {}
        }
    }

    // 3. OnHit 효과 적용
    for sourced in ctx.on_hit_effects {
        match &sourced.effect {
            Effect::BonusDamage {
                flat,
                percent,
                damage_type,
            } => {
                let flat_bonus = i128::from(*flat);
                let percent_basis = running_raw_damage.saturating_add(flat_bonus);
                let bonus_raw =
                    flat_bonus + percent_basis.saturating_mul(i128::from(*percent)) / 100;
                let bonus_raw = raw_damage_component(bonus_raw);
                if bonus_raw > 0 {
                    raw_components.push((*damage_type, bonus_raw));
                    running_raw_damage = running_raw_damage.saturating_add(i128::from(bonus_raw));
                }
            }
            _ => {}
        }
    }

    // 4. 최종 데미지 계산
    let pre_crit_raw_damage = raw_components
        .iter()
        .fold(0_u32, |acc, (_, raw)| acc.saturating_add(*raw));
    let mut breakdown: Vec<_> = raw_components
        .into_iter()
        .map(|(damage_type, raw_damage)| {
            let raw_damage = apply_critical_multiplier(raw_damage, modifiers, critical);
            DamageBreakdown {
                damage_type,
                raw_damage,
                final_damage: apply_post_mitigation_modifiers(
                    mitigate_damage(
                        raw_damage,
                        damage_type,
                        resistance_for_type(damage_type, ctx, modifiers),
                    ),
                    damage_type,
                    modifiers,
                ),
                critical,
            }
        })
        .collect();
    let raw_damage = breakdown.iter().fold(0_u32, |acc, component| {
        acc.saturating_add(component.raw_damage)
    });
    let mut final_damage = breakdown.iter().fold(0_u32, |acc, component| {
        acc.saturating_add(component.final_damage)
    });
    if pre_crit_raw_damage > 0 && request.minimum_damage > 0 {
        let adjusted = final_damage.max(request.minimum_damage);
        if adjusted > final_damage {
            if let Some(primary) = breakdown.first_mut() {
                primary.final_damage = primary
                    .final_damage
                    .saturating_add(adjusted.saturating_sub(final_damage));
            }
            final_damage = adjusted;
        }
    }
    let target_remaining_hp = ctx.target_current_hp.saturating_sub(final_damage);
    let target_killed = target_remaining_hp == 0;

    // 5. 사망 시 커맨드 추가
    if target_killed {
        commands.push(BattleCommand::UnitDied {
            unit_id: request.target_id,
            killer_id: Some(request.attacker_id),
        });
    }

    DamageResult {
        attacker_id: request.attacker_id,
        target_id: request.target_id,
        damage_source: request.source,
        damage_type: request.damage_type,
        raw_damage,
        final_damage,
        breakdown,
        critical,
        target_killed,
        target_remaining_hp,
        triggered_commands: commands,
    }
}

/// 데미지 결과를 유닛에 적용 (HP 감소만 처리)
pub fn apply_damage_to_unit(stats: &mut crate::game::stats::UnitStats, damage: u32) -> (u32, bool) {
    stats.current_health = stats.current_health.saturating_sub(damage);
    let killed = stats.current_health == 0;
    (stats.current_health, killed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn calculate_damage_clamps_extreme_bonus_damage() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 1,
            minimum_damage: 1,
            time_ms: 0,
        };

        let effects = [SourcedEffect {
            source: CooldownSource::Unit {
                unit_instance_id: attacker_id,
            },
            target: TriggerEffectTarget::SelfUnit,
            effect: Effect::BonusDamage {
                flat: i32::MAX,
                percent: i32::MAX,
                damage_type: DamageType::Physical,
            },
        }];

        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 0,
            target_magic_resist: 0,
            target_current_hp: 1,
            target_max_hp: 1,
            on_attack_effects: &effects,
            on_hit_effects: &[],
        };

        let result = calculate_damage(&request, &ctx);
        assert_eq!(result.final_damage, u32::MAX);
    }

    #[test]
    fn calculate_damage_uses_request_base_damage_instead_of_ctx_attack() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 10,
            minimum_damage: 1,
            time_ms: 0,
        };

        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 3,
            target_magic_resist: 0,
            target_current_hp: 100,
            target_max_hp: 100,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };

        let result = calculate_damage(&request, &ctx);
        assert_eq!(result.final_damage, 9);
    }

    #[test]
    fn calculate_damage_allows_zero_for_non_basic_sources() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Magic,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 0,
            minimum_damage: 0,
            time_ms: 0,
        };

        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 999,
            target_armor: 999,
            target_magic_resist: 999,
            target_current_hp: 100,
            target_max_hp: 100,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };

        let result = calculate_damage(&request, &ctx);
        assert_eq!(result.final_damage, 0);
    }

    #[test]
    fn calculate_damage_only_uses_bonus_damage_from_trigger_effects() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 10,
            minimum_damage: 1,
            time_ms: 0,
        };

        let item_source = CooldownSource::Item {
            item_instance_id: Uuid::from_u128(0xC),
        };
        let on_attack = [SourcedEffect {
            source: item_source,
            target: TriggerEffectTarget::SelfUnit,
            effect: Effect::ApplyBuff {
                buff_id: "poison".to_string(),
                duration_ms: 123,
            },
        }];

        let on_hit = [SourcedEffect {
            source: CooldownSource::Unit {
                unit_instance_id: attacker_id,
            },
            target: TriggerEffectTarget::SelfUnit,
            effect: Effect::BonusDamage {
                flat: 5,
                percent: 0,
                damage_type: DamageType::Physical,
            },
        }];

        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 0,
            target_magic_resist: 0,
            target_current_hp: 100,
            target_max_hp: 100,
            on_attack_effects: &on_attack,
            on_hit_effects: &on_hit,
        };

        let result = calculate_damage(&request, &ctx);
        assert_eq!(result.final_damage, 15);
        assert!(!result.triggered_commands.iter().any(|c| matches!(
            c,
            BattleCommand::ApplyBuff { .. } | BattleCommand::ApplyHeal { .. }
        )));
    }

    #[test]
    fn physical_magic_and_true_damage_use_their_own_mitigation_rules() {
        assert_eq!(mitigate_damage(100, DamageType::Physical, 100), 50);
        assert_eq!(mitigate_damage(100, DamageType::Magic, 25), 80);
        assert_eq!(mitigate_damage(100, DamageType::True, 10_000), 100);
        assert_eq!(mitigate_damage(100, DamageType::Magic, -100), 150);
    }

    #[test]
    fn magic_damage_uses_magic_resist_instead_of_armor() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 9_999,
            target_magic_resist: 100,
            target_current_hp: 100,
            target_max_hp: 100,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };

        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Magic,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 0,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);
        assert_eq!(result.raw_damage, 100);
        assert_eq!(result.final_damage, 50);
        assert_eq!(result.damage_type, DamageType::Magic);
        assert_eq!(result.damage_source, DamageSource::Ability);
    }

    #[test]
    fn typed_bonus_damage_uses_its_own_mitigation() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let bonus = [SourcedEffect {
            source: CooldownSource::Item {
                item_instance_id: Uuid::from_u128(0xC),
            },
            target: TriggerEffectTarget::CounterpartUnit,
            effect: Effect::BonusDamage {
                flat: 100,
                percent: 0,
                damage_type: DamageType::Magic,
            },
        }];
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 100,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &bonus,
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 1,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 150);
        assert_eq!(result.breakdown.len(), 2);
        assert_eq!(result.breakdown[0].damage_type, DamageType::Physical);
        assert_eq!(result.breakdown[0].final_damage, 50);
        assert_eq!(result.breakdown[1].damage_type, DamageType::Magic);
        assert_eq!(result.breakdown[1].final_damage, 100);
    }

    #[test]
    fn trigger_modify_damage_adjusts_current_damage_request() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let on_attack = [SourcedEffect {
            source: CooldownSource::Item {
                item_instance_id: Uuid::from_u128(0xC),
            },
            target: TriggerEffectTarget::SelfUnit,
            effect: Effect::ModifyDamage(DamageModifiers {
                armor_penetration_flat: 100,
                ..Default::default()
            }),
        }];
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 100,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &on_attack,
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 1,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 100);
    }

    #[test]
    fn flat_penetration_adjusts_only_matching_resistance_type() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 100,
            target_magic_resist: 100,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Physical,
            modifiers: DamageModifiers {
                armor_penetration_flat: 100,
                ..Default::default()
            },
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 0,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 100);
    }

    #[test]
    fn post_mitigation_modifiers_apply_after_resistance() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 100,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Physical,
            modifiers: DamageModifiers {
                damage_amp_percent: 50,
                damage_reduction_percent: 20,
                ..Default::default()
            },
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 0,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 60);
    }

    #[test]
    fn negative_resistance_from_signed_stats_increases_damage() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: -100,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 0,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 150);
    }

    #[test]
    fn percent_penetration_applies_before_flat_penetration() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 100,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Physical,
            modifiers: DamageModifiers {
                armor_penetration_percent: 50,
                ..Default::default()
            },
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 0,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 66);
    }

    #[test]
    fn type_specific_amp_and_reduction_only_affect_matching_damage_type() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 0,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Magic,
            modifiers: DamageModifiers {
                physical_damage_amp_percent: 100,
                magic_damage_reduction_percent: 25,
                ..Default::default()
            },
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 0,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 75);
    }

    #[test]
    fn defender_on_hit_modify_damage_can_reduce_incoming_damage() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let on_hit = [SourcedEffect {
            source: CooldownSource::Item {
                item_instance_id: Uuid::from_u128(0xC),
            },
            target: TriggerEffectTarget::SelfUnit,
            effect: Effect::ModifyDamage(DamageModifiers {
                damage_reduction_percent: 50,
                ..Default::default()
            }),
        }];
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 0,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &[],
            on_hit_effects: &on_hit,
        };
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 1,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert_eq!(result.final_damage, 50);
    }

    #[test]
    fn critical_hit_multiplies_raw_damage_before_mitigation() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xA).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB).into();
        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_armor: 100,
            target_magic_resist: 0,
            target_current_hp: 300,
            target_max_hp: 300,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: DamageModifiers {
                crit_chance_percent: 100,
                crit_damage_percent: 50,
                ..Default::default()
            },
            crit_roll_percent: Some(0),
            attacker_id,
            target_id,
            base_damage: 100,
            minimum_damage: 1,
            time_ms: 0,
        };

        let result = calculate_damage(&request, &ctx);

        assert!(result.critical);
        assert_eq!(result.raw_damage, 150);
        assert_eq!(result.final_damage, 75);
        assert_eq!(result.breakdown[0].raw_damage, 150);
        assert!(result.breakdown[0].critical);
    }
}
