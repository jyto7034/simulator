use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::shop_data::{
    ShopContext, ShopMerchandise, ShopRefreshPolicy, ShopSellPolicy,
};
use crate::game::{
    battle::{
        event_log::BattleEventLog,
        result_stats::BattleResultStatsDto,
        types::{BattleWinner, ParticipantBattleResult},
    },
    behavior::GameError,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    data::event_data::{EventCheckOutcome, EventChoiceId, EventChoiceKeyId, EventId, EventSceneId},
    employee::EventAptitude,
    enums::RewardMode,
    map::{HeadquartersContactOption, MapNodeId},
    reward::RewardOption,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOfferState {
    pub offer_id: Uuid,
    pub merchandise: ShopMerchandise,
    pub item_uuid: Uuid,
    pub quantity: u32,
    pub total_price: u32,
    pub repeatable: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopSessionState {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub context: ShopContext,
    pub node_id: MapNodeId,
    pub offer_count: usize,
    pub refresh_policy: Option<ShopRefreshPolicy>,
    pub sell_policy: Option<ShopSellPolicy>,
    pub refresh_uses: u32,
    pub page_index: u32,
    pub offers: Vec<ShopOfferState>,
    pub purchased_merchandise: HashSet<ShopMerchandise>,
}

impl ShopSessionState {
    pub fn remove_offer(&mut self, offer_id: Uuid) -> Result<ShopOfferState, GameError> {
        let pos = self
            .offers
            .iter()
            .position(|offer| offer.offer_id == offer_id)
            .ok_or(GameError::ShopItemNotFound)?;
        Ok(self.offers.remove(pos))
    }
}

#[derive(Debug, Clone)]
pub struct RewardSessionState {
    pub stage_uuid: Uuid,
    pub mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub selected_reward_uuid: Option<Uuid>,
    pub can_skip: bool,
}

impl RewardSessionState {
    pub fn get_selected_reward(&self) -> Option<&RewardOption> {
        self.selected_reward_uuid
            .and_then(|uuid| self.rewards.iter().find(|reward| reward.uuid == uuid))
    }
}

#[derive(Debug, Clone)]
pub struct SavePointSessionState {
    pub node_id: MapNodeId,
}

impl SavePointSessionState {
    pub const fn new(node_id: MapNodeId) -> Self {
        Self { node_id }
    }
}

#[derive(Debug, Clone)]
pub struct MaintenanceSessionState {
    pub node_id: MapNodeId,
}

impl MaintenanceSessionState {
    pub fn new(node_id: MapNodeId) -> Self {
        Self { node_id }
    }
}

#[derive(Debug, Clone)]
pub struct HeadquartersContactSessionState {
    pub node_id: MapNodeId,
    pub options: Vec<HeadquartersContactOption>,
    pub shop_pool_id: Option<String>,
}

impl HeadquartersContactSessionState {
    pub fn new(node_id: MapNodeId, shop_pool_id: Option<String>) -> Self {
        Self {
            node_id,
            options: vec![
                HeadquartersContactOption::RecruitEmployee,
                HeadquartersContactOption::RequestEmergencySupplies,
                HeadquartersContactOption::OpenHeadquartersShop,
            ],
            shop_pool_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventStartedCombatState {
    pub encounter_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_abnormality_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventEquipmentModifierEvidence {
    pub equipment_instance_uuid: Uuid,
    pub equipment_definition_id: String,
    pub aptitude: EventAptitude,
    pub amount: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventChoiceKeyEvidence {
    pub key_id: EventChoiceKeyId,
    pub equipment_instance_uuid: Uuid,
    pub equipment_definition_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventCheckModifierComponent {
    EmployeeAptitude {
        aptitude: EventAptitude,
        amount: u8,
    },
    Equipment {
        equipment_instance_uuid: Uuid,
        equipment_definition_id: String,
        aptitude: EventAptitude,
        amount: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCommittedCheckState {
    pub source_scene_id: EventSceneId,
    pub choice_id: EventChoiceId,
    pub employee_uuid: Uuid,
    pub raw_roll: u8,
    pub aptitude: EventAptitude,
    pub dc: u8,
    pub modifier_components: Vec<EventCheckModifierComponent>,
    pub modifier_total: u8,
    pub total_roll: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_key_evidence: Option<EventChoiceKeyEvidence>,
    pub outcome: EventCheckOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSessionState {
    pub node_id: MapNodeId,
    pub event_id: EventId,
    pub current_scene_id: EventSceneId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_check: Option<EventCommittedCheckState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_combat: Option<EventStartedCombatState>,
}

impl EventSessionState {
    pub fn new(node_id: MapNodeId, event_id: EventId, current_scene_id: EventSceneId) -> Self {
        Self {
            node_id,
            event_id,
            current_scene_id,
            committed_check: None,
            started_combat: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CombatBattleState {
    pub primary_abnormality_id: Option<String>,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub abnormality_uuid: Uuid,
    pub winner: BattleWinner,
    pub event_log: BattleEventLog,
    pub result_stats: BattleResultStatsDto,
    pub bonus_objectives: Vec<crate::game::pve_bonus_objectives::PveBonusObjectiveOutcomeDto>,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub participant_results: Vec<ParticipantBattleResult>,
    pub employee_conditions: crate::game::employee::EmployeeRoster,
}

#[derive(Debug, Clone)]
pub enum ActiveNodeContent {
    Shop(ShopSessionState),
    Reward(RewardSessionState),
    Event(EventSessionState),
    SavePoint(SavePointSessionState),
    Maintenance(MaintenanceSessionState),
    HeadquartersContact(HeadquartersContactSessionState),
    CombatBattle(CombatBattleState),
}

impl ActiveNodeContent {
    pub fn as_shop(&self) -> Result<&ShopSessionState, GameError> {
        match self {
            ActiveNodeContent::Shop(shop) => Ok(shop),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_shop_mut(&mut self) -> Result<&mut ShopSessionState, GameError> {
        match self {
            ActiveNodeContent::Shop(shop) => Ok(shop),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_reward(&self) -> Result<&RewardSessionState, GameError> {
        match self {
            ActiveNodeContent::Reward(reward) => Ok(reward),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_reward_mut(&mut self) -> Result<&mut RewardSessionState, GameError> {
        match self {
            ActiveNodeContent::Reward(reward) => Ok(reward),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_event(&self) -> Result<&EventSessionState, GameError> {
        match self {
            ActiveNodeContent::Event(event) => Ok(event),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_event_mut(&mut self) -> Result<&mut EventSessionState, GameError> {
        match self {
            ActiveNodeContent::Event(event) => Ok(event),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_save_point(&self) -> Result<&SavePointSessionState, GameError> {
        match self {
            ActiveNodeContent::SavePoint(save_point) => Ok(save_point),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_save_point_mut(&mut self) -> Result<&mut SavePointSessionState, GameError> {
        match self {
            ActiveNodeContent::SavePoint(save_point) => Ok(save_point),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_maintenance(&self) -> Result<&MaintenanceSessionState, GameError> {
        match self {
            ActiveNodeContent::Maintenance(maintenance) => Ok(maintenance),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_combat_battle(&self) -> Result<&CombatBattleState, GameError> {
        match self {
            ActiveNodeContent::CombatBattle(battle) => Ok(battle),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_headquarters_contact(&self) -> Result<&HeadquartersContactSessionState, GameError> {
        match self {
            ActiveNodeContent::HeadquartersContact(headquarters) => Ok(headquarters),
            _ => Err(GameError::EventTypeMismatch),
        }
    }
}
