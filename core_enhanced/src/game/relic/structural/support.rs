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
    pub range: Arc<TileRangePattern>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion: Option<SupportConversionDef>,
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
    },
}
crate::authoring::clone_shape::struct_shape!(SupportPolicyDef { range, conversion });
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
            speed_points
        }
    ]
});

impl SupportPolicyDef {
    pub fn validate(&self, shield: bool) -> Result<(), String> {
        self.range.validate()?;
        if let Some(conversion) = &self.conversion {
            if !super::validation::valid_id(&conversion.relic_definition) {
                return Err("support conversion requires a bounded relic definition".into());
            }
            conversion.selection_range.validate()?;
            match &conversion.replacement {
                SupportReplacementDef::PersonalShield { .. } if shield => (),
                SupportReplacementDef::RegularBasic {
                    damage_percent,
                    speed_points,
                } if !shield && *damage_percent <= 10000 && *speed_points <= 10000 => (),
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
    if let Some(support) = &definition.support {
        return SelectedSupportReach::Pattern {
            range: Arc::clone(&support.range),
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
                if policy(&effect.op)
                    .and_then(|p| p.conversion.as_ref())
                    .is_some_and(|c| c.relic_definition == definition && c.variant == variant)
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub(crate) fn validate_references(
    data: &crate::game::data::GameDataBase,
) -> Result<(), super::RelicPlanError> {
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
                let Some(conversion) = policy(&effect.op).and_then(|p| p.conversion.as_ref())
                else {
                    continue;
                };
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
    effects: &mut [super::EffectiveEffect],
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
        let Some(conversion) =
            policy(&skill.steps[*step].effects[*index].op).and_then(|p| p.conversion.as_ref())
        else {
            continue;
        };
        work.charge(copies.len(), "support_conversion_equipped_copy")?;
        // Copies are sorted by UUID. The authored 'once' policy selects one real
        // equipped copy, with that copy retained in normal contribution receipts.
        let Some((copy, definition)) = copies.iter().find(|(copy, _)| {
            copy.definition_id == conversion.relic_definition && copy.variant == conversion.variant
        }) else {
            continue;
        };
        super::compiler::admit_serialized_copy(conversion, work)?;
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
            } => {
                effect.include_recipe(
                    &StructuralRecipeDef::FixedRecipientSupport,
                    &contribution,
                    work,
                )?;
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
        super::compiler::admit_serialized_copy(&effect.key, work)?;
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
