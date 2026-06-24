use uuid::Uuid;

use crate::game::data::shop_data::{ShopMetadata, ShopType};
use crate::game::{
    battle::{
        event_log::BattleEventLog,
        result_stats::BattleResultStatsDto,
        types::{BattleWinner, ParticipantBattleResult},
    },
    behavior::GameError,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    employee::StarterEmployeeCandidate,
    enums::{RewardMode, ShopEventOption},
    map::{HeadquartersContactOption, MapNodeId, SupportNodeMode, SupportNodeType},
    reward::RewardOption,
};

#[derive(Debug, Clone)]
pub struct ShopSessionState {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub shop_type: ShopType,
    pub can_reroll: bool,
    pub visible_items: Vec<Uuid>,
    pub hidden_items: Vec<Uuid>,
}

impl ShopSessionState {
    pub fn remove_visible_item(&mut self, uuid: Uuid) -> Result<(), GameError> {
        let pos = self
            .visible_items
            .iter()
            .position(|item| *item == uuid)
            .ok_or(GameError::ShopItemNotFound)?;
        self.visible_items.remove(pos);
        Ok(())
    }

    pub fn reroll_items(&mut self) {
        std::mem::swap(&mut self.hidden_items, &mut self.visible_items);
    }
}

impl From<&ShopMetadata> for ShopSessionState {
    fn from(value: &ShopMetadata) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            uuid: value.uuid,
            shop_type: value.shop_type,
            can_reroll: value.can_reroll,
            visible_items: value.visible_items.clone(),
            hidden_items: value.hidden_items.clone(),
        }
    }
}

impl From<ShopMetadata> for ShopSessionState {
    fn from(value: ShopMetadata) -> Self {
        Self::from(&value)
    }
}

impl From<&ShopEventOption> for ShopSessionState {
    fn from(value: &ShopEventOption) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            uuid: value.uuid,
            shop_type: value.shop_type,
            can_reroll: value.can_reroll,
            visible_items: value.visible_items.clone(),
            hidden_items: Vec::new(),
        }
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
pub struct SupportSessionState {
    pub node_id: MapNodeId,
    pub support_mode: SupportNodeMode,
    pub support_type: Option<SupportNodeType>,
    pub choices: Vec<SupportNodeType>,
    pub selected_support_type: Option<SupportNodeType>,
}

impl SupportSessionState {
    pub fn known(node_id: MapNodeId, support_type: SupportNodeType) -> Self {
        Self {
            node_id,
            support_mode: SupportNodeMode::Known,
            support_type: Some(support_type),
            choices: Vec::new(),
            selected_support_type: None,
        }
    }

    pub fn choice(
        node_id: MapNodeId,
        support_mode: SupportNodeMode,
        choices: Vec<SupportNodeType>,
    ) -> Self {
        Self {
            node_id,
            support_mode,
            support_type: None,
            choices,
            selected_support_type: None,
        }
    }

    pub fn select(&mut self, support_type: SupportNodeType) -> Result<(), GameError> {
        if !matches!(
            self.support_mode,
            SupportNodeMode::LimitedChoice | SupportNodeMode::FullChoice
        ) {
            return Err(GameError::InvalidAction);
        }
        if !self.choices.contains(&support_type) {
            return Err(GameError::InvalidAction);
        }
        self.selected_support_type = Some(support_type);
        Ok(())
    }

    pub fn resolved_support_type(&self) -> Result<SupportNodeType, GameError> {
        match self.support_mode {
            SupportNodeMode::Known => self.support_type.ok_or(GameError::InvalidAction),
            SupportNodeMode::LimitedChoice | SupportNodeMode::FullChoice => {
                self.selected_support_type.ok_or(GameError::InvalidAction)
            }
        }
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
    pub recruitment_candidates: Vec<StarterEmployeeCandidate>,
    pub shop_pool_id: Option<String>,
}

impl HeadquartersContactSessionState {
    pub fn new(
        node_id: MapNodeId,
        recruitment_candidates: Vec<StarterEmployeeCandidate>,
        shop_pool_id: Option<String>,
    ) -> Self {
        Self {
            node_id,
            options: vec![
                HeadquartersContactOption::RecruitEmployee,
                HeadquartersContactOption::RequestEmergencySupplies,
                HeadquartersContactOption::OpenHeadquartersShop,
            ],
            recruitment_candidates,
            shop_pool_id,
        }
    }

    pub fn get_candidate(&self, candidate_id: &str) -> Option<&StarterEmployeeCandidate> {
        self.recruitment_candidates
            .iter()
            .find(|candidate| candidate.id == candidate_id)
    }
}

#[derive(Debug, Clone)]
pub struct CombatBattleState {
    pub abnormality_id: String,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub abnormality_uuid: Uuid,
    pub winner: BattleWinner,
    pub event_log: BattleEventLog,
    pub result_stats: BattleResultStatsDto,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub participant_results: Vec<ParticipantBattleResult>,
}

#[derive(Debug, Clone)]
pub enum ActiveNodeContent {
    Shop(ShopSessionState),
    Reward(RewardSessionState),
    Support(SupportSessionState),
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

    pub fn as_support(&self) -> Result<&SupportSessionState, GameError> {
        match self {
            ActiveNodeContent::Support(support) => Ok(support),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_support_mut(&mut self) -> Result<&mut SupportSessionState, GameError> {
        match self {
            ActiveNodeContent::Support(support) => Ok(support),
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
