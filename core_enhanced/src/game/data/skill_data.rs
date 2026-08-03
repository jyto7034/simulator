use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::game::{
    ability::{
        DeliveryDef, FocusPermissions, ProjectileHitPolicy, SkillAreaAnchorSource,
        SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillKind, SkillPresentationDef,
        SkillStepCondition, SkillStepDef, SkillStepRepeat, SkillTarget, SkillTargetReach,
        StepTargetingMode,
    },
    battle::buffs::BuffDatabase,
    battle::tile_range::TileRangePattern,
    data::build_string_index,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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
        let database = Self {
            range_presets: vec![],
            skills,
            by_id: OnceLock::new(),
        };
        database
            .validate_contract()
            .expect("invalid Skill authoring contract");
        database
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.skills, "skill id", |skill| &skill.id))
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .expect("invalid Skill authoring contract");
        let _ = self.by_id();
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        validate_skill_contracts(&self.skills)
    }

    pub(crate) fn validate_buff_references(
        &self,
        buff_data: &BuffDatabase,
    ) -> Result<(), crate::game::data::DataValidationError> {
        validate_skill_buff_references(&self.skills, buff_data)
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
        raw.try_into_database().map_err(D::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SkillDatabase")]
struct RawSkillDatabase {
    #[serde(default)]
    range_presets: Vec<SkillRangePresetDef>,
    skills: Vec<RawSkillDef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SkillDef")]
struct RawSkillDef {
    id: crate::game::ability::SkillId,
    name: String,
    kind: SkillKind,
    cast_targeting: RawSkillCastTargetingDef,
    focus_time_ms: u32,
    #[serde(default)]
    focus_permissions: FocusPermissions,
    steps: Vec<RawSkillStepDef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum RawSkillCastTargetingDef {
    Explicit {
        target: SkillTarget,
        reach: SkillTargetReach,
        #[serde(default)]
        target_range: Option<TileRangePattern>,
        #[serde(default)]
        target_range_preset: Option<String>,
        air_capable: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SkillStepDef")]
struct RawSkillStepDef {
    id: String,
    delay_ms: u32,
    target: SkillTarget,
    #[serde(default)]
    targeting: StepTargetingMode,
    #[serde(default)]
    when: SkillStepCondition,
    #[serde(default)]
    repeat: SkillStepRepeat,
    delivery: DeliveryDef,
    effects: Vec<SkillEffectDef>,
    #[serde(default)]
    presentation: SkillPresentationDef,
}

impl RawSkillDatabase {
    fn try_into_database(self) -> Result<SkillDatabase, String> {
        let mut preset_index = HashMap::new();
        for (index, preset) in self.range_presets.iter().enumerate() {
            if preset_index.insert(preset.id.clone(), index).is_some() {
                return Err(format!("duplicate skill range preset id '{}'", preset.id));
            }
        }
        for preset in &self.range_presets {
            preset.range.validate().map_err(|error| {
                format!(
                    "skill range preset '{}' has invalid defense_tile_range: {error}",
                    preset.id
                )
            })?;
        }

        let skills = self
            .skills
            .into_iter()
            .map(|skill| skill.try_into_skill_def(&self.range_presets, &preset_index))
            .collect::<Result<Vec<_>, _>>()?;
        let database = SkillDatabase {
            range_presets: self.range_presets,
            skills,
            by_id: OnceLock::new(),
        };
        database
            .validate_contract()
            .map_err(|error| format!("invalid Skill data: {error:?}"))?;
        Ok(database)
    }
}

impl RawSkillDef {
    fn try_into_skill_def(
        self,
        presets: &[SkillRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> Result<SkillDef, String> {
        Ok(SkillDef {
            id: self.id,
            name: self.name,
            kind: self.kind,
            cast_targeting: self
                .cast_targeting
                .try_into_cast_targeting(presets, preset_index)?,
            focus_time_ms: self.focus_time_ms,
            focus_permissions: self.focus_permissions,
            steps: self
                .steps
                .into_iter()
                .map(|step| step.try_into_step(presets, preset_index))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl RawSkillCastTargetingDef {
    fn try_into_cast_targeting(
        self,
        presets: &[SkillRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> Result<SkillCastTargetingDef, String> {
        Ok(match self {
            RawSkillCastTargetingDef::Explicit {
                target,
                reach,
                target_range,
                target_range_preset,
                air_capable,
            } => SkillCastTargetingDef::Explicit {
                target,
                reach,
                target_range: try_resolve_skill_range_source(
                    "explicit cast_targeting",
                    target_range,
                    target_range_preset,
                    presets,
                    preset_index,
                )?,
                air_capable,
            },
        })
    }
}

impl RawSkillStepDef {
    fn try_into_step(
        self,
        _presets: &[SkillRangePresetDef],
        _preset_index: &HashMap<String, usize>,
    ) -> Result<SkillStepDef, String> {
        let step_id = self.id;
        Ok(SkillStepDef {
            id: step_id.clone(),
            delay_ms: self.delay_ms,
            target: self.target,
            targeting: self.targeting,
            when: self.when,
            repeat: self.repeat,
            delivery: self.delivery,
            effects: self.effects,
            presentation: self.presentation,
        })
    }
}

fn try_resolve_skill_range_source(
    context: &str,
    inline_range: Option<TileRangePattern>,
    preset_id: Option<String>,
    presets: &[SkillRangePresetDef],
    preset_index: &HashMap<String, usize>,
) -> Result<Option<TileRangePattern>, String> {
    match (inline_range, preset_id) {
        (Some(_), Some(preset_id)) => Err(format!(
            "{context} defines both target_range and target_range_preset '{preset_id}'"
        )),
        (Some(range), None) => Ok(Some(range)),
        (None, Some(preset_id)) => {
            let Some(index) = preset_index.get(&preset_id) else {
                return Err(format!(
                    "{context} references unknown target_range_preset '{preset_id}'"
                ));
            };
            Ok(Some(presets[*index].range.clone()))
        }
        (None, None) => Ok(None),
    }
}

fn validate_skill_buff_references(
    skills: &[SkillDef],
    buff_data: &BuffDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (skill_index, skill) in skills.iter().enumerate() {
        let skill_id = skill.id.to_string();
        for (step_index, step) in skill.steps.iter().enumerate() {
            let step_path = format!("skills[{skill_index}].steps[{step_index}]");
            let validate_buff = |field_path: String,
                                 buff_id: &str|
             -> Result<(), crate::game::data::DataValidationError> {
                if buff_data.contains_name(buff_id) {
                    Ok(())
                } else {
                    Err(crate::game::data::DataValidationError::at(
                        &skill_id,
                        field_path,
                        format!("references unknown buff '{buff_id}'"),
                    ))
                }
            };
            match &step.when {
                crate::game::ability::SkillStepCondition::Always
                | crate::game::ability::SkillStepCondition::IfPreviousStepDealtDamage => {}
                crate::game::ability::SkillStepCondition::IfCasterHasBuff { buff_id, .. } => {
                    validate_buff(format!("{step_path}.when.IfCasterHasBuff.buff_id"), buff_id)?;
                }
            }

            if let crate::game::ability::SkillStepRepeat::ByBuffStacks { buff_id, .. } =
                &step.repeat
            {
                validate_buff(format!("{step_path}.repeat.ByBuffStacks.buff_id"), buff_id)?;
            }

            for (effect_index, effect) in step.effects.iter().enumerate() {
                if let crate::game::ability::SkillEffectDef::ApplyBuff { buff_id, .. } = effect {
                    validate_buff(
                        format!("{step_path}.effects[{effect_index}].ApplyBuff.buff_id"),
                        buff_id,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn validate_skill_contracts(
    skills: &[SkillDef],
) -> Result<(), crate::game::data::DataValidationError> {
    for (skill_index, skill) in skills.iter().enumerate() {
        let skill_id = skill.id.to_string();
        let skill_path = format!("skills[{skill_index}]");
        let error = |path: String, cause: String| {
            crate::game::data::DataValidationError::at(&skill_id, path, cause)
        };
        if skill.steps.is_empty() {
            return Err(error(
                format!("{skill_path}.steps"),
                "must define at least one step".to_string(),
            ));
        }
        let crate::game::ability::SkillCastTargetingDef::Explicit {
            reach,
            target,
            target_range,
            ..
        } = &skill.cast_targeting;
        validate_skill_target_reach(*reach, target_range.as_ref())
            .map_err(|cause| error(format!("{skill_path}.cast_targeting"), cause))?;
        if matches!(reach, SkillTargetReach::Global) && matches!(target, SkillTarget::SelfUnit) {
            return Err(error(
                format!("{skill_path}.cast_targeting.target"),
                "global reach requires an explicit non-self target".to_string(),
            ));
        }
        if matches!(target, SkillTarget::EnemySingle { .. })
            && matches!(reach, SkillTargetReach::Pattern)
            && target_range.is_none()
        {
            return Err(error(
                format!("{skill_path}.cast_targeting.target_range"),
                "pattern reach for EnemySingle requires target_range".to_string(),
            ));
        }
        let mut step_ids = HashSet::new();

        for (step_index, step) in skill.steps.iter().enumerate() {
            let step_path = format!("{skill_path}.steps[{step_index}]");
            if !step_ids.insert(step.id.clone()) {
                return Err(error(
                    format!("{step_path}.id"),
                    format!("duplicate step id '{}'", step.id),
                ));
            }

            let mut has_damage_effect = false;
            let mut has_modify_damage_effect = false;
            for (effect_index, effect) in step.effects.iter().enumerate() {
                match effect {
                    SkillEffectDef::Damage { amount, .. } => {
                        has_damage_effect = true;
                        if *amount < 0 {
                            return Err(error(
                                format!("{step_path}.effects[{effect_index}].Damage.amount"),
                                format!("Damage amount must not be negative: {amount}"),
                            ));
                        }
                    }
                    SkillEffectDef::ModifyDamage { .. } => {
                        has_modify_damage_effect = true;
                    }
                    SkillEffectDef::Heal { .. }
                    | SkillEffectDef::ModifyResonance { .. }
                    | SkillEffectDef::ModifyStabilization { .. }
                    | SkillEffectDef::ModifyStats { .. }
                    | SkillEffectDef::ApplyBuff { .. }
                    | SkillEffectDef::InterruptCast
                    | SkillEffectDef::ExtraAttack { .. } => {}
                }
            }
            if has_modify_damage_effect && !has_damage_effect {
                return Err(error(
                    format!("{step_path}.effects"),
                    "ModifyDamage requires a Damage effect in the same step".to_string(),
                ));
            }

            match &step.delivery {
                DeliveryDef::Projectile {
                    hit_policy,
                    allow_targetless_cast,
                    max_range_tiles,
                    max_lifetime_ms,
                    max_kills,
                    max_pierces,
                    collision,
                    ..
                } => {
                    collision.try_validate_runtime_contract().map_err(|cause| {
                        error(format!("{step_path}.delivery.Projectile.collision"), cause)
                    })?;
                    match hit_policy {
                        ProjectileHitPolicy::TargetLocked => {
                            if *allow_targetless_cast {
                                return Err(error(
                                    format!(
                                        "{step_path}.delivery.Projectile.allow_targetless_cast"
                                    ),
                                    "target-locked projectile must not allow targetless cast"
                                        .to_string(),
                                ));
                            }
                            if max_range_tiles.is_some()
                                || max_lifetime_ms.is_some()
                                || max_kills.is_some()
                                || max_pierces.is_some()
                            {
                                return Err(error(
                                    format!("{step_path}.delivery.Projectile"),
                                    "target-locked projectile must not define directional stop policy fields".to_string(),
                                ));
                            }
                            collision
                                .try_validate_homing_runtime_contract()
                                .map_err(|cause| {
                                    error(
                                        format!("{step_path}.delivery.Projectile.collision"),
                                        cause,
                                    )
                                })?;
                        }
                        ProjectileHitPolicy::DirectionalCollision => {
                            if max_range_tiles.is_none() && max_lifetime_ms.is_none() {
                                return Err(error(
                                    format!("{step_path}.delivery.Projectile"),
                                    "DirectionalCollision requires max_range_tiles or max_lifetime_ms".to_string(),
                                ));
                            }
                            if max_kills.is_none()
                                && max_pierces.is_none()
                                && collision.max_hits.is_none()
                                && max_range_tiles.is_none()
                                && max_lifetime_ms.is_none()
                            {
                                return Err(error(
                                    format!("{step_path}.delivery.Projectile"),
                                    "DirectionalCollision requires at least one stop policy"
                                        .to_string(),
                                ));
                            }
                        }
                    }
                }
                DeliveryDef::TileArea { footprint, area } => {
                    footprint.validate().map_err(|cause| {
                        error(format!("{step_path}.delivery.TileArea.footprint"), cause)
                    })?;
                    area.try_validate_runtime_contract().map_err(|cause| {
                        error(format!("{step_path}.delivery.TileArea.area"), cause)
                    })?;
                    let expected_origin = match area.anchor {
                        SkillAreaAnchorSource::Caster => {
                            crate::game::ability::SkillTileAreaOrigin::Caster
                        }
                        SkillAreaAnchorSource::CastTarget
                        | SkillAreaAnchorSource::CastTargetStart
                        | SkillAreaAnchorSource::ImpactContext
                        | SkillAreaAnchorSource::ImpactContextStart => {
                            crate::game::ability::SkillTileAreaOrigin::Anchor
                        }
                    };
                    if area.tile_origin != expected_origin {
                        return Err(error(
                            format!("{step_path}.delivery.TileArea.area.tile_origin"),
                            format!(
                                "{:?} area requires {:?} tile_origin",
                                area.anchor, expected_origin
                            ),
                        ));
                    }
                    if area.anchor != SkillAreaAnchorSource::Caster
                        && !footprint.is_rotation_invariant()
                    {
                        return Err(error(
                            format!("{step_path}.delivery.TileArea.footprint"),
                            "target/impact-centered footprint must be rotation-invariant until an explicit orientation policy exists".to_string(),
                        ));
                    }
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
                        if !has_prior_spatial_delivery {
                            return Err(error(
                                format!("{step_path}.delivery.TileArea.area.anchor"),
                                format!("{:?} requires a prior spatial delivery step", area.anchor),
                            ));
                        }
                    }
                }
                DeliveryDef::Instant => {}
            }
        }
    }
    Ok(())
}

fn validate_skill_target_reach(
    reach: SkillTargetReach,
    target_range: Option<&TileRangePattern>,
) -> Result<(), String> {
    match reach {
        SkillTargetReach::Pattern => {
            if let Some(pattern) = target_range {
                pattern
                    .validate()
                    .map_err(|error| format!("invalid target_range: {error}"))?;
            }
        }
        SkillTargetReach::Global => {
            if target_range.is_some() {
                return Err("global reach must not also define target_range".to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ability::{
        DeliveryDef, ProjectileHitPolicy, SkillAreaAnchorSource, SkillCastTargetingDef,
        SkillEffectDef, SkillId, SkillProjectileCollisionDef, SkillStepDef,
        SkillTileAreaDeliveryDef, UnitTargetRule,
    };
    use crate::game::battle::buffs::BuffDatabase;
    use crate::game::battle::damage::{DamageModifiers, DamageType};
    use crate::game::battle::tile_range::TileRangePattern;

    fn explicit_cast_targeting(target: SkillTarget) -> SkillCastTargetingDef {
        SkillCastTargetingDef::explicit(target, Default::default(), None, false)
    }

    fn explicit_enemy_cast_targeting() -> SkillCastTargetingDef {
        SkillCastTargetingDef::explicit(
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            SkillTargetReach::Pattern,
            Some(TileRangePattern {
                include_anchor_tile: false,
                rows: vec![".@X".to_string()],
            }),
            false,
        )
    }

    fn explicit_self_cast_targeting() -> SkillCastTargetingDef {
        explicit_cast_targeting(SkillTarget::SelfUnit)
    }

    fn projectile_step(collision: SkillProjectileCollisionDef) -> SkillStepDef {
        SkillStepDef {
            id: "shot".to_string(),
            delay_ms: 0,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1_000,
                hit_policy: ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
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
            target: SkillTarget::CastTarget,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec![".@.".to_string()],
                },
                area: SkillTileAreaDeliveryDef {
                    anchor,
                    tile_origin: if anchor == SkillAreaAnchorSource::Caster {
                        crate::game::ability::SkillTileAreaOrigin::Caster
                    } else {
                        crate::game::ability::SkillTileAreaOrigin::Anchor
                    },
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
                cast_targeting: explicit_self_cast_targeting(),
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
                cast_targeting: explicit_enemy_cast_targeting(),
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
    fn skill_database_rejects_global_reach_with_authored_target_pattern() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("ambiguous_whole_field"),
                name: "ambiguous_whole_field".to_string(),
                kind: SkillKind::Targeted,
                cast_targeting: SkillCastTargetingDef::explicit(
                    SkillTarget::EnemySingle {
                        rule: UnitTargetRule::Nearest,
                    },
                    SkillTargetReach::Global,
                    Some(TileRangePattern {
                        include_anchor_tile: false,
                        rows: vec![".@X".to_string()],
                    }),
                    false,
                ),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "hit".to_string(),
                    delay_ms: 0,
                    target: SkillTarget::EnemySingle {
                        rule: UnitTargetRule::Nearest,
                    },
                    targeting: Default::default(),
                    when: Default::default(),
                    repeat: Default::default(),
                    delivery: DeliveryDef::Instant,
                    effects: vec![],
                    presentation: Default::default(),
                }],
            }]);
        });

        assert!(
            result.is_err(),
            "global reach must not also define target_range"
        );
    }

    #[test]
    fn skill_database_rejects_duplicate_step_ids() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("duplicate_steps"),
                name: "duplicate_steps".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: explicit_enemy_cast_targeting(),
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
            cast_targeting: explicit_self_cast_targeting(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "step".to_string(),
                delay_ms: 0,
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
        let error = database
            .validate_buff_references(&BuffDatabase::new(vec![]))
            .expect_err("unknown buff references must be rejected");
        assert_eq!(error.object_id.as_deref(), Some("unknown_buff"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("skills[0].steps[0].when.IfCasterHasBuff.buff_id")
        );
        assert!(error.cause.contains("does_not_exist"));
    }

    #[test]
    fn skill_database_rejects_negative_damage_amounts() {
        let result = std::panic::catch_unwind(|| {
            SkillDatabase::new(vec![SkillDef {
                id: SkillId::from("negative_damage"),
                name: "negative_damage".to_string(),
                kind: SkillKind::Untargeted,
                cast_targeting: explicit_enemy_cast_targeting(),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "hit".to_string(),
                    delay_ms: 0,
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
                cast_targeting: explicit_enemy_cast_targeting(),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "modifier_only".to_string(),
                    delay_ms: 0,
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
                cast_targeting: explicit_cast_targeting(SkillTarget::CastTarget),
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
    fn skill_database_rejects_noncentered_or_oriented_target_area() {
        for defect in ["caster_origin", "implicit_orientation"] {
            let mut step = tile_area_step("area", SkillAreaAnchorSource::CastTarget);
            let DeliveryDef::TileArea { footprint, area } = &mut step.delivery else {
                unreachable!();
            };
            match defect {
                "caster_origin" => {
                    area.tile_origin = crate::game::ability::SkillTileAreaOrigin::Caster;
                }
                "implicit_orientation" => {
                    *footprint = TileRangePattern {
                        include_anchor_tile: false,
                        rows: vec![".@X".to_string()],
                    };
                }
                _ => unreachable!(),
            }
            let result = std::panic::catch_unwind(|| {
                SkillDatabase::new(vec![SkillDef {
                    id: SkillId::from(defect),
                    name: defect.to_string(),
                    kind: SkillKind::Targeted,
                    cast_targeting: explicit_cast_targeting(SkillTarget::CastTarget),
                    focus_time_ms: 0,
                    focus_permissions: Default::default(),
                    steps: vec![step],
                }]);
            });
            assert!(result.is_err(), "target-area defect must reject: {defect}");
        }
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
                        name: "Preset Skill",
                        kind: Targeted,
                        focus_time_ms: 0,
                        cast_targeting: Explicit(
                            target: EnemySingle(rule: Nearest),
                            reach: pattern,
                            target_range_preset: Some("front_1"),
                            air_capable: false,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                delay_ms: 0,
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                                effects: [],
                            ),
                        ],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        let skill = database.get_by_id("preset_skill").unwrap();
        let SkillCastTargetingDef::Explicit { target_range, .. } = &skill.cast_targeting;
        let range = target_range.as_ref().unwrap();
        assert_eq!(range.rows, vec![".@X"]);
        assert_eq!(database.range_presets.len(), 1);
    }

    #[test]
    fn skill_database_rejects_implicit_and_legacy_targeting_authority() {
        let missing_cast_targeting = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "implicit_skill",
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(missing_cast_targeting.is_err());

        let missing_selection_rule = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "missing_selection_rule",
                        cast_targeting: Explicit(
                            target: EnemySingle(),
                            reach: pattern,
                            target_range: Some((include_anchor_tile: false, rows: [".@X"])),
                            air_capable: false,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(missing_selection_rule.is_err());

        let missing_reach = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "missing_reach",
                        cast_targeting: Explicit(
                            target: SelfUnit,
                            air_capable: false,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                target: SelfUnit,
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(missing_reach.is_err());

        let missing_air_capable = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "missing_air_capable",
                        cast_targeting: Explicit(
                            target: SelfUnit,
                            reach: pattern,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                target: SelfUnit,
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(missing_air_capable.is_err());

        let missing_pattern_target_range = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "missing_pattern_target_range",
                        cast_targeting: Explicit(
                            target: EnemySingle(rule: Nearest),
                            reach: pattern,
                            air_capable: false,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(missing_pattern_target_range.is_err());

        let legacy_weapon_authority = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "legacy_weapon_authority",
                        cast_targeting: WeaponTargetingProfile,
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(legacy_weapon_authority.is_err());

        let legacy_range_units = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "legacy_range_skill",
                        cast_targeting: Explicit(
                            target: EnemySingle(rule: Nearest),
                            reach: pattern,
                            target_range: Some((include_anchor_tile: false, rows: [".@X"])),
                            air_capable: false,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                range_units: 3,
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(legacy_range_units.is_err());

        for legacy_step_field in [
            "range_policy: whole_field_valid_tiles,",
            "defense_tile_range: Some((include_anchor_tile: false, rows: [\".@X\"])),",
            "air_capable: true,",
        ] {
            let authored = format!(
                r#"
                SkillDatabase(
                    skills: [
                        SkillDef(
                            id: "legacy_step_authority",
                            cast_targeting: Explicit(
                                target: SelfUnit,
                                reach: pattern,
                                air_capable: false,
                            ),
                            steps: [
                                SkillStepDef(
                                    id: "hit",
                                    {legacy_step_field}
                                    target: SelfUnit,
                                    delivery: Instant,
                                ),
                            ],
                        ),
                    ],
                )
                "#
            );
            assert!(
                ron::de::from_str::<SkillDatabase>(&authored).is_err(),
                "legacy step authority must reject: {legacy_step_field}"
            );
        }

        let missing_tile_area_footprint = ron::de::from_str::<SkillDatabase>(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "missing_footprint",
                        cast_targeting: Explicit(
                            target: SelfUnit,
                            reach: pattern,
                            air_capable: false,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "area",
                                target: SelfUnit,
                                delivery: TileArea(area: (anchor: Caster)),
                            ),
                        ],
                    ),
                ],
            )
            "#,
        );
        assert!(missing_tile_area_footprint.is_err());
    }

    #[test]
    fn skill_database_rejects_inline_target_range_and_preset_conflict() {
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
                            cast_targeting: Explicit(
                                target: EnemySingle(rule: Nearest),
                                reach: pattern,
                                target_range: Some((include_anchor_tile: false, rows: [".@X"])),
                                target_range_preset: Some("front_1"),
                                air_capable: false,
                            ),
                            steps: [
                                SkillStepDef(
                                    id: "hit",
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
            "inline target_range and preset reference must be mutually exclusive"
        );
    }
}
