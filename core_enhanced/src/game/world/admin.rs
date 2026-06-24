mod catalog;
mod commands;
mod fixtures;
mod grants;
#[cfg(test)]
mod tests;

pub use commands::AdminCommand;

use serde::Serialize;
use serde_json::{json, Value};

use super::GameCore;
use crate::game::{behavior::GameError, enums::RewardMode, map::SupportNodeMode};

const ADMIN_NODE_NS: u64 = 0x4144_4d49_4e4e_4f44; // "ADMINNOD"

#[derive(Debug, Clone, Serialize)]
pub struct AdminCommandOutput {
    pub result_type: &'static str,
    pub payload: Value,
}

impl GameCore {
    pub fn execute_admin_command(
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
            AdminCommand::AdminDumpAllowedActions => self.admin_output(
                "AdminDumpAllowedActions",
                json!({ "allowed_actions": self.get_allowed_actions() }),
            ),
            AdminCommand::AdminDumpGrantCatalog => self.admin_dump_grant_catalog()?,
            AdminCommand::AdminEnterSupport {
                support_type,
                support_mode,
            } => self.admin_enter_support(
                support_type,
                support_mode.unwrap_or(SupportNodeMode::Known),
            )?,
            AdminCommand::AdminEnterMaintenance => self.admin_enter_maintenance()?,
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
            AdminCommand::AdminGrantArtifact { definition_id } => {
                self.admin_grant_artifact(&definition_id)?
            }
            AdminCommand::AdminGrantSkillFragment { fragment_id, count } => {
                self.admin_grant_skill_fragment(&fragment_id, count)?
            }
            AdminCommand::AdminGrantEquipmentMaterial {
                material_id,
                amount,
            } => self.admin_grant_equipment_material(&material_id, amount)?,
            AdminCommand::AdminGrantFragmentDust { amount } => {
                self.admin_grant_fragment_dust(amount)?
            }
            AdminCommand::AdminSetEmployeeHp {
                employee_uuid,
                current_hp,
                max_hp,
            } => self.admin_set_employee_hp(employee_uuid, current_hp, max_hp)?,
            AdminCommand::AdminSetEmployeeTrauma {
                employee_uuid,
                trauma,
            } => self.admin_set_employee_trauma(employee_uuid, trauma)?,
            AdminCommand::AdminSetEnkephalin { amount } => self.admin_set_enkephalin(amount),
        };

        self.sync_roster_order_with_owned_units()?;
        Ok(output)
    }

    fn admin_output(&self, result_type: &'static str, payload: Value) -> AdminCommandOutput {
        AdminCommandOutput {
            result_type,
            payload,
        }
    }
}
