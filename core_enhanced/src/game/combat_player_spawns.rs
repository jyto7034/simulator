use uuid::Uuid;

use crate::game::{
    battle::{
        scenario::{
            ScenarioArtifact, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
            ScenarioUnitSpawn,
        },
        types::{BattleEquipmentEnhancement, BattleUnitDraft, BattleUnitSource},
    },
    behavior::GameError,
    data::GameDataBase,
    employee::EmployeeRoster,
    enums::Side,
    resources::{Inventory, Position},
    skill_fragment::SkillFragmentInventory,
};

#[derive(Debug, Clone)]
pub(crate) struct PlayerScenarioStart {
    pub(crate) spawn_group: ScenarioSpawnGroup,
    pub(crate) artifacts: Vec<ScenarioArtifact>,
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
        let employee = roster.get(&employee_uuid).ok_or(GameError::UnitNotFound)?;
        if !employee.is_available_for_combat() {
            return Err(GameError::InvalidAction);
        }

        let equipped_items: Vec<Uuid> = employee
            .loadout
            .item_slot
            .iter()
            .map(|equipped| equipped.base_uuid)
            .collect();
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
            owned_uuid: employee_uuid,
            source: BattleUnitSource::Employee(
                employee
                    .combat_profile_for_battle(&game_data.skill_fragment_data, skill_fragments)?,
            ),
            level: employee.battle_tier(),
            growth_stacks: employee.combat_profile.growth_stacks.clone(),
            equipped_items,
            equipped_item_enhancements,
        };
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
            spawns,
        },
        artifacts: scenario_artifacts_from_inventory(inventory),
    })
}

pub(crate) fn scenario_artifacts_from_inventory(inventory: &Inventory) -> Vec<ScenarioArtifact> {
    let mut artifact_items = inventory.artifacts.get_all_items();
    artifact_items.sort_by(|a, b| a.uuid.as_bytes().cmp(b.uuid.as_bytes()));
    artifact_items
        .into_iter()
        .enumerate()
        .map(|(index, artifact)| ScenarioArtifact {
            side: Side::Player,
            base_uuid: artifact.uuid,
            instance_salt: index as u32,
        })
        .collect()
}
