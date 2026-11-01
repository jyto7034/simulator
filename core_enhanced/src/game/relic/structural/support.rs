//! Provider-authored support conversions. Inventory copies select a closed
//! recipe; the ordinary immutable structural compiler owns its execution.
use super::authoring::{PersonalShieldGrantDef, StructuralRecipeDef};
use crate::game::{
    ability::{sustained::SustainedEffectDef, SkillEffectDef},
    battle::tile_range::TileRangePattern,
    relic::RelicVariant,
    value_expr::AmountExpr,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub(crate) fn pattern_extent(range: &TileRangePattern) -> Option<usize> {
    crate::authoring::control::charge(range.rows.len() as u64, "support_pattern_extent").ok()?;
    let base = (std::mem::size_of::<TileRangePattern>() + 64).checked_add(
        range
            .rows
            .capacity()
            .checked_mul(std::mem::size_of::<String>())?,
    )?;
    range
        .rows
        .iter()
        .try_fold(base, |n, row| n.checked_add(row.capacity()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportPolicyDef {
    /// When absent, use the effect's authored circular radius. A present field
    /// keeps its original unwrapped pattern wire representation, including RON.
    #[serde(default, skip_serializing_if = "Option::is_none", serialize_with = "serialize_pattern", deserialize_with = "deserialize_pattern")]
    pub range: Option<Arc<TileRangePattern>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion: Option<SupportConversionDef>,
    /// Multiple mutually exclusive inventory variants; singular input remains supported.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conversions: Vec<SupportConversionDef>,
}
fn serialize_pattern<S: serde::Serializer>(value: &Option<Arc<TileRangePattern>>, serializer: S) -> Result<S::Ok, S::Error> {
    value.as_ref().ok_or_else(|| serde::ser::Error::custom("absent support pattern must be omitted"))?.serialize(serializer)
}
fn deserialize_pattern<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<Arc<TileRangePattern>>, D::Error> {
    Arc::<TileRangePattern>::deserialize(deserializer).map(Some)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportConversionDef {
    pub relic_definition: String,
    pub variant: RelicVariant,
    pub selection_range: Arc<TileRangePattern>,
    pub replacement: SupportReplacementDef,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SupportReplacementDef {
    PersonalShield {
        amount: Arc<AmountExpr>,
    },
    RegularBasic {
        damage_percent: u32,
        speed_points: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        health_upkeep: Option<crate::game::ability::sustained::ChannelHealthUpkeep>,
    },
}
crate::authoring::clone_shape::struct_shape!(SupportPolicyDef { range, conversion, conversions });
crate::authoring::clone_shape::struct_shape!(SupportConversionDef {
    relic_definition,
    variant,
    selection_range,
    replacement
});
crate::authoring::clone_shape::enum_shape!(SupportReplacementDef {
    units: [],
    tuples: [],
    records: [
        PersonalShield { amount },
        RegularBasic {
            damage_percent,
            speed_points,
            health_upkeep
        }
    ]
});

impl SupportPolicyDef {
    pub fn variants(&self) -> impl Iterator<Item = &SupportConversionDef> {
        self.conversion.iter().chain(self.conversions.iter())
    }
    pub fn validate(&self, shield: bool) -> Result<(), String> {
        if let Some(range) = &self.range { range.validate()?; }
        if self.conversions.len() > 32 { return Err("support allows at most 32 conversion alternatives".into()); }
        for (index, conversion) in self.variants().enumerate() {
            if self.variants().take(index).any(|c| c.relic_definition == conversion.relic_definition && c.variant == conversion.variant) {
                return Err("duplicate support conversion relic and variant".into());
            }
            if !super::validation::valid_id(&conversion.relic_definition) {
                return Err("support conversion requires a bounded relic definition".into());
            }
            conversion.selection_range.validate()?;
            match &conversion.replacement {
                SupportReplacementDef::PersonalShield { .. } if shield => (),
                SupportReplacementDef::RegularBasic {
                    damage_percent,
                    speed_points,
                    health_upkeep,
                } if !shield && *damage_percent <= 10000 && *speed_points <= 10000 => {
                    if let Some(upkeep) = health_upkeep { upkeep.validate()?; }
                },
                _ => return Err("support conversion replacement does not match its effect".into()),
            }
        }
        Ok(())
    }
}
pub(crate) fn policy(effect: &SkillEffectDef) -> Option<&SupportPolicyDef> {
    match effect {
        SkillEffectDef::PersonalShieldGrant { definition } => definition.support.as_ref(),
        SkillEffectDef::Sustained {
            definition: SustainedEffectDef::ChannelAura { aura, .. },
        } => aura.support.as_ref(),
        _ => None,
    }
}

/// Distinct from the support footprint: a converted consumer uses this range
/// for acceptance, then the original effect's persistence policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectedSupportReach {
    Radius(u32),
    Pattern {
        range: Arc<TileRangePattern>,
        end_on_exit: bool,
    },
}
crate::authoring::clone_shape::enum_shape!(SelectedSupportReach {
    units: [],
    tuples: [Radius(value)],
    records: [Pattern { range, end_on_exit }]
});

pub(crate) fn selection_recipe(effect: &super::EffectiveEffect) -> Option<SelectedSupportReach> {
    effect.recipes.iter().find_map(|recipe| match recipe {
        StructuralRecipeDef::SupportSelectionPattern { range, end_on_exit } => {
            Some(SelectedSupportReach::Pattern {
                range: Arc::clone(range),
                end_on_exit: *end_on_exit,
            })
        }
        _ => None,
    })
}

pub(crate) fn shield_reach(definition: &PersonalShieldGrantDef) -> SelectedSupportReach {
    if let Some(range) = definition.support.as_ref().and_then(|s| s.range.as_ref()) {
        return SelectedSupportReach::Pattern {
            range: Arc::clone(range),
            end_on_exit: false,
        };
    }
    use super::authoring::PersonalShieldRecipientsDef::*;
    SelectedSupportReach::Radius(match definition.recipients {
        PlayerEmployeesAtRelease { radius_units } => radius_units,
        SelectedPlayerEmployeeAtRelease {
            authored_reach_units,
        } => authored_reach_units,
    })
}

pub(crate) fn declares_conversion_for(
    data: &crate::game::data::GameDataBase,
    definition: &str,
    variant: RelicVariant,
) -> Result<bool, super::RelicPlanError> {
    for skill in &data.skill_data.skills {
        crate::authoring::control::charge(1, "support_conversion_skill_scan")
            .map_err(super::RelicPlanError::control)?;
        for step in &skill.steps {
            crate::authoring::control::charge(
                step.effects.len() as u64,
                "support_conversion_effect_scan",
            )
            .map_err(super::RelicPlanError::control)?;
            for effect in &step.effects {
                if policy(&effect.op).is_some_and(|p| p.variants()
                    .any(|c| c.relic_definition == definition && c.variant == variant))
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub(crate) fn preview_conversions(
    data: &crate::game::data::GameDataBase,
    definition: &str,
    variant: RelicVariant,
) -> Result<Vec<crate::game::relic::SupportConversionPreviewDto>, crate::game::behavior::GameError> {
    use crate::authoring::clone_shape::controlled_clone;
    let mut result = Vec::new();
    for skill in &data.skill_data.skills {
        crate::authoring::control::charge(1, "support_preview_skill")?;
        for step in &skill.steps {
            crate::authoring::control::charge(step.effects.len() as u64, "support_preview_effects")?;
            for effect in &step.effects {
                let Some(policy) = policy(&effect.op) else { continue };
                crate::authoring::control::charge(policy.variants().count() as u64, "support_preview_variants")?;
                for conversion in policy.variants().filter(|c| c.relic_definition == definition && c.variant == variant) {
                    crate::game::relic::allocation::admit_owned_projection(
                        &(&skill.id, &step.id, &effect.id, conversion), "support_preview_copy")?;
                    crate::game::battle::runtime_control::admit_retained_growth(0,
                        Some(std::mem::size_of::<crate::game::relic::SupportConversionPreviewDto>()), "support_preview_entry")?;
                    result.reserve_exact(1);
                    result.push(crate::game::relic::SupportConversionPreviewDto {
                        skill_id: controlled_clone(&skill.id)?, step_id: controlled_clone(&step.id)?,
                        effect_id: controlled_clone(&effect.id)?, conversion: controlled_clone(conversion)?,
                    });
                }
            }
        }
    }
    Ok(result)
}

pub(crate) fn validate_references(
    data: &crate::game::data::GameDataBase,
) -> Result<(), super::RelicPlanError> {
    crate::authoring::control::charge(data.relic_data.relics.len() as u64, "support_penalty_reference_scan").map_err(super::RelicPlanError::control)?;
    for definition in &data.relic_data.relics {
        if definition.corruption.as_ref().is_some_and(|v| v.penalties.contains(&crate::game::data::relic_data::RelicPenalty::AuthoredSupportUpkeep))
            && !declares_conversion_for(data, &definition.id, RelicVariant::Corrupt)? {
            return Err(super::RelicPlanError::new(super::FaultCode::InvalidReference,
                "authored support upkeep requires a real corrupt support conversion"));
        }
    }
    for skill in &data.skill_data.skills {
        crate::authoring::control::charge(1, "support_conversion_reference_skill")
            .map_err(super::RelicPlanError::control)?;
        for step in &skill.steps {
            crate::authoring::control::charge(
                step.effects.len() as u64,
                "support_conversion_reference_effect",
            )
            .map_err(super::RelicPlanError::control)?;
            for effect in &step.effects {
                let Some(policy) = policy(&effect.op)
                else {
                    continue;
                };
                crate::authoring::control::charge(policy.variants().count() as u64, "support_variant_references").map_err(super::RelicPlanError::control)?;
                for conversion in policy.variants() {
                let definition = data
                    .relic_data
                    .get_by_id(&conversion.relic_definition)
                    .ok_or_else(|| {
                        super::RelicPlanError::new(
                            super::FaultCode::InvalidReference,
                            "support conversion references an unknown relic",
                        )
                    })?;
                if conversion.variant == RelicVariant::Corrupt && definition.corruption.is_none() {
                    return Err(super::RelicPlanError::new(
                        super::FaultCode::InvalidReference,
                        "support conversion references an unavailable corrupt variant",
                    ));
                }
                if conversion.variant == RelicVariant::Corrupt && definition.corruption.as_ref().is_some_and(|v| v.penalties.contains(&crate::game::data::relic_data::RelicPenalty::AuthoredSupportUpkeep))
                    && !matches!(&conversion.replacement, SupportReplacementDef::RegularBasic { health_upkeep: Some(_), .. }) {
                    return Err(super::RelicPlanError::new(super::FaultCode::InvalidReference,
                        "every corrupt conversion for authored support upkeep must pair its benefit with HP upkeep"));
                }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn apply_conversions(
    data: &crate::game::data::GameDataBase,
    copies: &[(
        &crate::game::relic::OwnedRelic,
        &crate::game::data::relic_data::RelicDefinition,
    )],
    effects: &mut [super::SharedEffect],
    work: &mut super::Work,
) -> Result<Vec<super::dto::RelicRuleStateDto>, super::RelicPlanError> {
    use super::{dto::*, EffectBody, PlanContribution};
    use sha2::{Digest, Sha256};
    let mut states = Vec::new();
    for effect in effects {
        work.charge(1, "support_conversion_consumer")?;
        let EffectBody::Original {
            skill_id,
            step,
            effect: index,
        } = &effect.body
        else {
            continue;
        };
        let skill = data.skill_data.get_by_id(skill_id).ok_or_else(|| {
            super::RelicPlanError::new(
                super::FaultCode::InvalidReference,
                "support conversion provider unavailable",
            )
        })?;
        let Some(policy) = policy(&skill.steps[*step].effects[*index].op)
        else {
            continue;
        };
        work.charge(copies.len().checked_mul(policy.variants().count()).ok_or_else(||
            super::RelicPlanError::new(super::FaultCode::InvalidReference, "support variant work overflow"))?, "support_conversion_equipped_copy")?;
        // Copies are sorted by UUID. The authored 'once' policy selects one real
        // equipped copy, with that copy retained in normal contribution receipts.
        let Some((copy, definition, conversion)) = copies.iter().find_map(|(copy, definition)| {
            policy.variants().find(|c| copy.definition_id == c.relic_definition && copy.variant == c.variant)
                .map(|c| (*copy, *definition, c))
        }) else {
            continue;
        };
        super::compiler::admit_owned_copy(conversion, work)?;
        let site_bytes = super::compiler::serialized_size(&effect.key, work)?;
        work.reserve(Some(site_bytes + 128), "support_conversion_identity")?;
        let encoded = serde_json::to_vec(&effect.key).map_err(|e| {
            super::RelicPlanError::new(super::FaultCode::InvalidReference, e.to_string())
        })?;
        let id = format!("support-{:x}", Sha256::digest(encoded));
        if super::validation::rules_for(definition, copy.variant)
            .iter()
            .any(|rule| rule.id == id)
        {
            return Err(super::RelicPlanError::new(
                super::FaultCode::InvalidReference,
                "support conversion contribution ID collides with relic rule",
            ));
        }
        let contribution = PlanContribution {
            copy_uuid: copy.copy_uuid,
            definition_id: definition.id.clone(),
            variant: copy.variant,
            rule_id: id,
        };
        let effect = super::compiler::edit_effect(effect, work)?;
        effect.include_recipe(
            &StructuralRecipeDef::SupportSelectionPattern {
                range: Arc::clone(&conversion.selection_range),
                end_on_exit: matches!(
                    conversion.replacement,
                    SupportReplacementDef::RegularBasic { .. }
                ),
            },
            &contribution,
            work,
        )?;
        match &conversion.replacement {
            SupportReplacementDef::PersonalShield { amount } => effect.include_recipe(
                &StructuralRecipeDef::PersonalShieldSingle {
                    initial_amount: Arc::clone(amount),
                },
                &contribution,
                work,
            )?,
            SupportReplacementDef::RegularBasic {
                damage_percent,
                speed_points,
                health_upkeep,
            } => {
                effect.include_recipe(
                    &StructuralRecipeDef::FixedRecipientSupport,
                    &contribution,
                    work,
                )?;
                if let Some(upkeep) = health_upkeep {
                    effect.include_recipe(&StructuralRecipeDef::ChannelHealthUpkeep { policy: *upkeep }, &contribution, work)?;
                }
                effect.include_recipe(
                    &StructuralRecipeDef::RegularBasicSupport {
                        damage_percent: *damage_percent,
                        speed_points: *speed_points,
                    },
                    &contribution,
                    work,
                )?;
            }
        }
        work.reserve(
            Some(
                std::mem::size_of::<RelicRuleStateDto>()
                    + std::mem::size_of::<EffectiveEffectAddressDto>(),
            ),
            "support_conversion_receipt",
        )?;
        super::compiler::admit_owned_copy(&effect.key, work)?;
        states.push(RelicRuleStateDto {
            contribution,
            state: RelicRuleState::Active,
            targets: vec![EffectiveEffectAddressDto {
                provider_index: effect.provider,
                effect: effect.key.clone(),
            }],
        });
    }
    Ok(states)
}
