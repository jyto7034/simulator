//! Certified combat power derived only from persistent roster and loadout state.

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::game::{
    battle::types::{effective_basic_attack_tile_range, UnitCombatProfile},
    behavior::GameError,
    combat_setup::player_spawns::certified_combat_profile_for_employee,
    data::{run_policy_data::CertifiedCombatPowerWeights, GameDataBase, ValidatedGameData},
    employee::EmployeeRoster,
    resources::Inventory,
    skill_fragment::SkillFragmentInventory,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifiedCombatPower {
    pub power_model_version: u32,
    pub total: u64,
    pub members: Vec<CertifiedEmployeePower>,
}

/// Input identity is distinct from the existing power formula. SpellPower is
/// recorded but has no certified weight until a later model is approved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CertifiedInputFingerprint {
    pub schema_version: u32,
    pub sha256: String,
    pub spell_power_evaluation_model_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifiedEmployeePower {
    pub employee_uuid: Uuid,
    pub power: u64,
}

pub fn certified_combat_power_for_roster(
    roster: &EmployeeRoster,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
) -> Result<CertifiedCombatPower, GameError> {
    let policy = &game_data.run_policy.combat_difficulty;
    let mut employees = roster
        .iter()
        // Certification represents the whole roster that can enter combat now.
        // Magnitudes inside the eligible boundary remain readiness-only inputs.
        .filter(|employee| employee.is_battle_deployable())
        .collect::<Vec<_>>();
    employees.sort_by_key(|employee| *employee.uuid.as_bytes());

    let mut members = Vec::with_capacity(employees.len());
    let mut total = 0_u64;
    for employee in employees {
        let profile =
            certified_combat_profile_for_employee(employee, inventory, skill_fragments, game_data)?;
        let power = certified_power_for_profile(&profile, employee.level, policy.power_weights);
        total = total.saturating_add(power);
        members.push(CertifiedEmployeePower {
            employee_uuid: employee.uuid,
            power,
        });
    }

    Ok(CertifiedCombatPower {
        power_model_version: policy.power_model_version,
        total,
        members,
    })
}

/// Read-only input projection using the same persistent profile authority as power.
pub fn certified_input_fingerprint_for_roster(
    roster: &EmployeeRoster,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &ValidatedGameData,
) -> Result<CertifiedInputFingerprint, GameError> {
    let mut employees = roster
        .iter()
        .filter(|employee| employee.is_battle_deployable())
        .collect::<Vec<_>>();
    employees.sort_by_key(|employee| *employee.uuid.as_bytes());
    let mut inputs = Vec::with_capacity(employees.len());
    for employee in employees {
        let profile =
            certified_combat_profile_for_employee(employee, inventory, skill_fragments, game_data)?;
        let copies = employee
            .loadout
            .item_slot
            .iter_with_address()
            .map(|(slot, binding)| {
                let owned = inventory
                    .equipments
                    .get_item(&binding.instance_uuid)
                    .ok_or(GameError::InventoryItemNotFound)?;
                Ok((
                    slot,
                    binding.instance_uuid,
                    binding.base_uuid,
                    owned.enhancement_level,
                ))
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        let fragment_copy = employee
            .skill_fragments
            .active_copy_uuid()
            .map(|id| skill_fragments.get(&id).ok_or(GameError::InvalidAction))
            .transpose()?;
        inputs.push(
            crate::game::data::content_pack::canonical::bytes(&(
                employee.uuid,
                employee.level,
                &profile,
                &employee.combat_profile.growth_stacks,
                copies,
                &employee.skill_fragments,
                fragment_copy,
            ))
            .map_err(|e| GameError::InvalidStaticData(format!("certified input encoding: {e}")))?,
        );
    }
    let bytes = crate::game::data::content_pack::canonical::bytes(&(
        "certified_input_v1",
        game_data.identity(),
        inputs,
    ))
    .map_err(|e| GameError::InvalidStaticData(format!("certified input encoding: {e}")))?;
    Ok(CertifiedInputFingerprint {
        schema_version: 1,
        sha256: format!("{:x}", Sha256::digest(bytes)),
        spell_power_evaluation_model_pending: true,
    })
}

fn certified_power_for_profile(
    profile: &UnitCombatProfile,
    level: u32,
    weights: CertifiedCombatPowerWeights,
) -> u64 {
    let level = u64::from(level.max(1)).saturating_mul(u64::from(weights.level_flat));
    let health = weighted_percent(
        u64::from(profile.stats.max_health),
        weights.max_health_percent,
    );
    let attack_dps = u128::from(profile.stats.attack)
        .saturating_mul(1_000)
        .div_ceil(u128::from(profile.stats.attack_interval_ms.max(1)));
    let attack = weighted_percent(
        saturating_u128_to_u64(attack_dps),
        weights.attack_dps_percent,
    );
    let attack_range = effective_basic_attack_tile_range(&profile.basic_attack);
    let range_cells = attack_range
        .rows
        .iter()
        .flat_map(|row| row.chars())
        .filter(|cell| *cell == 'X')
        .count()
        .saturating_add(attack_range.include_anchor_tile as usize);

    let positive = i128::from(level)
        .saturating_add(i128::from(health))
        .saturating_add(i128::from(attack))
        .saturating_add(
            i128::from(
                profile
                    .weapon_profile
                    .as_ref()
                    .map(|weapon| weapon.block_capacity)
                    .unwrap_or(0),
            ) * i128::from(weights.block_capacity_per_unit),
        )
        .saturating_add(
            i128::try_from(range_cells).unwrap_or(i128::MAX)
                * i128::from(weights.basic_attack_range_per_cell),
        )
        .saturating_add(if profile.skill_id.is_some() {
            i128::from(weights.active_skill_flat)
        } else {
            0
        });
    let signed_defense =
        i128::from(profile.stats.defense).saturating_mul(i128::from(weights.defense_per_point));
    let signed_magic_resist = i128::from(profile.stats.magic_resist)
        .saturating_mul(i128::from(weights.magic_resist_per_point));
    let total = positive
        .saturating_add(signed_defense)
        .saturating_add(signed_magic_resist)
        .max(1);
    u64::try_from(total).unwrap_or(u64::MAX)
}

fn weighted_percent(value: u64, percent: u32) -> u64 {
    saturating_u128_to_u64(
        u128::from(value)
            .saturating_mul(u128::from(percent))
            .div_ceil(100),
    )
}

fn saturating_u128_to_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::game::{
        ability::SkillId,
        battle::tile_range::TileRangePattern,
        data::{
            consumable_data::{
                ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
                ConsumableTargetPolicy, ConsumableTier,
            },
            equipment_data::{EquipmentMetadata, EquipmentType, WeaponCombatProfile},
            GameDataBuilder,
        },
        employee::{Employee, EmployeeCondition, MAX_PHYSICAL_DAMAGE, MAX_TRAUMA},
        enums::RiskLevel,
        resources::{item_slot::EquippedRef, OwnedEquipment},
        stats::TriggeredEffects,
    };

    fn certification_weapon() -> EquipmentMetadata {
        EquipmentMetadata {
            content_family: None,
            id: "certification_weapon".to_string(),
            uuid: Uuid::from_u128(0xCE47_1F1C_A710_0001),
            name: "Certification Weapon".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: TriggeredEffects::default(),
            ability_activations: vec![],
            weapon_profile: Some(WeaponCombatProfile::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        }
    }

    fn arm_employee(
        employee: &mut Employee,
        inventory: &mut Inventory,
        weapon: &EquipmentMetadata,
    ) {
        let instance_uuid = employee.uuid;
        let mut owned = OwnedEquipment::new(instance_uuid, Arc::new(weapon.clone()));
        owned.equipped_to = Some(employee.uuid);
        inventory.equipments.add_item(owned).unwrap();
        employee
            .loadout
            .item_slot
            .equip(
                EquippedRef {
                    instance_uuid,
                    base_uuid: weapon.uuid,
                    equipment_type: EquipmentType::Weapon,
                },
                weapon.allow_duplicate_equip,
            )
            .unwrap();
    }

    #[test]
    fn provisional_formula_is_sensitive_to_every_authored_quantitative_input() {
        let weights = crate::game::data::run_policy_data::RunPolicyData::builtin()
            .combat_difficulty
            .power_weights;
        let baseline = UnitCombatProfile::employee_default();
        let baseline_power = certified_power_for_profile(&baseline, 1, weights);

        assert!(certified_power_for_profile(&baseline, 2, weights) > baseline_power);

        let mut profile = baseline.clone();
        profile.stats.max_health += 10;
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);

        let mut profile = baseline.clone();
        profile.stats.attack += 10;
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);

        let mut profile = baseline.clone();
        profile.stats.attack_interval_ms = (profile.stats.attack_interval_ms / 2).max(1);
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);

        let mut profile = baseline.clone();
        profile.stats.defense += 1;
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);

        let mut profile = baseline.clone();
        profile.stats.magic_resist += 1;
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);

        let mut profile = baseline.clone();
        profile
            .apply_weapon_profile(&WeaponCombatProfile {
                block_capacity: 1,
                ..WeaponCombatProfile::default()
            })
            .unwrap();
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);

        let mut profile = baseline.clone();
        profile.basic_attack.defense_tile_range = Some(TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
        });
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);

        let mut profile = baseline;
        profile.skill_id = Some(SkillId::new("certified_active_skill"));
        assert!(certified_power_for_profile(&profile, 1, weights) > baseline_power);
    }

    #[test]
    fn certification_is_deterministic_and_ignores_eligible_condition_magnitudes_and_current_hp() {
        let weapon = certification_weapon();
        let game_data = GameDataBuilder::empty()
            .with_equipment(vec![weapon.clone()])
            .build();
        let mut inventory = Inventory::new();
        let fragments = SkillFragmentInventory::new();
        let employee_uuid = Uuid::from_u128(7);
        let mut employee = Employee::new(employee_uuid, "certified", "Certified");
        employee.combat_profile.battle_profile.stats.current_health = 1;
        arm_employee(&mut employee, &mut inventory, &weapon);
        let mut roster = EmployeeRoster::new();
        roster.add(employee.clone());

        let baseline =
            certified_combat_power_for_roster(&roster, &inventory, &fragments, &game_data).unwrap();
        employee.condition = EmployeeCondition::new(4, MAX_TRAUMA - 1);
        roster.add(employee.clone());
        let distressed =
            certified_combat_power_for_roster(&roster, &inventory, &fragments, &game_data).unwrap();

        assert_eq!(baseline, distressed);
        assert_eq!(baseline.power_model_version, 2);
        assert_eq!(baseline.members[0].employee_uuid, employee_uuid);
        assert!(baseline.total > 0);

        employee.apply_consumable_modifier(
            Uuid::from_u128(8),
            &ConsumableMetadata {
                id: "temporary_offense".to_string(),
                uuid: Uuid::from_u128(8),
                name: "Temporary Offense".to_string(),
                description: String::new(),
                tier: ConsumableTier::Rare,
                rarity: RiskLevel::ZAYIN,
                price: 0,
                target_policy: ConsumableTargetPolicy::SingleEmployee,
                duration_policy: ConsumableDurationPolicy::NextCombatNode,
                effect: ConsumableEffect::OffenseBoost {
                    attack_bonus_percent: 500,
                },
                live_pool: false,
            },
        );
        roster.add(employee);
        let temporary_boosted =
            certified_combat_power_for_roster(&roster, &inventory, &fragments, &game_data).unwrap();
        assert_eq!(baseline, temporary_boosted);
    }

    #[test]
    fn certification_membership_matches_combat_eligibility_boundaries() {
        let weapon = certification_weapon();
        let game_data = GameDataBuilder::empty()
            .with_equipment(vec![weapon.clone()])
            .build();
        let fragments = SkillFragmentInventory::new();

        for (physical_damage, trauma, expected_eligible) in [
            (0, 0, true),
            (4, 0, true),
            (4, MAX_TRAUMA - 1, true),
            (5, 0, false),
            (0, MAX_TRAUMA, false),
            (MAX_PHYSICAL_DAMAGE, 0, false),
        ] {
            let employee_uuid = Uuid::from_u128(
                0xE110_0000 + u128::from(physical_damage) * 1_000 + u128::from(trauma),
            );
            let mut employee = Employee::new(employee_uuid, "eligibility", "Eligibility");
            employee.condition = EmployeeCondition::new(physical_damage, trauma);
            let mut inventory = Inventory::new();
            arm_employee(&mut employee, &mut inventory, &weapon);
            let mut roster = EmployeeRoster::new();
            roster.add(employee);

            let certified =
                certified_combat_power_for_roster(&roster, &inventory, &fragments, &game_data)
                    .unwrap();

            assert_eq!(
                certified
                    .members
                    .iter()
                    .any(|member| member.employee_uuid == employee_uuid),
                expected_eligible,
                "physical_damage={physical_damage}, trauma={trauma}"
            );
            assert_eq!(certified.total > 0, expected_eligible);
        }
    }

    #[test]
    fn permanently_dead_employee_is_not_certified() {
        let game_data = GameDataBuilder::empty().build();
        let mut employee = Employee::new(Uuid::from_u128(9), "dead", "Dead");
        employee.condition = EmployeeCondition::new(MAX_PHYSICAL_DAMAGE, 0);
        let mut roster = EmployeeRoster::new();
        roster.add(employee);

        let certified = certified_combat_power_for_roster(
            &roster,
            &Inventory::new(),
            &SkillFragmentInventory::new(),
            &game_data,
        )
        .unwrap();

        assert_eq!(certified.total, 0);
        assert!(certified.members.is_empty());
    }
    #[test]
    fn r06_fingerprint_is_order_stable_and_includes_persistent_spell_power_and_copy() {
        let mut weapon = certification_weapon();
        weapon.weapon_profile.as_mut().unwrap().attack = 40;
        weapon.weapon_profile.as_mut().unwrap().spell_power = 20;
        let game_data: ValidatedGameData = Arc::new(
            GameDataBuilder::empty()
                .with_equipment(vec![weapon.clone()])
                .build(),
        )
        .into();
        let mut inventory = Inventory::new();
        let fragments = SkillFragmentInventory::new();
        let mut a = Employee::new(Uuid::from_u128(1), "a", "A");
        let mut b = Employee::new(Uuid::from_u128(2), "b", "B");
        arm_employee(&mut a, &mut inventory, &weapon);
        arm_employee(&mut b, &mut inventory, &weapon);
        let mut roster = EmployeeRoster::new();
        roster.add(a.clone());
        roster.add(b.clone());
        let fingerprint = |r: &EmployeeRoster, i: &Inventory| {
            certified_input_fingerprint_for_roster(r, i, &fragments, &game_data).unwrap()
        };
        let original = fingerprint(&roster, &inventory);
        assert!(original.spell_power_evaluation_model_pending);
        let power =
            certified_combat_power_for_roster(&roster, &inventory, &fragments, &game_data).unwrap();
        let mut reordered = EmployeeRoster::new();
        reordered.add(b);
        reordered.add(a.clone());
        assert_eq!(original, fingerprint(&reordered, &inventory));
        a.combat_profile.battle_profile.stats.current_health = 1;
        a.condition = EmployeeCondition::new(3, 4);
        reordered.add(a.clone());
        assert_eq!(original, fingerprint(&reordered, &inventory));
        a.combat_profile.battle_profile.stats.spell_power = 1;
        reordered.add(a.clone());
        assert_ne!(original, fingerprint(&reordered, &inventory));
        assert_eq!(
            power,
            certified_combat_power_for_roster(&reordered, &inventory, &fragments, &game_data)
                .unwrap()
        );
        a.combat_profile.battle_profile.stats.spell_power = 0;
        let old_binding = a.loadout.item_slot.iter().next().unwrap().clone();
        let new_uuid = Uuid::from_u128(77);
        let mut owned = OwnedEquipment::new(new_uuid, Arc::new(weapon.clone()));
        owned.equipped_to = Some(a.uuid);
        inventory.equipments.add_item(owned).unwrap();
        // Recreate the slot with the same definition and different persistent copy.
        a.loadout
            .item_slot
            .remove_by_instance(old_binding.instance_uuid)
            .unwrap();
        a.loadout
            .item_slot
            .equip(
                EquippedRef {
                    instance_uuid: new_uuid,
                    base_uuid: weapon.uuid,
                    equipment_type: EquipmentType::Weapon,
                },
                true,
            )
            .unwrap();
        reordered.add(a);
        assert_ne!(original, fingerprint(&reordered, &inventory));
        assert_eq!(
            power,
            certified_combat_power_for_roster(&reordered, &inventory, &fragments, &game_data)
                .unwrap()
        );
    }
}
