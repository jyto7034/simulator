//! Certified combat power derived only from persistent roster and loadout state.

use uuid::Uuid;

use crate::game::{
    battle::types::{effective_basic_attack_tile_range, UnitCombatProfile},
    behavior::GameError,
    combat_setup::player_spawns::certified_combat_profile_for_employee,
    data::{run_policy_data::CertifiedCombatPowerWeights, GameDataBase},
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
        .filter(|employee| employee.is_combat_eligible())
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
            i128::from(profile.block_capacity) * i128::from(weights.block_capacity_per_unit),
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
    use super::*;
    use crate::game::{
        ability::SkillId,
        battle::tile_range::TileRangePattern,
        data::{
            consumable_data::{
                ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
                ConsumableTargetPolicy, ConsumableTier,
            },
            GameDataBuilder,
        },
        employee::{Employee, EmployeeCondition, MAX_PHYSICAL_DAMAGE, MAX_TRAUMA},
        enums::RiskLevel,
    };

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
        profile.block_capacity += 1;
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
        let game_data = GameDataBuilder::empty().build();
        let inventory = Inventory::new();
        let fragments = SkillFragmentInventory::new();
        let employee_uuid = Uuid::from_u128(7);
        let mut employee = Employee::new(employee_uuid, "certified", "Certified");
        employee.combat_profile.battle_profile.stats.current_health = 1;
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
        let game_data = GameDataBuilder::empty().build();
        let inventory = Inventory::new();
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
}
