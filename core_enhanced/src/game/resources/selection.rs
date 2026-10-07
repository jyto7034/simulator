use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::shop_data::{
    ShopContext, ShopMerchandise, ShopRefreshPolicy, ShopSellPolicy,
};
use crate::game::{
    battle::{
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

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
use crate::game::battle::event_log::BattleEventLog;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct CommittedRelicCorruptionSelection {
    pub source_scene_id: EventSceneId,
    pub copy_uuid: Option<Uuid>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relic_corruption_selections: Option<Vec<CommittedRelicCorruptionSelection>>,
}

impl EventSessionState {
    pub fn new(node_id: MapNodeId, event_id: EventId, current_scene_id: EventSceneId) -> Self {
        Self {
            node_id,
            event_id,
            current_scene_id,
            committed_check: None,
            started_combat: None,
            relic_corruption_selections: None,
        }
    }
}

// Native membership is the added receipt Vec/String payload only. Existing
// existing OwnedHashMap event_sessions backing belongs to the legacy scope.
// Its closed insertion/removal history is preserved; no new table migration
// or raw capacity heuristic is introduced for the receipt-payload scope.
impl EventSessionState {
    pub(crate) fn has_relic_history(&self) -> bool {
        self.relic_corruption_selections.is_some()
    }
    pub(crate) fn relic_history_extent(&self) -> Result<usize, GameError> {
        let Some(history) = &self.relic_corruption_selections else {
            return Ok(0);
        };
        crate::authoring::control::charge(history.len() as u64, "relic_event_history_extent")?;
        let mut bytes = history
            .capacity()
            .checked_mul(std::mem::size_of::<CommittedRelicCorruptionSelection>())
            .ok_or(GameError::InvalidAction)?;
        for receipt in history {
            bytes = bytes
                .checked_add(receipt.source_scene_id.0.capacity())
                .ok_or(GameError::InvalidAction)?;
        }
        Ok(bytes)
    }
    /// The new opt-in transition copies the entire closed session leaf, so its
    /// legacy strings are included in admission too. Outside-native World
    /// capture and None-only legacy transitions preserve their former scope.
    pub(crate) fn preflight_relic_clone(&self) -> Result<(), GameError> {
        if !crate::game::battle::runtime_control::is_active() {
            return Ok(());
        }
        crate::game::relic::allocation::admit_owned_projection(self, "relic_event_session_clone")?;
        if !crate::authoring::control::is_active() {
            crate::game::resource::store::preflight_resource_clone(self)
                .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        }
        Ok(())
    }
    pub(crate) fn clone_for_relic_transition(&self) -> Result<Self, GameError> {
        if !self.has_relic_history() {
            return Ok(self.clone());
        }
        self.preflight_relic_clone()?;
        crate::authoring::clone_shape::controlled_clone(self)
    }
    pub(crate) fn history_snapshot(
        &self,
    ) -> Result<Option<Vec<CommittedRelicCorruptionSelection>>, GameError> {
        let Some(history) = &self.relic_corruption_selections else {
            return Ok(None);
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(self.relic_history_extent()?),
            "relic_event_history_projection",
        )?;
        if !crate::authoring::control::is_active() {
            crate::game::resource::store::preflight_resource_clone(history)
                .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        }
        Ok(Some(crate::authoring::clone_shape::controlled_clone(
            history,
        )?))
    }
}
impl crate::authoring::clone_shape::CloneShape for CommittedRelicCorruptionSelection {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), GameError> {
        meter.scan_storage(Some(1))?;
        self.source_scene_id.0.visit_clone_shape(meter)?;
        self.copy_uuid.visit_clone_shape(meter)
    }
}

pub(crate) fn has_relic_event_histories(
    sessions: &crate::owned_collections::OwnedHashMap<MapNodeId, EventSessionState>,
) -> bool {
    sessions.values().any(EventSessionState::has_relic_history)
}
pub(crate) fn relic_event_histories_extent(
    sessions: &crate::owned_collections::OwnedHashMap<MapNodeId, EventSessionState>,
) -> Result<usize, GameError> {
    if !has_relic_event_histories(sessions) {
        return Ok(0);
    }
    crate::authoring::control::charge(sessions.len() as u64, "relic_event_history_membership")?;
    sessions.values().try_fold(0usize, |bytes, session| {
        bytes
            .checked_add(session.relic_history_extent()?)
            .ok_or(GameError::InvalidAction)
    })
}
pub(crate) fn preflight_relic_event_histories(
    sessions: &crate::owned_collections::OwnedHashMap<MapNodeId, EventSessionState>,
) -> Result<(), GameError> {
    if !crate::game::battle::runtime_control::is_active() || !has_relic_event_histories(sessions) {
        return Ok(());
    }
    crate::authoring::control::charge(
        sessions.len() as u64,
        "relic_event_history_clone_membership",
    )?;
    for session in sessions
        .values()
        .filter(|session| session.has_relic_history())
    {
        session.preflight_relic_clone()?;
    }
    Ok(())
}
pub(crate) fn active_relic_history_extent(
    content: &Option<ActiveNodeContent>,
) -> Result<usize, GameError> {
    match content {
        Some(ActiveNodeContent::Event(session)) => session.relic_history_extent(),
        _ => Ok(0),
    }
}
pub(crate) fn has_active_relic_history(content: &Option<ActiveNodeContent>) -> bool {
    matches!(content, Some(ActiveNodeContent::Event(session)) if session.has_relic_history())
}

#[derive(Debug, Clone)]
pub struct CombatBattleState {
    pub(crate) typed_resource_result: Option<crate::game::resource::durable::RunResourceResult>,
    pub runtime_facts:
        Option<std::sync::Arc<crate::game::battle::runtime_facts::RuntimeFactSnapshot>>,
    pub primary_abnormality_id: Option<String>,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub abnormality_uuid: Uuid,
    pub outcome_id: crate::game::battle::types::BattleOutcomeId,
    pub verdict: crate::game::battle::types::BattleVerdict,
    pub winner: BattleWinner,
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub event_log: BattleEventLog,
    pub result_stats: BattleResultStatsDto,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub grant_node_bonus: bool,
    pub advance_encounter_progression: bool,
    pub participant_results: Vec<ParticipantBattleResult>,
    pub employee_conditions: crate::game::employee::EmployeeRoster,
    pub compendium_candidates: Vec<crate::game::battle::core::CompendiumCandidate>,
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

// In the opaque application family this set is freshly constructed/deserialized
// and only inserted into (world/shop). Removing an offer edits the Vec, not the
// set. Arbitrary external mutation of this public raw field is not certified.
crate::authoring::clone_shape::struct_shape_with_sets!(ShopSessionState {
    id, name, uuid, context, node_id, offer_count, refresh_policy, sell_policy,
    refresh_uses, page_index, offers
} sets { purchased_merchandise });
crate::authoring::clone_shape::struct_shape!(RewardSessionState {
    stage_uuid,
    mode,
    rewards,
    selected_reward_uuid,
    can_skip
});
crate::authoring::clone_shape::struct_shape!(SavePointSessionState { node_id });
crate::authoring::clone_shape::struct_shape!(MaintenanceSessionState { node_id });
crate::authoring::clone_shape::struct_shape!(HeadquartersContactSessionState {
    node_id,
    options,
    shop_pool_id
});
crate::authoring::clone_shape::struct_shape!(CombatBattleState {
    typed_resource_result,
    runtime_facts,
    primary_abnormality_id,
    encounter_id,
    node_type,
    mission_variant,
    abnormality_uuid,
    outcome_id,
    verdict,
    winner,
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    event_log,
    result_stats,
    reward_mode,
    rewards,
    grant_node_bonus,
    advance_encounter_progression,
    participant_results,
    employee_conditions,
    compendium_candidates
});
crate::authoring::clone_shape::enum_shape!(ActiveNodeContent {
    units: [],
    tuples: [
        Shop(value),
        Reward(value),
        Event(value),
        SavePoint(value),
        Maintenance(value),
        HeadquartersContact(value),
        CombatBattle(value)
    ],
    records: []
});
