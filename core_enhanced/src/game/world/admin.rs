mod catalog;
mod commands;
mod fixtures;
mod grants;
#[cfg(test)]
mod tests;

pub(crate) use commands::AdminCommand;

use serde::Serialize;
use serde_json::{json, Value};

use super::GameCore;
use crate::game::{behavior::GameError, enums::RewardMode};

const ADMIN_NODE_NS: u64 = 0x4144_4d49_4e4e_4f44; // "ADMINNOD"

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdminCommandOutput {
    pub result_type: &'static str,
    pub payload: Value,
}

impl GameCore {
    pub(super) fn prepare_lab_draft(
        &mut self,
        recipe: &crate::authoring::LabPreparation,
    ) -> Result<crate::authoring::LabPreparationReceipt, GameError> {
        use crate::{authoring::control, game::map::MapNodePayload};
        use uuid::Uuid;
        let count = recipe.validate()?;
        if self.state.active_battle.is_some() || !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        self.run_state()?;
        // The recipe names an actual published encounter; failed references are
        // rejected before changing a normal map node or granting bulk loadouts.
        control::charge(
            self.game_data.pve_data.encounters.len() as u64,
            "lab_encounter_reference",
        )?;
        self.game_data
            .pve_data
            .get_by_id(&recipe.encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;
        control::admit(
            control::Resource::CloneSlots,
            0,
            count,
            "lab_bulk_employees",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            count.checked_mul(std::mem::size_of::<(
                Uuid,
                crate::game::employee::EmployeeDefinition,
            )>()),
            "lab_bulk_employees",
        )?;
        let mut plans = Vec::with_capacity(count);
        let mut relic_assignments = Vec::new();
        for group in &recipe.employees {
            control::charge(
                self.game_data.employee_definition_data.definitions.len() as u64,
                "lab_employee_definition",
            )?;
            let definition = self
                .game_data
                .employee_definition_data
                .get_by_id(&group.definition_id)
                .ok_or_else(|| {
                    GameError::InvalidCommandData(format!(
                        "lab preparation references missing employee '{}'",
                        group.definition_id
                    ))
                })?;
            for _ in 0..group.count {
                control::poll("lab_employee_prepare")?;
                let mut definition = crate::authoring::clone_shape::controlled_clone(definition)?;
                if let Some(equipment) = &group.equipment_ids {
                    definition.initial_loadout.equipment_ids =
                        crate::authoring::clone_shape::controlled_clone(equipment)?;
                }
                if let Some(fragment) = &group.active_fragment_id {
                    control::charge_serialized(fragment, "lab_fragment_override")?;
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        Some(fragment.len()),
                        "lab_fragment_override",
                    )?;
                    definition.initial_loadout.active_skill_fragment_id =
                        Some(fragment.as_str().into());
                }
                let id = self.state.uuid_manager.next_employee();
                for (slot, relic) in group.relics.iter().enumerate() {
                    control::admit(
                        control::Resource::CloneSlots,
                        relic_assignments.len(),
                        1,
                        "lab_relic_assignment",
                    )?;
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        Some(64),
                        "lab_relic_assignment",
                    )?;
                    relic_assignments.push((id, slot, relic));
                }
                plans.push((id, definition));
            }
        }
        self.initialize_employees_with_loadouts(&plans)?;
        for (employee, slot, relic) in relic_assignments {
            self.admin_grant_relic(&relic.definition_id, 1)?;
            let copy = self
                .state
                .relics
                .copies()
                .filter(|c| {
                    c.definition_id == relic.definition_id
                        && self.state.relics.equipped_to(c.copy_uuid).ok() == Some(None)
                })
                .map(|c| c.copy_uuid)
                .last()
                .ok_or(GameError::InvalidAction)?;
            if relic.corrupt {
                self.admin_grant_relic_resources(
                    crate::game::reward::RewardEffect::GrantRelicCorruptionCatalyst { amount: 1 },
                )?;
                self.handle_corrupt_relic(copy)?;
            }
            self.handle_equip_relic(copy, employee, slot as u8)?;
        }
        self.validate_structural_loadouts()?;
        // Use an eligible manifested node and its original difficulty authority.
        // The detached admin fixture has no Combat/Boss authority and cannot be
        // used as a substitute for the normal progression and entry snapshot.
        control::charge(
            self.run_state()?.map.nodes.len() as u64,
            "lab_selectable_combat_node",
        )?;
        let selectable = self.selectable_map_node_ids();
        control::charge_serialized(&selectable, "lab_selectable_combat_node")?;
        let node = {
            let run = self.run_state()?;
            selectable.into_iter().find(|id| {
                run.map.node(*id).is_some_and(|node| node.category == recipe.category)
            })
        }.ok_or_else(|| GameError::InvalidCommandData(
            "lab preparation requires a currently selectable manifested combat node of the requested category".into()
        ))?;
        control::charge_serialized(&recipe.encounter_id, "lab_encounter_binding")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<MapNodePayload>() + recipe.encounter_id.len()),
            "lab_encounter_binding",
        )?;
        self.run_state_mut()?
            .map
            .node_mut(node)
            .ok_or(GameError::InvalidAction)?
            .payload = MapNodePayload::Encounter {
            encounter_id: Some(recipe.encounter_id.clone()),
        };
        // These are the production selection, progression, entry-snapshot,
        // preview, plan and battle-commit operations. The outer authoring draft
        // discards every change if any of those operations or deployment fails.
        self.handle_select_map_node(node)?;
        // The outer Lab transaction uses the Admin route. Capture the same
        // pre-entry Continue root as the normal ConfirmEnterNode boundary,
        // before battle construction takes its entry snapshot.
        self.prepare_continue_before_operation(
            super::application::ApplicationRoute::Behavior(
                crate::game::behavior::ActionKind::ConfirmEnterNode,
            ),
        )?;
        self.handle_confirm_enter_node()?;
        for placement in &recipe.placements {
            control::poll("lab_bulk_deploy")?;
            self.handle_deploy_unit(
                plans[placement.employee_index].0,
                placement.position,
                placement.facing,
                None,
                0,
            )?;
        }
        let battle_id = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .battle_uuid;
        for (_, definition) in &plans {
            self.record_lab_definition(
                "Employee",
                definition.employee_definition_id.as_str(),
                "employee_profile",
            )?;
            for fragment in &definition.initial_loadout.baseline_skill_fragment_ids {
                self.record_lab_definition("SkillFragment", fragment.as_str(), "fragment_profile")?;
            }
        }
        self.record_lab_definition("PveEncounter", &recipe.encounter_id, "encounter_plan")?;
        Ok(crate::authoring::LabPreparationReceipt {
            employee_ids: plans.into_iter().map(|p| p.0).collect(),
            battle_id,
            deployed_count: recipe.placements.len(),
        })
    }

    pub(crate) fn prepare_admin_command(
        &mut self,
        command: AdminCommand,
    ) -> Result<super::PreparedApplicationCommit<AdminCommandOutput>, GameError> {
        self.prepare_application_for_publication(
            super::application::ApplicationRoute::AdminCommand,
            move |draft| draft.execute_admin_command_draft(command),
        )
    }

    pub fn execute_admin_command(
        &mut self,
        command: AdminCommand,
    ) -> Result<AdminCommandOutput, GameError> {
        self.execute_application(
            super::application::ApplicationRoute::AdminCommand,
            move |draft| draft.execute_admin_command_draft(command),
        )
    }

    fn execute_admin_command_draft(
        &mut self,
        command: AdminCommand,
    ) -> Result<AdminCommandOutput, GameError> {
        let output = match command {
            AdminCommand::AdminDumpState => {
                self.admin_output("AdminDumpState", self.get_run_snapshot_json()?)
            }
            AdminCommand::AdminDumpSelectedEvent => self.admin_output(
                "AdminDumpSelectedEvent",
                json!({ "selected_event": self.get_selected_event_snapshot_json()? }),
            ),
            AdminCommand::AdminDumpInventory => self.admin_output(
                "AdminDumpInventory",
                json!({ "inventory": self.get_inventory_snapshot_json()? }),
            ),
            AdminCommand::AdminDumpRoster => self.admin_output(
                "AdminDumpRoster",
                json!({ "roster": self.get_employee_roster_snapshot_json()? }),
            ),
            AdminCommand::AdminDumpActionCapabilities => self.admin_output(
                "AdminDumpActionCapabilities",
                json!({ "action_capabilities": self.get_action_capabilities() }),
            ),
            AdminCommand::AdminDumpGrantCatalog => self.admin_dump_grant_catalog()?,
            AdminCommand::AdminEnterSavePoint => self.admin_enter_save_point()?,
            AdminCommand::AdminEnterMaintenance => self.admin_enter_maintenance()?,
            AdminCommand::AdminEnterEvent { event_id } => self.admin_enter_event(event_id)?,
            AdminCommand::AdminEnterShop {
                shop_id,
                shop_pool_id,
            } => self.admin_enter_shop(shop_id, shop_pool_id)?,
            AdminCommand::AdminEnterReward {
                reward_pool_id,
                mode,
                can_skip,
            } => self.admin_enter_reward(
                reward_pool_id,
                mode.unwrap_or(RewardMode::ClaimAll),
                can_skip.unwrap_or(false),
            )?,
            AdminCommand::AdminEnterHeadquartersContact {
                shop_pool_id,
                candidate_count,
            } => {
                self.admin_enter_headquarters_contact(shop_pool_id, candidate_count.unwrap_or(3))?
            }
            AdminCommand::AdminGrantEquipment {
                definition_id,
                count,
            } => self.admin_grant_equipment(&definition_id, count)?,
            AdminCommand::AdminGrantConsumable {
                definition_id,
                count,
            } => self.admin_grant_consumable(&definition_id, count)?,
            AdminCommand::AdminGrantSkillFragment { fragment_id, count } => {
                self.admin_grant_skill_fragment(&fragment_id, count)?
            }
            AdminCommand::AdminGrantEquipmentMaterial {
                material_id,
                amount,
            } => self.admin_grant_equipment_material(&material_id, amount)?,
            AdminCommand::AdminGrantRelic {
                definition_id,
                count,
            } => self.admin_grant_relic(&definition_id, count)?,
            AdminCommand::AdminGrantRelicCorruptionCatalyst { amount } => self
                .admin_grant_relic_resources(
                    crate::game::reward::RewardEffect::GrantRelicCorruptionCatalyst { amount },
                )?,
            AdminCommand::AdminGrantEquipmentDust { amount } => self.admin_grant_relic_resources(
                crate::game::reward::RewardEffect::GrantEquipmentDust { amount },
            )?,
            AdminCommand::AdminGrantFragmentDust { amount } => {
                self.admin_grant_fragment_dust(amount)?
            }
            AdminCommand::AdminSetEmployeeCondition {
                employee_uuid,
                physical_damage,
                trauma,
            } => self.admin_set_employee_condition(employee_uuid, physical_damage, trauma)?,
            AdminCommand::AdminSetEnkephalin { amount } => self.admin_set_enkephalin(amount),
        };

        Ok(output)
    }

    fn admin_output(&self, result_type: &'static str, payload: Value) -> AdminCommandOutput {
        AdminCommandOutput {
            result_type,
            payload,
        }
    }
}
