//! Immutable authored relics. Mutable copies and loadouts belong to the run.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::battle::damage::DamageType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RelicDamageTarget {
    #[default]
    AnyDamage,
    SkillDamage,
    BasicAttackDamage,
}

/// Every field is an AND condition on one actual effect/provider.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicDamageSelector {
    #[serde(default)]
    pub target: RelicDamageTarget,
    #[serde(default)]
    pub all_tags: Vec<String>,
    #[serde(default)]
    pub damage_type: Option<DamageType>,
    #[serde(default)]
    pub equipment_id: Option<String>,
    #[serde(default)]
    pub fragment_id: Option<String>,
    #[serde(default)]
    pub form_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicDamageEffect {
    pub id: String,
    #[serde(default)]
    pub selector: RelicDamageSelector,
    #[serde(default)]
    pub flat: i64,
    #[serde(default)]
    pub percent: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RelicPenalty {
    DefenseFlat { amount: i32 },
    MagicResistFlat { amount: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicCorruptionVariant {
    pub effects: Vec<RelicDamageEffect>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<crate::game::relic::structural::authoring::RelicRuleDef>,
    pub penalties: Vec<RelicPenalty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicDefinition {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub rarity: String,
    pub allow_duplicates: bool,
    #[serde(default)]
    pub incompatible_with: Vec<String>,
    pub effects: Vec<RelicDamageEffect>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<crate::game::relic::structural::authoring::RelicRuleDef>,
    #[serde(default)]
    pub corruption: Option<RelicCorruptionVariant>,
    pub dismantle_equipment_dust: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicDatabase {
    pub relics: Vec<RelicDefinition>,
}

crate::authoring::clone_shape::copy_shape!(RelicDamageTarget, RelicPenalty);
crate::authoring::clone_shape::struct_shape!(RelicDamageSelector {
    target,
    all_tags,
    damage_type,
    equipment_id,
    fragment_id,
    form_id
});
crate::authoring::clone_shape::struct_shape!(RelicDamageEffect {
    id,
    selector,
    flat,
    percent
});
crate::authoring::clone_shape::struct_shape!(RelicCorruptionVariant {
    effects,
    rules,
    penalties
});
crate::authoring::clone_shape::struct_shape!(RelicDefinition {
    id,
    uuid,
    name,
    description,
    rarity,
    allow_duplicates,
    incompatible_with,
    effects,
    rules,
    corruption,
    dismantle_equipment_dust
});
crate::authoring::clone_shape::struct_shape!(RelicDatabase { relics });

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

impl RelicDatabase {
    pub fn get_by_id(&self, id: &str) -> Option<&RelicDefinition> {
        self.relics.iter().find(|definition| definition.id == id)
    }

    pub(crate) fn validate_references(
        &self,
        equipment: &super::equipment_data::EquipmentDatabase,
        fragments: &super::skill_fragment_data::SkillFragmentDatabase,
    ) -> Result<(), String> {
        for definition in &self.relics {
            for effect in definition.effects.iter().chain(
                definition
                    .corruption
                    .iter()
                    .flat_map(|variant| variant.effects.iter()),
            ) {
                crate::authoring::control::charge(1, "relic_provider_reference")
                    .map_err(|error| format!("{error:?}"))?;
                let selector = &effect.selector;
                if let Some(id) = &selector.equipment_id {
                    if equipment.get_by_id(id).is_none() {
                        return Err(format!(
                            "relic '{}' effect '{}' references unknown equipment '{id}'",
                            definition.id, effect.id
                        ));
                    }
                }
                if let Some(id) = &selector.fragment_id {
                    let fragment = fragments
                        .get_by_id(&super::skill_fragment_data::SkillFragmentId::from(
                            id.clone(),
                        ))
                        .ok_or_else(|| {
                            format!(
                                "relic '{}' effect '{}' references unknown fragment '{id}'",
                                definition.id, effect.id
                            )
                        })?;
                    if let Some(form_id) = &selector.form_id {
                        if !matches!(&fragment.effect, super::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { forms, .. } if forms.contains_key(form_id))
                        {
                            return Err(format!("relic '{}' effect '{}' references unknown fragment form '{form_id}'", definition.id, effect.id));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Run before publishing winners/indexes. No unsupported recipe is decoded
    /// as a generic unconditional numeric effect.
    pub(crate) fn validate_contract(&self) -> Result<(), String> {
        for (index, definition) in self.relics.iter().enumerate() {
            crate::authoring::control::charge(1, "relic_definition_validation")
                .map_err(|error| format!("{error:?}"))?;
            let fail = |field: &str, cause: &str| format!("relics[{index}].{field}: {cause}");
            if definition.dismantle_equipment_dust == 0 {
                return Err(fail("dismantle_equipment_dust", "must be nonzero"));
            }
            if !valid_id(&definition.id) || definition.uuid.is_nil() {
                return Err(fail("id", "invalid definition identity"));
            }
            let duplicate_work = index
                .checked_mul(definition.id.len() + std::mem::size_of::<Uuid>())
                .and_then(|n| n.checked_add(definition.name.len()))
                .and_then(|n| n.checked_add(definition.rarity.len()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or("relic definition validation accounting overflow")?;
            crate::authoring::control::charge(duplicate_work, "relic_definition_identity_scan")
                .map_err(|error| format!("{error:?}"))?;
            if self.relics[..index]
                .iter()
                .any(|other| other.id == definition.id || other.uuid == definition.uuid)
            {
                return Err(fail("id", "duplicate definition identity"));
            }
            if definition.name.trim().is_empty() || !valid_id(&definition.rarity) {
                return Err(fail("name", "missing name or invalid rarity"));
            }
            for (i, other) in definition.incompatible_with.iter().enumerate() {
                let work = self
                    .relics
                    .len()
                    .checked_add(i)
                    .and_then(|n| n.checked_mul(other.len()))
                    .and_then(|n| u64::try_from(n).ok())
                    .ok_or("relic incompatibility accounting overflow")?;
                crate::authoring::control::charge(work, "relic_incompatibility_reference")
                    .map_err(|error| format!("{error:?}"))?;
                if other == &definition.id
                    || self.get_by_id(other).is_none()
                    || definition.incompatible_with[..i].contains(other)
                {
                    return Err(fail(
                        "incompatible_with",
                        "invalid or repeated relic reference",
                    ));
                }
            }
            if definition.effects.is_empty() && definition.rules.is_empty() {
                return Err(fail(
                    "effects/rules",
                    "at least one numeric effect or structural rule is required",
                ));
            }
            if !definition.effects.is_empty() {
                Self::validate_effects(&definition.effects)
                    .map_err(|cause| fail("effects", &cause))?;
            }
            crate::game::relic::structural::validate_rules(&definition.id, &definition.rules)
                .map_err(|cause| fail("rules", &cause.to_string()))?;
            if let Some(variant) = &definition.corruption {
                if variant.effects.is_empty() && variant.rules.is_empty() {
                    return Err(fail(
                        "corruption.effects/rules",
                        "at least one effect or rule is required",
                    ));
                }
                if !variant.effects.is_empty() {
                    Self::validate_effects(&variant.effects)
                        .map_err(|cause| fail("corruption.effects", &cause))?;
                }
                crate::game::relic::structural::validate_rules(&definition.id, &variant.rules)
                    .map_err(|cause| fail("corruption.rules", &cause.to_string()))?;
                if variant.penalties.is_empty()
                    || variant.penalties.iter().any(|penalty| match penalty {
                        RelicPenalty::DefenseFlat { amount }
                        | RelicPenalty::MagicResistFlat { amount } => *amount >= 0,
                    })
                {
                    return Err(fail(
                        "corruption.penalties",
                        "a corruption requires authored negative flat penalties",
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_effects(effects: &[RelicDamageEffect]) -> Result<(), String> {
        if effects.is_empty() {
            return Err("at least one numeric effect is required".into());
        }
        for (index, effect) in effects.iter().enumerate() {
            crate::authoring::control::charge(1, "relic_effect_validation")
                .map_err(|error| format!("{error:?}"))?;
            let work = index
                .checked_add(1)
                .and_then(|n| n.checked_mul(effect.id.len()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or("relic effect identity accounting overflow")?;
            crate::authoring::control::charge(work, "relic_effect_identity_scan")
                .map_err(|error| format!("{error:?}"))?;
            if !valid_id(&effect.id) || effects[..index].iter().any(|other| other.id == effect.id) {
                return Err(format!(
                    "[{index}].id: invalid or duplicate effect identity"
                ));
            }
            if effect.flat < 0 || effect.percent < 0 || (effect.flat == 0 && effect.percent == 0) {
                return Err(format!(
                    "[{index}]: damage amplification must be nonnegative and nonzero"
                ));
            }
            let selector = &effect.selector;
            if selector.form_id.is_some() && selector.fragment_id.is_none() {
                return Err(format!(
                    "[{index}].selector.form_id: requires an exact fragment"
                ));
            }
            for id in selector
                .equipment_id
                .iter()
                .chain(&selector.fragment_id)
                .chain(&selector.form_id)
            {
                crate::authoring::control::charge(id.len() as u64, "relic_selector_identity")
                    .map_err(|error| format!("{error:?}"))?;
                if !valid_id(id) {
                    return Err(format!("[{index}].selector: invalid provider identity"));
                }
            }
            for (tag_index, tag) in selector.all_tags.iter().enumerate() {
                let work = tag_index
                    .checked_add(1)
                    .and_then(|n| n.checked_mul(tag.len()))
                    .and_then(|n| u64::try_from(n).ok())
                    .ok_or("relic tag validation accounting overflow")?;
                crate::authoring::control::charge(work, "relic_selector_tag_scan")
                    .map_err(|error| format!("{error:?}"))?;
                if !valid_id(tag) || selector.all_tags[..tag_index].contains(tag) {
                    return Err(format!(
                        "[{index}].selector.all_tags: invalid or repeated tag"
                    ));
                }
            }
        }
        Ok(())
    }
}

impl RelicDamageEffect {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = std::mem::size_of::<Self>()
            .checked_add(self.id.len())?
            .checked_add(
                self.selector
                    .all_tags
                    .len()
                    .checked_mul(std::mem::size_of::<String>())?,
            )?;
        for tag in &self.selector.all_tags {
            bytes = bytes.checked_add(tag.len())?;
        }
        for id in [
            &self.selector.equipment_id,
            &self.selector.fragment_id,
            &self.selector.form_id,
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.checked_add(id.len())?;
        }
        Some(bytes)
    }
}
