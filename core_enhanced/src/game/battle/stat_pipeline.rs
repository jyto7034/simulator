//! Battle-entry stat pipeline.
//!
//! Employee units pass through this pipeline in two runtime stages because the
//! battle scenario first stores a combat profile in `BattleUnitDraft`, then the
//! battle builder resolves final stats from that draft:
//!
//! 1. Employee profile stage:
//!    base employee profile -> skill fragments -> active consumable battle modifier.
//! 2. Draft final-stat stage:
//!    draft combat profile, including weapon overrides -> growth stacks ->
//!    equipped item Permanent effects -> equipment enhancement modifiers ->
//!    exact employee trauma starting HP or non-employee current HP ratio preservation.
//!
//! Keep tests for both the individual stages and their composition. The split is
//! an execution boundary, not permission for call sites to invent another order.

use crate::game::{
    battle::types::{BattleUnitDraft, UnitCombatProfile},
    behavior::GameError,
    data::{
        consumable_data::ConsumableEffect, skill_fragment_data::SkillFragmentDatabase, GameDataBase,
    },
    employee::{percent_amount_ceil, Employee},
    growth::GrowthId,
    skill_fragment::SkillFragmentInventory,
};

/// Builds the employee-side battle profile before scenario-level equipment effects are applied.
///
/// Current order:
///
/// 1. Base employee combat profile.
/// 2. Skill fragment loadout.
/// 3. Active consumable battle profile modifier.
pub(crate) fn employee_combat_profile_for_battle(
    employee: &Employee,
    skill_fragments: &SkillFragmentDatabase,
    fragment_inventory: &SkillFragmentInventory,
) -> Result<UnitCombatProfile, GameError> {
    let mut profile = employee.skill_fragments.apply_to_profile(
        skill_fragments,
        fragment_inventory,
        employee.uuid,
        &employee.combat_profile.battle_profile,
    )?;
    apply_consumable_battle_profile_effects(employee, &mut profile);
    Ok(profile)
}

/// Builds the employee profile from persistent loadout inputs only.
///
/// Active consumables and condition-derived HP are deliberately excluded from
/// certified power so readiness cannot rewrite the run's difficulty authority.
pub(crate) fn employee_combat_profile_for_certified_power(
    employee: &Employee,
    skill_fragments: &SkillFragmentDatabase,
    fragment_inventory: &SkillFragmentInventory,
) -> Result<UnitCombatProfile, GameError> {
    employee.skill_fragments.apply_to_profile(
        skill_fragments,
        fragment_inventory,
        employee.uuid,
        &employee.combat_profile.battle_profile,
    )
}

/// Computes final battle-entry stats from an already assembled battle draft.
///
/// Current order:
///
/// 1. Draft combat profile, including weapon profile overrides for employees.
/// 2. Growth stacks.
/// 3. Equipped item Permanent effects.
/// 4. Equipped item enhancement modifiers.
/// 5. Exact employee trauma starting HP after final max HP, or non-employee current HP ratio.
pub(crate) fn effective_stats_for_draft(
    draft: &BattleUnitDraft,
    game_data: &GameDataBase,
) -> Result<crate::game::stats::UnitStats, GameError> {
    let base_profile = draft.combat_profile(game_data)?;
    let base_current_health = base_profile.stats.current_health;
    let base_max_health = base_profile.stats.max_health.max(1);
    let mut stats = persistent_stats_for_draft_profile(draft, game_data, &base_profile)?;

    apply_draft_stat_scale(&mut stats, draft.stat_scale);

    stats.current_health = if let Some(condition) = draft.employee_start_condition {
        let remaining = crate::game::employee::MAX_TRAUMA.saturating_sub(condition.trauma);
        if remaining == 0 {
            0
        } else {
            let condition_hp = (u64::from(stats.max_health) * u64::from(remaining)
                / u64::from(crate::game::employee::MAX_TRAUMA))
                as u32;
            let condition_hp = condition_hp.max(1);
            let bonus = percent_amount_ceil(stats.max_health, condition.battle_hp_bonus_percent);
            condition_hp.saturating_add(bonus).min(stats.max_health)
        }
    } else if base_current_health == 0 {
        0
    } else {
        let scaled = (u128::from(base_current_health) * u128::from(stats.max_health)
            / u128::from(base_max_health))
        .min(u128::from(u32::MAX)) as u32;
        scaled.max(1).min(stats.max_health)
    };

    Ok(stats)
}

/// Resolves permanent employee stats, including growth, equipped item effects,
/// and enhancement levels, without condition HP or scenario difficulty scale.
pub(crate) fn persistent_stats_for_draft(
    draft: &BattleUnitDraft,
    game_data: &GameDataBase,
) -> Result<crate::game::stats::UnitStats, GameError> {
    let base_profile = draft.combat_profile(game_data)?;
    persistent_stats_for_draft_profile(draft, game_data, &base_profile)
}

fn persistent_stats_for_draft_profile(
    draft: &BattleUnitDraft,
    game_data: &GameDataBase,
    base_profile: &UnitCombatProfile,
) -> Result<crate::game::stats::UnitStats, GameError> {
    let mut stats = base_profile.stats;

    // Only the selected weapon contributes base numbers. Behavioral profile
    // application does not add these values, so rebuilding cannot accumulate.
    if let Some(weapon) = &base_profile.weapon_profile {
        stats.attack = stats.attack.checked_add(weapon.attack).ok_or_else(|| {
            GameError::InvalidStaticData("weapon Attack contribution overflowed".into())
        })?;
        stats.spell_power = stats
            .spell_power
            .checked_add(weapon.spell_power)
            .ok_or_else(|| {
                GameError::InvalidStaticData("weapon SpellPower contribution overflowed".into())
            })?;
    }

    let mut bound_copies = std::collections::BTreeMap::new();
    for binding in &draft.equipped_items {
        if let Some(copy) = binding.persistent_copy_uuid {
            if bound_copies.insert(copy, binding.base_uuid).is_some() {
                return Err(GameError::InvalidStaticData(
                    "duplicate equipped copy binding".into(),
                ));
            }
        }
    }
    let mut enhancements = std::collections::BTreeMap::new();
    for enhancement in &draft.equipped_item_enhancements {
        let Some(copy) = enhancement.persistent_copy_uuid else {
            return Err(GameError::InvalidStaticData(
                "enhancement requires an exact copy".into(),
            ));
        };
        if bound_copies.get(&copy) != Some(&enhancement.base_uuid)
            || enhancements
                .insert(copy, enhancement.enhancement_level)
                .is_some()
        {
            return Err(GameError::InvalidStaticData(
                "enhancement copy binding mismatch or duplicate".into(),
            ));
        }
    }

    for (stat_id, value) in &draft.growth_stacks.stacks {
        match stat_id {
            GrowthId::KillStack => {
                stats.add_attack(*value);
            }
        }
    }

    for binding in &draft.equipped_items {
        let origin_item = game_data
            .equipment_data
            .get_by_uuid(&binding.base_uuid)
            .ok_or(GameError::MissingResource(""))?;

        stats
            .apply_permanent_effects(&origin_item.triggered_effects)
            .map_err(|err| match err {
                GameError::InvalidStaticData(message) => GameError::InvalidStaticData(format!(
                    "equipment '{}' invalid Permanent effect: {}",
                    origin_item.id, message
                )),
                other => other,
            })?;

        let enhancement_level = binding
            .persistent_copy_uuid
            .and_then(|copy| enhancements.get(&copy).copied())
            .unwrap_or(0);
        if enhancement_level > 0 {
            if let Some(recipe) = game_data
                .equipment_data
                .get_enhancement_recipe_by_equipment_id(&origin_item.id)
            {
                for modifier in &recipe.modifiers_per_level {
                    for _ in 0..enhancement_level {
                        stats.apply_modifier(*modifier);
                    }
                }
            }
        }
    }

    stats.current_health = stats.max_health;
    Ok(stats)
}

fn apply_draft_stat_scale(
    stats: &mut crate::game::stats::UnitStats,
    scale: crate::game::battle::types::BattleUnitStatScale,
) {
    stats.set_max_health(percent_amount_ceil(
        stats.max_health,
        scale.max_health_percent,
    ));
    stats.attack = percent_amount_ceil(stats.attack, scale.attack_percent);
    stats.defense = scale_i32_percent_ceil(stats.defense, scale.defense_percent);
    if let Some(exact) = scale.exact_override {
        stats.set_max_health(exact.max_health);
        stats.attack = exact.attack;
        stats.defense = exact.defense;
        stats.magic_resist = exact.magic_resistance;
    }
}

fn scale_i32_percent_ceil(value: i32, percent: u32) -> i32 {
    if value == 0 || percent == 100 {
        return value;
    }
    let sign = value.signum();
    let magnitude = value.unsigned_abs();
    let scaled = percent_amount_ceil(magnitude, percent).min(i32::MAX as u32) as i32;
    scaled.saturating_mul(sign)
}

fn apply_consumable_battle_profile_effects(employee: &Employee, profile: &mut UnitCombatProfile) {
    let Some(modifier) = employee.active_consumable_modifier.as_ref() else {
        return;
    };

    match &modifier.effect {
        ConsumableEffect::BattleHpSetup { .. } => {}
        ConsumableEffect::OffenseBoost {
            attack_bonus_percent,
        } if modifier.tier.allows_offense_boost() => {
            let bonus = percent_amount_ceil(profile.stats.attack, *attack_bonus_percent);
            profile.stats.attack = profile.stats.attack.saturating_add(bonus);
        }
        ConsumableEffect::InitialSkillCharge { percent } => {
            let bonus = percent_amount_ceil(profile.resonance.max, *percent);
            profile.resonance.start = profile
                .resonance
                .start
                .saturating_add(bonus)
                .min(profile.resonance.max);
        }
        ConsumableEffect::DefenseMitigation { percent } => {
            let percent = (*percent).min(i32::MAX as u32) as i32;
            profile.incoming_damage_modifiers.damage_reduction_percent = profile
                .incoming_damage_modifiers
                .damage_reduction_percent
                .saturating_add(percent);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    mod r06_stats;
    use std::collections::HashMap;
    use uuid::Uuid;

    use super::*;
    use crate::game::{
        battle::types::BattleUnitSource,
        data::{
            abnormality_data::AbnormalityMetadata,
            consumable_data::{
                ConsumableDurationPolicy, ConsumableMetadata, ConsumableTargetPolicy,
                ConsumableTier,
            },
            equipment_data::{EquipmentMetadata, EquipmentType},
            GameDataBuilder,
        },
        enums::{RiskLevel, Tier},
        growth::GrowthStack,
        skill_fragment::{starter_basic_attack_fragment_id, SkillFragmentLoadout},
        stats::{Effect, StatId, StatModifier, StatModifierKind, TriggerType, TriggeredEffect},
    };

    fn consumable(effect: ConsumableEffect, tier: ConsumableTier) -> ConsumableMetadata {
        ConsumableMetadata {
            id: "test_consumable".to_string(),
            uuid: Uuid::from_u128(0xC0),
            name: "Test Consumable".to_string(),
            description: String::new(),
            tier,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            target_policy: ConsumableTargetPolicy::SingleEmployee,
            duration_policy: ConsumableDurationPolicy::NextCombatNode,
            effect,
            live_pool: false,
        }
    }

    fn weapon_with_permanent_modifier(uuid: Uuid, modifier: StatModifier) -> EquipmentMetadata {
        let mut triggered_effects = HashMap::new();
        triggered_effects.insert(
            TriggerType::Permanent,
            vec![TriggeredEffect::legacy(Effect::Modifier(modifier))],
        );

        EquipmentMetadata {
            id: "test_weapon".to_string(),
            uuid,
            name: "Test Weapon".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects,
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        }
    }

    fn abnormality(uuid: Uuid, attack: u32) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: "test_abnormality".to_string(),
            uuid,
            name: "Test Abnormality".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        }
    }

    #[test]
    fn employee_profile_pipeline_applies_skill_fragments_before_consumable_offense_boost() {
        let mut employee =
            Employee::new(Uuid::from_u128(1), "pipeline_employee", "Pipeline Employee");
        employee.skill_fragments =
            SkillFragmentLoadout::with_baseline_ids(vec![starter_basic_attack_fragment_id()]);
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &consumable(
                ConsumableEffect::OffenseBoost {
                    attack_bonus_percent: 50,
                },
                ConsumableTier::Rare,
            ),
        );

        let profile = employee_combat_profile_for_battle(
            &employee,
            &SkillFragmentDatabase::with_builtin_starter(vec![]),
            &SkillFragmentInventory::new(),
        )
        .unwrap();

        assert_eq!(profile.stats.attack, 18);
    }

    #[test]
    fn draft_stats_pipeline_applies_growth_before_equipment_percent_modifiers() {
        let abno_uuid = Uuid::from_u128(1);
        let weapon_uuid = Uuid::from_u128(2);
        let weapon = weapon_with_permanent_modifier(
            weapon_uuid,
            StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Percent,
                value: 10,
            },
        );
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abnormality(abno_uuid, 100)])
            .with_equipment(vec![weapon])
            .build();

        let mut growth_stacks = GrowthStack::new();
        growth_stacks.add(crate::game::growth::GrowthId::KillStack, 10);
        let draft = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks,
            equipment_layout: None,
            equipped_items: vec![crate::game::battle::types::BattleEquipmentBinding {
                base_uuid: weapon_uuid,
                persistent_copy_uuid: None,
                slot: None,
            }],
            equipped_item_enhancements: vec![],
            employee_start_condition: None,
        };

        let stats = effective_stats_for_draft(&draft, &game_data).unwrap();

        assert_eq!(stats.attack, 121);
    }

    #[test]
    fn draft_stats_pipeline_applies_manifested_difficulty_scale_to_enemy_stats() {
        let abno_uuid = Uuid::from_u128(11);
        let mut abnormality = abnormality(abno_uuid, 10);
        abnormality.max_health = 101;
        abnormality.defense = 3;
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abnormality])
            .build();
        let draft = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(12),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            level: Tier::I,
            stat_scale: crate::game::battle::types::BattleUnitStatScale {
                max_health_percent: 150,
                attack_percent: 125,
                defense_percent: 150,
                exact_override: None,
            },
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
            employee_start_condition: None,
        };

        let stats = effective_stats_for_draft(&draft, &game_data).unwrap();

        assert_eq!(stats.max_health, 152);
        assert_eq!(stats.current_health, 152);
        assert_eq!(stats.attack, 13);
        assert_eq!(stats.defense, 5);
        assert_eq!(stats.magic_resist, 0);
    }

    #[test]
    fn authored_stage_three_scale_saturates_stats_and_preserves_full_health() {
        let mut profile = UnitCombatProfile::employee_default();
        profile.stats =
            crate::game::stats::UnitStats::with_values(u32::MAX, u32::MAX, u32::MAX, -3, 1_000);
        let draft = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(0x207),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(0x208),
                profile,
            },
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            level: Tier::I,
            stat_scale: crate::game::battle::types::BattleUnitStatScale {
                max_health_percent: 207,
                attack_percent: 182,
                defense_percent: 182,
                exact_override: None,
            },
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
            employee_start_condition: None,
        };

        let stats = effective_stats_for_draft(&draft, &GameDataBuilder::empty().build()).unwrap();

        assert_eq!(stats.max_health, i32::MAX as u32);
        assert_eq!(stats.current_health, i32::MAX as u32);
        assert_eq!(stats.attack, u32::MAX);
        assert_eq!(stats.defense, -6);
    }

    #[test]
    fn employee_start_hp_uses_final_max_hp_then_trauma_floor_and_battle_hp_bonus() {
        let weapon_uuid = Uuid::from_u128(0xA11);
        let weapon = weapon_with_permanent_modifier(
            weapon_uuid,
            StatModifier {
                stat: StatId::MaxHealth,
                kind: StatModifierKind::Percent,
                value: 50,
            },
        );
        let game_data = GameDataBuilder::empty()
            .with_equipment(vec![weapon])
            .build();
        let draft = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(0xE11),
            source: BattleUnitSource::Employee(UnitCombatProfile::employee_default()),
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: vec![crate::game::battle::types::BattleEquipmentBinding {
                base_uuid: weapon_uuid,
                persistent_copy_uuid: None,
                slot: None,
            }],
            equipped_item_enhancements: vec![],
            employee_start_condition: Some(
                crate::game::battle::types::EmployeeBattleStartCondition {
                    trauma: 25,
                    battle_hp_bonus_percent: 20,
                },
            ),
        };

        let stats = effective_stats_for_draft(&draft, &game_data).unwrap();

        assert_eq!(stats.max_health, 150);
        assert_eq!(stats.current_health, 142); // floor(150 * .75) + ceil(150 * .20)
    }

    #[test]
    fn final_employee_stats_compose_consumable_boost_before_equipment_percent_effects() {
        let weapon_uuid = Uuid::from_u128(2);
        let weapon = weapon_with_permanent_modifier(
            weapon_uuid,
            StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Percent,
                value: 10,
            },
        );
        let game_data = GameDataBuilder::empty()
            .with_equipment(vec![weapon])
            .build();
        let mut employee =
            Employee::new(Uuid::from_u128(1), "pipeline_employee", "Pipeline Employee");
        employee.skill_fragments =
            SkillFragmentLoadout::with_baseline_ids(vec![starter_basic_attack_fragment_id()]);
        employee.apply_consumable_modifier(
            Uuid::from_u128(4),
            &consumable(
                ConsumableEffect::OffenseBoost {
                    attack_bonus_percent: 50,
                },
                ConsumableTier::Rare,
            ),
        );
        let profile = employee_combat_profile_for_battle(
            &employee,
            &SkillFragmentDatabase::with_builtin_starter(vec![]),
            &SkillFragmentInventory::new(),
        )
        .unwrap();
        let draft = BattleUnitDraft {
            owned_uuid: employee.uuid,
            source: BattleUnitSource::Employee(profile),
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: vec![crate::game::battle::types::BattleEquipmentBinding {
                base_uuid: weapon_uuid,
                persistent_copy_uuid: None,
                slot: None,
            }],
            equipped_item_enhancements: vec![],
            employee_start_condition: Some(
                crate::game::battle::types::EmployeeBattleStartCondition {
                    trauma: employee.condition.trauma,
                    battle_hp_bonus_percent: 0,
                },
            ),
        };

        let stats = effective_stats_for_draft(&draft, &game_data).unwrap();

        // Base employee attack is 12. The committed order is:
        // consumable +50% => 18, then equipment +10% => 19.
        // If equipment percent applied before the consumable stage, this would be 20.
        assert_eq!(stats.attack, 19);
    }
}
