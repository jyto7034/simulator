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
