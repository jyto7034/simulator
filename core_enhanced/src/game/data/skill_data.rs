use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::game::{
    ability::{DeliveryDef, SkillDef, SkillKind, SkillTarget},
    data::{build_string_index, once_lock_with},
};

fn validate_registered_buff(skill_id: &str, context: &str, buff_id: &str) {
    assert!(
        crate::game::battle::buffs::contains_name(buff_id),
        "skill '{}' references unknown buff '{}' in {}",
        skill_id,
        buff_id,
        context
    );
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDatabase {
    pub skills: Vec<SkillDef>,

    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl SkillDatabase {
    pub fn new(skills: Vec<SkillDef>) -> Self {
        validate_skill_contracts(&skills);
        let by_id = once_lock_with(build_string_index(&skills, "skill id", |skill| &skill.id));

        Self { skills, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.skills, "skill id", |skill| &skill.id))
    }

    pub(crate) fn validate_indexes(&self) {
        validate_skill_contracts(&self.skills);
        let _ = self.by_id();
    }

    pub fn get_by_id(&self, id: impl AsRef<str>) -> Option<&SkillDef> {
        self.by_id()
            .get(id.as_ref())
            .and_then(|&index| self.skills.get(index))
    }
}

fn validate_skill_contracts(skills: &[SkillDef]) {
    for skill in skills {
        assert!(
            !skill.steps.is_empty(),
            "skill '{}' must define at least one step",
            skill.id
        );
        let mut step_ids = HashSet::new();

        for step in &skill.steps {
            assert!(
                step_ids.insert(step.id.clone()),
                "skill '{}' contains duplicate step id '{}'",
                skill.id,
                step.id
            );

            match &step.when {
                crate::game::ability::SkillStepCondition::Always
                | crate::game::ability::SkillStepCondition::IfPreviousStepDealtDamage => {}
                crate::game::ability::SkillStepCondition::IfCasterHasBuff { buff_id, .. } => {
                    validate_registered_buff(&skill.id, "step condition", buff_id);
                }
            }

            if let crate::game::ability::SkillStepRepeat::ByBuffStacks { buff_id, .. } =
                &step.repeat
            {
                validate_registered_buff(&skill.id, "step repeat", buff_id);
            }

            for effect in &step.effects {
                match effect {
                    crate::game::ability::SkillEffectDef::ApplyBuff { buff_id, .. } => {
                        validate_registered_buff(&skill.id, "step effect", buff_id);
                    }
                    crate::game::ability::SkillEffectDef::Damage { amount, .. } => {
                        assert!(
                            *amount >= 0,
                            "skill '{}' step '{}' has negative Damage amount {}",
                            skill.id,
                            step.id,
                            amount
                        );
                    }
                    _ => {}
                }
            }

            match &step.delivery {
                DeliveryDef::Projectile { collision, .. } => {
                    collision.validate_runtime_contract();
                    if matches!(skill.kind, SkillKind::Targeted)
                        && matches!(step.target, SkillTarget::EnemySingle { .. })
                    {
                        collision.validate_homing_runtime_contract();
                    }
                }
                DeliveryDef::Area { area } => area.validate_runtime_contract(),
                DeliveryDef::Instant => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ability::{
        DeliveryDef, SkillCastTargetingDef, SkillEffectDef, SkillId, SkillProjectileCollisionDef,
        SkillStepDef, UnitTargetRule,
    };
    use crate::game::battle::damage::DamageType;

    fn projectile_step(collision: SkillProjectileCollisionDef) -> SkillStepDef {
        SkillStepDef {
            id: "shot".to_string(),
            delay_ms: 0,
            range_units: 3.0,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1_000,
                collision,
            },
            effects: vec![],
            presentation: Default::default(),
        }
    }

    #[test]
    fn skill_database_rejects_empty_step_skills() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("empty"),
                name: "empty".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: SkillCastTargetingDef::FirstStepTarget,
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![],
            }]);
        });

        assert!(result.is_err(), "empty-step skills must be rejected");
    }

    #[test]
    fn skill_database_rejects_nondefault_homing_collision_contracts() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("homing"),
                name: "homing".to_string(),
                kind: SkillKind::Targeted,
                cast_targeting: SkillCastTargetingDef::FirstStepTarget,
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![projectile_step(SkillProjectileCollisionDef {
                    piercing: true,
                    ..Default::default()
                })],
            }]);
        });

        assert!(
            result.is_err(),
            "targeted homing projectile should reject unsupported collision modifiers"
        );
    }

    #[test]
    fn skill_database_rejects_duplicate_step_ids() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("duplicate_steps"),
                name: "duplicate_steps".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: SkillCastTargetingDef::FirstStepTarget,
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![
                    projectile_step(Default::default()),
                    projectile_step(Default::default()),
                ],
            }]);
        });

        assert!(result.is_err(), "duplicate step ids must be rejected");
    }

    #[test]
    fn skill_database_rejects_unknown_buff_references() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("unknown_buff"),
                name: "unknown_buff".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: SkillCastTargetingDef::FirstStepTarget,
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "step".to_string(),
                    delay_ms: 0,
                    range_units: 1.0,
                    target: SkillTarget::SelfUnit,
                    targeting: Default::default(),
                    when: crate::game::ability::SkillStepCondition::IfCasterHasBuff {
                        buff_id: "does_not_exist".to_string(),
                        min_stacks: 1,
                    },
                    repeat: crate::game::ability::SkillStepRepeat::ByBuffStacks {
                        unit: crate::game::ability::SkillUnitReference::SelfUnit,
                        buff_id: "does_not_exist".to_string(),
                        max: None,
                    },
                    delivery: DeliveryDef::Instant,
                    effects: vec![crate::game::ability::SkillEffectDef::ApplyBuff {
                        buff_id: "does_not_exist".to_string(),
                        duration_ms: 100,
                    }],
                    presentation: Default::default(),
                }],
            }]);
        });

        assert!(result.is_err(), "unknown buff references must be rejected");
    }

    #[test]
    fn skill_database_rejects_negative_damage_amounts() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("negative_damage"),
                name: "negative_damage".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: SkillCastTargetingDef::FirstStepTarget,
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "hit".to_string(),
                    delay_ms: 0,
                    range_units: 1.0,
                    target: SkillTarget::EnemySingle {
                        rule: UnitTargetRule::Nearest,
                    },
                    targeting: Default::default(),
                    when: Default::default(),
                    repeat: Default::default(),
                    delivery: DeliveryDef::Instant,
                    effects: vec![SkillEffectDef::Damage {
                        amount: -1,
                        damage_type: DamageType::Magic,
                    }],
                    presentation: Default::default(),
                }],
            }]);
        });

        assert!(result.is_err(), "negative Damage amounts must be rejected");
    }
}
