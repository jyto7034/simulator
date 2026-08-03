//! Converts the player's selected employees into scenario drafts.
//!
//! This is the bridge between safe-zone inventory/loadout state and battle-only
//! unit spawn data.

use uuid::Uuid;

use crate::game::{
    battle::{
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        types::{
            BattleEquipmentEnhancement, BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass,
            EmployeeBattleStartCondition, UnitCombatProfile,
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

pub(crate) fn certified_combat_profile_for_employee(
    employee: &Employee,
    inventory: &Inventory,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
) -> Result<UnitCombatProfile, GameError> {
    let equipped_items = employee
        .loadout
        .item_slot
        .iter()
        .map(|equipped| equipped.base_uuid)
        .collect::<Vec<_>>();
    let equipped_item_enhancements = employee
        .loadout
        .item_slot
        .iter()
        .map(|equipped| {
            let owned = inventory
                .equipments
                .get_item(&equipped.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            Ok(BattleEquipmentEnhancement {
                base_uuid: equipped.base_uuid,
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
        equipped_items,
        equipped_item_enhancements,
        employee_start_condition: None,
    };
    let mut profile = draft.combat_profile(game_data)?;
    profile.stats =
        crate::game::battle::stat_pipeline::persistent_stats_for_draft(&draft, game_data)?;
    Ok(profile)
}

fn battle_unit_draft_for_employee_with_availability(
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
    let equipped_items: Vec<Uuid> = item_slot
        .iter()
        .map(|equipped| equipped.base_uuid)
        .collect();
    let equipped_item_enhancements = item_slot
        .iter()
        .map(|equipped| {
            let owned = inventory
                .equipments
                .get_item(&equipped.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            Ok(BattleEquipmentEnhancement {
                base_uuid: equipped.base_uuid,
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
