use uuid::Uuid;

use crate::game::{enums::Side, stats::Effect};

use super::buffs::BuffId;

/// 데미지 요청 - 데미지 계산에 필요한 모든 정보
#[derive(Debug, Clone)]
pub struct DamageRequest {
    pub source: DamageSource,
    pub attacker_id: Uuid,
    pub target_id: Uuid,
    pub base_damage: u32,
    pub time_ms: u64,
}

/// 데미지 출처 구분
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// 데미지 계산 결과
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub attacker_id: Uuid,
    pub target_id: Uuid,
    pub final_damage: u32,
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
        unit_id: Uuid,
        killer_id: Option<Uuid>,
    },
    /// 어빌리티 실행 요청
    ExecuteAbility {
        ability_id: crate::game::ability::AbilityId,
        caster_id: Uuid,
        target_id: Option<Uuid>,
    },
    /// 스탯 변경 요청
    ApplyModifier {
        target_id: Uuid,
        modifier: crate::game::stats::StatModifier,
    },
    /// 힐 적용 요청
    ApplyHeal {
        target_id: Uuid,
        flat: i32,
        percent: i32,
        /// 커맨드를 유발한 주체 (킬 크레딧/트리거용). 없으면 환경/미상.
        source_id: Option<Uuid>,
    },
    /// 다음 공격 예약
    ScheduleAttack {
        attacker_id: Uuid,
        target_id: Option<Uuid>,
        /// 현재 시각 기준 딜레이(ms)
        time_ms: u64,
    },
    /// 버프 적용 요청
    ApplyBuff {
        caster_id: Uuid,
        target_id: Uuid,
        buff_id: BuffId,
        duration_ms: u64,
    },
}

/// 데미지 계산 컨텍스트 - 트리거 수집에 필요한 정보
pub struct DamageContext<'a> {
    pub attacker_side: Side,
    pub target_side: Side,
    pub attacker_attack: u32,
    pub target_defense: u32,
    pub target_current_hp: u32,
    pub target_max_hp: u32,
    pub on_attack_effects: &'a [Effect],
    pub on_hit_effects: &'a [Effect],
}

/// 데미지 계산 및 결과 생성
pub fn calculate_damage(request: &DamageRequest, ctx: &DamageContext) -> DamageResult {
    let mut commands = Vec::new();

    // 1. 기본 데미지 계산 (attack - defense, 최소 1)
    let base_damage = ctx
        .attacker_attack
        .saturating_sub(ctx.target_defense)
        .max(1);
    let mut damage: i128 = i128::from(base_damage);

    // 2. OnAttack 효과 적용
    for effect in ctx.on_attack_effects {
        match effect {
            Effect::BonusDamage { flat, percent } => {
                damage += i128::from(*flat);
                damage += damage.saturating_mul(i128::from(*percent)) / 100;
            }
            Effect::ApplyBuff {
                buff_id,
                duration_ms,
            } => {
                commands.push(BattleCommand::ApplyBuff {
                    caster_id: request.attacker_id,
                    target_id: request.target_id,
                    buff_id: BuffId::from_name(buff_id.as_str()),
                    duration_ms: *duration_ms,
                });
            }
            Effect::Ability(ability_id) => {
                commands.push(BattleCommand::ExecuteAbility {
                    ability_id: *ability_id,
                    caster_id: request.attacker_id,
                    target_id: Some(request.target_id),
                });
            }
            _ => {}
        }
    }

    // 3. OnHit 효과 적용
    for effect in ctx.on_hit_effects {
        match effect {
            Effect::BonusDamage { flat, percent } => {
                damage += i128::from(*flat);
                damage += damage.saturating_mul(i128::from(*percent)) / 100;
            }
            Effect::Heal { flat, percent } => {
                commands.push(BattleCommand::ApplyHeal {
                    target_id: request.target_id,
                    flat: *flat,
                    percent: *percent,
                    source_id: Some(request.target_id),
                });
            }
            Effect::ApplyBuff {
                buff_id,
                duration_ms,
            } => {
                commands.push(BattleCommand::ApplyBuff {
                    caster_id: request.target_id,
                    target_id: request.attacker_id,
                    buff_id: BuffId::from_name(buff_id.as_str()),
                    duration_ms: *duration_ms,
                });
            }
            Effect::Ability(ability_id) => {
                commands.push(BattleCommand::ExecuteAbility {
                    ability_id: *ability_id,
                    caster_id: request.target_id,
                    target_id: Some(request.attacker_id),
                });
            }
            _ => {}
        }
    }

    // 4. 최종 데미지 계산 (최소 0)
    let final_damage = damage.clamp(0, u32::MAX as i128) as u32;
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
        final_damage,
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

    #[test]
    fn calculate_damage_clamps_extreme_bonus_damage() {
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            attacker_id: Uuid::from_u128(0xA),
            target_id: Uuid::from_u128(0xB),
            base_damage: 1,
            time_ms: 0,
        };

        let effects = [Effect::BonusDamage {
            flat: i32::MAX,
            percent: i32::MAX,
        }];

        let ctx = DamageContext {
            attacker_side: Side::Player,
            target_side: Side::Opponent,
            attacker_attack: 1,
            target_defense: 0,
            target_current_hp: 1,
            target_max_hp: 1,
            on_attack_effects: &effects,
            on_hit_effects: &[],
        };

        let result = calculate_damage(&request, &ctx);
        assert_eq!(result.final_damage, u32::MAX);
    }
}
