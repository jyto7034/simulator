use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::{AbilityEffect, AbilityId, SkillDef, TargetScope},
        enums::Side,
        stats::UnitStats,
    },
};

use super::damage::BattleCommand;

/// 어빌리티 실행 요청
#[derive(Debug, Clone)]
pub struct AbilityRequest {
    pub ability_id: AbilityId,
    pub caster_id: Uuid,
    pub target_id: Option<Uuid>,
    pub time_ms: u64,
}

/// 어빌리티 실행 결과
#[derive(Debug, Clone)]
pub struct AbilityResult {
    pub executed: bool,
    pub commands: Vec<BattleCommand>,
}

/// 어빌리티 실행에 필요한 유닛 정보
#[derive(Debug, Clone)]
pub struct UnitSnapshot {
    pub id: Uuid,
    pub owner: Side,
    pub position: Position,
    pub stats: UnitStats,
}

/// 어빌리티 실행기 - 데이터 드리븐 방식으로 어빌리티 처리
#[derive(Clone)]
pub struct AbilityExecutor {
    /// 어빌리티 정의 테이블
    skill_defs: HashMap<AbilityId, SkillDef>,
    /// 쿨다운 테이블 (unit_id, ability_id) -> next_ready_time_ms
    cooldowns: HashMap<(Uuid, AbilityId), u64>,
}

impl AbilityExecutor {
    pub fn new() -> Self {
        Self {
            skill_defs: Self::build_default_skill_defs(),
            cooldowns: HashMap::new(),
        }
    }

    /// 기본 스킬 정의 빌드
    fn build_default_skill_defs() -> HashMap<AbilityId, SkillDef> {
        use AbilityEffect::*;
        use AbilityId::*;
        use TargetScope::*;

        let mut defs = HashMap::new();

        // Scorched Girl 폭발 - 주변 적에게 데미지
        defs.insert(
            ScorchedExplosion,
            SkillDef {
                id: ScorchedExplosion,
                trigger: crate::game::ability::AbilityTrigger::OnHit,
                cooldown_ms: 0,
                effects: vec![DirectDamage {
                    amount: 30,
                    target: EnemyAll, // TODO: EnemyNearby(1) 같은 범위 지정 필요
                }],
            },
        );

        // Plague Doctor 광역 힐
        defs.insert(
            PlagueMassHeal,
            SkillDef {
                id: PlagueMassHeal,
                trigger: crate::game::ability::AbilityTrigger::OnAttack,
                cooldown_ms: 5000,
                effects: vec![Heal {
                    amount: 20,
                    target: AllyAll,
                }],
            },
        );

        // Red Shoes 광폭화
        defs.insert(
            RedShoesBerserk,
            SkillDef {
                id: RedShoesBerserk,
                trigger: crate::game::ability::AbilityTrigger::OnAttack,
                cooldown_ms: 0,
                effects: vec![DirectDamage {
                    amount: 15,
                    target: EnemySingle,
                }],
            },
        );

        // Fragment of Universe 전체 공격
        defs.insert(
            FragmentOfUniverseNova,
            SkillDef {
                id: FragmentOfUniverseNova,
                trigger: crate::game::ability::AbilityTrigger::OnAttack,
                cooldown_ms: 10000,
                effects: vec![DirectDamage {
                    amount: 25,
                    target: EnemyAll,
                }],
            },
        );

        // Fairy Festival 버프
        defs.insert(
            FairyFestivalBlessing,
            SkillDef {
                id: FairyFestivalBlessing,
                trigger: crate::game::ability::AbilityTrigger::OnBattleStart,
                cooldown_ms: 0,
                effects: vec![StatModifiers {
                    modifiers: vec![crate::game::stats::StatModifier {
                        stat: crate::game::stats::StatId::Attack,
                        kind: crate::game::stats::StatModifierKind::Flat,
                        value: 5,
                    }],
                    target: AllyAll,
                }],
            },
        );

        // Unknown Distortion 강타
        defs.insert(
            UnknownDistortionStrike,
            SkillDef {
                id: UnknownDistortionStrike,
                trigger: crate::game::ability::AbilityTrigger::OnAttack,
                cooldown_ms: 8000,
                effects: vec![DirectDamage {
                    amount: 50,
                    target: EnemySingle,
                }],
            },
        );

        defs
    }

    /// 어빌리티 실행
    pub fn execute(
        &mut self,
        request: &AbilityRequest,
        caster: &UnitSnapshot,
        units: &[UnitSnapshot],
    ) -> AbilityResult {
        let mut commands = Vec::new();

        // 스킬 정의 조회
        let Some(skill_def) = self.skill_defs.get(&request.ability_id) else {
            // 정의가 없으면 커스텀 로직으로 폴백
            return self.execute_custom(request, caster, units);
        };

        // 쿨다운 체크
        if skill_def.cooldown_ms > 0 {
            let cooldown_key = (request.caster_id, request.ability_id);
            if let Some(&ready_time) = self.cooldowns.get(&cooldown_key) {
                if request.time_ms < ready_time {
                    return AbilityResult {
                        executed: false,
                        commands,
                    };
                }
            }
            // 쿨다운 갱신
            self.cooldowns
                .insert(cooldown_key, request.time_ms + skill_def.cooldown_ms);
        }

        // 각 효과 처리
        for effect in &skill_def.effects {
            let effect_commands = self.process_effect(effect, caster, request.target_id, units);
            commands.extend(effect_commands);
        }

        AbilityResult {
            executed: true,
            commands,
        }
    }

    /// 개별 효과 처리
    fn process_effect(
        &self,
        effect: &AbilityEffect,
        caster: &UnitSnapshot,
        hinted_target: Option<Uuid>,
        units: &[UnitSnapshot],
    ) -> Vec<BattleCommand> {
        let mut commands = Vec::new();

        match effect {
            AbilityEffect::DirectDamage { amount, target } => {
                let targets = self.resolve_targets(*target, caster, hinted_target, units);
                for target_id in targets {
                    commands.push(BattleCommand::ApplyHeal {
                        target_id,
                        flat: -(*amount), // 음수 힐 = 데미지
                        percent: 0,
                        source_id: Some(caster.id),
                    });
                }
            }
            AbilityEffect::Heal { amount, target } => {
                let targets = self.resolve_targets(*target, caster, hinted_target, units);
                for target_id in targets {
                    commands.push(BattleCommand::ApplyHeal {
                        target_id,
                        flat: *amount,
                        percent: 0,
                        source_id: Some(caster.id),
                    });
                }
            }
            AbilityEffect::ExtraAttack { count, target } => {
                let targets = self.resolve_targets(*target, caster, hinted_target, units);
                for _ in 0..*count {
                    for &target_id in &targets {
                        // 추가 공격은 1회성 Attack 이벤트로 스케줄
                        commands.push(BattleCommand::ScheduleAttack {
                            attacker_id: caster.id,
                            target_id: Some(target_id),
                            time_ms: 0, // 즉시 실행
                        });
                    }
                }
            }
            AbilityEffect::StatModifiers { modifiers, target } => {
                let targets = self.resolve_targets(*target, caster, hinted_target, units);
                for target_id in targets {
                    for modifier in modifiers {
                        commands.push(BattleCommand::ApplyModifier {
                            target_id,
                            modifier: *modifier,
                        });
                    }
                }
            }
        }

        commands
    }

    /// 타겟 해석
    fn resolve_targets(
        &self,
        scope: TargetScope,
        caster: &UnitSnapshot,
        hinted_target: Option<Uuid>,
        units: &[UnitSnapshot],
    ) -> Vec<Uuid> {
        match scope {
            TargetScope::SelfUnit => vec![caster.id],
            TargetScope::AllySingle => {
                // 힌트된 타겟이 아군이면 사용, 아니면 자기 자신
                if let Some(target_id) = hinted_target {
                    if units
                        .iter()
                        .any(|u| u.id == target_id && u.owner == caster.owner)
                    {
                        return vec![target_id];
                    }
                }
                vec![caster.id]
            }
            TargetScope::AllyAll => units
                .iter()
                .filter(|u| u.owner == caster.owner)
                .map(|u| u.id)
                .collect(),
            TargetScope::EnemySingle => {
                // 힌트된 타겟이 적이면 사용
                if let Some(target_id) = hinted_target {
                    if units
                        .iter()
                        .any(|u| u.id == target_id && u.owner != caster.owner)
                    {
                        return vec![target_id];
                    }
                }
                // 아니면 적 중에서 가장 작은 ID (결정성 보장)
                units
                    .iter()
                    .filter(|u| u.owner != caster.owner)
                    .min_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()))
                    .map(|u| vec![u.id])
                    .unwrap_or_default()
            }
            TargetScope::EnemyAll => units
                .iter()
                .filter(|u| u.owner != caster.owner)
                .map(|u| u.id)
                .collect(),
        }
    }

    /// 데이터 드리븐으로 처리 불가능한 커스텀 어빌리티
    fn execute_custom(
        &self,
        request: &AbilityRequest,
        _caster: &UnitSnapshot,
        _units: &[UnitSnapshot],
    ) -> AbilityResult {
        let commands = Vec::new();

        match request.ability_id {
            AbilityId::SpiderBudPoisonStack => {
                // TODO: 독 스택 시스템 구현 필요
                // 버프 시스템이 완성되면 ApplyBuff 커맨드로 전환
            }
            _ => {}
        }

        AbilityResult {
            executed: true,
            commands,
        }
    }

    /// 쿨다운 초기화
    pub fn reset_cooldowns(&mut self) {
        self.cooldowns.clear();
    }
}

impl Default for AbilityExecutor {
    fn default() -> Self {
        Self::new()
    }
}
