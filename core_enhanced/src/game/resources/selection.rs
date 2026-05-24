use uuid::Uuid;

use crate::game::data::shop_data::{ShopMetadata, ShopType};
use crate::game::{
    battle::{
        timeline::Timeline,
        types::{BattleWinner, ParticipantBattleResult},
    },
    behavior::GameError,
    combat_preview::CombatNodeType,
    employee::StarterEmployeeCandidate,
    enums::{RewardMode, ShopEventOption},
    map::{
        HeadquartersContactOption, MapNodeId, MedicalTreatmentKind, SupportNodeMode,
        SupportNodeType,
    },
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
    pub target_candidates: Vec<Uuid>,
    pub selected_employee_uuid: Option<Uuid>,
    pub selected_medical_treatment: Option<MedicalTreatmentKind>,
}

impl SupportSessionState {
    pub fn known(node_id: MapNodeId, support_type: SupportNodeType) -> Self {
        Self {
            node_id,
            support_mode: SupportNodeMode::Known,
            support_type: Some(support_type),
            choices: Vec::new(),
            selected_support_type: None,
            target_candidates: Vec::new(),
            selected_employee_uuid: None,
            selected_medical_treatment: None,
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
            target_candidates: Vec::new(),
            selected_employee_uuid: None,
            selected_medical_treatment: None,
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
        self.selected_employee_uuid = None;
        self.selected_medical_treatment = None;
        Ok(())
    }

    pub fn set_target_candidates(&mut self, mut candidates: Vec<Uuid>) {
        candidates.sort();
        candidates.dedup();
        self.target_candidates = candidates;
        if self
            .selected_employee_uuid
            .is_some_and(|employee_uuid| !self.target_candidates.contains(&employee_uuid))
        {
            self.selected_employee_uuid = None;
        }
    }

    pub fn select_target(&mut self, employee_uuid: Uuid) -> Result<(), GameError> {
        if self.target_candidates.is_empty() || !self.target_candidates.contains(&employee_uuid) {
            return Err(GameError::InvalidAction);
        }
        self.selected_employee_uuid = Some(employee_uuid);
        Ok(())
    }

    pub fn select_medical_treatment(
        &mut self,
        treatment: MedicalTreatmentKind,
    ) -> Result<(), GameError> {
        if self.resolved_support_type()? != SupportNodeType::Medical {
            return Err(GameError::InvalidAction);
        }
        self.selected_medical_treatment = Some(treatment);
        Ok(())
    }

    pub fn requires_employee_target(support_type: SupportNodeType) -> bool {
        matches!(support_type, SupportNodeType::Medical)
    }

    pub fn needs_target_selection(&self) -> Result<bool, GameError> {
        let support_type = self.resolved_support_type()?;
        Ok(Self::requires_employee_target(support_type)
            && !self.target_candidates.is_empty()
            && self.selected_employee_uuid.is_none())
    }

    pub fn needs_medical_treatment_selection(&self) -> Result<bool, GameError> {
        Ok(self.resolved_support_type()? == SupportNodeType::Medical
            && !self.target_candidates.is_empty()
            && self.selected_employee_uuid.is_some()
            && self.selected_medical_treatment.is_none())
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
    pub abnormality_uuid: Uuid,
    pub winner: BattleWinner,
    pub timeline: Timeline,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub participant_results: Vec<ParticipantBattleResult>,
}

#[derive(Debug, Clone)]
pub enum SelectedEventState {
    Shop(ShopSessionState),
    Reward(RewardSessionState),
    Support(SupportSessionState),
    HeadquartersContact(HeadquartersContactSessionState),
    CombatBattle(CombatBattleState),
}

#[derive(Debug)]
pub struct SelectedEvent {
    pub event: SelectedEventState,
}

impl SelectedEvent {
    pub fn new(event: SelectedEventState) -> Self {
        Self { event }
    }

    pub fn as_shop(&self) -> Result<&ShopSessionState, GameError> {
        match &self.event {
            SelectedEventState::Shop(shop) => Ok(shop),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_shop_mut(&mut self) -> Result<&mut ShopSessionState, GameError> {
        match &mut self.event {
            SelectedEventState::Shop(shop) => Ok(shop),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_reward(&self) -> Result<&RewardSessionState, GameError> {
        match &self.event {
            SelectedEventState::Reward(reward) => Ok(reward),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_reward_mut(&mut self) -> Result<&mut RewardSessionState, GameError> {
        match &mut self.event {
            SelectedEventState::Reward(reward) => Ok(reward),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_support(&self) -> Result<&SupportSessionState, GameError> {
        match &self.event {
            SelectedEventState::Support(support) => Ok(support),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_support_mut(&mut self) -> Result<&mut SupportSessionState, GameError> {
        match &mut self.event {
            SelectedEventState::Support(support) => Ok(support),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_combat_battle(&self) -> Result<&CombatBattleState, GameError> {
        match &self.event {
            SelectedEventState::CombatBattle(battle) => Ok(battle),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_headquarters_contact(&self) -> Result<&HeadquartersContactSessionState, GameError> {
        match &self.event {
            SelectedEventState::HeadquartersContact(headquarters) => Ok(headquarters),
            _ => Err(GameError::EventTypeMismatch),
        }
    }
}
