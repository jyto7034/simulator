//! Battle-entry stat pipeline.
//!
//! Employee units pass through this pipeline in two runtime stages because the
//! battle scenario first stores a combat profile in `BattleUnitDraft`, then the
//! battle builder resolves final stats from that draft:
//!
//! 1. Employee profile stage:
//!    base employee profile -> skill fragments -> run HP/trauma condition ->
//!    active consumable battle modifier.
//! 2. Draft final-stat stage:
//!    draft combat profile, including weapon overrides -> growth stacks ->
//!    equipped item Permanent effects -> equipment enhancement modifiers ->
//!    artifact Permanent effects -> current HP ratio preservation.
//!
//! Keep tests for both the individual stages and their composition. The split is
//! an execution boundary, not permission for call sites to invent another order.

use uuid::Uuid;

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

/// Builds the employee-side battle profile before scenario-level equipment and artifact
/// effects are applied.
///
/// Current order:
///
/// 1. Base employee combat profile.
/// 2. Skill fragment loadout.
/// 3. Run HP and trauma battle-start condition.
/// 4. Active consumable battle profile modifier.
pub(crate) fn employee_combat_profile_for_battle(
    employee: &Employee,
    skill_fragments: &SkillFragmentDatabase,
    fragment_inventory: &SkillFragmentInventory,
) -> Result<UnitCombatProfile, GameError> {
    let mut profile = employee.skill_fragments.apply_to_profile(
        skill_fragments,
        fragment_inventory,
        &employee.combat_profile.battle_profile,
    )?;
    profile.stats.current_health = employee.battle_start_hp_for_max(profile.stats.max_health);
    apply_consumable_battle_profile_effects(employee, &mut profile);
    Ok(profile)
}

/// Computes final battle-entry stats from an already assembled battle draft.
///
/// Current order:
///
/// 1. Draft combat profile, including weapon profile overrides for employees.
/// 2. Growth stacks.
/// 3. Equipped item Permanent effects.
/// 4. Equipped item enhancement modifiers.
/// 5. Artifact Permanent effects.
/// 6. Run HP ratio preservation after max HP changes.
pub(crate) fn effective_stats_for_draft(
    draft: &BattleUnitDraft,
    game_data: &GameDataBase,
    artifacts: &[Uuid],
) -> Result<crate::game::stats::UnitStats, GameError> {
    let base_profile = draft.combat_profile(game_data)?;
    let base_current_health = base_profile.stats.current_health;
    let base_max_health = base_profile.stats.max_health.max(1);
    let mut stats = base_profile.stats;

    for (stat_id, value) in &draft.growth_stacks.stacks {
        match stat_id {
            GrowthId::KillStack => {
                stats.add_attack(*value);
            }
        }
    }

    for item_uuid in &draft.equipped_items {
        let origin_item = game_data
            .equipment_data
            .get_by_uuid(item_uuid)
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

        let enhancement_level: u32 = draft
            .equipped_item_enhancements
            .iter()
            .filter(|enhancement| enhancement.base_uuid == *item_uuid)
            .map(|enhancement| u32::from(enhancement.enhancement_level))
            .sum();
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

    for artifact_uuid in artifacts {
        let origin_artifact = game_data
            .artifact_data
            .get_by_uuid(artifact_uuid)
            .ok_or(GameError::MissingResource(""))?;

        stats
            .apply_permanent_effects(&origin_artifact.triggered_effects)
            .map_err(|err| match err {
                GameError::InvalidStaticData(message) => GameError::InvalidStaticData(format!(
                    "artifact '{}' invalid Permanent effect: {}",
                    origin_artifact.id, message
                )),
                other => other,
            })?;
    }

    stats.current_health = if base_current_health == 0 {
        0
    } else {
        let scaled = base_current_health.saturating_mul(stats.max_health) / base_max_health;
        scaled.max(1).min(stats.max_health)
    };

    Ok(stats)
}

fn apply_consumable_battle_profile_effects(employee: &Employee, profile: &mut UnitCombatProfile) {
    let Some(modifier) = employee.active_consumable_modifier.as_ref() else {
        return;
    };

    match &modifier.effect {
        ConsumableEffect::BattleHpSetup { bonus_percent } => {
            let bonus = percent_amount_ceil(profile.stats.max_health, *bonus_percent);
            profile.stats.current_health = profile
                .stats
                .current_health
                .saturating_add(bonus)
                .min(profile.stats.max_health);
        }
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
    use std::collections::HashMap;

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
        let mut employee = Employee::new(Uuid::from_u128(1), "pipeline_employee");
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
            growth_stacks,
            equipped_items: vec![weapon_uuid],
            equipped_item_enhancements: vec![],
        };

        let stats = effective_stats_for_draft(&draft, &game_data, &[]).unwrap();

        assert_eq!(stats.attack, 121);
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
        let mut employee = Employee::new(Uuid::from_u128(1), "pipeline_employee");
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
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![weapon_uuid],
            equipped_item_enhancements: vec![],
        };

        let stats = effective_stats_for_draft(&draft, &game_data, &[]).unwrap();

        // Base employee attack is 12. The committed order is:
        // consumable +50% => 18, then equipment +10% => 19.
        // If equipment percent applied before the consumable stage, this would be 20.
        assert_eq!(stats.attack, 19);
    }
}
