use std::collections::{HashMap, HashSet};

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
        equipment_data::EquipmentDatabase,
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
    enums::RewardMode,
    map::{MapNodeCategory, MapNodeId},
    resources::item_slot::{EquippedRef, ItemSlot},
    reward::{RewardEffect, RewardOption},
    reward_policy::CombatRewardPolicy,
};

use super::GameDataBase;

pub(super) fn validate_monolith_references(
    run_policy: &crate::game::data::run_policy_data::RunPolicyData,
    shop_data: &ShopDatabase,
    visual_novel_data: &VisualNovelDatabase,
    pve_data: &PveEncounterDatabase,
    abnormality_data: &AbnormalityDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    let policy = &run_policy.monolith;
    let shop = shop_data
        .get_by_id(&policy.wandering_merchant_shop_id)
        .ok_or_else(|| {
            crate::game::data::DataValidationError::at(
                &policy.wandering_merchant_shop_id,
                "run_policy.monolith.wandering_merchant_shop_id",
                "references missing Shop",
            )
        })?;
    if shop.context != crate::game::data::shop_data::ShopContext::UnknownExploration
        || shop.offer_count != policy.first_page_offer_count
        || shop
            .refresh_policy
            .is_none_or(|refresh| refresh.max_uses != u32::from(policy.paid_refresh_uses))
    {
        return Err(crate::game::data::DataValidationError::at(
            &shop.id,
            "run_policy.monolith.wandering_merchant_shop_id",
            "monolith merchant must be an unknown-exploration Shop with the authored three-offer/one-refresh contract",
        ));
    }
    if visual_novel_data
        .get_by_id(&policy.exchange_visual_novel_id)
        .is_none()
    {
        return Err(crate::game::data::DataValidationError::at(
            &policy.exchange_visual_novel_id,
            "run_policy.monolith.exchange_visual_novel_id",
            "references missing visual novel",
        ));
    }
    for encounter in pve_data
        .encounters
        .iter()
        .filter(|encounter| encounter.encounter_class == PveEncounterClass::Elite)
    {
        let Some(abnormality_id) = encounter.primary_abnormality_id() else {
            continue;
        };
        let metadata = abnormality_data.get_by_id(abnormality_id).ok_or_else(|| {
            crate::game::data::DataValidationError::at(
                &encounter.id,
                "pve.primary_abnormality_id",
                "monolith-eligible Elite references missing abnormality",
            )
        })?;
        if metadata.response_complete_skill_fragment_id.is_none() {
            return Err(crate::game::data::DataValidationError::at(
                &encounter.id,
                "abnormality.response_complete_skill_fragment_id",
                "monolith-eligible Elite abnormality must define a unique skill fragment",
            ));
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
) {
    skill_data
        .validate_buff_references(buff_data)
        .expect("invalid Skill buff reference");
    validate_skill_fragment_skill_references(skill_fragment_data, abnormality_data, skill_data)
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
    );
    validate_pve_enemy_references(
        pve_data,
        abnormality_data,
        corroded_employee_data,
        corroded_wave_data,
        reward_data,
    )
    .expect("invalid PVE cross-catalog reference");
    validate_event_combat_references(event_data, pve_data).expect("invalid Event combat reference");
    validate_event_choice_key_references(event_data, equipment_data)
        .expect("invalid Event choice-key reference");
    validate_visual_novel_trigger_references(visual_novel_trigger_data, visual_novel_data)
        .expect("invalid visual novel trigger reference");
    validate_no_duplicate_run_start_triggers(visual_novel_trigger_data);
    validate_abnormality_omen_references(abnormality_data, boss_omen_data)
        .expect("invalid abnormality BossOmen reference");
    validate_boss_omen_references(boss_omen_data, abnormality_data, event_data, pve_data)
        .expect("invalid BossOmen chain reference");
}

pub(super) fn validate_shop_item_references(
    shop_data: &ShopDatabase,
    consumable_data: &ConsumableDatabase,
    equipment_data: &EquipmentDatabase,
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
        {
            return Err(crate::game::data::DataValidationError::at(
                &definition.employee_definition_id,
                format!("{loadout_path}.baseline_skill_fragment_ids"),
                "must declare baseline skill fragments",
            ));
        }
        for (fragment_index, fragment_id) in definition
            .initial_loadout
            .baseline_skill_fragment_ids
            .iter()
            .enumerate()
        {
            if skill_fragment_data.get_by_id(fragment_id).is_none() {
                return Err(crate::game::data::DataValidationError::at(
                    &definition.employee_definition_id,
                    format!("{loadout_path}.baseline_skill_fragment_ids[{fragment_index}]"),
                    format!("references missing baseline skill fragment '{fragment_id}'"),
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
    }
    Ok(())
}

pub(super) fn validate_generated_combat_preview_contracts(
    game_data: &GameDataBase,
) -> Result<(), crate::game::data::DataValidationError> {
    validate_combat_preview_threat_warning_contract(game_data)
}

pub(super) fn validate_skill_fragment_skill_references(
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
    abnormality_data: &AbnormalityDatabase,
    skill_data: &SkillDatabase,
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

        let (imitation_skill_id, upgrade_skill_ids, awakened_skill_id) = match &fragment.effect {
            SkillFragmentEffectDef::BasicAttackModifier { .. } => continue,
            SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id,
                upgrade_skill_ids,
                awakened_skill_id,
            } => (imitation_skill_id, upgrade_skill_ids, awakened_skill_id),
        };

        let effect_path = format!("{fragment_path}.effect.ActiveSkill");
        let imitation_skill = skill_data.get_by_id(imitation_skill_id).ok_or_else(|| {
            error(
                format!("{effect_path}.imitation_skill_id"),
                format!("references unknown imitation skill '{imitation_skill_id}'"),
            )
        })?;
        validate_defense_route_skill_contract(imitation_skill)
            .map_err(|cause| error(format!("{effect_path}.imitation_skill_id"), cause))?;
        for (level, skill_id) in upgrade_skill_ids {
            let path = format!("{effect_path}.upgrade_skill_ids[{level}]");
            let upgrade_skill = skill_data.get_by_id(skill_id).ok_or_else(|| {
                error(
                    path.clone(),
                    format!("references unknown upgrade skill '{skill_id}'"),
                )
            })?;
            let _ = (path, upgrade_skill);
        }
        if let Some(awakened_skill_id) = awakened_skill_id {
            let path = format!("{effect_path}.awakened_skill_id");
            let awakened_skill = skill_data.get_by_id(awakened_skill_id).ok_or_else(|| {
                error(
                    path.clone(),
                    format!("references unknown awakened skill '{awakened_skill_id}'"),
                )
            })?;
            let _ = (path, awakened_skill);
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
        if !skill_fragment_data.fragments.is_empty() {
            let response_fragment_id = abnormality
                .response_complete_skill_fragment_id
                .as_ref()
                .ok_or_else(|| {
                    error(
                        format!("{abnormality_path}.response_complete_skill_fragment_id"),
                        "must declare response_complete_skill_fragment_id".to_string(),
                    )
                })?;
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
) {
    for reward in &reward_data.rewards {
        for effect in &reward.effects {
            validate_reward_effect(
                effect,
                reward_data,
                equipment_data,
                consumable_data,
                skill_fragment_data,
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
        RewardEffect::GrantSkillFragmentResearch {
            fragment_id,
            amount,
        } => {
            if *amount == 0 {
                return Err("grants zero skill fragment research progress".to_string());
            }
            skill_fragment_data
                .get_by_id(fragment_id)
                .map(|_| ())
                .ok_or_else(|| {
                    format!("references unknown skill fragment research target '{fragment_id}'")
                })
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
            PveEncounterClass::Normal => {
                if encounter.primary_abnormality_id().is_some() {
                    return Err(error(
                        format!("{encounter_path}.primary_abnormality_id"),
                        "normal encounter must not define primary_abnormality_id".to_string(),
                    ));
                }
            }
            PveEncounterClass::Elite
            | PveEncounterClass::NormalBoss
            | PveEncounterClass::FinalBoss => {
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
                    PveEncounterClass::NormalBoss | PveEncounterClass::FinalBoss
                        if abnormality.threat_class != BattleUnitThreatClass::Boss =>
                    {
                        return Err(error(
                            format!("{encounter_path}.primary_abnormality_id"),
                            format!("primary abnormality '{primary_abnormality_id}' must be Boss"),
                        ));
                    }
                    PveEncounterClass::Normal => unreachable!(),
                    _ => {}
                }
            }
        }

        if encounter.has_bonus_objectives() && encounter.primary_abnormality_id().is_none() {
            return Err(error(
                format!("{encounter_path}.primary_abnormality_id"),
                "encounter defines bonus objectives but has no primary abnormality".to_string(),
            ));
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
        for (reward_index, reward_uuid) in encounter.reward_uuids.iter().enumerate() {
            if reward_data.get_by_uuid(reward_uuid).is_none() {
                return Err(error(
                    format!("{encounter_path}.reward_uuids[{reward_index}]"),
                    format!("references missing reward uuid {reward_uuid}"),
                ));
            }
        }
        if let Some(survive_timer_ms) = encounter.survive_timer_ms {
            if survive_timer_ms == 0 {
                return Err(error(
                    format!("{encounter_path}.survive_timer_ms"),
                    "must be greater than zero".to_string(),
                ));
            }
            if encounter.node_type != Some(CombatNodeType::Defense) {
                return Err(error(
                    format!("{encounter_path}.survive_timer_ms"),
                    "survive_timer_ms requires an explicit Defense encounter".to_string(),
                ));
            }
        }
        if let Some(node_type) = encounter.node_type {
            if encounter.reward_mode != RewardMode::ClaimAll {
                return Err(error(
                    format!("{encounter_path}.reward_mode"),
                    format!(
                        "live combat result rewards are automatically granted; expected ClaimAll, found {:?}",
                        encounter.reward_mode
                    ),
                ));
            }
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
            let rewards = encounter
                .reward_uuids
                .iter()
                .filter_map(|reward_uuid| reward_data.get_by_uuid(reward_uuid))
                .map(RewardOption::from_metadata)
                .collect::<Vec<_>>();
            CombatRewardPolicy::for_mission(node_type, mission_variant)
                .validate_rewards(&rewards)
                .map_err(|message| {
                    error(
                        format!("{encounter_path}.reward_uuids"),
                        format!("invalid {node_type:?} reward policy: {message}"),
                    )
                })?;
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

fn validate_combat_preview_threat_warning_contract(
    game_data: &GameDataBase,
) -> Result<(), crate::game::data::DataValidationError> {
    const PREVIEW_VALIDATION_SEEDS: [u64; 5] = [0, 1, 17, 41, 99];

    for (encounter_index, encounter) in game_data.pve_data.encounters.iter().enumerate() {
        for seed in PREVIEW_VALIDATION_SEEDS {
            let node_id = MapNodeId::new(Uuid::from_u128(
                0xADAD_0000_0000_0000_0000_0000_0000_0000_u128 + u128::from(seed),
            ));
            let preview = CombatPreview::try_generate_for_node_at_floor(
                    node_id,
                    MapNodeCategory::Combat,
                    Some(encounter.id.as_str()),
                    game_data,
                    seed,
                    0,
                )
                .map_err(|preview_error| {
                    crate::game::data::DataValidationError::at(
                        &encounter.id,
                        format!("encounters[{encounter_index}]"),
                        format!(
                            "combat preview generation failed for baseline difficulty seed {}: {preview_error:?}",
                            seed
                        ),
                    )
                })?;
            let required =
                required_briefing_warning_tags_for_spawn_waves(&preview.spawn_waves, game_data);
            let briefing_tags = preview
                .threat_warnings
                .iter()
                .filter(|warning| warning.source == ThreatWarningSource::Briefing)
                .map(|warning| warning.tag)
                .collect::<HashSet<_>>();

            for required_tag in required {
                if !briefing_tags.contains(&required_tag) {
                    return Err(crate::game::data::DataValidationError::at(
                            &encounter.id,
                            format!("encounters[{encounter_index}]"),
                            format!(
                                "combat preview baseline difficulty seed {} omits required briefing warning {required_tag:?}",
                                seed
                            ),
                        ));
                }
            }
        }
    }
    Ok(())
}
