use std::collections::{HashMap, HashSet};

use std::sync::Arc;

use uuid::Uuid;

use crate::game::{
    ability::{DeliveryDef, SkillDef, SkillTarget, SkillTargetReach},
    battle::{buffs::BuffDatabase, types::BattleUnitThreatClass},
    combat_preview::{
        required_briefing_warning_tags_for_spawn_waves, CombatMissionVariant, CombatNodeType,
        CombatPreview, ThreatWarningSource,
    },
    combat_setup::mission_policy::CombatMissionPolicy,
    data::{
        abnormality_data::AbnormalityDatabase,
        boss_omen_data::{BossOmenChainDatabase, BossOmenSourceKind},
        consumable_data::ConsumableDatabase,
        corroded_employee_data::CorrodedEmployeeProfileDatabase,
        corroded_wave_data::CorrodedWavePresetDatabase,
        employee_data::EmployeeDefinitionDatabase,
        equipment_data::{EquipmentDatabase, EquipmentType},
        event_data::{EventDatabase, EventSuccessAction},
        pve_data::{
            PveEncounterClass, PveEncounterDatabase, PveWaveData, PveWaveEnemyData, PveWaveSource,
        },
        reward_data::RewardDatabase,
        shop_data::{ShopDatabase, ShopMerchandise},
        skill_data::SkillDatabase,
        skill_fragment_data::{
            self, SkillFragmentAcquisitionSource, SkillFragmentDependency, SkillFragmentEffectDef,
            SkillFragmentOrigin,
        },
        visual_novel_data::VisualNovelDatabase,
        visual_novel_trigger_data::{
            validate_no_duplicate_run_start_triggers, validate_visual_novel_trigger_references,
            VisualNovelTriggerDatabase,
        },
    },
    events::combat::CombatExecutor,
    map::{
        MapNodeCategory, MapNodeCombatDifficulty, MapNodeCombatDifficultyClass, MapNodeId,
        RevivalStage,
    },
    resources::item_slot::{EquippedRef, ItemSlot},
    reward::{RewardEffect, RewardOption},
    reward_policy::CombatRewardPolicy,
};

use super::GameDataBase;

pub(super) fn validate_pve_reward_source_references(
    pve_data: &PveEncounterDatabase,
    abnormality_data: &AbnormalityDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (source_index, source) in pve_data.reward_sources.iter().enumerate() {
        let source_path = format!("reward_sources[{source_index}]");
        let encounter = pve_data.get_by_id(&source.encounter_id).ok_or_else(|| {
            crate::game::data::DataValidationError::at(
                &source.reward_source_id,
                format!("{source_path}.encounter_id"),
                format!("references missing PVE encounter '{}'", source.encounter_id),
            )
        })?;
        let abnormality_id = encounter.primary_abnormality_id().ok_or_else(|| {
            crate::game::data::DataValidationError::at(
                &source.reward_source_id,
                format!("{source_path}.encounter_id"),
                "reward source encounter must author a primary abnormality",
            )
        })?;
        let abnormality = abnormality_data.get_by_id(abnormality_id).ok_or_else(|| {
            crate::game::data::DataValidationError::at(
                &source.reward_source_id,
                format!("{source_path}.encounter_id"),
                format!("references missing primary abnormality '{abnormality_id}'"),
            )
        })?;
        if abnormality.response_complete_skill_fragment_id != source.fragment_id {
            return Err(crate::game::data::DataValidationError::at(
                &source.reward_source_id,
                format!("{source_path}.fragment_id"),
                format!("must equal primary abnormality '{}' source fragment {:?}",
                    abnormality_id, abnormality.response_complete_skill_fragment_id),
            ));
        }
        if let Some(variant) = source.revival.as_ref().and_then(|policy| policy.variant.as_ref()) {
            let target = abnormality_data.get_by_id(&variant.abnormality_id).ok_or_else(||
                crate::game::data::DataValidationError::at(&source.reward_source_id,
                    format!("{source_path}.revival.variant.abnormality_id"), "missing variant abnormality"))?;
            if target.response_complete_skill_fragment_id.as_ref() != Some(&variant.fragment_id)
                || skill_fragment_data.get_by_id(&variant.fragment_id).is_none() {
                return Err(crate::game::data::DataValidationError::at(&source.reward_source_id,
                    format!("{source_path}.revival.variant.fragment_id"), "variant fragment must exist and match its abnormality declaration"));
            }
        }
        if let Some(fragment_id) = &source.fragment_id {
            if skill_fragment_data.get_by_id(fragment_id).is_none() {
                return Err(crate::game::data::DataValidationError::at(
                    &source.reward_source_id,
                    format!("{source_path}.fragment_id"),
                    format!("references missing Skill Fragment '{fragment_id}'"),
                ));
            }
        }
    }
    Ok(())
}

/// Cross-catalog reward validation is shared by the live loader and fixture assembly.
pub(super) fn validate_revival_rewards(
    pve: &PveEncounterDatabase, equipment: &EquipmentDatabase,
    rewards: &RewardDatabase, consumables: &ConsumableDatabase,
    fragments: &skill_fragment_data::SkillFragmentDatabase,
    relics: &super::relic_data::RelicDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    use crate::game::data::equipment_data::EquipmentMaterialType;
    for (index, source) in pve.reward_sources.iter().enumerate() {
        let fail = |field: &str, cause: String| crate::game::data::DataValidationError::at(
            &source.reward_source_id, format!("reward_sources[{index}].{field}"), cause);
        let encounter = pve.get_by_id(&source.encounter_id)
            .ok_or_else(|| fail("encounter_id", "missing encounter".into()))?;
        let validate_items = |field: &str, items: &[String]| {
            let mut seen = HashSet::new();
            for id in items {
                if equipment.get_by_id(id).is_none() || !seen.insert(id) {
                    return Err(fail(field, format!("missing or duplicate equipment '{id}'")));
                }
            }
            Ok(())
        };
        let validate_effects = |field: &str, effects: &[RewardEffect]| {
            for effect in effects {
                let valid = match effect {
                    RewardEffect::GrantEnkephalin { amount } | RewardEffect::GrantEquipmentDust { amount }
                    | RewardEffect::GrantEquipmentMaterial { amount, .. } => *amount > 0,
                    RewardEffect::GrantExperience { amount, target: crate::game::reward::ExperienceTargetPolicy::AliveRoster } => *amount > 0,
                    RewardEffect::GrantRelic { .. } => true,
                    _ => false,
                };
                if !valid { return Err(fail(field, "expected positive quantity reward or explicit relic; equipment belongs in choices and fragments are first-only".into())); }
                validate_reward_effect(effect, rewards, equipment, consumables, fragments, relics)
                    .map_err(|cause| fail(field, cause))?;
            }
            Ok(())
        };
        validate_items("first_equipment_ids", &source.first_equipment_ids)?;
        validate_effects("repeat_rewards", &source.repeat_rewards)?;
        if encounter.encounter_class == PveEncounterClass::Elite && source.first_equipment_ids.is_empty() {
            return Err(fail("first_equipment_ids", "elite first victory requires equipment".into()));
        }
        if encounter.is_rewardless_story_encounter() {
            if source.revival.is_some() || !source.first_equipment_ids.is_empty() || !source.repeat_rewards.is_empty() {
                return Err(fail("revival", "rewardless story encounter cannot grant rewards or eggs".into()));
            }
            continue;
        }
        let Some(policy) = &source.revival else {
            if encounter.encounter_class == PveEncounterClass::NormalBoss {
                return Err(fail("revival", "normal boss requires its two revival stages".into()));
            }
            continue;
        };
        if encounter.encounter_class != PveEncounterClass::NormalBoss {
            return Err(fail("revival", "only normal bosses leave eggs".into()));
        }
        if let Some(variant) = &policy.variant {
            if source.fragment_id.is_some() || !source.first_equipment_ids.is_empty() || !source.repeat_rewards.is_empty() {
                return Err(fail("revival.variant", "variant source must leave only its egg before hatching".into()));
            }
            if variant.abnormality_id.trim().is_empty() || Some(variant.abnormality_id.as_str()) == encounter.primary_abnormality_id() {
                return Err(fail("revival.variant.abnormality_id", "variant must explicitly name a different abnormality kind".into()));
            }
            if variant.equipment_ids.len() > 128 {
                return Err(fail("revival.variant.equipment_ids", "too many guaranteed variant equipment rewards".into()));
            }
            validate_items("revival.variant.equipment_ids", &variant.equipment_ids)?;
        }
        if policy.catalyst_amount == 0 || !equipment.get_material_by_id(&policy.catalyst_material_id)
            .is_some_and(|m| m.material_type == EquipmentMaterialType::RevivalCatalyst) {
            return Err(fail("revival.catalyst_material_id", "requires a positive common revival catalyst recipe".into()));
        }
        if !equipment.get_material_by_id(&policy.part_material_id)
            .is_some_and(|m| m.material_type == EquipmentMaterialType::AbnormalityPart) {
            return Err(fail("revival.part_material_id", "requires the boss's authored abnormality part".into()));
        }
        if policy.stage_2.stat_multiplier_percent <= 100
            || policy.stage_3.stat_multiplier_percent <= policy.stage_2.stat_multiplier_percent {
            return Err(fail("revival", "stage 2 must exceed 100%; stage 3 must exceed stage 2".into()));
        }
        for (field, stage) in [("stage_2", &policy.stage_2), ("stage_3", &policy.stage_3)] {
            if !pve.get_by_id(&stage.encounter_id).is_some_and(|e|
                e.encounter_class == PveEncounterClass::NormalBoss && e.primary_abnormality_id() == policy.variant.as_ref().map(|v| v.abnormality_id.as_str()).or_else(|| encounter.primary_abnormality_id())) {
                return Err(fail(&format!("revival.{field}.encounter_id"), "revival must target its declared normal boss kind".into()));
            }
            validate_items(&format!("revival.{field}.equipment_choices"), &stage.equipment_choices)?;
            validate_effects(&format!("revival.{field}.rewards"), &stage.rewards)?;
            if !stage.rewards.iter().any(|effect| matches!(effect,
                RewardEffect::GrantEquipmentMaterial { material_id, amount } if material_id == &policy.part_material_id && *amount > 0)) {
                return Err(fail(&format!("revival.{field}.rewards"), "revival victory must grant its authored boss part".into()));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_ordinary_reward_source_exclusion(
    reward_data: &RewardDatabase,
    pve_data: &PveEncounterDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    let source_fragment_ids = pve_data
        .reward_sources
        .iter()
        .flat_map(|source| source.granted_fragment_ids())
        .collect::<HashSet<_>>();
    for (reward_index, reward) in reward_data.rewards.iter().enumerate() {
        for (effect_index, effect) in reward.effects.iter().enumerate() {
            let RewardEffect::GrantSkillFragment { fragment_id } = effect else {
                continue;
            };
            if source_fragment_ids.contains(fragment_id) {
                return Err(crate::game::data::DataValidationError::at(
                    &reward.id,
                    format!("rewards[{reward_index}].effects[{effect_index}].fragment_id"),
                    format!(
                        "ordinary Reward must not grant encounter-source fragment '{fragment_id}'"
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_data_references(
    abnormality_data: &AbnormalityDatabase,
    corroded_employee_data: &CorrodedEmployeeProfileDatabase,
    corroded_wave_data: &CorrodedWavePresetDatabase,
    consumable_data: &ConsumableDatabase,
    equipment_data: &EquipmentDatabase,
    reward_data: &RewardDatabase,
    event_data: &EventDatabase,
    visual_novel_data: &VisualNovelDatabase,
    visual_novel_trigger_data: &VisualNovelTriggerDatabase,
    pve_data: &PveEncounterDatabase,
    boss_omen_data: &BossOmenChainDatabase,
    skill_data: &SkillDatabase,
    buff_data: &BuffDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
    relic_data: &super::relic_data::RelicDatabase,
) {
    skill_data
        .validate_buff_references(buff_data)
        .expect("invalid Skill buff reference");
    equipment_data
        .validate_effect_and_activation_contract(skill_data, buff_data)
        .expect("invalid Equipment effect or activation contract");
    validate_skill_fragment_skill_references(
        skill_fragment_data,
        abnormality_data,
        skill_data,
        equipment_data,
    )
    .expect("invalid Skill Fragment cross-catalog reference");
    validate_abnormality_unit_skill_references(abnormality_data, skill_data, skill_fragment_data)
        .expect("invalid Abnormality Skill reference");
    validate_corroded_employee_skill_references(corroded_employee_data, skill_data)
        .expect("invalid CorrodedEmployee Skill reference");
    validate_reward_references(
        reward_data,
        equipment_data,
        consumable_data,
        skill_fragment_data,
        relic_data,
    );
    validate_pve_enemy_references(
        pve_data,
        abnormality_data,
        corroded_employee_data,
        corroded_wave_data,
        reward_data,
    )
    .expect("invalid PVE cross-catalog reference");
    validate_pve_reward_source_references(pve_data, abnormality_data, skill_fragment_data)
        .expect("invalid PVE reward-source cross-catalog reference");
    validate_revival_rewards(pve_data, equipment_data, reward_data, consumable_data, skill_fragment_data, relic_data)
        .expect("invalid revival reward contract");
    validate_ordinary_reward_source_exclusion(reward_data, pve_data)
        .expect("ordinary Reward grants an encounter-source fragment");
    validate_event_combat_references(event_data, pve_data).expect("invalid Event combat reference");
    validate_event_choice_key_references(event_data, equipment_data)
        .expect("invalid Event choice-key reference");
    validate_visual_novel_trigger_references(visual_novel_trigger_data, visual_novel_data)
        .expect("invalid visual novel trigger reference");
    validate_no_duplicate_run_start_triggers(visual_novel_trigger_data);
    validate_abnormality_omen_references(abnormality_data, boss_omen_data)
        .expect("invalid abnormality BossOmen reference");
    validate_boss_omen_references(
        boss_omen_data,
        abnormality_data,
        event_data,
        pve_data,
        skill_fragment_data,
    )
    .expect("invalid BossOmen chain reference");
}

pub(super) fn validate_shop_item_references(
    shop_data: &ShopDatabase,
    consumable_data: &ConsumableDatabase,
    equipment_data: &EquipmentDatabase,
    relic_data: &crate::game::data::relic_data::RelicDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    let mut tags_by_merchandise: HashMap<ShopMerchandise, HashSet<String>> = HashMap::new();
    let mut minimum_buy_quote: HashMap<ShopMerchandise, u32> = HashMap::new();
    for (shop_index, shop) in shop_data.shops.iter().enumerate() {
        let entries = shop_data.flattened_entries(shop).map_err(|cause| {
            crate::game::data::DataValidationError::at(
                &shop.id,
                format!("shops[{shop_index}]"),
                cause,
            )
        })?;
        for (entry_index, entry) in entries.iter().enumerate() {
            let base_price = match &entry.merchandise {
                ShopMerchandise::Equipment { equipment_id } => equipment_data
                    .get_by_id(equipment_id)
                    .map(|item| item.price),
                ShopMerchandise::Consumable { consumable_id } => consumable_data
                    .get_by_id(consumable_id)
                    .map(|item| item.price),
                ShopMerchandise::Relic { relic_id } => relic_data.get_by_id(relic_id).and(entry.base_price),
                ShopMerchandise::RevivalCatalyst { material_id } => equipment_data.get_material_by_id(material_id)
                    .filter(|material| material.material_type == crate::game::data::equipment_data::EquipmentMaterialType::RevivalCatalyst)
                    .and(entry.base_price),
                ShopMerchandise::RelicCorruptionCatalyst => entry.base_price,
                ShopMerchandise::Monolith { .. } => None,
            }
            .ok_or_else(|| {
                crate::game::data::DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].merchandise[{entry_index}]"),
                    format!(
                        "references missing typed merchandise '{}'",
                        entry.merchandise.identity_key()
                    ),
                )
            })?;
            if base_price == 0 {
                return Err(crate::game::data::DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].merchandise[{entry_index}]"),
                    format!(
                        "Shop merchandise '{}' has zero base price",
                        entry.merchandise.identity_key()
                    ),
                ));
            }
            let buy_percent = entry.buy_percent.unwrap_or(shop.default_buy_percent);
            let numerator = u64::from(base_price)
                .checked_mul(u64::from(buy_percent))
                .and_then(|value| value.checked_add(99))
                .ok_or_else(|| {
                    crate::game::data::DataValidationError::at(
                        &shop.id,
                        format!("shops[{shop_index}].merchandise[{entry_index}].buy_percent"),
                        "Shop buy quote overflows",
                    )
                })?;
            let unit_quote = u32::try_from(numerator / 100).map_err(|_| {
                crate::game::data::DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].merchandise[{entry_index}].buy_percent"),
                    "Shop buy quote exceeds u32",
                )
            })?;
            if unit_quote == 0 {
                return Err(crate::game::data::DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].merchandise[{entry_index}].buy_percent"),
                    "Shop buy quote must be positive",
                ));
            }
            unit_quote.checked_mul(entry.quantity).ok_or_else(|| {
                crate::game::data::DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].merchandise[{entry_index}].quantity"),
                    "total Shop buy quote exceeds u32",
                )
            })?;
            minimum_buy_quote
                .entry(entry.merchandise.clone())
                .and_modify(|current| *current = (*current).min(unit_quote))
                .or_insert(unit_quote);

            let tags = entry.tags.iter().cloned().collect::<HashSet<_>>();
            if let Some(existing) = tags_by_merchandise.get(&entry.merchandise) {
                if existing != &tags {
                    return Err(crate::game::data::DataValidationError::at(
                        &shop.id,
                        format!("shops[{shop_index}].merchandise[{entry_index}].tags"),
                        format!(
                            "merchandise '{}' must use the same tags in every Shop entry",
                            entry.merchandise.identity_key()
                        ),
                    ));
                }
            } else {
                tags_by_merchandise.insert(entry.merchandise.clone(), tags);
            }
        }
    }

    for (merchandise, min_buy) in minimum_buy_quote {
        let base_price = match &merchandise {
            ShopMerchandise::Equipment { equipment_id } => equipment_data
                .get_by_id(equipment_id)
                .map(|item| item.price)
                .expect("validated equipment merchandise"),
            ShopMerchandise::Consumable { consumable_id } => consumable_data
                .get_by_id(consumable_id)
                .map(|item| item.price)
                .expect("validated consumable merchandise"),
            ShopMerchandise::Relic { .. } | ShopMerchandise::RevivalCatalyst { .. }
                | ShopMerchandise::RelicCorruptionCatalyst => continue, // These goods cannot be sold.
            ShopMerchandise::Monolith { .. } => {
                unreachable!("dynamic monolith merchandise cannot be authored in Shop data")
            }
        };
        let tags = tags_by_merchandise
            .get(&merchandise)
            .cloned()
            .unwrap_or_default();
        let mut maximum_sell = None::<u32>;
        for shop in &shop_data.shops {
            let Some(policy) = &shop.sell_policy else {
                continue;
            };
            if policy
                .accepted_tags
                .as_ref()
                .is_some_and(|accepted| !accepted.iter().any(|tag| tags.contains(tag)))
            {
                continue;
            }
            let quote =
                u32::try_from(u64::from(base_price) * u64::from(policy.price_percent) / 100)
                    .map_err(|_| {
                        crate::game::data::DataValidationError::at(
                            &shop.id,
                            "sell_policy.price_percent",
                            "Shop sell quote exceeds u32",
                        )
                    })?;
            maximum_sell = Some(maximum_sell.map_or(quote, |current| current.max(quote)));
        }
        if maximum_sell.is_some_and(|max_sell| max_sell >= min_buy) {
            return Err(crate::game::data::DataValidationError::at(
                merchandise.identity_key(),
                "shop_economy.no_arbitrage",
                format!(
                    "maximum live sell quote {maximum_sell:?} must be strictly lower than minimum live buy quote {min_buy}"
                ),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_starter_employee_loadout_references(
    employee_definition_data: &EmployeeDefinitionDatabase,
    equipment_data: &EquipmentDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (definition_index, definition) in employee_definition_data.definitions.iter().enumerate() {
        let loadout_path = format!("definitions[{definition_index}].initial_loadout");
        if definition.starter_eligible
            && definition
                .initial_loadout
                .baseline_skill_fragment_ids
                .is_empty()
            && definition
                .initial_loadout
                .active_skill_fragment_id
                .is_none()
        {
            return Err(crate::game::data::DataValidationError::at(
                &definition.employee_definition_id,
                format!("{loadout_path}.baseline_skill_fragment_ids"),
                "must declare an active starting fragment or explicit baseline fragments",
            ));
        }
        for (fragment_index, fragment_id) in definition
            .initial_loadout
            .baseline_skill_fragment_ids
            .iter()
            .enumerate()
        {
            if !skill_fragment_data
                .get_by_id(fragment_id)
                .is_some_and(|fragment| {
                    matches!(
                        fragment.effect,
                        SkillFragmentEffectDef::BasicAttackModifier { .. }
                    )
                })
            {
                return Err(crate::game::data::DataValidationError::at(
                    &definition.employee_definition_id,
                    format!("{loadout_path}.baseline_skill_fragment_ids[{fragment_index}]"),
                    format!("requires an existing baseline BasicAttackModifier fragment '{fragment_id}'"),
                ));
            }
        }

        if definition.starter_eligible && definition.initial_loadout.equipment_ids.is_empty() {
            return Err(crate::game::data::DataValidationError::at(
                &definition.employee_definition_id,
                format!("{loadout_path}.equipment_ids"),
                "must declare starting equipment",
            ));
        }
        let mut slot = ItemSlot::default();
        let mut has_weapon = false;
        let stats = definition.profile.base_combat_stats;
        let mut profile = crate::game::battle::types::UnitCombatProfile::for_employee(
            stats.max_hp,
            stats.attack,
            stats.defense,
            stats.magic_resist,
            definition.profile.basic_attack_speed_percent,
        );
        for (equipment_index, equipment_id) in
            definition.initial_loadout.equipment_ids.iter().enumerate()
        {
            let equipment_path = format!("{loadout_path}.equipment_ids[{equipment_index}]");
            let equipment = equipment_data.get_by_id(equipment_id).ok_or_else(|| {
                crate::game::data::DataValidationError::at(
                    &definition.employee_definition_id,
                    &equipment_path,
                    format!("references missing equipment '{equipment_id}'"),
                )
            })?;
            has_weapon |= equipment.equipment_type == EquipmentType::Weapon;
            if equipment.equipment_type == EquipmentType::Weapon {
                if let Some(weapon) = &equipment.weapon_profile {
                    if definition.starter_eligible {
                        use crate::game::employee::EmployeeRole;
                        use crate::game::data::equipment_data::WeaponArchetype;
                        let matches_role = match definition.role {
                            EmployeeRole::Healer => weapon.action.is_heal(),
                            EmployeeRole::Tank => weapon.weapon_archetype == WeaponArchetype::Shield && !weapon.action.is_heal(),
                            EmployeeRole::Dealer => weapon.weapon_archetype != WeaponArchetype::Shield && !weapon.action.is_heal(),
                        };
                        if !matches_role { return Err(crate::game::data::DataValidationError::at(
                            &definition.employee_definition_id, &equipment_path, "starter weapon must match its declared role")); }
                    }
                    profile.apply_weapon_profile(weapon).map_err(|error| {
                        crate::game::data::DataValidationError::at(
                            &definition.employee_definition_id,
                            &equipment_path,
                            format!("{error:?}"),
                        )
                    })?;
                }
            }
            slot.equip(
                EquippedRef {
                    instance_uuid: equipment.uuid,
                    base_uuid: equipment.uuid,
                    equipment_type: equipment.equipment_type,
                },
                equipment.allow_duplicate_equip,
            )
            .map_err(|error| {
                crate::game::data::DataValidationError::at(
                    &definition.employee_definition_id,
                    equipment_path,
                    format!("invalid starting equipment layout: {error:?}"),
                )
            })?;
        }
        if definition.starter_eligible && !has_weapon {
            return Err(crate::game::data::DataValidationError::at(
                &definition.employee_definition_id,
                format!("{loadout_path}.equipment_ids"),
                "starter employee must declare an initial weapon",
            ));
        }
        if let Some(id) = &definition.initial_loadout.active_skill_fragment_id {
            let compatible = skill_fragment_data
                .get_by_id(id)
                .and_then(|fragment| fragment.form_compatibility_report("base", &profile))
                .is_some_and(|report| report.is_compatible);
            if !compatible {
                return Err(crate::game::data::DataValidationError::at(
                    &definition.employee_definition_id,
                    format!("{loadout_path}.active_skill_fragment_id"),
                    format!("requires an existing ActiveSkill base form compatible with the initial weapon: '{id}'"),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_generated_combat_preview_contracts(
    game_data: &Arc<GameDataBase>,
) -> Result<(), super::live_ron::LiveRonError> {
    validate_combat_preview_threat_warning_contract(game_data)
}

pub(super) fn validate_skill_fragment_skill_references(
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
    abnormality_data: &AbnormalityDatabase,
    skill_data: &SkillDatabase,
    equipment_data: &EquipmentDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (fragment_index, fragment) in skill_fragment_data.fragments.iter().enumerate() {
        let fragment_id = fragment.id.as_str();
        let fragment_path = format!("fragments[{fragment_index}]");
        let error = |path: String, cause: String| {
            crate::game::data::DataValidationError::at(fragment_id, path, cause)
        };
        if let Some(SkillFragmentOrigin::Abnormality { abnormality_id }) = &fragment.origin {
            if abnormality_data.get_by_id(abnormality_id).is_none() {
                return Err(error(
                    format!("{fragment_path}.origin.Abnormality.abnormality_id"),
                    format!("references unknown origin abnormality '{abnormality_id}'"),
                ));
            }
        }

        for (source_index, source) in fragment.sources.iter().enumerate() {
            match source {
                SkillFragmentAcquisitionSource::AbnormalityContainment { abnormality_id } => {
                    if abnormality_data.get_by_id(abnormality_id).is_none() {
                        return Err(error(
                            format!("{fragment_path}.sources[{source_index}].AbnormalityContainment.abnormality_id"),
                            format!("references unknown source abnormality '{abnormality_id}'"),
                        ));
                    }
                }
                SkillFragmentAcquisitionSource::RareReward
                | SkillFragmentAcquisitionSource::DependentConcept { .. } => {}
            }
        }

        for (dependency_index, dependency) in fragment.dependencies.iter().enumerate() {
            if let SkillFragmentDependency::SourceAbnormality { abnormality_id } = dependency {
                if abnormality_data.get_by_id(abnormality_id).is_none() {
                    return Err(error(
                        format!("{fragment_path}.dependencies[{dependency_index}].SourceAbnormality.abnormality_id"),
                        format!("references unknown dependency abnormality '{abnormality_id}'"),
                    ));
                }
            }
        }

        let SkillFragmentEffectDef::ActiveSkill {
            forms,
            dismantle_recipe,
            passive_guard,
        } = &fragment.effect
        else {
            continue;
        };
        let effect_path = format!("{fragment_path}.effect.ActiveSkill");
        let mut recipes = Vec::new();
        if let Some(recipe) = dismantle_recipe {
            recipes.push((format!("{effect_path}.dismantle_recipe"), recipe));
        }
        for (form_id, form) in forms {
            let path = format!("{effect_path}.forms[{form_id}]");
            let skill = skill_data.get_by_id(&form.skill_id).ok_or_else(|| {
                error(
                    format!("{path}.skill_id"),
                    format!("references unknown skill '{}'", form.skill_id),
                )
            })?;
            if skill.resonance_admission.is_some_and(|policy| policy.capacity != form.resonance_max) {
                return Err(error(
                    format!("{path}.resonance_max"),
                    "fragment form capacity must match its compiled skill admission capacity".into(),
                ));
            }
            if skill.steps.iter().any(|step| {
                step.targeting == crate::game::ability::StepTargetingMode::RetargetOnStep
            }) {
                return Err(error(
                    format!("{path}.skill_id"),
                    "manual employee fragment cannot retarget after admission".into(),
                ));
            }
            let compiled_guard = skill.fragment_guard();
            if passive_guard.is_some() && compiled_guard.is_some() && *passive_guard != compiled_guard {
                return Err(error(format!("{path}.skill_id"), "compiled and fragment metadata guard declarations disagree".into()));
            }
            if passive_guard.is_some() || compiled_guard.is_some() {
                let parry_steps: Vec<_> = skill.steps.iter().filter(|step| step.effects.iter().any(|effect| matches!(&effect.op, crate::game::ability::SkillEffectDef::Sustained { definition: crate::game::ability::sustained::SustainedEffectDef::Parry { .. } }))).collect();
                if skill.cast_selection() != crate::game::ability::SkillCastSelection::SelfUnit
                    || !matches!(skill.cast_target_definition(), Some((crate::game::ability::SkillTarget::SelfUnit, ..)))
                    || parry_steps.is_empty() || parry_steps.iter().any(|step| step.target != crate::game::ability::SkillTarget::SelfUnit || !matches!(step.delivery, crate::game::ability::DeliveryDef::Instant) || step.effects.iter().any(|effect| matches!(&effect.op, crate::game::ability::SkillEffectDef::Sustained { definition: crate::game::ability::sustained::SustainedEffectDef::Parry { reduction_percent, .. } } if *reduction_percent != 0))) {
                    return Err(error(format!("{path}.skill_id"), "passive guard requires a SelfUnit Instant parry with zero finite reduction in every form".into()));
                }
            }
            if form_id == "base" {
                validate_defense_route_skill_contract(skill)
                    .map_err(|cause| error(format!("{path}.skill_id"), cause))?;
            }
            if let Some(recipe) = &form.bloom_recipe {
                if recipe.materials.iter().any(|cost| equipment_data.get_material_by_id(&cost.material_id).is_some_and(|m| m.material_type != crate::game::data::equipment_data::EquipmentMaterialType::AbnormalityPart)) {
                    return Err(error(format!("{path}.bloom_recipe"), "bloom requires abnormality parts".into()));
                }
                recipes.push((format!("{path}.bloom_recipe"), recipe));
            }
        }
        for (path, recipe) in recipes {
            let costs = recipe
                .aggregated_materials()
                .ok_or_else(|| error(path.clone(), "invalid/overflowing fragment recipe".into()))?;
            for id in costs.keys() {
                if equipment_data.get_material_by_id(id).is_none() {
                    return Err(error(
                        path.clone(),
                        format!("unknown fragment material '{id}'"),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_defense_route_skill_contract(skill: &SkillDef) -> Result<(), String> {
    let Some((target, reach, target_range, _)) = skill.cast_target_definition() else {
        return Err(format!("skill '{}' has no cast targeting", skill.id));
    };
    if matches!(target, SkillTarget::EnemySingle { .. })
        && matches!(reach, SkillTargetReach::Pattern)
        && target_range.is_none()
    {
        return Err(format!(
            "skill '{}' has pattern unit reach without target_range",
            skill.id
        ));
    }
    for step in &skill.steps {
        if let DeliveryDef::TileArea { footprint, .. } = &step.delivery {
            footprint.validate().map_err(|cause| {
                format!(
                    "skill '{}' step '{}' has invalid TileArea footprint: {cause}",
                    skill.id, step.id
                )
            })?;
        }
    }
    Ok(())
}

pub(super) fn validate_abnormality_unit_skill_references(
    abnormality_data: &AbnormalityDatabase,
    skill_data: &SkillDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (abnormality_index, abnormality) in abnormality_data.items.iter().enumerate() {
        let abnormality_path = format!("items[{abnormality_index}]");
        let error = |path: String, cause: String| {
            crate::game::data::DataValidationError::at(&abnormality.id, path, cause)
        };
        if matches!(abnormality.threat_class, BattleUnitThreatClass::Normal) {
            return Err(error(
                format!("{abnormality_path}.threat_class"),
                "must use Elite or Boss threat_class".to_string(),
            ));
        }
        if let Some(response_fragment_id) = &abnormality.response_complete_skill_fragment_id {
            if skill_fragment_data
                .get_by_id(response_fragment_id)
                .is_none()
            {
                return Err(error(
                    format!("{abnormality_path}.response_complete_skill_fragment_id"),
                    format!("references unknown Skill Fragment '{response_fragment_id}'"),
                ));
            }
        }
        if let Some(skill_id) = &abnormality.skill_id {
            let path = format!("{abnormality_path}.skill_id");
            let skill = skill_data.get_by_id(skill_id).ok_or_else(|| {
                error(
                    path.clone(),
                    format!("references unknown Skill '{skill_id}'"),
                )
            })?;
            validate_defense_route_skill_contract(skill).map_err(|cause| error(path, cause))?;
        }
    }
    Ok(())
}

pub(super) fn validate_corroded_employee_skill_references(
    corroded_employee_data: &CorrodedEmployeeProfileDatabase,
    skill_data: &SkillDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (profile_index, profile) in corroded_employee_data.profiles.iter().enumerate() {
        if let Some(skill_id) = &profile.skill_id {
            let path = format!("profiles[{profile_index}].skill_id");
            let skill = skill_data.get_by_id(skill_id).ok_or_else(|| {
                crate::game::data::DataValidationError::at(
                    &profile.id,
                    &path,
                    format!("references unknown Skill '{skill_id}'"),
                )
            })?;
            validate_defense_route_skill_contract(skill).map_err(|cause| {
                crate::game::data::DataValidationError::at(&profile.id, path, cause)
            })?;
        }
    }
    Ok(())
}

fn validate_reward_references(
    reward_data: &RewardDatabase,
    equipment_data: &EquipmentDatabase,
    consumable_data: &ConsumableDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
    relic_data: &super::relic_data::RelicDatabase,
) {
    for reward in &reward_data.rewards {
        for effect in &reward.effects {
            validate_reward_effect(
                effect,
                reward_data,
                equipment_data,
                consumable_data,
                skill_fragment_data,
                relic_data,
            )
            .unwrap_or_else(|cause| panic!("reward '{}': {cause}", reward.id));
        }
    }

    validate_equipment_reward_pool_references(reward_data, equipment_data)
        .expect("invalid equipment reward pool reference");
}

pub(super) fn validate_equipment_reward_pool_references(
    reward_data: &RewardDatabase,
    equipment_data: &EquipmentDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (pool_index, pool) in reward_data.equipment_pools.iter().enumerate() {
        let pool_path = format!("equipment_pools[{pool_index}]");
        let mut excluded_ids = HashSet::new();
        for (excluded_index, equipment_id) in pool.filter.exclude_equipment_ids.iter().enumerate() {
            let excluded_path =
                format!("{pool_path}.filter.exclude_equipment_ids[{excluded_index}]");
            if !excluded_ids.insert(equipment_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &pool.id,
                    excluded_path,
                    "duplicate excluded equipment id",
                ));
            }
            if equipment_data.get_by_id(equipment_id).is_none() {
                return Err(crate::game::data::DataValidationError::at(
                    &pool.id,
                    excluded_path,
                    format!("references unknown equipment '{equipment_id}'"),
                ));
            }
        }
        for (entry_index, entry) in pool.entries.iter().enumerate() {
            let entry_path = format!("{pool_path}.entries[{entry_index}]");
            if entry.weight == 0 {
                return Err(crate::game::data::DataValidationError::at(
                    &pool.id,
                    format!("{entry_path}.weight"),
                    "weight must be greater than zero",
                ));
            }
            if equipment_data.get_by_id(&entry.equipment_id).is_none() {
                return Err(crate::game::data::DataValidationError::at(
                    &pool.id,
                    format!("{entry_path}.equipment_id"),
                    format!("references unknown equipment '{}'", entry.equipment_id),
                ));
            }
        }
        if pool.candidates(equipment_data).is_empty() {
            return Err(crate::game::data::DataValidationError::at(
                &pool.id,
                format!("{pool_path}.entries"),
                "must produce at least one candidate",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_reward_effect(
    effect: &RewardEffect,
    reward_data: &RewardDatabase,
    equipment_data: &EquipmentDatabase,
    consumable_data: &ConsumableDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
    relic_data: &super::relic_data::RelicDatabase,
) -> Result<(), String> {
    match effect {
        RewardEffect::GrantEquipment { equipment_id } => equipment_data
            .get_by_id(equipment_id)
            .map(|_| ())
            .ok_or_else(|| format!("references unknown equipment '{equipment_id}'")),
        RewardEffect::GrantEquipmentFromPool { pool_id } => {
            let pool = reward_data
                .equipment_pool_by_id(pool_id)
                .ok_or_else(|| format!("references unknown equipment reward pool '{pool_id}'"))?;
            if pool.candidates(equipment_data).is_empty() {
                return Err(format!(
                    "equipment reward pool '{pool_id}' has no candidates"
                ));
            }
            Ok(())
        }
        RewardEffect::GrantEquipmentMaterial {
            material_id,
            amount,
        } => {
            if *amount == 0 {
                return Err("grants zero equipment material".to_string());
            }
            equipment_data
                .get_material_by_id(material_id)
                .map(|_| ())
                .ok_or_else(|| format!("references unknown equipment material '{material_id}'"))
        }
        RewardEffect::GrantConsumable {
            consumable_id,
            quantity,
        } => {
            if *quantity == 0 {
                return Err("grants zero consumable quantity".to_string());
            }
            consumable_data
                .get_by_id(consumable_id)
                .map(|_| ())
                .ok_or_else(|| format!("references unknown consumable '{consumable_id}'"))
        }
        RewardEffect::GrantSkillFragment { fragment_id } => skill_fragment_data
            .get_by_id(fragment_id)
            .map(|_| ())
            .ok_or_else(|| format!("references unknown skill fragment '{fragment_id}'")),
        RewardEffect::GrantRelic { relic_id } => relic_data
            .get_by_id(relic_id)
            .map(|_| ())
            .ok_or_else(|| format!("references unknown relic '{relic_id}'")),
        RewardEffect::GrantRelicCorruptionCatalyst { amount }
        | RewardEffect::GrantEquipmentDust { amount } => {
            if *amount == 0 {
                Err("grants zero run crafting resource".into())
            } else {
                Ok(())
            }
        }
        RewardEffect::GrantEnkephalin { .. } => Ok(()),
        RewardEffect::GrantExperience { amount, .. } if *amount == 0 => {
            Err("grants zero experience".to_string())
        }
        RewardEffect::GrantFragmentDust { amount } if *amount == 0 => {
            Err("grants zero skill fragment dust".to_string())
        }
        RewardEffect::GrantExperience { .. } | RewardEffect::GrantFragmentDust { .. } => Ok(()),
    }
}

pub(super) fn validate_pve_enemy_references(
    pve_data: &PveEncounterDatabase,
    abnormality_data: &AbnormalityDatabase,
    corroded_employee_data: &CorrodedEmployeeProfileDatabase,
    corroded_wave_data: &CorrodedWavePresetDatabase,
    reward_data: &RewardDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (encounter_index, encounter) in pve_data.encounters.iter().enumerate() {
        let encounter_path = format!("encounters[{encounter_index}]");
        let error = |path: String, cause: String| {
            crate::game::data::DataValidationError::at(&encounter.id, path, cause)
        };
        match encounter.encounter_class {
            PveEncounterClass::Normal | PveEncounterClass::StressCombat => {
                if encounter.primary_abnormality_id().is_some() {
                    return Err(error(
                        format!("{encounter_path}.primary_abnormality_id"),
                        "normal encounter must not define primary_abnormality_id".to_string(),
                    ));
                }
            }
            PveEncounterClass::Elite
            | PveEncounterClass::NormalBoss
            | PveEncounterClass::FinalBoss
            | PveEncounterClass::ReferenceFinalBoss => {
                let primary_abnormality_id =
                    encounter.primary_abnormality_id().ok_or_else(|| {
                        error(
                            format!("{encounter_path}.primary_abnormality_id"),
                            format!(
                                "{:?} encounter must define primary_abnormality_id",
                                encounter.encounter_class
                            ),
                        )
                    })?;
                let abnormality = abnormality_data
                    .get_by_id(primary_abnormality_id)
                    .ok_or_else(|| {
                        error(
                            format!("{encounter_path}.primary_abnormality_id"),
                            format!(
                                "references missing primary abnormality '{primary_abnormality_id}'"
                            ),
                        )
                    })?;
                match encounter.encounter_class {
                    PveEncounterClass::Elite
                        if abnormality.threat_class != BattleUnitThreatClass::Elite =>
                    {
                        return Err(error(
                            format!("{encounter_path}.primary_abnormality_id"),
                            format!("primary abnormality '{primary_abnormality_id}' must be Elite"),
                        ));
                    }
                    PveEncounterClass::NormalBoss
                    | PveEncounterClass::FinalBoss
                    | PveEncounterClass::ReferenceFinalBoss
                        if abnormality.threat_class != BattleUnitThreatClass::Boss =>
                    {
                        return Err(error(
                            format!("{encounter_path}.primary_abnormality_id"),
                            format!("primary abnormality '{primary_abnormality_id}' must be Boss"),
                        ));
                    }
                    PveEncounterClass::Normal | PveEncounterClass::StressCombat => unreachable!(),
                    _ => {}
                }
            }
        }

        for (wave_index, wave) in encounter.waves.iter().enumerate() {
            validate_pve_wave_references(
                wave,
                abnormality_data,
                corroded_employee_data,
                corroded_wave_data,
            )
            .map_err(|cause| error(format!("{encounter_path}.waves[{wave_index}]"), cause))?;
        }
        for (policy_index, policy) in encounter.outcome_policies.iter().enumerate() {
            for (reward_index, reward_uuid) in policy.reward_uuids.iter().enumerate() {
                if reward_data.get_by_uuid(reward_uuid).is_none() {
                    return Err(error(
                        format!(
                            "{encounter_path}.outcome_policies[{policy_index}].reward_uuids[{reward_index}]"
                        ),
                        format!("references missing reward uuid {reward_uuid}"),
                    ));
                }
            }
        }
        if encounter.survive_timer_ms().is_some() {
            if encounter.node_type != Some(CombatNodeType::Defense) {
                return Err(error(
                    format!("{encounter_path}.terminal_outcomes"),
                    "TimeReached Victory requires an explicit Defense encounter".to_string(),
                ));
            }
        }
        if let Some(node_type) = encounter.node_type {
            let mission_variant = encounter
                .mission_variant
                .unwrap_or_else(|| CombatMissionVariant::default_for_node_type(node_type));
            if !mission_variant.is_compatible_with(node_type) {
                return Err(error(
                    format!("{encounter_path}.mission_variant"),
                    format!("{mission_variant:?} is incompatible with node_type {node_type:?}"),
                ));
            }
            if !CombatMissionPolicy::is_supported_encounter_node_type(node_type) {
                return Err(error(
                    format!("{encounter_path}.node_type"),
                    format!("uses deferred combat node type {node_type:?}"),
                ));
            }
            for (policy_index, policy) in encounter.outcome_policies.iter().enumerate() {
                let rewards = policy
                    .reward_uuids
                    .iter()
                    .filter_map(|reward_uuid| reward_data.get_by_uuid(reward_uuid))
                    .map(RewardOption::from_metadata)
                    .collect::<Vec<_>>();
                CombatRewardPolicy::for_mission(node_type, mission_variant)
                    .validate_rewards(&rewards)
                    .map_err(|message| {
                        error(
                            format!(
                                "{encounter_path}.outcome_policies[{policy_index}].reward_uuids"
                            ),
                            format!("invalid {node_type:?} reward policy: {message}"),
                        )
                    })?;
            }
        }
    }
    Ok(())
}

pub(super) fn validate_pve_wave_references(
    wave: &PveWaveData,
    abnormality_data: &AbnormalityDatabase,
    corroded_employee_data: &CorrodedEmployeeProfileDatabase,
    corroded_wave_data: &CorrodedWavePresetDatabase,
) -> Result<(), String> {
    if let PveWaveSource::GeneratedCorroded { preset_id, .. } = &wave.source {
        let preset = corroded_wave_data
            .get_by_id(preset_id)
            .ok_or_else(|| format!("references unknown corroded wave preset '{preset_id}'"))?;
        for role in &preset.role_mix {
            if corroded_employee_data.get_by_id(&role.profile_id).is_none() {
                return Err(format!(
                    "corroded wave preset '{}' references unknown corroded employee profile '{}'",
                    preset.id, role.profile_id
                ));
            }
        }
    }

    for enemy in wave.manual_enemies() {
        match enemy {
            PveWaveEnemyData::Abnormality { abnormality_id, .. } => {
                if abnormality_data.get_by_id(abnormality_id).is_none() {
                    return Err(format!(
                        "references unknown abnormality enemy '{abnormality_id}'"
                    ));
                }
            }
            PveWaveEnemyData::CorrodedEmployee { profile_id, .. } => {
                if corroded_employee_data.get_by_id(profile_id).is_none() {
                    return Err(format!(
                        "references unknown corroded employee profile '{profile_id}'"
                    ));
                }
            }
            PveWaveEnemyData::FacilityEntity { profile_id, .. } => {
                return Err(format!(
                    "uses unsupported FacilityEntity profile '{profile_id}'"
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_abnormality_omen_references(
    abnormality_data: &AbnormalityDatabase,
    boss_omen_data: &BossOmenChainDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    let chain_ids = boss_omen_data
        .chains
        .iter()
        .map(|chain| chain.id.as_str())
        .collect::<HashSet<_>>();
    for (abnormality_index, abnormality) in abnormality_data.items.iter().enumerate() {
        let Some(omen_chain_id) = &abnormality.omen_chain_id else {
            continue;
        };
        if !chain_ids.contains(omen_chain_id.as_str()) {
            return Err(crate::game::data::DataValidationError::at(
                &abnormality.id,
                format!("items[{abnormality_index}].omen_chain_id"),
                format!("references unknown omen chain '{}'", omen_chain_id.as_str()),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_boss_omen_references(
    boss_omen_data: &BossOmenChainDatabase,
    abnormality_data: &AbnormalityDatabase,
    event_data: &EventDatabase,
    pve_data: &PveEncounterDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (chain_index, chain) in boss_omen_data.chains.iter().enumerate() {
        let chain_path = format!("chains[{chain_index}]");
        let abnormality = abnormality_data
            .get_by_id(&chain.boss_abnormality_id)
            .ok_or_else(|| {
                crate::game::data::DataValidationError::at(
                    chain.id.as_str(),
                    format!("{chain_path}.boss_abnormality_id"),
                    format!(
                        "references missing boss abnormality '{}'",
                        chain.boss_abnormality_id
                    ),
                )
            })?;
        if !abnormality
            .omen_chain_id
            .as_ref()
            .is_some_and(|id| id == &chain.id)
        {
            return Err(crate::game::data::DataValidationError::at(
                chain.id.as_str(),
                format!("{chain_path}.boss_abnormality_id"),
                format!(
                    "boss abnormality '{}' does not reference this chain through omen_chain_id",
                    abnormality.id
                ),
            ));
        }

        let boss_encounter = pve_data
            .get_by_id(&chain.boss_encounter_id)
            .ok_or_else(|| {
                crate::game::data::DataValidationError::at(
                    chain.id.as_str(),
                    format!("{chain_path}.boss_encounter_id"),
                    format!(
                        "references missing boss encounter '{}'",
                        chain.boss_encounter_id
                    ),
                )
            })?;
        if boss_encounter.primary_abnormality_id() != Some(chain.boss_abnormality_id.as_str()) {
            return Err(crate::game::data::DataValidationError::at(
                chain.id.as_str(),
                format!("{chain_path}.boss_encounter_id"),
                format!(
                    "boss encounter '{}' does not target abnormality '{}'",
                    boss_encounter.id, chain.boss_abnormality_id
                ),
            ));
        }
        if !matches!(boss_encounter.encounter_class, PveEncounterClass::FinalBoss) {
            return Err(crate::game::data::DataValidationError::at(
                chain.id.as_str(),
                format!("{chain_path}.boss_encounter_id"),
                format!("boss encounter '{}' must be FinalBoss", boss_encounter.id),
            ));
        }

        if let Some(required) = &chain.required_fragment {
            let fragment = skill_fragment_data
                .get_by_id(&required.fragment_id)
                .ok_or_else(|| {
                    crate::game::data::DataValidationError::at(
                        chain.id.as_str(),
                        format!("{chain_path}.required_fragment.fragment_id"),
                        format!(
                            "references missing Skill Fragment '{}'",
                            required.fragment_id
                        ),
                    )
                })?;
            if required.awakening_required
                && !matches!(
                    &fragment.effect,
                    SkillFragmentEffectDef::ActiveSkill { forms, .. } if forms.contains_key("awakened")
                )
            {
                return Err(crate::game::data::DataValidationError::at(
                    chain.id.as_str(),
                    format!("{chain_path}.required_fragment.awakening_required"),
                    format!(
                        "fragment '{}' cannot be awakened under its authored effect",
                        required.fragment_id
                    ),
                ));
            }
            let source = pve_data
                .reward_sources
                .iter()
                .find(|source| source.granted_fragment_ids().any(|id| id == &required.fragment_id))
                .ok_or_else(|| {
                    crate::game::data::DataValidationError::at(
                        chain.id.as_str(),
                        format!("{chain_path}.required_fragment.fragment_id"),
                        "required fragment has no reachable authored encounter source",
                    )
                })?;
            if source.encounter_id == chain.boss_encounter_id {
                return Err(crate::game::data::DataValidationError::at(
                    chain.id.as_str(),
                    format!("{chain_path}.required_fragment.fragment_id"),
                    "chain cannot require the fragment granted by its own FinalBoss",
                ));
            }
            if pve_data
                .get_by_id(&source.encounter_id)
                .is_some_and(|encounter| encounter.encounter_class == PveEncounterClass::FinalBoss)
            {
                let producing_chain = boss_omen_data
                    .chains
                    .iter()
                    .find(|candidate| candidate.boss_encounter_id == source.encounter_id)
                    .ok_or_else(|| {
                        crate::game::data::DataValidationError::at(
                            chain.id.as_str(),
                            format!("{chain_path}.required_fragment.fragment_id"),
                            "required FinalBoss fragment source is dormant and unreachable",
                        )
                    })?;
                if !chain.dependencies.contains(&producing_chain.id) {
                    return Err(crate::game::data::DataValidationError::at(
                        chain.id.as_str(),
                        format!("{chain_path}.dependencies"),
                        format!(
                            "must depend on FinalBoss chain '{}' that grants required fragment '{}'",
                            producing_chain.id.as_str(),
                            required.fragment_id
                        ),
                    ));
                }
            }
        }

        for (step_index, step) in chain.steps.iter().enumerate() {
            let step_path = format!("{chain_path}.steps[{step_index}]");
            match step.source_kind {
                BossOmenSourceKind::Event => {
                    let event_id = step.event_id.as_ref().ok_or_else(|| {
                        crate::game::data::DataValidationError::at(
                            chain.id.as_str(),
                            format!("{step_path}.event_id"),
                            "Event step must define event_id",
                        )
                    })?;
                    if event_data.get_by_id(event_id.as_str()).is_none() {
                        return Err(crate::game::data::DataValidationError::at(
                            chain.id.as_str(),
                            format!("{step_path}.event_id"),
                            format!("references missing Event '{}'", event_id.as_str()),
                        ));
                    }
                }
                BossOmenSourceKind::Combat => {
                    let encounter_id = step.encounter_id.as_ref().ok_or_else(|| {
                        crate::game::data::DataValidationError::at(
                            chain.id.as_str(),
                            format!("{step_path}.encounter_id"),
                            "Combat step must define encounter_id",
                        )
                    })?;
                    let encounter = pve_data.get_by_id(encounter_id).ok_or_else(|| {
                        crate::game::data::DataValidationError::at(
                            chain.id.as_str(),
                            format!("{step_path}.encounter_id"),
                            format!("references missing encounter '{encounter_id}'"),
                        )
                    })?;
                    if !matches!(
                        encounter.encounter_class,
                        PveEncounterClass::Elite
                            | PveEncounterClass::NormalBoss
                            | PveEncounterClass::FinalBoss
                    ) {
                        return Err(crate::game::data::DataValidationError::at(
                            chain.id.as_str(),
                            format!("{step_path}.encounter_id"),
                            format!(
                                "combat encounter '{}' must be abnormality-bearing",
                                encounter.id
                            ),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn validate_event_combat_references(
    event_data: &EventDatabase,
    pve_data: &PveEncounterDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (event_index, event) in event_data.events.iter().enumerate() {
        for (scene_index, scene) in event.scenes.iter().enumerate() {
            let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices {
                choices, ..
            } = &scene.next
            else {
                continue;
            };
            for (choice_index, choice) in choices.iter().enumerate() {
                let Some(EventSuccessAction::StartCombat {
                    encounter_id,
                    primary_abnormality_id,
                }) = &choice.success_action
                else {
                    continue;
                };
                let effect_path = format!(
                        "events[{event_index}].scenes[{scene_index}].next.EmployeeCheckChoices.choices[{choice_index}].success_action.StartCombat"
                    );
                let encounter = pve_data.get_by_id(encounter_id).ok_or_else(|| {
                    crate::game::data::DataValidationError::at(
                        event.id.as_str(),
                        format!("{effect_path}.encounter_id"),
                        format!("references missing combat encounter '{encounter_id}'"),
                    )
                })?;
                if let Some(primary_abnormality_id) = primary_abnormality_id {
                    if encounter.primary_abnormality_id() != Some(primary_abnormality_id.as_str()) {
                        return Err(crate::game::data::DataValidationError::at(
                                event.id.as_str(),
                                format!("{effect_path}.primary_abnormality_id"),
                                format!(
                                    "primary_abnormality_id '{primary_abnormality_id}' does not match encounter '{encounter_id}'"
                                ),
                            ));
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn validate_event_choice_key_references(
    event_data: &EventDatabase,
    equipment_data: &EquipmentDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    let provided_keys = equipment_data
        .items
        .iter()
        .flat_map(|item| item.event_choice_keys.iter().map(|key| key.as_str()))
        .collect::<HashSet<_>>();

    for (event_index, event) in event_data.events.iter().enumerate() {
        for (scene_index, scene) in event.scenes.iter().enumerate() {
            let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices { choices } =
                &scene.next
            else {
                continue;
            };
            for (choice_index, choice) in choices.iter().enumerate() {
                let Some(required_key) = &choice.check.required_choice_key else {
                    continue;
                };
                if !provided_keys.contains(required_key.as_str()) {
                    return Err(crate::game::data::DataValidationError::at(
                        event.id.as_str(),
                        format!(
                            "events[{event_index}].scenes[{scene_index}].next.EmployeeCheckChoices.choices[{choice_index}].check.required_choice_key"
                        ),
                        format!(
                            "references event choice key '{}' that no equipment provides",
                            required_key.as_str()
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

struct GeneratedValidationContext<'a> {
    game_data: &'a GameDataBase,
    encounter_index: usize,
    tuning_path: String,
    revival: Option<crate::game::revival::RevivalNodeTuning>,
}

impl GeneratedValidationContext<'_> {
    fn error(
        &self,
        cause: String,
        failure: Option<&crate::game::combat_preview::CombatGenerationFailure>,
        preview: Option<&CombatPreview>,
    ) -> super::live_ron::LiveRonError {
        use super::live_ron::{LiveRonError, LiveRonPhase, LiveRonSource};
        let encounter = &self.game_data.pve_data.encounters[self.encounter_index];
        let encounter_source = LiveRonSource {
            source_file: "data/pve/encounters.ron".into(),
            object_id: Some(encounter.id.clone()),
            field_path: Some(format!("encounters[{}]", self.encounter_index)),
            ..Default::default()
        };
        let policy_source = LiveRonSource {
            source_file: "data/run/policy.ron".into(),
            object_id: Some("run_policy".into()),
            field_path: Some(self.tuning_path.clone()),
            ..Default::default()
        };
        let mut primary = encounter_source.clone();
        if let Some((wave_index, field)) = failure.and_then(|failure| failure.wave_field.as_ref()) {
            if *wave_index < encounter.waves.len() {
                primary.field_path = Some(format!(
                    "encounters[{}].waves[{wave_index}].{field}",
                    self.encounter_index
                ));
            } else {
                primary = policy_source.clone();
                primary.field_path = Some(format!(
                    "{}.extra_waves[{}].{field}",
                    self.tuning_path,
                    wave_index - encounter.waves.len()
                ));
            }
        }
        let mut related_sources = vec![encounter_source, policy_source];
        if self.revival.is_some() {
            related_sources.push(LiveRonSource {
                source_file: "data/pve/encounters.ron".into(),
                object_id: Some(encounter.id.clone()),
                field_path: Some("reward_sources.revival".into()),
                ..Default::default()
            });
        }
        let template_id = failure
            .and_then(|failure| failure.template_id.as_deref())
            .or_else(|| preview.map(|preview| preview.battlefield_template_id.as_str()));
        if let Some((index, template)) = template_id.and_then(|id| {
            self.game_data
                .map_catalogs
                .battlefield_templates
                .templates
                .iter()
                .enumerate()
                .find(|(_, template)| template.id == id)
        }) {
            related_sources.push(LiveRonSource {
                source_file: "data/map/battlefield_templates.ron".into(),
                object_id: Some(template.id.clone()),
                field_path: Some(format!("[{index}]")),
                ..Default::default()
            });
        }
        if let Some(binding) = &encounter.boss_runtime {
            related_sources.push(LiveRonSource {
                source_file: "data/boss_runtime/base.ron".into(),
                object_id: Some(binding.encounter_id.as_str().to_owned()),
                field_path: Some("encounters".into()),
                ..Default::default()
            });
            // Setup feasibility consumes every PatternCall candidate in the
            // selected encounter. Retain that actual compiled dependency graph,
            // including definitions replaced in another source file.
            if let Some(definition) = self
                .game_data
                .boss_runtime_data
                .get_encounter(&binding.encounter_id)
            {
                use crate::game::data::boss_runtime_data::CompiledBossControlNode;
                let mut seen = HashSet::new();
                for phase in &definition.phases {
                    for actor in &phase.actor_behaviors {
                        let behavior_id = actor.behavior.id.as_str();
                        if seen.insert(("actor_behaviors", behavior_id)) {
                            related_sources.push(LiveRonSource {
                                source_file: "data/boss_runtime/base.ron".into(),
                                object_id: Some(behavior_id.to_owned()),
                                field_path: Some("actor_behaviors".into()),
                                ..Default::default()
                            });
                        }
                        for node in &actor.behavior.nodes {
                            if let CompiledBossControlNode::PatternCall { candidates, .. } = node {
                                for candidate in candidates {
                                    let pattern_id = candidate.pattern.id.as_str();
                                    if seen.insert(("patterns", pattern_id)) {
                                        related_sources.push(LiveRonSource {
                                            source_file: "data/boss_runtime/base.ron".into(),
                                            object_id: Some(pattern_id.to_owned()),
                                            field_path: Some("patterns".into()),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        related_sources.retain(|source| source != &primary);
        let mut error = LiveRonError::validation(
            primary.source_file,
            "generated_combat_preview",
            primary.object_id,
            primary.field_path,
            cause,
        );
        error.related_sources = related_sources;
        error.with_phase(LiveRonPhase::GeneratedSetup)
    }
}

fn baseline_tuning_path(game_data: &GameDataBase) -> String {
    use crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand;
    let (profile_index, profile) = game_data
        .run_policy
        .combat_difficulty
        .profiles
        .iter()
        .enumerate()
        .find(|(_, profile)| profile.id == "default_facility")
        .expect("validated default combat difficulty profile");
    let band_index = profile
        .ordinary_bands
        .iter()
        .position(|entry| entry.band == OrdinaryCombatDifficultyBand::Standard)
        .expect("validated Standard combat tuning");
    format!("combat_difficulty.profiles[{profile_index}].ordinary_bands[{band_index}].tuning")
}

fn validate_combat_preview_threat_warning_contract(
    game_data: &Arc<GameDataBase>,
) -> Result<(), super::live_ron::LiveRonError> {
    const PREVIEW_VALIDATION_SEEDS: [u64; 5] = [0, 1, 17, 41, 99];
    use crate::authoring::control::{self, ExecutionPhase};
    let _phase = control::phase(ExecutionPhase::GeneratedSetup);
    let control_error = |error| {
        super::live_ron::LiveRonError::validation(
            "generated_setup",
            "execution_control",
            None::<String>,
            None::<String>,
            format!("{error:?}"),
        )
        .with_phase(super::live_ron::LiveRonPhase::GeneratedSetup)
    };
    control::charge(1, "generated_validation_entry").map_err(control_error)?;

    for (encounter_index, encounter) in game_data.pve_data.encounters.iter().enumerate() {
        control::charge(1, "generated_encounter").map_err(control_error)?;
        let baseline_context = GeneratedValidationContext {
            game_data,
            encounter_index,
            tuning_path: baseline_tuning_path(game_data),
            revival: None,
        };
        for seed in PREVIEW_VALIDATION_SEEDS {
            control::charge(1, "generated_baseline_seed").map_err(control_error)?;
            let node_id = MapNodeId::new(Uuid::from_u128(
                0xADAD_0000_0000_0000_0000_0000_0000_0000_u128 + u128::from(seed),
            ));
            let preview = CombatPreview::try_generate_from_request_with_diagnostic(
                    node_id,
                    crate::game::combat_preview::BattlefieldGenerationRequest {
                        category: MapNodeCategory::Combat,
                        encounter_id: Some(encounter.id.as_str()), seed, floor_index: 0,
                    },
                    game_data.as_ref(), None,
                )
                .map_err(|preview_error| {
                    baseline_context.error(
                        format!(
                            "combat preview generation failed for baseline difficulty seed {seed}: {:?}",
                            preview_error.error
                        ), Some(&preview_error), None,
                    )
                })?;
            let required = required_briefing_warning_tags_for_spawn_waves(
                &preview.spawn_waves,
                game_data.as_ref(),
            );
            let briefing_tags = preview
                .threat_warnings
                .iter()
                .filter(|warning| warning.source == ThreatWarningSource::Briefing)
                .map(|warning| warning.tag)
                .collect::<HashSet<_>>();

            for required_tag in required {
                if !briefing_tags.contains(&required_tag) {
                    return Err(baseline_context.error(
                            format!(
                                "combat preview baseline difficulty seed {seed} omits required briefing warning {required_tag:?}"
                            ), None, Some(&preview),
                        ));
                }
            }
        }

        let category = if encounter.node_type == Some(CombatNodeType::Boss) {
            MapNodeCategory::Boss
        } else {
            MapNodeCategory::Combat
        };
        for (
            difficulty_index,
            (difficulty_label, difficulty, reward_multiplier_percent, tuning_path),
        ) in manifested_difficulty_cases(game_data.as_ref(), &encounter.id)
            .map_err(control_error)?
            .into_iter()
            .enumerate()
        {
            let context = GeneratedValidationContext {
                game_data,
                encounter_index,
                tuning_path,
                revival: difficulty.revival,
            };
            for seed in PREVIEW_VALIDATION_SEEDS {
                control::charge(1, "generated_manifested_seed").map_err(control_error)?;
                let node_id = MapNodeId::new(Uuid::from_u128(
                    0xB055_0500_0000_0000_0000_0000_0000_0000_u128
                        + ((encounter_index as u128) << 32)
                        + ((difficulty_index as u128) << 16)
                        + u128::from(seed),
                ));
                let preview = CombatPreview::try_generate_from_request_with_diagnostic(
                    node_id,
                    crate::game::combat_preview::BattlefieldGenerationRequest {
                        category, encounter_id: Some(encounter.id.as_str()), seed, floor_index: 0,
                    },
                    game_data.as_ref(), Some(&difficulty),
                )
                .map_err(|preview_error| {
                    context.error(
                        format!(
                            "combat preview generation failed for manifested difficulty {difficulty_label} seed {seed}: {:?}", preview_error.error
                        ), Some(&preview_error), None,
                    )
                })?;
                validate_preview_briefing_warnings(&context, &difficulty_label, seed, &preview)?;
                CombatExecutor::validate_live_combat_setup_feasibility(
                    game_data.clone(),
                    &encounter.id,
                    seed,
                    &preview,
                    reward_multiplier_percent,
                )
                .map_err(|setup_error| {
                    context.error(
                        format!(
                            "resolved battle setup failed for manifested difficulty {difficulty_label} seed {seed}: {setup_error:?}"
                        ), None, Some(&preview),
                    )
                })?;
            }
        }
    }
    control::poll("generated_validation_complete").map_err(control_error)?;
    Ok(())
}

fn validate_preview_briefing_warnings(
    context: &GeneratedValidationContext<'_>,
    difficulty_label: &str,
    seed: u64,
    preview: &CombatPreview,
) -> Result<(), super::live_ron::LiveRonError> {
    let required =
        required_briefing_warning_tags_for_spawn_waves(&preview.spawn_waves, context.game_data);
    let briefing_tags = preview
        .threat_warnings
        .iter()
        .filter(|warning| warning.source == ThreatWarningSource::Briefing)
        .map(|warning| warning.tag)
        .collect::<HashSet<_>>();
    for required_tag in required {
        if !briefing_tags.contains(&required_tag) {
            return Err(context.error(
                format!(
                    "combat preview manifested difficulty {difficulty_label} seed {seed} omits required briefing warning {required_tag:?}"
                ), None, Some(preview),
            ));
        }
    }
    Ok(())
}

fn manifested_difficulty_cases(
    game_data: &GameDataBase,
    encounter_id: &str,
) -> Result<Vec<(String, MapNodeCombatDifficulty, u32, String)>, crate::game::behavior::GameError> {
    use crate::authoring::control;
    let power_model_version = game_data.run_policy.combat_difficulty.power_model_version;
    let mut cases = Vec::new();
    for (profile_index, profile) in game_data
        .run_policy
        .combat_difficulty
        .profiles
        .iter()
        .enumerate()
    {
        control::charge(1, "generated_difficulty_profile")?;
        for (band_index, entry) in profile.ordinary_bands.iter().enumerate() {
            control::charge(1, "generated_ordinary_case")?;
            push_manifested_difficulty_case(
                game_data,
                &mut cases,
                profile.id.as_str(),
                format!("{}::Ordinary({:?})", profile.id, entry.band),
                power_model_version,
                MapNodeCombatDifficultyClass::Ordinary(entry.band),
                None,
                format!("combat_difficulty.profiles[{profile_index}].ordinary_bands[{band_index}].tuning"),
            );
        }
        for (role_index, entry) in profile.role_tuning.iter().enumerate() {
            control::charge(1, "generated_role_case")?;
            if entry.role
                == crate::game::data::run_policy_data::CombatNodeDifficultyRole::Revival
            {
                for source in &game_data.pve_data.reward_sources {
                    let Some(policy) = &source.revival else { continue; };
                    for stage in [RevivalStage::Stage2, RevivalStage::Stage3] {
                        let authored = policy.stage(stage);
                        if authored.encounter_id != encounter_id { continue; }
                        control::charge(1, "generated_revival_case")?;
                        push_manifested_difficulty_case(game_data, &mut cases, &profile.id,
                            format!("{}::Revival::{stage:?}", source.reward_source_id), power_model_version,
                            MapNodeCombatDifficultyClass::Role(entry.role),
                            Some(crate::game::revival::RevivalNodeTuning { stage, stat_multiplier_percent: authored.stat_multiplier_percent }),
                            format!("combat_difficulty.profiles[{profile_index}].role_tuning[{role_index}].tuning"));
                    }
                }
            } else {
                push_manifested_difficulty_case(
                    game_data,
                    &mut cases,
                    profile.id.as_str(),
                    format!("{}::Role({:?})", profile.id, entry.role),
                    power_model_version,
                    MapNodeCombatDifficultyClass::Role(entry.role),
                    None,
                    format!("combat_difficulty.profiles[{profile_index}].role_tuning[{role_index}].tuning"),
                );
            }
        }
    }
    Ok(cases)
}

fn push_manifested_difficulty_case(
    game_data: &GameDataBase,
    cases: &mut Vec<(String, MapNodeCombatDifficulty, u32, String)>,
    profile_id: &str,
    label: String,
    power_model_version: u32,
    class: MapNodeCombatDifficultyClass,
    revival: Option<crate::game::revival::RevivalNodeTuning>,
    tuning_path: String,
) {
    let mut difficulty = MapNodeCombatDifficulty {
        power_model_version,
        difficulty_profile_id: profile_id.to_string(),
        class,
        revival,
        floor_power_anchor: 100,
        target_power: 100,
    };
    let tuning = game_data
        .run_policy
        .resolved_tuning_for_manifested_node(&difficulty)
        .expect("validated live difficulty case must resolve");
    difficulty.target_power = u64::from(tuning.target_power_percent);
    cases.push((
        label,
        difficulty,
        tuning.reward_multiplier_percent,
        tuning_path,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::pve_data::PveEncounterRewardSource;
    use crate::game::data::reward_data::RewardMetadata;
    use crate::game::data::skill_fragment_data::SkillFragmentId;

    #[test]
    fn compiled_parry_guard_metadata_must_agree_with_graph_declaration() {
        let source = crate::authoring::node_graph::tests::parry_fixture().to_string();
        let skills = crate::authoring::node_graph::compile(&source, "r08_strike".into()).unwrap().load_native().unwrap();
        let base = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r09/fragments.ron"));
        for percent in [30,31] {
            let declared = base.replace("ActiveSkill(forms:", &format!("ActiveSkill(passive_guard: Some((reduction_percent: {percent}, reflection_chance_percent: 100, reflection_percent: 50)), forms:"));
            let fragments = ron::de::from_str(&declared).unwrap();
            let result = validate_skill_fragment_skill_references(&fragments, &AbnormalityDatabase::new(vec![]), &skills, &EquipmentDatabase::new(vec![]));
            assert_eq!(result.is_ok(),percent==30,"{result:?}");
        }
    }

    #[test]
    fn compiled_graph_capacity_must_match_fragment_form() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../content_assembler/docs/examples/compiler-preflight/G01.node-graph.json"));
        let skills = crate::authoring::node_graph::compile(source, "r08_strike".into()).unwrap().load_native().unwrap();
        let base = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r09/fragments.ron"));
        for capacity in [99, 100, 101] {
            let fragments = ron::de::from_str(&base.replace("resonance_max: 100", &format!("resonance_max: {capacity}"))).unwrap();
            let result = validate_skill_fragment_skill_references(&fragments, &AbnormalityDatabase::new(vec![]), &skills, &EquipmentDatabase::new(vec![]));
            if capacity == 100 { result.unwrap(); }
            else { assert!(result.unwrap_err().field_path.unwrap().ends_with("resonance_max")); }
        }
    }

    #[test]
    fn r12_passive_guard_requires_bounded_policy_and_parry_in_every_selected_form() {
        let authored = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/r12/passive_guard.ron"
        ));
        let base = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/r09/fragments.ron"
        ));
        let text = base.replace("ActiveSkill(forms:", "ActiveSkill(passive_guard: Some((reduction_percent: 30, reflection_chance_percent: 50, reflection_percent: 25)), forms:");
        let fragments: skill_fragment_data::SkillFragmentDatabase =
            ron::de::from_str(&text).unwrap();
        fragments.validate_contract().unwrap();
        let skills = SkillDatabase::from_ron("r12/guard.ron", authored).unwrap();
        validate_skill_fragment_skill_references(
            &fragments,
            &AbnormalityDatabase::new(vec![]),
            &skills,
            &EquipmentDatabase::new(vec![]),
        )
        .unwrap();
        for field in [
            "reduction_percent: 30",
            "reflection_chance_percent: 50",
            "reflection_percent: 25",
        ] {
            let invalid =
                text.replace(field, &format!("{}: 101", field.split(':').next().unwrap()));
            let fragments: skill_fragment_data::SkillFragmentDatabase =
                ron::de::from_str(&invalid).unwrap();
            let error = fragments.validate_contract().unwrap_err();
            assert!(format!("{error:?}").contains("passive_guard"));
        }
        for authored in [authored.replace("reduction_percent: 0", "reduction_percent: 10"), authored.replace("Sustained(definition: Parry(duration_ms: 200, window_ms: 30, invulnerable_ms: 40, miss_recovery_ms: 20, reduction_percent: 0, counter_skill: \"r12_counter\"))", "Heal(amount: 1)").replace("use_lifetime: ActionAndEffects(effects: [(step: \"guard\", effect: \"parry\")])", "use_lifetime: Action")] {
            let skills = SkillDatabase::from_ron("r12/invalid_guard.ron", &authored).unwrap();
            assert!(validate_skill_fragment_skill_references(&fragments, &AbnormalityDatabase::new(vec![]), &skills, &EquipmentDatabase::new(vec![])).is_err());
        }
    }

    #[test]
    fn ordinary_reward_cannot_grant_an_encounter_source_fragment() {
        let fragment_id = SkillFragmentId::from("source_fragment");
        let rewards = RewardDatabase::new(vec![RewardMetadata {
            id: "invalid_ordinary_reward".to_string(),
            uuid: Uuid::from_u128(1),
            name: "invalid".to_string(),
            description: "invalid".to_string(),
            icon: "invalid".to_string(),
            effects: vec![RewardEffect::GrantSkillFragment {
                fragment_id: fragment_id.clone(),
            }],
        }]);
        let encounters = PveEncounterDatabase::new_with_reward_sources(
            Vec::new(),
            vec![PveEncounterRewardSource {
                reward_source_id: "source".to_string(),
                encounter_id: "encounter".to_string(),
                fragment_id: Some(fragment_id),
                first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 5 }],
                    revival: None,
            }],
        );

        let error = validate_ordinary_reward_source_exclusion(&rewards, &encounters)
            .expect_err("ordinary reward must not bypass the source ledger");
        assert_eq!(error.object_id.as_deref(), Some("invalid_ordinary_reward"));
        assert!(error.cause.contains("encounter-source fragment"));
    }
}
