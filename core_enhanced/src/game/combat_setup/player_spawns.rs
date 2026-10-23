//! Converts the player's selected employees into scenario drafts.
//!
//! This is the bridge between safe-zone inventory/loadout state and battle-only
//! unit spawn data.

use uuid::Uuid;

use crate::game::{
    battle::{
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        types::{
            BattleEquipmentBinding, BattleEquipmentEnhancement, BattleUnitDraft, BattleUnitSource,
            BattleUnitThreatClass, EmployeeBattleStartCondition, UnitCombatProfile,
        },
    },
    behavior::GameError,
    data::GameDataBase,
    employee::{Employee, EmployeeRoster},
    enums::Side,
    resources::{item_slot::ItemSlot, Inventory, Position},
    skill_fragment::SkillFragmentInventory,
};

#[derive(Debug, Clone)]
pub(crate) struct PlayerScenarioStart {
    pub(crate) spawn_group: ScenarioSpawnGroup,
}

pub(crate) fn player_scenario_start_from_positions(
    roster: &EmployeeRoster,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    mut placements: Vec<(Uuid, Position)>,
) -> Result<PlayerScenarioStart, GameError> {
    placements.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let group_id = ScenarioGroupId::new("player_initial");
    let mut spawns = Vec::new();

    for (index, (employee_uuid, pos)) in placements.into_iter().enumerate() {
        let draft = battle_unit_draft_for_employee(
            roster,
            inventory,
            skill_fragments,
            game_data,
            employee_uuid,
        )?;
        spawns.push(ScenarioUnitSpawn {
            unit_ref: ScenarioUnitRef::new(format!("{}_{}", group_id.0, index)),
            side: Side::Player,
            draft,
            position: pos,
            instance_salt: index as u32,
        });
    }

    Ok(PlayerScenarioStart {
        spawn_group: ScenarioSpawnGroup {
            id: group_id,
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns,
        },
    })
}

pub(crate) fn battle_unit_draft_for_employee(
    roster: &EmployeeRoster,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    employee_uuid: Uuid,
) -> Result<BattleUnitDraft, GameError> {
    battle_unit_draft_for_employee_with_availability(
        roster,
        inventory,
        skill_fragments,
        game_data,
        employee_uuid,
        true,
    )
}

pub(crate) fn effective_combat_profile_for_employee(
    roster: &EmployeeRoster,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    employee_uuid: Uuid,
) -> Result<UnitCombatProfile, GameError> {
    let employee = roster.get(&employee_uuid).ok_or(GameError::UnitNotFound)?;
    effective_combat_profile_for_employee_with_item_slot(
        employee,
        inventory,
        skill_fragments,
        game_data,
        &employee.loadout.item_slot,
    )
}

/// The same final battle projection as spawning, with only the equipped relic
/// penalties appended. Persistent and certified profiles use their old paths.
pub(crate) fn effective_combat_profile_for_employee_with_relics(
    roster: &EmployeeRoster,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    relics: &crate::game::relic::RelicInventory,
    game_data: &GameDataBase,
    employee_uuid: Uuid,
) -> Result<UnitCombatProfile, GameError> {
    let mut profile = effective_combat_profile_for_employee(
        roster,
        inventory,
        skill_fragments,
        game_data,
        employee_uuid,
    )?;
    if relics
        .employee_slots(employee_uuid)
        .iter()
        .any(Option::is_some)
    {
        let draft = battle_unit_draft_for_employee_with_availability(
            roster,
            inventory,
            skill_fragments,
            game_data,
            employee_uuid,
            false,
        )?;
        profile.stats = crate::game::battle::core::relic_numeric::penalize_employee_stats(
            relics,
            game_data,
            employee_uuid,
            draft.effective_stats(game_data)?,
        )?;
    }
    Ok(profile)
}

pub(crate) fn effective_combat_profile_for_employee_with_item_slot(
    employee: &Employee,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    item_slot: &ItemSlot,
) -> Result<UnitCombatProfile, GameError> {
    battle_unit_draft_for_employee_from_item_slot(
        employee.uuid,
        employee,
        inventory,
        skill_fragments,
        game_data,
        item_slot,
    )?
    .combat_profile(game_data)
}

/// Builds a safe-zone-only profile for skill-fragment compatibility checks.
/// A missing weapon remains explicit (`weapon_profile == None`) so this helper
/// cannot be used as a battle gameplay source.
pub(crate) fn skill_fragment_compatibility_profile_for_employee_with_item_slot(
    employee: &Employee,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    item_slot: &ItemSlot,
) -> Result<UnitCombatProfile, GameError> {
    let mut profile =
        employee.combat_profile_for_battle(&game_data.skill_fragment_data, skill_fragments)?;
    let mut weapon_profile = None;
    for equipped in item_slot.iter() {
        let owned = inventory
            .equipments
            .get_item(&equipped.instance_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if owned.meta.uuid != equipped.base_uuid {
            return Err(GameError::InvalidStaticData(
                "equipped copy/base identity mismatch".into(),
            ));
        }
        let equipment = game_data
            .equipment_data
            .get_by_uuid(&equipped.base_uuid)
            .ok_or(GameError::MissingResource("EquipmentMetadata"))?;
        if equipment.equipment_type != crate::game::data::equipment_data::EquipmentType::Weapon {
            continue;
        }
        let next = equipment.weapon_profile.as_ref().ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "weapon equipment '{}' is missing weapon_profile",
                equipment.id
            ))
        })?;
        if weapon_profile.replace(next).is_some() {
            return Err(GameError::InvalidStaticData(
                "employee loadout contains multiple equipped weapons".to_string(),
            ));
        }
    }
    if let Some(weapon_profile) = weapon_profile {
        profile.apply_weapon_profile(weapon_profile)?;
    }
    Ok(profile)
}

pub(crate) fn certified_combat_profile_for_employee(
    employee: &Employee,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
) -> Result<UnitCombatProfile, GameError> {
    let equipped_items = employee
        .loadout
        .item_slot
        .iter_with_address()
        .map(|(slot, equipped)| Ok(BattleEquipmentBinding {
            weapon_stage: inventory.equipments.get_item(&equipped.instance_uuid).ok_or(GameError::InventoryItemNotFound)?.weapon_stage,
            base_uuid: equipped.base_uuid,
            persistent_copy_uuid: Some(equipped.instance_uuid),
            slot: Some(slot),
        }))
        .collect::<Result<Vec<_>, GameError>>()?;
    let equipped_item_enhancements = employee
        .loadout
        .item_slot
        .iter()
        .map(|equipped| {
            let owned = inventory
                .equipments
                .get_item(&equipped.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if owned.meta.uuid != equipped.base_uuid {
                return Err(GameError::InvalidStaticData(
                    "equipped copy/base identity mismatch".into(),
                ));
            }
            Ok(BattleEquipmentEnhancement {
                base_uuid: equipped.base_uuid,
                persistent_copy_uuid: Some(equipped.instance_uuid),
                enhancement_level: owned.enhancement_level,
            })
        })
        .collect::<Result<Vec<_>, GameError>>()?;
    let draft = BattleUnitDraft {
        owned_uuid: employee.uuid,
        source: BattleUnitSource::Employee(
            crate::game::battle::stat_pipeline::employee_combat_profile_for_certified_power(
                employee,
                &game_data.skill_fragment_data,
                skill_fragments,
            )?,
        ),
        threat_class: BattleUnitThreatClass::Normal,
        level: employee.battle_tier(game_data.run_policy.as_ref()),
        stat_scale: Default::default(),
        growth_stacks: employee.combat_profile.growth_stacks.clone(),
        equipment_layout: Some(employee.loadout.item_slot.layout_kind()),
        equipped_items,
        equipped_item_enhancements,
        employee_start_condition: None,
    };
    let mut profile = draft.combat_profile(game_data)?;
    profile.stats =
        crate::game::battle::stat_pipeline::persistent_stats_for_draft(&draft, game_data)?;
    Ok(profile)
}

pub(crate) fn battle_unit_draft_for_employee_with_availability(
    roster: &EmployeeRoster,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    employee_uuid: Uuid,
    require_combat_eligible: bool,
) -> Result<BattleUnitDraft, GameError> {
    let employee = roster.get(&employee_uuid).ok_or(GameError::UnitNotFound)?;
    if require_combat_eligible && !employee.is_combat_eligible() {
        return Err(GameError::InvalidAction);
    }

    battle_unit_draft_for_employee_from_item_slot(
        employee_uuid,
        employee,
        inventory,
        skill_fragments,
        game_data,
        &employee.loadout.item_slot,
    )
}

fn battle_unit_draft_for_employee_from_item_slot(
    employee_uuid: Uuid,
    employee: &Employee,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    item_slot: &ItemSlot,
) -> Result<BattleUnitDraft, GameError> {
    let equipped_items: Vec<BattleEquipmentBinding> = item_slot
        .iter_with_address()
        .map(|(slot, equipped)| Ok(BattleEquipmentBinding {
            weapon_stage: inventory.equipments.get_item(&equipped.instance_uuid).ok_or(GameError::InventoryItemNotFound)?.weapon_stage,
            base_uuid: equipped.base_uuid,
            persistent_copy_uuid: Some(equipped.instance_uuid),
            slot: Some(slot),
        }))
        .collect::<Result<Vec<_>, GameError>>()?;
    let equipped_item_enhancements = item_slot
        .iter()
        .map(|equipped| {
            let owned = inventory
                .equipments
                .get_item(&equipped.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if owned.meta.uuid != equipped.base_uuid {
                return Err(GameError::InvalidStaticData(
                    "equipped copy/base identity mismatch".into(),
                ));
            }
            Ok(BattleEquipmentEnhancement {
                base_uuid: equipped.base_uuid,
                persistent_copy_uuid: Some(equipped.instance_uuid),
                enhancement_level: owned.enhancement_level,
            })
        })
        .collect::<Result<Vec<_>, GameError>>()?;

    Ok(BattleUnitDraft {
        owned_uuid: employee_uuid,
        source: BattleUnitSource::Employee(
            employee.combat_profile_for_battle(&game_data.skill_fragment_data, skill_fragments)?,
        ),
        threat_class: BattleUnitThreatClass::Normal,
        level: employee.battle_tier(game_data.run_policy.as_ref()),
        stat_scale: Default::default(),
        growth_stacks: employee.combat_profile.growth_stacks.clone(),
        equipment_layout: Some(item_slot.layout_kind()),
        equipped_items,
        equipped_item_enhancements,
        employee_start_condition: Some(EmployeeBattleStartCondition {
            trauma: employee.condition.trauma,
            battle_hp_bonus_percent: employee
                .active_consumable_modifier
                .as_ref()
                .and_then(|modifier| match modifier.effect {
                    crate::game::data::consumable_data::ConsumableEffect::BattleHpSetup {
                        bonus_percent,
                    } => Some(bonus_percent),
                    _ => None,
                })
                .unwrap_or(0),
        }),
    })
}
