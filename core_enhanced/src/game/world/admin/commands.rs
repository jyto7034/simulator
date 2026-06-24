use serde::Deserialize;
use uuid::Uuid;

use crate::game::{
    enums::RewardMode,
    map::{SupportNodeMode, SupportNodeType},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdminCommand {
    AdminDumpState,
    AdminDumpSelectedEvent,
    AdminDumpInventory,
    AdminDumpRoster,
    AdminDumpAllowedActions,
    AdminDumpGrantCatalog,
    AdminEnterSupport {
        support_type: SupportNodeType,
        #[serde(default)]
        support_mode: Option<SupportNodeMode>,
    },
    AdminEnterMaintenance,
    AdminEnterShop {
        #[serde(default)]
        shop_id: Option<String>,
        #[serde(default)]
        shop_pool_id: Option<String>,
    },
    AdminEnterReward {
        #[serde(default)]
        reward_pool_id: Option<String>,
        #[serde(default)]
        mode: Option<RewardMode>,
        #[serde(default)]
        can_skip: Option<bool>,
    },
    AdminEnterHeadquartersContact {
        #[serde(default)]
        shop_pool_id: Option<String>,
        #[serde(default)]
        candidate_count: Option<usize>,
    },
    AdminGrantEquipment {
        definition_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantConsumable {
        definition_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantArtifact {
        definition_id: String,
    },
    AdminGrantSkillFragment {
        fragment_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantEquipmentMaterial {
        material_id: String,
        amount: u32,
    },
    AdminGrantFragmentDust {
        amount: u32,
    },
    AdminSetEmployeeHp {
        employee_uuid: Uuid,
        current_hp: u32,
        #[serde(default)]
        max_hp: Option<u32>,
    },
    AdminSetEmployeeTrauma {
        employee_uuid: Uuid,
        trauma: u32,
    },
    AdminSetEnkephalin {
        amount: u32,
    },
}

fn default_admin_count() -> u32 {
    1
}
