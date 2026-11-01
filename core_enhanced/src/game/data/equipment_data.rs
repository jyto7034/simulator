use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::{AbilityActivationBinding, AbilityActivationDef},
    battle::{
        buffs::BuffDatabase, damage::DamageType, tile_range::TileRangePattern,
        types::DeploymentAffinity,
    },
    behavior::GameError,
    data::event_data::EventChoiceKeyId,
    data::{build_string_index, build_uuid_index, skill_data::SkillDatabase},
    employee::EventAptitude,
    enums::RiskLevel,
    stats::{
        classify_triggered_effect_execution, triggered_effect_kind_name, Effect, StatModifier,
        TriggerType, TriggeredEffects,
    },
};

fn default_allow_duplicate_equip() -> bool {
    true
}

fn default_cannot_unequip_reason() -> String {
    "equipment_bound".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeaponRangeRole {
    #[default]
    Melee,
    Ranged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeaponArchetype {
    #[default]
    Sword,
    Spear,
    Shield,
    Bow,
    Gun,
    Staff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetingProfile {
    #[default]
    DefaultForward,
    HighestRouteProgress,
    AirFirst,
    LowDefenseFirst,
    LowMagicResistFirst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponCombatProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_weapon: Option<crate::game::ability::weapon_marks::MarkWeaponDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kill_growth: Option<crate::game::resource::WeaponKillGrowthDef>,
    #[serde(
        default,
        skip_serializing_if = "super::abnormality_data::BasicAttackAction::is_default_damage"
    )]
    pub action: super::abnormality_data::BasicAttackAction,
    #[serde(default, skip_serializing_if = "crate::game::stats::is_zero_u32")]
    pub attack: u32,
    #[serde(default, skip_serializing_if = "crate::game::stats::is_zero_u32")]
    pub spell_power: u32,
    pub range_role: WeaponRangeRole,
    pub weapon_archetype: WeaponArchetype,
    pub deployment_affinity: DeploymentAffinity,
    pub block_capacity: u32,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub ground_defense_focus: Option<GroundDefenseFocusProfile>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub platform_rest: Option<PlatformRestProfile>,
    pub damage_type: DamageType,
    pub targeting_profile: TargetingProfile,
    #[serde(default)]
    pub air_capable: bool,
    pub defense_tile_range: TileRangePattern,
    #[serde(default = "default_weapon_attack_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_weapon_attack_windup_ms")]
    pub windup_ms: u32,
    #[serde(default = "default_weapon_attack_reposition_ms")]
    pub ranged_reposition_ms: u64,
    #[serde(default = "default_weapon_attack_delivery")]
    pub delivery: crate::game::ability::DeliveryDef,
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundDefenseFocusProfile {
    pub block_capacity_bonus: u32,
    pub incoming_damage_reduction_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformRestProfile {
    pub hp_recovery_per_tick: u32,
}

impl Default for WeaponCombatProfile {
    fn default() -> Self {
        Self {
            mark_weapon: None,
            kill_growth: None,
            action: Default::default(),
            attack: 0,
            spell_power: 0,
            range_role: WeaponRangeRole::Melee,
            weapon_archetype: WeaponArchetype::Sword,
            deployment_affinity: DeploymentAffinity::GroundOnly,
            block_capacity: 0,
            ground_defense_focus: Some(GroundDefenseFocusProfile {
                block_capacity_bonus: 0,
                incoming_damage_reduction_percent: 0,
            }),
            platform_rest: None,
            damage_type: DamageType::Physical,
            targeting_profile: TargetingProfile::DefaultForward,
            air_capable: false,
            defense_tile_range: TileRangePattern {
                include_anchor_tile: false,
                rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
            },
            interval_ms: default_weapon_attack_interval_ms(),
            windup_ms: default_weapon_attack_windup_ms(),
            ranged_reposition_ms: default_weapon_attack_reposition_ms(),
            delivery: default_weapon_attack_delivery(),
        }
    }
}

fn default_weapon_attack_interval_ms() -> u64 {
    1500
}

fn default_weapon_attack_windup_ms() -> u32 {
    crate::game::data::abnormality_data::DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS
}

fn default_weapon_attack_reposition_ms() -> u64 {
    1000
}

fn default_weapon_attack_delivery() -> crate::game::ability::DeliveryDef {
    crate::game::ability::DeliveryDef::Instant
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentMaterialType {
    EquipmentDust,
    AbnormalityPart,
    RevivalCatalyst,
    Fragment,
    Core,
    Blueprint,
    Residue,
    Generic,
}

impl EquipmentMaterialType {
    pub fn is_storable(self) -> bool { matches!(self, Self::AbnormalityPart | Self::RevivalCatalyst) }
}

pub type EquipmentItem = EquipmentMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventCheckModifierDefinition {
    pub aptitude: EventAptitude,
    pub amount: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentMetadata {
    /// Explicit source-family identity used by source-scoped combat rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_family: Option<String>,
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub equipment_type: EquipmentType,
    pub rarity: RiskLevel,
    pub price: u32,
    #[serde(default = "default_allow_duplicate_equip")]
    pub allow_duplicate_equip: bool,
    #[serde(default)]
    pub bound: bool,
    #[serde(default = "default_cannot_unequip_reason")]
    pub cannot_unequip_reason: String,
    /// 트리거 기반 효과 (Permanent = 상시 적용)
    #[serde(default)]
    pub triggered_effects: TriggeredEffects,
    /// 장기적으로 사용하는 proc/activation 기반 ability 연결
    #[serde(default)]
    pub ability_activations: Vec<AbilityActivationBinding>,
    #[serde(default)]
    pub weapon_profile: Option<WeaponCombatProfile>,
    #[serde(default)]
    pub event_check_modifiers: Vec<EventCheckModifierDefinition>,
    #[serde(default)]
    pub event_choice_keys: Vec<EventChoiceKeyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentMaterialMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub material_type: EquipmentMaterialType,
    pub rarity: RiskLevel,
    #[serde(default)]
    pub equipment_type: Option<EquipmentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentMaterialCost {
    pub material_id: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentDismantleRecipeMetadata {
    pub equipment_id: String,
    pub yields: Vec<EquipmentMaterialCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentEnhancementRecipeMetadata {
    pub equipment_id: String,
    pub max_level: u8,
    pub costs_per_level: Vec<EquipmentMaterialCost>,
    #[serde(default)]
    pub modifiers_per_level: Vec<StatModifier>,
}

impl EquipmentEnhancementRecipeMetadata {
    pub(crate) fn aggregated_costs(&self) -> Option<Vec<EquipmentMaterialCost>> {
        let mut indexes = HashMap::<&str, usize>::new();
        let mut aggregated = Vec::<EquipmentMaterialCost>::new();
        for cost in &self.costs_per_level {
            if let Some(index) = indexes.get(cost.material_id.as_str()).copied() {
                aggregated[index].amount = aggregated[index].amount.checked_add(cost.amount)?;
            } else {
                indexes.insert(cost.material_id.as_str(), aggregated.len());
                aggregated.push(cost.clone());
            }
        }
        Some(aggregated)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentDatabase {
    pub items: Vec<EquipmentMetadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_attacks: Vec<crate::game::ability::equipment_attack::EquipmentAttackBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_defenses: Vec<crate::game::ability::equipment_defense::EquipmentDefenseBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_supports: Vec<crate::game::ability::equipment_support::EquipmentSupportBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_states: Vec<crate::game::ability::equipment_state::EquipmentStateBinding>,
    #[serde(default)]
    pub materials: Vec<EquipmentMaterialMetadata>,
    #[serde(default)]
    pub dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
    #[serde(default)]
    pub enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_material_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_dismantle_equipment_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_enhancement_equipment_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl EquipmentDatabase {
    pub fn new(items: Vec<EquipmentMetadata>) -> Self {
        Self::with_materials(items, Vec::new())
    }

    pub fn with_materials(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
    ) -> Self {
        Self::with_materials_and_dismantles(items, materials, Vec::new())
    }

    pub fn with_materials_and_dismantles(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
    ) -> Self {
        Self::with_all(items, materials, dismantle_recipes, Vec::new())
    }

    pub fn with_all(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
        enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
    ) -> Self {
        Self {
            items,
            equipment_attacks: Vec::new(),
            equipment_defenses: Vec::new(),
            equipment_supports: Vec::new(),
            equipment_states: Vec::new(),
            materials,
            dismantle_recipes,
            enhancement_recipes,
            by_id: OnceLock::new(),
            by_material_id: OnceLock::new(),
            by_dismantle_equipment_id: OnceLock::new(),
            by_enhancement_equipment_id: OnceLock::new(),
            by_uuid: OnceLock::new(),
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.items, "equipment id", |item| &item.id))
    }

    fn by_material_id(&self) -> &HashMap<String, usize> {
        self.by_material_id.get_or_init(|| {
            build_string_index(&self.materials, "equipment material id", |item| &item.id)
        })
    }

    fn by_dismantle_equipment_id(&self) -> &HashMap<String, usize> {
        self.by_dismantle_equipment_id.get_or_init(|| {
            build_string_index(
                &self.dismantle_recipes,
                "equipment dismantle recipe equipment id",
                |recipe| &recipe.equipment_id,
            )
        })
    }

    fn by_enhancement_equipment_id(&self) -> &HashMap<String, usize> {
        self.by_enhancement_equipment_id.get_or_init(|| {
            build_string_index(
                &self.enhancement_recipes,
                "equipment enhancement recipe equipment id",
                |recipe| &recipe.equipment_id,
            )
        })
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.items, "equipment uuid", |item| item.uuid))
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .expect("invalid equipment authoring contract");
        let _ = self.by_id();
        let _ = self.by_material_id();
        let _ = self.by_dismantle_equipment_id();
        let _ = self.by_enhancement_equipment_id();
        let _ = self.by_uuid();
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for item in &self.items {
            if item.content_family.as_ref().is_some_and(|family|family.trim().is_empty()) {return Err(crate::game::data::DataValidationError::at(&item.id,"content_family","content family must be nonempty"));}
        }
        for (index, material) in self.materials.iter().enumerate() {
            if material.id == "equipment_dust" && material.material_type != EquipmentMaterialType::EquipmentDust {
                return Err(crate::game::data::DataValidationError::at(&material.id,
                    format!("materials[{index}].material_type"), "reserved equipment_dust requires EquipmentDust purpose"));
            }
        }
        self.validate_weapon_profiles()?;
        self.validate_event_check_metadata()?;
        self.validate_dismantle_recipe_references()?;
        self.validate_enhancement_recipe_references()
    }

    pub(crate) fn validate_effect_and_activation_contract(
        &self,
        skill_data: &SkillDatabase,
        buff_data: &BuffDatabase,
    ) -> Result<(), crate::game::data::DataValidationError> {
        const TRIGGERS: [TriggerType; 7] = [
            TriggerType::Permanent,
            TriggerType::OnBattleStart,
            TriggerType::OnAttack,
            TriggerType::OnHit,
            TriggerType::OnDeath,
            TriggerType::OnKill,
            TriggerType::OnAllyDeath,
        ];

        for (item_index, item) in self.items.iter().enumerate() {
            for trigger in TRIGGERS {
                let Some(effects) = item.triggered_effects.get(&trigger) else {
                    continue;
                };
                for (effect_index, triggered_effect) in effects.iter().enumerate() {
                    let effect_path = format!(
                        "items[{item_index}].triggered_effects.{trigger:?}[{effect_index}]"
                    );
                    if classify_triggered_effect_execution(
                        trigger,
                        triggered_effect.target(),
                        triggered_effect.effect(),
                    )
                    .is_none()
                    {
                        return Err(crate::game::data::DataValidationError::at(
                            &item.id,
                            effect_path,
                            format!(
                                "unsupported equipment trigger effect: trigger={trigger:?}, target={:?}, effect={}",
                                triggered_effect.target(),
                                triggered_effect_kind_name(triggered_effect.effect())
                            ),
                        ));
                    }
                    if let Effect::BasicLifesteal { percent, max_heal } = triggered_effect.effect()
                    {
                        if item.equipment_type != EquipmentType::Weapon
                            || *percent == 0
                            || *percent > 100
                            || *max_heal == 0
                            || *max_heal > i32::MAX as u32
                        {
                            return Err(crate::game::data::DataValidationError::at(&item.id, effect_path, "BasicLifesteal requires a weapon, percent 1..100 and a positive signed-range per-attack max_heal"));
                        }
                    }
                    if let Effect::ApplyBuff { buff_id, .. } = triggered_effect.effect() {
                        if !buff_data.contains_name(buff_id) {
                            return Err(crate::game::data::DataValidationError::at(
                                &item.id,
                                format!("{effect_path}.effect.ApplyBuff.buff_id"),
                                format!("references unknown equipment effect buff '{buff_id}'"),
                            ));
                        }
                    }
                }
            }

            for (activation_index, binding) in item.ability_activations.iter().enumerate() {
                let AbilityActivationDef::TriggerProc { trigger, .. } = binding.activation;
                let activation_path =
                    format!("items[{item_index}].ability_activations[{activation_index}]");
                if trigger == TriggerType::Permanent {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("{activation_path}.activation.TriggerProc.trigger"),
                        "unsupported equipment ability activation trigger Permanent",
                    ));
                }
                if skill_data.get_by_id(&binding.ability_id).is_none() {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("{activation_path}.ability_id"),
                        format!(
                            "references unknown equipment activation skill '{}'",
                            binding.ability_id
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_event_check_metadata(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (item_index, item) in self.items.iter().enumerate() {
            let mut seen_aptitudes = HashSet::new();
            for (modifier_index, modifier) in item.event_check_modifiers.iter().enumerate() {
                let path = format!("items[{item_index}].event_check_modifiers[{modifier_index}]");
                if !matches!(modifier.amount, 1 | 2) {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("{path}.amount"),
                        "event check modifier amount must be 1 or 2",
                    ));
                }
                if !seen_aptitudes.insert(modifier.aptitude) {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("{path}.aptitude"),
                        "duplicate event check modifier aptitude",
                    ));
                }
            }

            let mut seen_keys = HashSet::new();
            for (key_index, key) in item.event_choice_keys.iter().enumerate() {
                let path = format!("items[{item_index}].event_choice_keys[{key_index}]");
                if !key.is_valid() {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        path,
                        "event choice key must be a non-empty lower_snake_case identifier",
                    ));
                }
                if !seen_keys.insert(key.as_str()) {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        path,
                        "duplicate event choice key",
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_weapon_profiles(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (item_index, item) in self.items.iter().enumerate() {
            let profile_path = format!("items[{item_index}].weapon_profile");
            match (item.equipment_type, item.weapon_profile.as_ref()) {
                (EquipmentType::Weapon, Some(profile)) => {
                    profile
                        .try_validate_runtime_contract(&item.id)
                        .map_err(|error| {
                            crate::game::data::DataValidationError::at(
                                &item.id,
                                format!("{profile_path}.{}", error.field_suffix),
                                error.cause,
                            )
                        })?;
                }
                (EquipmentType::Weapon, None) => {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        profile_path,
                        "weapon equipment is missing weapon_profile",
                    ));
                }
                (_, Some(_)) => {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        profile_path,
                        "non-weapon equipment must not define weapon_profile",
                    ));
                }
                (_, None) => {}
            }
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&EquipmentMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.items.get(index))
    }

    pub fn get_material_by_id(&self, id: &str) -> Option<&EquipmentMaterialMetadata> {
        self.by_material_id()
            .get(id)
            .and_then(|&index| self.materials.get(index))
    }

    pub fn get_dismantle_recipe_by_equipment_id(
        &self,
        equipment_id: &str,
    ) -> Option<&EquipmentDismantleRecipeMetadata> {
        self.by_dismantle_equipment_id()
            .get(equipment_id)
            .and_then(|&index| self.dismantle_recipes.get(index))
    }

    pub fn get_enhancement_recipe_by_equipment_id(
        &self,
        equipment_id: &str,
    ) -> Option<&EquipmentEnhancementRecipeMetadata> {
        self.by_enhancement_equipment_id()
            .get(equipment_id)
            .and_then(|&index| self.enhancement_recipes.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&EquipmentMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.items.get(index))
    }

    fn validate_dismantle_recipe_references(
        &self,
    ) -> Result<(), crate::game::data::DataValidationError> {
        let mut equipment_ids = HashSet::new();
        for (recipe_index, recipe) in self.dismantle_recipes.iter().enumerate() {
            let recipe_path = format!("dismantle_recipes[{recipe_index}]");
            if !equipment_ids.insert(recipe.equipment_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "duplicate dismantle recipe equipment_id",
                ));
            }
            if !self.items.iter().any(|item| item.id == recipe.equipment_id) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "references missing equipment",
                ));
            }

            if recipe.yields.is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.yields"),
                    "must have at least one yield",
                ));
            }

            for (yield_index, material) in recipe.yields.iter().enumerate() {
                let yield_path = format!("{recipe_path}.yields[{yield_index}]");
                if material.amount == 0 {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{yield_path}.amount"),
                        "yield amount must be greater than zero",
                    ));
                }
                if !self
                    .materials
                    .iter()
                    .any(|candidate| candidate.id == material.material_id)
                {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{yield_path}.material_id"),
                        format!("references missing material '{}'", material.material_id),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_enhancement_recipe_references(
        &self,
    ) -> Result<(), crate::game::data::DataValidationError> {
        let mut equipment_ids = HashSet::new();
        for (recipe_index, recipe) in self.enhancement_recipes.iter().enumerate() {
            let recipe_path = format!("enhancement_recipes[{recipe_index}]");
            if !equipment_ids.insert(recipe.equipment_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "duplicate enhancement recipe equipment_id",
                ));
            }
            if !self.items.iter().any(|item| item.id == recipe.equipment_id) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "references missing equipment",
                ));
            }
            if self.get_by_id(&recipe.equipment_id).is_none_or(|item| item.equipment_type != EquipmentType::Weapon)
                || recipe.costs_per_level.iter().any(|cost| self.get_material_by_id(&cost.material_id).is_none_or(|m| m.material_type != EquipmentMaterialType::EquipmentDust))
                || recipe.modifiers_per_level.iter().any(|m| !matches!(m.stat, crate::game::stats::StatId::Attack | crate::game::stats::StatId::SpellPower)) {
                return Err(crate::game::data::DataValidationError::at(&recipe.equipment_id, &recipe_path, "enhancement requires a weapon, equipment dust and attack/spell power modifiers"));
            }
            if recipe.max_level == 0 {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.max_level"),
                    "max_level must be greater than zero",
                ));
            }
            if recipe.costs_per_level.is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.costs_per_level"),
                    "must have at least one cost",
                ));
            }
            for (cost_index, cost) in recipe.costs_per_level.iter().enumerate() {
                let cost_path = format!("{recipe_path}.costs_per_level[{cost_index}]");
                if cost.amount == 0 {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{cost_path}.amount"),
                        "cost amount must be greater than zero",
                    ));
                }
                if !self
                    .materials
                    .iter()
                    .any(|candidate| candidate.id == cost.material_id)
                {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{cost_path}.material_id"),
                        format!("references missing material '{}'", cost.material_id),
                    ));
                }
            }
            if recipe.aggregated_costs().is_none() {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.costs_per_level"),
                    "aggregate cost amount overflow",
                ));
            }
        }

        Ok(())
    }
}

impl WeaponCombatProfile {
    pub(crate) fn validate_runtime_contract(&self, owner_label: &str) -> Result<(), GameError> {
        self.try_validate_runtime_contract(owner_label)
            .map_err(|error| GameError::InvalidStaticData(error.cause))
    }

    pub(crate) fn try_validate_runtime_contract(
        &self,
        owner_label: &str,
    ) -> Result<(), WeaponProfileValidationError> {
        if let Some(marks) = &self.mark_weapon {
            marks.validate().map_err(|cause| WeaponProfileValidationError::new("mark_weapon", cause))?;
        }
        if let Some(growth) = &self.kill_growth {
            growth.validate().map_err(|cause| WeaponProfileValidationError::new("kill_growth", cause))?;
        }
        self.action
            .validate_delivery(&self.delivery)
            .map_err(|error| WeaponProfileValidationError::new(error.field_suffix, error.cause))?;
        match self.deployment_affinity {
            DeploymentAffinity::GroundOnly => {
                if self.ground_defense_focus.is_none() || self.platform_rest.is_some() {
                    return Err(WeaponProfileValidationError::new(
                        "deployment_affinity",
                        format!(
                            "weapon '{}' GroundOnly weapon_profile requires ground_defense_focus and forbids platform_rest",
                            owner_label
                        ),
                    ));
                }
            }
            DeploymentAffinity::PlatformOnly => {
                if self.ground_defense_focus.is_some() || self.platform_rest.is_none() {
                    return Err(WeaponProfileValidationError::new(
                        "deployment_affinity",
                        format!(
                            "weapon '{}' PlatformOnly weapon_profile requires platform_rest and forbids ground_defense_focus",
                            owner_label
                        ),
                    ));
                }
                if self.block_capacity != 0 {
                    return Err(WeaponProfileValidationError::new(
                        "block_capacity",
                        format!(
                            "weapon '{}' PlatformOnly weapon_profile block_capacity must be 0",
                            owner_label
                        ),
                    ));
                }
            }
            DeploymentAffinity::Any => {
                if self.ground_defense_focus.is_none() || self.platform_rest.is_none() {
                    return Err(WeaponProfileValidationError::new(
                        "deployment_affinity",
                        format!(
                            "weapon '{}' Any weapon_profile requires both ground_defense_focus and platform_rest",
                            owner_label
                        ),
                    ));
                }
            }
        }
        if self
            .ground_defense_focus
            .is_some_and(|profile| profile.incoming_damage_reduction_percent > 100)
        {
            return Err(WeaponProfileValidationError::new(
                "ground_defense_focus.incoming_damage_reduction_percent",
                format!(
                    "weapon '{}' weapon_profile ground_defense_focus incoming_damage_reduction_percent must be <= 100",
                    owner_label
                ),
            ));
        }
        if self.interval_ms == 0 {
            return Err(WeaponProfileValidationError::new(
                "interval_ms",
                format!(
                    "weapon '{}' weapon_profile interval_ms must be > 0",
                    owner_label
                ),
            ));
        }
        self.defense_tile_range.validate().map_err(|error| {
            WeaponProfileValidationError::new(
                "defense_tile_range",
                format!(
                    "weapon '{}' weapon_profile has invalid defense_tile_range: {}",
                    owner_label, error
                ),
            )
        })?;
        (if matches!(self.delivery, crate::game::ability::DeliveryDef::Projectile {
            hit_policy: crate::game::ability::ProjectileHitPolicy::DirectionalCollision, .. }) {
            self.delivery.try_validate_directional_basic_attack_contract()
        } else { self.delivery.try_validate_basic_attack_consumer_contract() })
            .map_err(|error| {
                WeaponProfileValidationError::new(
                    format!("delivery.{}", error.field_suffix),
                    error.cause,
                )
            })?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeaponProfileValidationError {
    pub field_suffix: String,
    pub cause: String,
}

impl WeaponProfileValidationError {
    fn new(field_suffix: impl Into<String>, cause: impl Into<String>) -> Self {
        Self {
            field_suffix: field_suffix.into(),
            cause: cause.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EquipmentDatabase, PlatformRestProfile, WeaponArchetype, WeaponCombatProfile};
    use crate::game::battle::types::DeploymentAffinity;
    use ron::de::from_str;

    #[test]
    fn removed_combination_recipes_field_is_rejected() {
        let error = from_str::<EquipmentDatabase>("(items: [], recipes: [])")
            .expect_err("removed equipment combination recipe schema must fail closed");

        assert!(error.to_string().contains("recipes"));
    }

    #[test]
    fn weapon_profile_schema_rejects_removed_range_units() {
        let error = from_str::<WeaponCombatProfile>("(range_units: 1.0)")
            .expect_err("removed scalar range must fail closed");

        assert!(error.to_string().contains("range_units"));
    }

    #[test]
    fn weapon_tactical_profile_accepts_explicit_zero_and_rejects_tile_mismatches() {
        let ground = WeaponCombatProfile::default();
        assert!(ground.validate_runtime_contract("ground_zero").is_ok());

        let platform = WeaponCombatProfile {
            deployment_affinity: DeploymentAffinity::PlatformOnly,
            block_capacity: 0,
            ground_defense_focus: None,
            platform_rest: Some(PlatformRestProfile {
                hp_recovery_per_tick: 0,
            }),
            ..WeaponCombatProfile::default()
        };
        assert!(platform.validate_runtime_contract("platform_zero").is_ok());

        let mut invalid_platform_block = platform.clone();
        invalid_platform_block.block_capacity = 1;
        assert!(invalid_platform_block
            .validate_runtime_contract("platform_block")
            .is_err());

        let mut invalid_ground_rest = WeaponCombatProfile::default();
        invalid_ground_rest.platform_rest = platform.platform_rest;
        assert!(invalid_ground_rest
            .validate_runtime_contract("ground_rest")
            .is_err());

        let mut invalid_any = WeaponCombatProfile::default();
        invalid_any.deployment_affinity = DeploymentAffinity::Any;
        assert!(invalid_any
            .validate_runtime_contract("any_missing")
            .is_err());
    }

    #[test]
    fn removed_grenade_launcher_weapon_archetype_is_rejected() {
        let error = from_str::<WeaponArchetype>("GrenadeLauncher")
            .expect_err("removed grenade launcher schema must fail closed");

        assert!(error.to_string().contains("GrenadeLauncher"));
    }

    #[test]
    fn duplicate_enhancement_material_total_overflow_is_rejected_at_publication() {
        let mut database = from_str::<EquipmentDatabase>(
            r#"EquipmentDatabase(
                items: [(
                    id: "overflow_accessory",
                    uuid: "00000000-0000-0000-0000-000000000011",
                    name: "Overflow Accessory",
                    equipment_type: Weapon,
                    rarity: ZAYIN,
                    price: 1,
                )],
                materials: [(
                    id: "overflow_dust",
                    uuid: "00000000-0000-0000-0000-000000000012",
                    name: "Overflow Dust",
                    description: "Overflow fixture",
                    material_type: EquipmentDust,
                    rarity: ZAYIN,
                    equipment_type: Some(Weapon),
                )],
                enhancement_recipes: [(
                    equipment_id: "overflow_accessory",
                    max_level: 1,
                    costs_per_level: [
                        (material_id: "overflow_dust", amount: 4294967295),
                        (material_id: "overflow_dust", amount: 1),
                    ],
                )],
            )"#,
        )
        .expect("overflow fixture should deserialize before contract validation");

        database.items[0].weapon_profile = Some(WeaponCombatProfile::default());
        let error = database
            .validate_contract()
            .expect_err("duplicate aggregate overflow must fail publication");
        assert_eq!(error.object_id.as_deref(), Some("overflow_accessory"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("enhancement_recipes[0].costs_per_level")
        );
        assert_eq!(error.cause, "aggregate cost amount overflow");
    }

    fn event_metadata_database(metadata: &str) -> EquipmentDatabase {
        from_str::<EquipmentDatabase>(&format!(
            r#"EquipmentDatabase(items: [(
                id: "event_tool",
                uuid: "00000000-0000-0000-0000-000000000001",
                name: "Event Tool",
                equipment_type: Accessory,
                rarity: ZAYIN,
                price: 1,
                {metadata}
            )])"#
        ))
        .expect("event equipment fixture should deserialize")
    }

    #[test]
    fn event_check_modifier_accepts_only_one_or_two() {
        let database =
            event_metadata_database("event_check_modifiers: [(aptitude: Analysis, amount: 3)],");

        let error = database
            .validate_contract()
            .expect_err("out-of-policy modifier must fail closed");
        assert!(error.cause.contains("amount must be 1 or 2"));
    }

    #[test]
    fn duplicate_event_modifier_aptitude_is_rejected() {
        let database = event_metadata_database(
            "event_check_modifiers: [(aptitude: Analysis, amount: 1), (aptitude: Analysis, amount: 2)],",
        );

        let error = database
            .validate_contract()
            .expect_err("duplicate aptitude must fail closed");
        assert!(error
            .cause
            .contains("duplicate event check modifier aptitude"));
    }

    #[test]
    fn duplicate_or_invalid_event_choice_keys_are_rejected() {
        let duplicate =
            event_metadata_database("event_choice_keys: [\"sealed_archive\", \"sealed_archive\"],");
        let duplicate_error = duplicate
            .validate_contract()
            .expect_err("duplicate key must fail closed");
        assert!(duplicate_error.cause.contains("duplicate event choice key"));

        let invalid = event_metadata_database("event_choice_keys: [\"Sealed Archive\"],");
        let invalid_error = invalid
            .validate_contract()
            .expect_err("invalid key must fail closed");
        assert!(invalid_error.cause.contains("lower_snake_case"));
    }

    #[test]
    fn programmatic_game_data_builder_rejects_invalid_equipment_effect_edges() {
        let database = from_str::<EquipmentDatabase>(
            r#"EquipmentDatabase(items: [(
                id: "invalid_effect_accessory",
                uuid: "00000000-0000-0000-0000-000000000021",
                name: "Invalid Effect Accessory",
                equipment_type: Accessory,
                rarity: ZAYIN,
                price: 1,
                triggered_effects: {
                    OnBattleStart: [(
                        target: SelfUnit,
                        effect: ApplyBuff(
                            buff_id: "__missing_programmatic_buff__",
                            duration_ms: 100,
                        ),
                    )],
                },
            )])"#,
        )
        .expect("invalid reference fixture should deserialize");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::game::data::GameDataBuilder::empty()
                .with_equipment_data(std::sync::Arc::new(database))
                .build()
        }));
        let payload = match result {
            Ok(_) => panic!("programmatic publication must reject invalid equipment"),
            Err(payload) => payload,
        };
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .expect("validation panic should contain a string message");
        assert!(message.contains("invalid Equipment effect or activation contract"));
        assert!(message.contains("invalid_effect_accessory"));
        assert!(message
            .contains("items[0].triggered_effects.OnBattleStart[0].effect.ApplyBuff.buff_id"));
        assert!(message.contains("__missing_programmatic_buff__"));
    }
}
