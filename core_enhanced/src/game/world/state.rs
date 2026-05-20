use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::behavior::ActionKind;
use crate::game::combat_preview::{CombatDeployment, CombatPreview, DEFAULT_RECON_CHARGES};
use crate::game::employee::{EmployeeRoster, StarterEmployeeCandidate};
use crate::game::employee_trust::EmployeeTrustPolicy;
use crate::game::managers::action_scheduler::ActionScheduler;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::map::{MapProgression, NodeSession, RunMap, RunProgression};
use crate::game::resources::{
    ActionValidator, Bench, Enkephalin, Field, GameState, Inventory, Qliphoth, SelectedEvent,
};
use crate::game::skill_fragment::{
    ResearchDeliveryPolicy, SkillFragmentInventory, SkillFragmentPolicy,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub name: String,
}

impl PlayerInfo {
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}

pub struct GameCoreState {
    pub player: Option<PlayerInfo>,
    pub game_state: GameState,
    pub action_validator: ActionValidator,
    pub selected_event: Option<SelectedEvent>,
    pub node_session: Option<NodeSession>,
    pub run: Option<RunState>,
    pub uuid_manager: UuidManager,
    pub enkephalin: Enkephalin,
    pub qliphoth: Qliphoth,
    pub inventory: Inventory,
    pub field: Field,
    pub bench: Bench,
    pub roster: EmployeeRoster,
    pub starter_candidates: Vec<StarterEmployeeCandidate>,
    pub skill_fragments: SkillFragmentInventory,
    pub skill_fragment_policy: SkillFragmentPolicy,
    pub research_delivery_policy: ResearchDeliveryPolicy,
    pub employee_trust_policy: EmployeeTrustPolicy,
}

pub struct RunState {
    pub map: RunMap,
    pub map_progression: MapProgression,
    pub run_progression: RunProgression,
    pub recon_charge: u32,
    pub combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    pub combat_deployments: HashMap<crate::game::map::MapNodeId, CombatDeployment>,
    pub recon_scanned_nodes: HashMap<crate::game::map::MapNodeId, u32>,
}

impl RunState {
    pub fn new(
        map: RunMap,
        map_progression: MapProgression,
        run_progression: RunProgression,
    ) -> Self {
        Self {
            map,
            map_progression,
            run_progression,
            recon_charge: DEFAULT_RECON_CHARGES,
            combat_previews: HashMap::new(),
            combat_deployments: HashMap::new(),
            recon_scanned_nodes: HashMap::new(),
        }
    }
}

impl GameCoreState {
    pub fn new(run_seed: u64) -> Self {
        let game_state = GameState::NotStarted;
        let initial_actions = ActionScheduler::get_allowed_actions(&game_state);
        let mut action_validator = ActionValidator::new();
        action_validator.set_allowed_actions(initial_actions);
        Self {
            player: None,
            game_state,
            action_validator,
            selected_event: None,
            node_session: None,
            run: None,
            uuid_manager: UuidManager::new(run_seed),
            enkephalin: Enkephalin::new(0),
            qliphoth: Qliphoth::new(),
            inventory: Inventory::new(),
            field: Field::new(super::METAGAME_FIELD_WIDTH, super::METAGAME_FIELD_HEIGHT),
            bench: Bench::new(super::METAGAME_BENCH_SLOTS),
            roster: EmployeeRoster::new(),
            starter_candidates: Vec::new(),
            skill_fragments: SkillFragmentInventory::new(),
            skill_fragment_policy: SkillFragmentPolicy::default_run_policy(),
            research_delivery_policy: ResearchDeliveryPolicy::default(),
            employee_trust_policy: EmployeeTrustPolicy::narrative_only(),
        }
    }

    pub fn transition_to(&mut self, new_state: GameState, allowed_actions: Vec<ActionKind>) {
        self.game_state = new_state;
        self.action_validator.set_allowed_actions(allowed_actions);
    }

    pub fn initialize_player(&mut self, player_id: Uuid) -> bool {
        if self
            .player
            .as_ref()
            .is_some_and(|player| player.id == player_id)
        {
            return false;
        }

        self.player = Some(PlayerInfo::new(player_id, "Hero"));
        true
    }
}
