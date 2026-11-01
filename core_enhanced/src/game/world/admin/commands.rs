use serde::Deserialize;
use uuid::Uuid;

use crate::game::enums::RewardMode;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum AdminCommand {
    AdminDumpState,
    AdminDumpSelectedEvent,
    AdminDumpInventory,
    AdminDumpRoster,
    AdminDumpActionCapabilities,
    AdminDumpGrantCatalog,
    AdminEnterSavePoint,
    AdminEnterMaintenance,
    AdminEnterEvent {
        event_id: String,
    },
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
    AdminGrantSkillFragment {
        fragment_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantEquipmentMaterial {
        material_id: String,
        amount: u32,
    },
    AdminGrantRelic {
        definition_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantRelicCorruptionCatalyst {
        amount: u32,
    },
    AdminGrantEquipmentDust {
        amount: u32,
    },
    AdminGrantFragmentDust {
        amount: u32,
    },
    AdminSetEmployeeCondition {
        employee_uuid: Uuid,
        physical_damage: u8,
        trauma: u32,
    },
    AdminSetEnkephalin {
        amount: u32,
    },
}

fn default_admin_count() -> u32 {
    1
}
