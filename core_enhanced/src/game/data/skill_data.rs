use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Deserializer, Serialize};

use crate::game::{
    ability::{
        DeliveryDef, FocusPermissions, SkillAreaAnchorSource, SkillCastTargetingDef, SkillDef,
        SkillEffectDef, SkillKind, SkillPresentationDef, SkillStepCondition, SkillStepDef,
        SkillStepRepeat, SkillTarget, StepTargetingMode,
    },
    battle::buffs::BuffDatabase,
    battle::tile_range::TileRangePattern,
    data::{build_string_index, once_lock_with},
};

fn validate_registered_buff(
    buff_data: &BuffDatabase,
    skill_id: &str,
    context: &str,
    buff_id: &str,
) {
    assert!(
        buff_data.contains_name(buff_id),
        "skill '{}' references unknown buff '{}' in {}",
        skill_id,
        buff_id,
        context
    );
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRangePresetDef {
    pub id: String,
    pub range: TileRangePattern,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillDatabase {
    #[serde(default)]
    pub range_presets: Vec<SkillRangePresetDef>,
    pub skills: Vec<SkillDef>,

    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl SkillDatabase {
    pub fn new(skills: Vec<SkillDef>) -> Self {
        validate_skill_contracts(&skills);
        let by_id = once_lock_with(build_string_index(&skills, "skill id", |skill| &skill.id));

        Self {
            range_presets: vec![],
            skills,
            by_id,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.skills, "skill id", |skill| &skill.id))
    }

    pub(crate) fn validate_indexes(&self) {
        validate_skill_contracts(&self.skills);
        let _ = self.by_id();
    }

    pub(crate) fn validate_buff_references(&self, buff_data: &BuffDatabase) {
        validate_skill_buff_references(&self.skills, buff_data);
    }

    pub fn get_by_id(&self, id: impl AsRef<str>) -> Option<&SkillDef> {
        self.by_id()
            .get(id.as_ref())
            .and_then(|&index| self.skills.get(index))
    }
}

impl<'de> Deserialize<'de> for SkillDatabase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawSkillDatabase::deserialize(deserializer)?;
        Ok(raw.into_database())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "SkillDatabase")]
struct RawSkillDatabase {
    #[serde(default)]
    range_presets: Vec<SkillRangePresetDef>,
    skills: Vec<RawSkillDef>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "SkillDef")]
struct RawSkillDef {
    id: crate::game::ability::SkillId,
    #[serde(default = "raw_default_skill_name")]
    name: String,
    #[serde(default)]
    kind: SkillKind,
    #[serde(default)]
    cast_targeting: RawSkillCastTargetingDef,
    #[serde(default)]
    focus_time_ms: u32,
    #[serde(default)]
    focus_permissions: FocusPermissions,
    #[serde(default)]
    steps: Vec<RawSkillStepDef>,
}

#[derive(Debug, Deserialize, Default)]
enum RawSkillCastTargetingDef {
    #[default]
    FirstStepTarget,
    Explicit {
        #[serde(default = "raw_default_step_range_units")]
        range_units: f32,
        target: SkillTarget,
        #[serde(default)]
        defense_tile_range: Option<TileRangePattern>,
        #[serde(default)]
        defense_tile_range_preset: Option<String>,
        #[serde(default)]
        air_capable: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename = "SkillStepDef")]
struct RawSkillStepDef {
    #[serde(default = "raw_default_step_id")]
    id: String,
    #[serde(default)]
    delay_ms: u32,
    #[serde(default = "raw_default_step_range_units")]
    range_units: f32,
    #[serde(default)]
    defense_tile_range: Option<TileRangePattern>,
    #[serde(default)]
    defense_tile_range_preset: Option<String>,
    #[serde(default)]
    air_capable: bool,
    target: SkillTarget,
    #[serde(default)]
    targeting: StepTargetingMode,
    #[serde(default)]
    when: SkillStepCondition,
    #[serde(default)]
    repeat: SkillStepRepeat,
    #[serde(default)]
    delivery: DeliveryDef,
    #[serde(default)]
    effects: Vec<SkillEffectDef>,
    #[serde(default)]
    presentation: SkillPresentationDef,
}

fn raw_default_skill_name() -> String {
    "Unnamed Skill".to_string()
}

fn raw_default_step_id() -> String {
    "step".to_string()
}

fn raw_default_step_range_units() -> f32 {
    1.0
}

impl RawSkillDatabase {
    fn into_database(self) -> SkillDatabase {
        let preset_index =
            build_string_index(&self.range_presets, "skill range preset id", |preset| {
                &preset.id
            });
        for preset in &self.range_presets {
            preset.range.validate().unwrap_or_else(|error| {
                panic!(
                    "skill range preset '{}' has invalid defense_tile_range: {}",
                    preset.id, error
                )
            });
        }

        let skills = self
            .skills
            .into_iter()
            .map(|skill| skill.into_skill_def(&self.range_presets, &preset_index))
            .collect::<Vec<_>>();
        validate_skill_contracts(&skills);
        let by_id = once_lock_with(build_string_index(&skills, "skill id", |skill| &skill.id));
        SkillDatabase {
            range_presets: self.range_presets,
            skills,
            by_id,
        }
    }
}

impl RawSkillDef {
    fn into_skill_def(
        self,
        presets: &[SkillRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> SkillDef {
        SkillDef {
            id: self.id,
            name: self.name,
            kind: self.kind,
            cast_targeting: self
                .cast_targeting
                .into_cast_targeting(presets, preset_index),
            focus_time_ms: self.focus_time_ms,
            focus_permissions: self.focus_permissions,
            steps: self
                .steps
                .into_iter()
                .map(|step| step.into_step(presets, preset_index))
                .collect(),
        }
    }
}

impl RawSkillCastTargetingDef {
    fn into_cast_targeting(
        self,
        presets: &[SkillRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> SkillCastTargetingDef {
        match self {
            RawSkillCastTargetingDef::FirstStepTarget => SkillCastTargetingDef::FirstStepTarget,
            RawSkillCastTargetingDef::Explicit {
                range_units,
                target,
                defense_tile_range,
                defense_tile_range_preset,
                air_capable,
            } => SkillCastTargetingDef::Explicit {
                range_units,
                target,
                defense_tile_range: resolve_skill_range_source(
                    "explicit cast_targeting",
                    defense_tile_range,
                    defense_tile_range_preset,
                    presets,
                    preset_index,
                ),
                air_capable,
            },
        }
    }
}

impl RawSkillStepDef {
    fn into_step(
        self,
        presets: &[SkillRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> SkillStepDef {
        let step_id = self.id;
        SkillStepDef {
            id: step_id.clone(),
            delay_ms: self.delay_ms,
            range_units: self.range_units,
            defense_tile_range: resolve_skill_range_source(
                &format!("step '{step_id}'"),
                self.defense_tile_range,
                self.defense_tile_range_preset,
                presets,
                preset_index,
            ),
            air_capable: self.air_capable,
            target: self.target,
            targeting: self.targeting,
            when: self.when,
            repeat: self.repeat,
            delivery: self.delivery,
            effects: self.effects,
            presentation: self.presentation,
        }
    }
}

fn resolve_skill_range_source(
    context: &str,
    inline_range: Option<TileRangePattern>,
    preset_id: Option<String>,
    presets: &[SkillRangePresetDef],
    preset_index: &HashMap<String, usize>,
) -> Option<TileRangePattern> {
    match (inline_range, preset_id) {
        (Some(_), Some(preset_id)) => {
            panic!("{context} defines both defense_tile_range and defense_tile_range_preset '{preset_id}'")
        }
        (Some(range), None) => Some(range),
        (None, Some(preset_id)) => {
            let Some(index) = preset_index.get(&preset_id) else {
                panic!("{context} references unknown defense_tile_range_preset '{preset_id}'");
            };
            Some(presets[*index].range.clone())
        }
        (None, None) => None,
    }
}

fn validate_skill_buff_references(skills: &[SkillDef], buff_data: &BuffDatabase) {
    for skill in skills {
        for step in &skill.steps {
            match &step.when {
                crate::game::ability::SkillStepCondition::Always
                | crate::game::ability::SkillStepCondition::IfPreviousStepDealtDamage => {}
                crate::game::ability::SkillStepCondition::IfCasterHasBuff { buff_id, .. } => {
                    validate_registered_buff(buff_data, &skill.id, "step condition", buff_id);
                }
            }

            if let crate::game::ability::SkillStepRepeat::ByBuffStacks { buff_id, .. } =
                &step.repeat
            {
                validate_registered_buff(buff_data, &skill.id, "step repeat", buff_id);
            }

            for effect in &step.effects {
                if let crate::game::ability::SkillEffectDef::ApplyBuff { buff_id, .. } = effect {
                    validate_registered_buff(buff_data, &skill.id, "step effect", buff_id);
                }
            }
        }
    }
}

fn validate_skill_contracts(skills: &[SkillDef]) {
    for skill in skills {
        assert!(
            !skill.steps.is_empty(),
            "skill '{}' must define at least one step",
            skill.id
        );
        if let crate::game::ability::SkillCastTargetingDef::Explicit {
            defense_tile_range: Some(pattern),
            ..
        } = &skill.cast_targeting
        {
            pattern.validate().unwrap_or_else(|error| {
                panic!(
                    "skill '{}' explicit cast_targeting has invalid defense_tile_range: {}",
                    skill.id, error
                )
            });
        }
        let mut step_ids = HashSet::new();

        for (step_index, step) in skill.steps.iter().enumerate() {
            if let Some(pattern) = &step.defense_tile_range {
                pattern.validate().unwrap_or_else(|error| {
                    panic!(
                        "skill '{}' step '{}' has invalid defense_tile_range: {}",
                        skill.id, step.id, error
                    )
                });
            }
            assert!(
                step_ids.insert(step.id.clone()),
                "skill '{}' contains duplicate step id '{}'",
                skill.id,
                step.id
            );

            let mut has_damage_effect = false;
            let mut has_modify_damage_effect = false;
            for effect in &step.effects {
                match effect {
                    crate::game::ability::SkillEffectDef::ApplyBuff { .. } => {}
                    crate::game::ability::SkillEffectDef::Damage { amount, .. } => {
                        has_damage_effect = true;
                        assert!(
                            *amount >= 0,
                            "skill '{}' step '{}' has negative Damage amount {}",
                            skill.id,
                            step.id,
                            amount
                        );
                    }
                    crate::game::ability::SkillEffectDef::ModifyDamage { .. } => {
                        has_modify_damage_effect = true;
                    }
                    _ => {}
                }
            }
            assert!(
                !has_modify_damage_effect || has_damage_effect,
                "skill '{}' step '{}' has ModifyDamage but no Damage effect",
                skill.id,
                step.id
            );

            match &step.delivery {
                DeliveryDef::Projectile { collision, .. } => {
                    collision.validate_runtime_contract();
                    if matches!(skill.kind, SkillKind::Targeted)
                        && matches!(step.target, SkillTarget::EnemySingle { .. })
                    {
                        collision.validate_homing_runtime_contract();
                    }
                }
                DeliveryDef::TileArea { area } => {
                    area.validate_runtime_contract();
                    assert!(
                        step.defense_tile_range.is_some(),
                        "skill '{}' step '{}' uses TileArea but is missing defense_tile_range",
                        skill.id,
                        step.id
                    );
                    if matches!(
                        area.anchor,
                        SkillAreaAnchorSource::ImpactContext
                            | SkillAreaAnchorSource::ImpactContextStart
                    ) {
                        let has_prior_spatial_delivery =
                            skill.steps[..step_index].iter().any(|prior_step| {
                                matches!(
                                    prior_step.delivery,
                                    DeliveryDef::Projectile { .. } | DeliveryDef::TileArea { .. }
                                )
                            });
                        assert!(
                            has_prior_spatial_delivery,
                            "skill '{}' step '{}' uses {:?} anchor but no prior spatial delivery step can provide impact context",
                            skill.id,
                            step.id,
                            area.anchor
                        );
                    }
                }
                DeliveryDef::Instant => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ability::{
        DeliveryDef, SkillAreaAnchorSource, SkillCastTargetingDef, SkillEffectDef, SkillId,
        SkillProjectileCollisionDef, SkillStepDef, SkillTileAreaDeliveryDef, UnitTargetRule,
    };
    use crate::game::battle::buffs::BuffDatabase;
    use crate::game::battle::damage::{DamageModifiers, DamageType};
    use crate::game::battle::tile_range::TileRangePattern;

    fn projectile_step(collision: SkillProjectileCollisionDef) -> SkillStepDef {
        SkillStepDef {
            id: "shot".to_string(),
            delay_ms: 0,
            range_units: 3.0,
            defense_tile_range: None,
            air_capable: false,
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

    fn tile_area_step(id: &str, anchor: SkillAreaAnchorSource) -> SkillStepDef {
        SkillStepDef {
            id: id.to_string(),
            delay_ms: 0,
            range_units: 3.0,
            defense_tile_range: Some(TileRangePattern {
                include_anchor_tile: true,
                rows: vec![".@.".to_string()],
            }),
            air_capable: false,
            target: SkillTarget::CastTarget,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::TileArea {
                area: SkillTileAreaDeliveryDef {
                    anchor,
                    ..Default::default()
                },
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
        let skill = SkillDef {
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
                defense_tile_range: None,
                air_capable: false,
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
        };
        let database = SkillDatabase::new(vec![skill]);
        let result = std::panic::catch_unwind(|| {
            database.validate_buff_references(&BuffDatabase::new(vec![]));
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
                    defense_tile_range: None,
                    air_capable: false,
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

    #[test]
    fn skill_database_rejects_modify_damage_without_damage_effect() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("dangling_modify_damage"),
                name: "dangling_modify_damage".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: SkillCastTargetingDef::FirstStepTarget,
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "modifier_only".to_string(),
                    delay_ms: 0,
                    range_units: 1.0,
                    defense_tile_range: None,
                    air_capable: false,
                    target: SkillTarget::EnemySingle {
                        rule: UnitTargetRule::Nearest,
                    },
                    targeting: Default::default(),
                    when: Default::default(),
                    repeat: Default::default(),
                    delivery: DeliveryDef::Instant,
                    effects: vec![SkillEffectDef::ModifyDamage {
                        modifiers: DamageModifiers {
                            damage_amp_percent: 25,
                            ..Default::default()
                        },
                    }],
                    presentation: Default::default(),
                }],
            }]);
        });

        assert!(
            result.is_err(),
            "ModifyDamage without Damage is a no-op and must be rejected"
        );
    }

    #[test]
    fn skill_database_rejects_impact_context_anchor_without_prior_spatial_delivery() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("dangling_impact_context"),
                name: "dangling_impact_context".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: SkillCastTargetingDef::FirstStepTarget,
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![tile_area_step("area", SkillAreaAnchorSource::ImpactContext)],
            }]);
        });

        assert!(
            result.is_err(),
            "ImpactContext tile area must require a prior spatial delivery"
        );
    }

    #[test]
    fn skill_database_resolves_range_preset_from_ron() {
        let database: SkillDatabase = ron::de::from_str(
            r#"
            SkillDatabase(
                range_presets: [
                    SkillRangePresetDef(
                        id: "front_1",
                        range: (include_anchor_tile: false, rows: [".@X"]),
                    ),
                ],
                skills: [
                    SkillDef(
                        id: "preset_skill",
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                defense_tile_range_preset: Some("front_1"),
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        let skill = database.get_by_id("preset_skill").unwrap();
        let range = skill.steps[0].defense_tile_range.as_ref().unwrap();
        assert_eq!(range.rows, vec![".@X"]);
        assert_eq!(database.range_presets.len(), 1);
    }

    #[test]
    fn skill_database_rejects_inline_range_and_preset_conflict() {
        let result = std::panic::catch_unwind(|| {
            let _: SkillDatabase = ron::de::from_str(
                r#"
                SkillDatabase(
                    range_presets: [
                        SkillRangePresetDef(
                            id: "front_1",
                            range: (include_anchor_tile: false, rows: [".@X"]),
                        ),
                    ],
                    skills: [
                        SkillDef(
                            id: "conflict_skill",
                            steps: [
                                SkillStepDef(
                                    id: "hit",
                                    defense_tile_range: Some((include_anchor_tile: false, rows: [".@X"])),
                                    defense_tile_range_preset: Some("front_1"),
                                    target: EnemySingle(rule: Nearest),
                                    delivery: Instant,
                                ),
                            ],
                        ),
                    ],
                )
                "#,
            )
            .unwrap();
        });

        assert!(
            result.is_err(),
            "inline defense_tile_range and preset reference must be mutually exclusive"
        );
    }
}
