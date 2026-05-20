use game_core::game::{
    ability::{
        DeliveryDef, FocusPermissions, SkillArea, SkillAreaAnchorSource, SkillAreaDeliveryDef,
        SkillAreaShapeDef, SkillAreaTickPolicy, SkillCastTargetingDef, SkillDef, SkillEffectDef,
        SkillHitTargetFilter, SkillKind, SkillProjectileCollisionDef, SkillStepCondition,
        SkillStepDef, SkillStepRepeat, SkillTarget, SkillUnitReference, StepTargetingMode,
        UnitTargetRule,
    },
    battle::damage::{DamageModifiers, DamageType},
    data::skill_data::SkillDatabase,
    stats::{StatId, StatModifier, StatModifierKind},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize)]
struct SkillFileText {
    path: String,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillDraftDto {
    id: String,
    name: String,
    kind: String,
    focus_time_ms: u32,
    focus_permissions: FocusPermissionsDto,
    steps: Vec<SkillStepDraftDto>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FocusPermissionsDto {
    allows_move: bool,
    allows_basic_attack: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillStepDraftDto {
    id: String,
    delay_ms: u32,
    range_tiles: u8,
    condition: StepConditionDto,
    repeat: StepRepeatDto,
    target: TargetDto,
    delivery: DeliveryDto,
    effects: Vec<EffectDto>,
    presentation: PresentationDto,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum StepConditionDto {
    Always,
    IfPreviousStepDealtDamage,
    IfCasterHasBuff { buff_id: String, min_stacks: u8 },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum StepRepeatDto {
    None,
    Times {
        count: u8,
    },
    ByBuffStacks {
        unit: String,
        buff_id: String,
        max: Option<u8>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum TargetDto {
    SelfUnit,
    EnemySingle { rule: String },
    Allies { area: AreaDto },
    Enemies { area: AreaDto },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum AreaDto {
    All,
    RadiusChebyshev { radius_tiles: u8 },
    Line { length_tiles: u8 },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum DeliveryDto {
    Instant,
    Projectile {
        speed_units_per_ms: u32,
        radius_units: u32,
        hit_targets: String,
        piercing: bool,
        despawn_on_hit: Option<bool>,
        max_hits: Option<u8>,
    },
    Area {
        shape: AreaShapeDto,
        anchor: String,
        hit_targets: String,
        include_caster: bool,
        tick_policy: String,
        duration_ms: u32,
        tick_interval_ms: Option<u32>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum AreaShapeDto {
    Circle {
        radius_units: u32,
    },
    Line {
        length_units: u32,
    },
    Box {
        width_units: u32,
        height_units: u32,
    },
    Rectangle {
        width_units: u32,
        length_units: u32,
    },
    Cone {
        angle_degrees: u16,
        length_units: u32,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum EffectDto {
    Damage { amount: i32, damage_type: String },
    ModifyDamage { modifiers: DamageModifiersDto },
    Heal { amount: i32 },
    ModifyResonance { amount: i32 },
    ModifyStats { modifier: StatModifierDto },
    ApplyBuff { buff_id: String, duration_ms: u32 },
    ExtraAttack { count: u8 },
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DamageModifiersDto {
    #[serde(default)]
    armor_penetration_flat: i32,
    #[serde(default)]
    magic_resist_penetration_flat: i32,
    #[serde(default)]
    armor_penetration_percent: i32,
    #[serde(default)]
    magic_resist_penetration_percent: i32,
    #[serde(default)]
    damage_amp_percent: i32,
    #[serde(default)]
    damage_reduction_percent: i32,
    #[serde(default)]
    physical_damage_amp_percent: i32,
    #[serde(default)]
    magic_damage_amp_percent: i32,
    #[serde(default)]
    true_damage_amp_percent: i32,
    #[serde(default)]
    physical_damage_reduction_percent: i32,
    #[serde(default)]
    magic_damage_reduction_percent: i32,
    #[serde(default)]
    true_damage_reduction_percent: i32,
    #[serde(default)]
    crit_chance_percent: i32,
    #[serde(default)]
    crit_damage_percent: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatModifierDto {
    stat: String,
    kind: String,
    value: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PresentationDto {
    cast_state: Option<String>,
    projectile_vfx_id: Option<String>,
    impact_vfx_id: Option<String>,
    target_anchor: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportResult {
    path: String,
    skill_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationResult {
    skill_count: usize,
}

#[tauri::command]
fn read_skill_file(path: String) -> Result<SkillFileText, String> {
    let path_buf = PathBuf::from(&path);
    let text = fs::read_to_string(&path_buf)
        .map_err(|err| format!("failed to read skill file '{}': {err}", path_buf.display()))?;

    Ok(SkillFileText { path, text })
}

#[tauri::command]
fn load_default_skills() -> Result<Vec<SkillDraftDto>, String> {
    let path = default_skill_path();
    load_skills_from_path(path)
}

#[tauri::command]
fn load_skills(path: String) -> Result<Vec<SkillDraftDto>, String> {
    load_skills_from_path(PathBuf::from(path))
}

#[tauri::command]
fn export_generated_skills(skills: Vec<SkillDraftDto>) -> Result<ExportResult, String> {
    let path = default_generated_skill_path();
    export_skills_to_path(skills, path)
}

#[tauri::command]
fn validate_skills(skills: Vec<SkillDraftDto>) -> Result<ValidationResult, String> {
    let skill_count = skills.len();
    let skill_defs = skills
        .into_iter()
        .map(skill_from_draft)
        .collect::<Result<Vec<_>, _>>()?;
    let _database = SkillDatabase::new(skill_defs);

    Ok(ValidationResult { skill_count })
}

fn default_skill_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("game_resources/data/skills/base.ron")
}

fn default_generated_skill_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("game_resources/data/skills/base.generated.ron")
}

fn load_skills_from_path(path: PathBuf) -> Result<Vec<SkillDraftDto>, String> {
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read skill file '{}': {err}", path.display()))?;
    let database: SkillDatabase = ron::de::from_str(&text)
        .map_err(|err| format!("failed to parse skill RON '{}': {err}", path.display()))?;

    Ok(database.skills.iter().map(skill_to_draft).collect())
}

fn skill_to_draft(skill: &SkillDef) -> SkillDraftDto {
    SkillDraftDto {
        id: skill.id.clone(),
        name: skill.name.clone(),
        kind: match skill.kind {
            SkillKind::Targeted => "Targeted",
            SkillKind::Untargeted => "Untargeted",
        }
        .to_string(),
        focus_time_ms: skill.focus_time_ms,
        focus_permissions: FocusPermissionsDto {
            allows_move: skill.focus_permissions.allows_move,
            allows_basic_attack: skill.focus_permissions.allows_basic_attack,
        },
        steps: skill.steps.iter().map(step_to_draft).collect(),
    }
}

fn step_to_draft(step: &game_core::game::ability::SkillStepDef) -> SkillStepDraftDto {
    SkillStepDraftDto {
        id: step.id.clone(),
        delay_ms: step.delay_ms,
        range_tiles: step.range_tiles,
        condition: condition_to_dto(&step.when),
        repeat: repeat_to_dto(&step.repeat),
        target: target_to_dto(&step.target),
        delivery: delivery_to_dto(&step.delivery),
        effects: step.effects.iter().map(effect_to_dto).collect(),
        presentation: PresentationDto {
            cast_state: step.presentation.cast_state.clone(),
            projectile_vfx_id: step.presentation.projectile_vfx_id.clone(),
            impact_vfx_id: step.presentation.impact_vfx_id.clone(),
            target_anchor: step.presentation.target_anchor.clone(),
        },
    }
}

fn condition_to_dto(condition: &SkillStepCondition) -> StepConditionDto {
    match condition {
        SkillStepCondition::Always => StepConditionDto::Always,
        SkillStepCondition::IfPreviousStepDealtDamage => {
            StepConditionDto::IfPreviousStepDealtDamage
        }
        SkillStepCondition::IfCasterHasBuff {
            buff_id,
            min_stacks,
        } => StepConditionDto::IfCasterHasBuff {
            buff_id: buff_id.clone(),
            min_stacks: *min_stacks,
        },
    }
}

fn repeat_to_dto(repeat: &SkillStepRepeat) -> StepRepeatDto {
    match repeat {
        SkillStepRepeat::Once => StepRepeatDto::None,
        SkillStepRepeat::Times { count } => StepRepeatDto::Times { count: *count },
        SkillStepRepeat::ByBuffStacks { unit, buff_id, max } => StepRepeatDto::ByBuffStacks {
            unit: match unit {
                SkillUnitReference::SelfUnit => "SelfUnit",
                SkillUnitReference::StepTarget => "StepTarget",
            }
            .to_string(),
            buff_id: buff_id.clone(),
            max: *max,
        },
    }
}

fn target_to_dto(target: &SkillTarget) -> TargetDto {
    match target {
        SkillTarget::SelfUnit => TargetDto::SelfUnit,
        SkillTarget::EnemySingle { rule } => TargetDto::EnemySingle {
            rule: match rule {
                UnitTargetRule::Nearest => "Nearest",
                UnitTargetRule::CurrentTarget => "CurrentTarget",
                UnitTargetRule::LowestHealthEnemy => "LowestHealthEnemy",
            }
            .to_string(),
        },
        SkillTarget::Allies { area } => TargetDto::Allies {
            area: area_to_dto(area),
        },
        SkillTarget::Enemies { area } => TargetDto::Enemies {
            area: area_to_dto(area),
        },
    }
}

fn area_to_dto(area: &SkillArea) -> AreaDto {
    match area {
        SkillArea::All => AreaDto::All,
        SkillArea::RadiusChebyshev { radius_tiles } => AreaDto::RadiusChebyshev {
            radius_tiles: *radius_tiles,
        },
        SkillArea::Line { length_tiles } => AreaDto::Line {
            length_tiles: *length_tiles,
        },
    }
}

fn delivery_to_dto(delivery: &DeliveryDef) -> DeliveryDto {
    match delivery {
        DeliveryDef::Instant => DeliveryDto::Instant,
        DeliveryDef::Projectile {
            speed_units_per_ms,
            collision,
        } => DeliveryDto::Projectile {
            speed_units_per_ms: *speed_units_per_ms,
            radius_units: collision.radius_units,
            hit_targets: hit_filter_label(collision.hit_targets).to_string(),
            piercing: collision.piercing,
            despawn_on_hit: collision.despawn_on_hit,
            max_hits: collision.max_hits,
        },
        DeliveryDef::Area { area } => DeliveryDto::Area {
            shape: shape_to_dto(area.shape),
            anchor: anchor_label(area.anchor).to_string(),
            hit_targets: hit_filter_label(area.hit_targets).to_string(),
            include_caster: area.include_caster,
            tick_policy: format!("{:?}", area.tick_policy),
            duration_ms: area.duration_ms,
            tick_interval_ms: area.tick_interval_ms,
        },
    }
}

fn shape_to_dto(shape: SkillAreaShapeDef) -> AreaShapeDto {
    match shape {
        SkillAreaShapeDef::Circle { radius_units } => AreaShapeDto::Circle { radius_units },
        SkillAreaShapeDef::Line { length_units } => AreaShapeDto::Line { length_units },
        SkillAreaShapeDef::Box {
            width_units,
            height_units,
        } => AreaShapeDto::Box {
            width_units,
            height_units,
        },
        SkillAreaShapeDef::Rectangle {
            width_units,
            length_units,
        } => AreaShapeDto::Rectangle {
            width_units,
            length_units,
        },
        SkillAreaShapeDef::Cone {
            angle_degrees,
            length_units,
        } => AreaShapeDto::Cone {
            angle_degrees,
            length_units,
        },
    }
}

fn effect_to_dto(effect: &SkillEffectDef) -> EffectDto {
    match effect {
        SkillEffectDef::Damage {
            amount,
            damage_type,
        } => EffectDto::Damage {
            amount: *amount,
            damage_type: damage_type_to_dto(*damage_type),
        },
        SkillEffectDef::ModifyDamage { modifiers } => EffectDto::ModifyDamage {
            modifiers: damage_modifiers_to_dto(*modifiers),
        },
        SkillEffectDef::Heal { amount } => EffectDto::Heal { amount: *amount },
        SkillEffectDef::ModifyResonance { amount } => {
            EffectDto::ModifyResonance { amount: *amount }
        }
        SkillEffectDef::ModifyStats { modifier } => EffectDto::ModifyStats {
            modifier: stat_modifier_to_dto(*modifier),
        },
        SkillEffectDef::ApplyBuff {
            buff_id,
            duration_ms,
        } => EffectDto::ApplyBuff {
            buff_id: buff_id.clone(),
            duration_ms: *duration_ms,
        },
        SkillEffectDef::ExtraAttack { count } => EffectDto::ExtraAttack { count: *count },
    }
}

fn hit_filter_label(filter: SkillHitTargetFilter) -> &'static str {
    match filter {
        SkillHitTargetFilter::Allies => "Allies",
        SkillHitTargetFilter::Enemies => "Enemies",
        SkillHitTargetFilter::Any => "Any",
    }
}

fn anchor_label(anchor: SkillAreaAnchorSource) -> &'static str {
    match anchor {
        SkillAreaAnchorSource::CastTarget => "CastTarget",
        SkillAreaAnchorSource::ImpactContext => "ImpactContext",
        SkillAreaAnchorSource::Caster => "Caster",
        SkillAreaAnchorSource::CastTargetStart => "CastTargetStart",
        SkillAreaAnchorSource::ImpactContextStart => "ImpactContextStart",
    }
}

fn export_skills_to_path(
    skills: Vec<SkillDraftDto>,
    path: PathBuf,
) -> Result<ExportResult, String> {
    let skill_count = skills.len();
    let skill_defs = skills
        .into_iter()
        .map(skill_from_draft)
        .collect::<Result<Vec<_>, _>>()?;

    let database = SkillDatabase::new(skill_defs);
    let pretty = ron::ser::PrettyConfig {
        depth_limit: 4,
        separate_tuple_members: true,
        enumerate_arrays: true,
        ..Default::default()
    };
    let text = ron::ser::to_string_pretty(&database, pretty)
        .map_err(|err| format!("failed to serialize generated skill RON: {err}"))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create export directory '{}': {err}",
                parent.display()
            )
        })?;
    }
    fs::write(&path, text).map_err(|err| {
        format!(
            "failed to write generated skill RON '{}': {err}",
            path.display()
        )
    })?;

    Ok(ExportResult {
        path: path.display().to_string(),
        skill_count,
    })
}

fn skill_from_draft(skill: SkillDraftDto) -> Result<SkillDef, String> {
    Ok(SkillDef {
        id: skill.id,
        name: skill.name,
        kind: kind_from_label(&skill.kind)?,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: skill.focus_time_ms,
        focus_permissions: FocusPermissions {
            allows_move: skill.focus_permissions.allows_move,
            allows_basic_attack: skill.focus_permissions.allows_basic_attack,
        },
        steps: skill
            .steps
            .into_iter()
            .map(step_from_draft)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn step_from_draft(step: SkillStepDraftDto) -> Result<SkillStepDef, String> {
    Ok(SkillStepDef {
        id: step.id,
        delay_ms: step.delay_ms,
        range_tiles: step.range_tiles,
        target: target_from_dto(step.target)?,
        targeting: StepTargetingMode::ReuseCastTarget,
        when: condition_from_dto(step.condition),
        repeat: repeat_from_dto(step.repeat)?,
        delivery: delivery_from_dto(step.delivery)?,
        effects: step
            .effects
            .into_iter()
            .map(effect_from_dto)
            .collect::<Result<Vec<_>, _>>()?,
        presentation: game_core::game::ability::SkillPresentationDef {
            cast_state: step.presentation.cast_state,
            projectile_vfx_id: step.presentation.projectile_vfx_id,
            impact_vfx_id: step.presentation.impact_vfx_id,
            target_anchor: step.presentation.target_anchor,
        },
    })
}

fn kind_from_label(label: &str) -> Result<SkillKind, String> {
    match label {
        "Targeted" => Ok(SkillKind::Targeted),
        "Untargeted" => Ok(SkillKind::Untargeted),
        other => Err(format!("unsupported skill kind '{other}'")),
    }
}

fn target_from_dto(target: TargetDto) -> Result<SkillTarget, String> {
    match target {
        TargetDto::SelfUnit => Ok(SkillTarget::SelfUnit),
        TargetDto::EnemySingle { rule } => Ok(SkillTarget::EnemySingle {
            rule: unit_rule_from_label(&rule)?,
        }),
        TargetDto::Allies { area } => Ok(SkillTarget::Allies {
            area: area_from_dto(area),
        }),
        TargetDto::Enemies { area } => Ok(SkillTarget::Enemies {
            area: area_from_dto(area),
        }),
    }
}

fn unit_rule_from_label(label: &str) -> Result<UnitTargetRule, String> {
    match label {
        "Nearest" => Ok(UnitTargetRule::Nearest),
        "CurrentTarget" => Ok(UnitTargetRule::CurrentTarget),
        "LowestHealthEnemy" => Ok(UnitTargetRule::LowestHealthEnemy),
        other => Err(format!("unsupported enemy target rule '{other}'")),
    }
}

fn area_from_dto(area: AreaDto) -> SkillArea {
    match area {
        AreaDto::All => SkillArea::All,
        AreaDto::RadiusChebyshev { radius_tiles } => SkillArea::RadiusChebyshev { radius_tiles },
        AreaDto::Line { length_tiles } => SkillArea::Line { length_tiles },
    }
}

fn condition_from_dto(condition: StepConditionDto) -> SkillStepCondition {
    match condition {
        StepConditionDto::Always => SkillStepCondition::Always,
        StepConditionDto::IfPreviousStepDealtDamage => {
            SkillStepCondition::IfPreviousStepDealtDamage
        }
        StepConditionDto::IfCasterHasBuff {
            buff_id,
            min_stacks,
        } => SkillStepCondition::IfCasterHasBuff {
            buff_id,
            min_stacks,
        },
    }
}

fn repeat_from_dto(repeat: StepRepeatDto) -> Result<SkillStepRepeat, String> {
    match repeat {
        StepRepeatDto::None => Ok(SkillStepRepeat::Once),
        StepRepeatDto::Times { count } => Ok(SkillStepRepeat::Times { count }),
        StepRepeatDto::ByBuffStacks { unit, buff_id, max } => Ok(SkillStepRepeat::ByBuffStacks {
            unit: unit_reference_from_label(&unit)?,
            buff_id,
            max,
        }),
    }
}

fn unit_reference_from_label(label: &str) -> Result<SkillUnitReference, String> {
    match label {
        "SelfUnit" => Ok(SkillUnitReference::SelfUnit),
        "StepTarget" => Ok(SkillUnitReference::StepTarget),
        other => Err(format!("unsupported repeat unit reference '{other}'")),
    }
}

fn delivery_from_dto(delivery: DeliveryDto) -> Result<DeliveryDef, String> {
    match delivery {
        DeliveryDto::Instant => Ok(DeliveryDef::Instant),
        DeliveryDto::Projectile {
            speed_units_per_ms,
            radius_units,
            hit_targets,
            piercing,
            despawn_on_hit,
            max_hits,
        } => Ok(DeliveryDef::Projectile {
            speed_units_per_ms,
            collision: SkillProjectileCollisionDef {
                radius_units,
                hit_targets: hit_filter_from_label(&hit_targets)?,
                piercing,
                despawn_on_hit,
                max_hits,
            },
        }),
        DeliveryDto::Area {
            shape,
            anchor,
            hit_targets,
            include_caster,
            tick_policy,
            duration_ms,
            tick_interval_ms,
        } => Ok(DeliveryDef::Area {
            area: SkillAreaDeliveryDef {
                shape: shape_from_dto(shape),
                anchor: anchor_from_label(&anchor)?,
                hit_targets: hit_filter_from_label(&hit_targets)?,
                include_caster,
                tick_policy: tick_policy_from_label(&tick_policy)?,
                duration_ms,
                tick_interval_ms,
            },
        }),
    }
}

fn shape_from_dto(shape: AreaShapeDto) -> SkillAreaShapeDef {
    match shape {
        AreaShapeDto::Circle { radius_units } => SkillAreaShapeDef::Circle { radius_units },
        AreaShapeDto::Line { length_units } => SkillAreaShapeDef::Line { length_units },
        AreaShapeDto::Box {
            width_units,
            height_units,
        } => SkillAreaShapeDef::Box {
            width_units,
            height_units,
        },
        AreaShapeDto::Rectangle {
            width_units,
            length_units,
        } => SkillAreaShapeDef::Rectangle {
            width_units,
            length_units,
        },
        AreaShapeDto::Cone {
            angle_degrees,
            length_units,
        } => SkillAreaShapeDef::Cone {
            angle_degrees,
            length_units,
        },
    }
}

fn effect_from_dto(effect: EffectDto) -> Result<SkillEffectDef, String> {
    match effect {
        EffectDto::Damage {
            amount,
            damage_type,
        } => Ok(SkillEffectDef::Damage {
            amount,
            damage_type: damage_type_from_dto(&damage_type)?,
        }),
        EffectDto::ModifyDamage { modifiers } => Ok(SkillEffectDef::ModifyDamage {
            modifiers: damage_modifiers_from_dto(modifiers),
        }),
        EffectDto::Heal { amount } => Ok(SkillEffectDef::Heal { amount }),
        EffectDto::ModifyResonance { amount } => Ok(SkillEffectDef::ModifyResonance { amount }),
        EffectDto::ModifyStats { modifier } => Ok(SkillEffectDef::ModifyStats {
            modifier: stat_modifier_from_dto(modifier)?,
        }),
        EffectDto::ApplyBuff {
            buff_id,
            duration_ms,
        } => Ok(SkillEffectDef::ApplyBuff {
            buff_id,
            duration_ms,
        }),
        EffectDto::ExtraAttack { count } => Ok(SkillEffectDef::ExtraAttack { count }),
    }
}

fn damage_modifiers_to_dto(modifiers: DamageModifiers) -> DamageModifiersDto {
    DamageModifiersDto {
        armor_penetration_flat: modifiers.armor_penetration_flat,
        magic_resist_penetration_flat: modifiers.magic_resist_penetration_flat,
        armor_penetration_percent: modifiers.armor_penetration_percent,
        magic_resist_penetration_percent: modifiers.magic_resist_penetration_percent,
        damage_amp_percent: modifiers.damage_amp_percent,
        damage_reduction_percent: modifiers.damage_reduction_percent,
        physical_damage_amp_percent: modifiers.physical_damage_amp_percent,
        magic_damage_amp_percent: modifiers.magic_damage_amp_percent,
        true_damage_amp_percent: modifiers.true_damage_amp_percent,
        physical_damage_reduction_percent: modifiers.physical_damage_reduction_percent,
        magic_damage_reduction_percent: modifiers.magic_damage_reduction_percent,
        true_damage_reduction_percent: modifiers.true_damage_reduction_percent,
        crit_chance_percent: modifiers.crit_chance_percent,
        crit_damage_percent: modifiers.crit_damage_percent,
    }
}

fn damage_modifiers_from_dto(modifiers: DamageModifiersDto) -> DamageModifiers {
    DamageModifiers {
        armor_penetration_flat: modifiers.armor_penetration_flat,
        magic_resist_penetration_flat: modifiers.magic_resist_penetration_flat,
        armor_penetration_percent: modifiers.armor_penetration_percent,
        magic_resist_penetration_percent: modifiers.magic_resist_penetration_percent,
        damage_amp_percent: modifiers.damage_amp_percent,
        damage_reduction_percent: modifiers.damage_reduction_percent,
        physical_damage_amp_percent: modifiers.physical_damage_amp_percent,
        magic_damage_amp_percent: modifiers.magic_damage_amp_percent,
        true_damage_amp_percent: modifiers.true_damage_amp_percent,
        physical_damage_reduction_percent: modifiers.physical_damage_reduction_percent,
        magic_damage_reduction_percent: modifiers.magic_damage_reduction_percent,
        true_damage_reduction_percent: modifiers.true_damage_reduction_percent,
        crit_chance_percent: modifiers.crit_chance_percent,
        crit_damage_percent: modifiers.crit_damage_percent,
    }
}

fn stat_modifier_to_dto(modifier: StatModifier) -> StatModifierDto {
    StatModifierDto {
        stat: match modifier.stat {
            StatId::MaxHealth => "MaxHealth",
            StatId::Attack => "Attack",
            StatId::Defense => "Defense",
            StatId::MagicResist => "MagicResist",
            StatId::AttackIntervalMs => "AttackIntervalMs",
            StatId::MoveSpeedUnitsPerMs => "MoveSpeedUnitsPerMs",
        }
        .to_string(),
        kind: match modifier.kind {
            StatModifierKind::Flat => "Flat",
            StatModifierKind::Percent => "Percent",
        }
        .to_string(),
        value: modifier.value,
    }
}

fn stat_modifier_from_dto(modifier: StatModifierDto) -> Result<StatModifier, String> {
    Ok(StatModifier {
        stat: stat_id_from_label(&modifier.stat)?,
        kind: stat_modifier_kind_from_label(&modifier.kind)?,
        value: modifier.value,
    })
}

fn damage_type_to_dto(damage_type: DamageType) -> String {
    match damage_type {
        DamageType::Physical => "Physical",
        DamageType::Magic => "Magic",
        DamageType::True => "True",
    }
    .to_string()
}

fn damage_type_from_dto(label: &str) -> Result<DamageType, String> {
    match label {
        "Physical" | "physical" => Ok(DamageType::Physical),
        "Magic" | "magic" => Ok(DamageType::Magic),
        "True" | "true" => Ok(DamageType::True),
        other => Err(format!("unsupported damage type '{other}'")),
    }
}

fn stat_id_from_label(label: &str) -> Result<StatId, String> {
    match label {
        "MaxHealth" => Ok(StatId::MaxHealth),
        "Attack" => Ok(StatId::Attack),
        "Defense" => Ok(StatId::Defense),
        "MagicResist" => Ok(StatId::MagicResist),
        "AttackIntervalMs" => Ok(StatId::AttackIntervalMs),
        "MoveSpeedUnitsPerMs" => Ok(StatId::MoveSpeedUnitsPerMs),
        other => Err(format!("unsupported stat id '{other}'")),
    }
}

fn stat_modifier_kind_from_label(label: &str) -> Result<StatModifierKind, String> {
    match label {
        "Flat" => Ok(StatModifierKind::Flat),
        "Percent" => Ok(StatModifierKind::Percent),
        other => Err(format!("unsupported stat modifier kind '{other}'")),
    }
}

fn hit_filter_from_label(label: &str) -> Result<SkillHitTargetFilter, String> {
    match label {
        "Allies" => Ok(SkillHitTargetFilter::Allies),
        "Enemies" => Ok(SkillHitTargetFilter::Enemies),
        "Any" => Ok(SkillHitTargetFilter::Any),
        other => Err(format!("unsupported hit target filter '{other}'")),
    }
}

fn anchor_from_label(label: &str) -> Result<SkillAreaAnchorSource, String> {
    match label {
        "CastTarget" => Ok(SkillAreaAnchorSource::CastTarget),
        "ImpactContext" => Ok(SkillAreaAnchorSource::ImpactContext),
        "Caster" => Ok(SkillAreaAnchorSource::Caster),
        "CastTargetStart" => Ok(SkillAreaAnchorSource::CastTargetStart),
        "ImpactContextStart" => Ok(SkillAreaAnchorSource::ImpactContextStart),
        other => Err(format!("unsupported area anchor '{other}'")),
    }
}

fn tick_policy_from_label(label: &str) -> Result<SkillAreaTickPolicy, String> {
    match label {
        "EveryTick" => Ok(SkillAreaTickPolicy::EveryTick),
        "OncePerArea" => Ok(SkillAreaTickPolicy::OncePerArea),
        "OnEnter" => Ok(SkillAreaTickPolicy::OnEnter),
        other => Err(format!("unsupported area tick policy '{other}'")),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read_skill_file,
            load_default_skills,
            load_skills,
            validate_skills,
            export_generated_skills
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
