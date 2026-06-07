use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::battle::{
    core::{
        sim::{BattleExecutionState, BattleLiveSignal},
        BattleCore,
    },
    ids::UnitInstanceId,
    tile_range::FacingDirection,
    timeline::TimelineEntry,
};
use crate::game::behavior::{
    AbnormalityAttemptDto, ActionKind, BattlePlaybackState, LiveBattleDeployedUnitDto,
    LiveBattleDeploymentDto, LiveBattleRedeployUnitDto, LiveBattleUnitDeployCostDto,
};
use crate::game::combat_preview::{CombatMissionVariant, CombatNodeType, CombatPreview};
use crate::game::employee::{EmployeeRoster, StarterEmployeeCandidate};
use crate::game::employee_trust::EmployeeTrustPolicy;
use crate::game::enums::RewardMode;
use crate::game::managers::action_scheduler::ActionScheduler;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::map::{MapProgression, NodeSession, RunMap, RunProgression};
use crate::game::resources::{
    ActionValidator, ActiveNodeContent, Enkephalin, GameState, Inventory, Qliphoth, RosterOrder,
};
use crate::game::reward::RewardOption;
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

pub const ABNORMALITY_MAX_ATTEMPTS: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbnormalityAttemptState {
    pub max_attempts: u8,
    pub attempts_started: u8,
}

impl AbnormalityAttemptState {
    pub const fn new(max_attempts: u8) -> Self {
        Self {
            max_attempts,
            attempts_started: 0,
        }
    }

    pub fn remaining_attempts(self) -> u8 {
        self.max_attempts.saturating_sub(self.attempts_started)
    }

    pub fn start_next_attempt(&mut self) -> bool {
        if self.remaining_attempts() == 0 {
            return false;
        }
        self.attempts_started = self.attempts_started.saturating_add(1);
        true
    }

    pub fn is_exhausted(self) -> bool {
        self.remaining_attempts() == 0
    }

    pub fn to_dto(self, node_id: crate::game::map::MapNodeId) -> AbnormalityAttemptDto {
        AbnormalityAttemptDto {
            node_id,
            max_attempts: self.max_attempts,
            attempts_started: self.attempts_started,
            remaining_attempts: self.remaining_attempts(),
        }
    }
}

pub struct GameCoreState {
    pub player: Option<PlayerInfo>,
    pub game_state: GameState,
    pub action_validator: ActionValidator,
    pub active_node_content: Option<ActiveNodeContent>,
    pub node_session: Option<NodeSession>,
    pub run: Option<RunState>,
    pub uuid_manager: UuidManager,
    pub enkephalin: Enkephalin,
    pub qliphoth: Qliphoth,
    pub inventory: Inventory,
    pub roster_order: RosterOrder,
    pub roster: EmployeeRoster,
    pub starter_candidates: Vec<StarterEmployeeCandidate>,
    pub skill_fragments: SkillFragmentInventory,
    pub skill_fragment_policy: SkillFragmentPolicy,
    pub research_delivery_policy: ResearchDeliveryPolicy,
    pub employee_trust_policy: EmployeeTrustPolicy,
    pub active_battle: Option<ActiveBattleSession>,
}

pub struct ActiveBattleSession {
    pub battle_uuid: Uuid,
    pub abnormality_id: String,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub combat_preview: CombatPreview,
    pub battle: BattleCore,
    pub execution: BattleExecutionState,
    pub last_pushed_timeline_seq: Option<u64>,
    pub live_deployment: Option<LiveBattleDeploymentState>,
    pub playback: BattlePlaybackState,
    pub playback_delta_remainder: u64,
}

pub struct LiveBattleDeploymentState {
    pub current_cost: u32,
    pub max_cost: u32,
    pub cost_per_second: u32,
    pub last_cost_update_ms: u64,
    pub base_deploy_cost: u32,
    pub redeploy_cooldown_ms: u64,
    pub redeploy_cost_multiplier_pct: u32,
    pub next_instance_salt: u32,
    pub deployed_units: HashMap<Uuid, LiveBattleDeployedUnitState>,
    pub redeploy_locks: HashMap<Uuid, LiveBattleRedeployState>,
}

#[derive(Debug, Clone, Copy)]
pub struct LiveBattleDeploymentPolicy {
    pub initial_cost: u32,
    pub max_cost: u32,
    pub cost_per_second: u32,
    pub base_deploy_cost: u32,
    pub redeploy_cooldown_ms: u64,
    pub redeploy_cost_multiplier_pct: u32,
    pub first_instance_salt: u32,
}

pub struct LiveBattleRedeployState {
    pub ready_at_ms: u64,
    pub deploy_cost: u32,
}

pub struct LiveBattleDeployedUnitState {
    pub unit_instance_id: UnitInstanceId,
    pub facing: FacingDirection,
}

impl LiveBattleDeploymentState {
    pub fn new(policy: LiveBattleDeploymentPolicy) -> Self {
        Self {
            current_cost: policy.initial_cost.min(policy.max_cost),
            max_cost: policy.max_cost,
            cost_per_second: policy.cost_per_second,
            last_cost_update_ms: 0,
            base_deploy_cost: policy.base_deploy_cost,
            redeploy_cooldown_ms: policy.redeploy_cooldown_ms,
            redeploy_cost_multiplier_pct: policy.redeploy_cost_multiplier_pct,
            next_instance_salt: policy.first_instance_salt,
            deployed_units: HashMap::new(),
            redeploy_locks: HashMap::new(),
        }
    }
}

impl ActiveBattleSession {
    pub fn playback_state(&self) -> BattlePlaybackState {
        self.playback
    }

    pub fn simulation_delta_for_tick(&mut self, raw_delta_ms: u64) -> Option<u64> {
        if self.playback.paused {
            return None;
        }
        let (numerator, denominator) = self.playback.speed.ratio();
        let scaled = raw_delta_ms
            .saturating_mul(numerator)
            .saturating_add(self.playback_delta_remainder);
        let delta_ms = scaled / denominator;
        self.playback_delta_remainder = scaled % denominator;
        if delta_ms == 0 {
            return None;
        }
        Some(delta_ms)
    }

    pub fn refresh_live_deployment_cost(&mut self) {
        let Some(deployment) = self.live_deployment.as_mut() else {
            return;
        };
        let now_ms = self.execution.last_event_time_ms();
        if now_ms <= deployment.last_cost_update_ms {
            return;
        }
        if deployment.cost_per_second == 0 {
            deployment.last_cost_update_ms = now_ms;
            return;
        }
        let elapsed_ms = now_ms - deployment.last_cost_update_ms;
        let gained = (elapsed_ms / 1_000).saturating_mul(deployment.cost_per_second as u64) as u32;
        if gained > 0 {
            deployment.current_cost = deployment
                .max_cost
                .min(deployment.current_cost.saturating_add(gained));
            deployment.last_cost_update_ms += (gained / deployment.cost_per_second) as u64 * 1_000;
        }
    }

    pub fn apply_live_signals_to_deployment(&mut self) {
        let signals = self.battle.drain_live_signals();
        if signals.is_empty() {
            return;
        }
        let Some(deployment) = self.live_deployment.as_mut() else {
            return;
        };

        for signal in signals {
            match signal {
                BattleLiveSignal::StabilizationDelta { amount, .. } if amount > 0 => {
                    deployment.current_cost = deployment
                        .max_cost
                        .min(deployment.current_cost.saturating_add(amount as u32));
                }
                BattleLiveSignal::StabilizationDelta { amount, .. } if amount < 0 => {
                    deployment.current_cost = deployment
                        .current_cost
                        .saturating_sub(amount.unsigned_abs());
                }
                BattleLiveSignal::StabilizationDelta { .. } => {}
            }
        }
    }

    pub fn live_deployment_dto(&self, roster: &EmployeeRoster) -> Option<LiveBattleDeploymentDto> {
        let deployment = self.live_deployment.as_ref()?;
        let mut deployed_units = deployment
            .deployed_units
            .iter()
            .map(|(employee_uuid, deployed)| LiveBattleDeployedUnitDto {
                employee_uuid: *employee_uuid,
                unit_instance_id: deployed.unit_instance_id,
                facing: deployed.facing,
                skill_readiness: self.battle.live_skill_readiness(
                    self.execution.last_event_time_ms(),
                    deployed.unit_instance_id,
                ),
            })
            .collect::<Vec<_>>();
        deployed_units.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        let mut redeploying_units = deployment
            .redeploy_locks
            .iter()
            .map(|(employee_uuid, state)| LiveBattleRedeployUnitDto {
                employee_uuid: *employee_uuid,
                ready_at_ms: state.ready_at_ms,
                deploy_cost: state.deploy_cost,
            })
            .collect::<Vec<_>>();
        redeploying_units.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        let mut unit_deploy_costs = roster
            .iter()
            .filter(|employee| employee.is_available_for_combat())
            .filter(|employee| !deployment.deployed_units.contains_key(&employee.uuid))
            .map(|employee| {
                let base_deploy_cost = deployment
                    .redeploy_locks
                    .get(&employee.uuid)
                    .map(|lock| lock.deploy_cost)
                    .unwrap_or(deployment.base_deploy_cost);
                LiveBattleUnitDeployCostDto {
                    employee_uuid: employee.uuid,
                    base_deploy_cost,
                    effective_deploy_cost: employee
                        .deployment_cost_after_consumable(base_deploy_cost),
                }
            })
            .collect::<Vec<_>>();
        unit_deploy_costs.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        Some(LiveBattleDeploymentDto {
            battle_time_ms: self.execution.last_event_time_ms(),
            current_cost: deployment.current_cost,
            max_cost: deployment.max_cost,
            base_deploy_cost: deployment.base_deploy_cost,
            cost_per_second: deployment.cost_per_second,
            unit_deploy_costs,
            deployed_units,
            redeploying_units,
        })
    }

    pub fn drain_event_log_delta(&mut self) -> Vec<TimelineEntry> {
        let delta = self
            .last_pushed_timeline_seq
            .map(|last_seen_seq| self.battle.event_log_entries_after_seq(last_seen_seq))
            .unwrap_or_else(|| self.battle.timeline.entries.clone());
        if let Some(last) = delta.last() {
            self.last_pushed_timeline_seq = Some(last.seq);
        }
        delta
    }

    pub fn event_log_delta_after(&self, last_seen_seq: Option<u64>) -> Vec<TimelineEntry> {
        last_seen_seq
            .map(|seq| self.battle.event_log_entries_after_seq(seq))
            .unwrap_or_else(|| self.battle.timeline.entries.clone())
    }

    pub fn last_event_log_seq(&self) -> u64 {
        self.battle
            .timeline
            .entries
            .last()
            .map(|entry| entry.seq)
            .unwrap_or(0)
    }
}

pub struct RunState {
    pub map: RunMap,
    pub map_progression: MapProgression,
    pub run_progression: RunProgression,
    pub combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    pub abnormality_attempts: HashMap<crate::game::map::MapNodeId, AbnormalityAttemptState>,
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
            combat_previews: HashMap::new(),
            abnormality_attempts: HashMap::new(),
        }
    }

    pub fn abnormality_attempt_state(
        &self,
        node_id: crate::game::map::MapNodeId,
    ) -> AbnormalityAttemptState {
        self.abnormality_attempts
            .get(&node_id)
            .copied()
            .unwrap_or_else(|| AbnormalityAttemptState::new(ABNORMALITY_MAX_ATTEMPTS))
    }

    pub fn abnormality_attempt_dto(
        &self,
        node_id: crate::game::map::MapNodeId,
    ) -> AbnormalityAttemptDto {
        self.abnormality_attempt_state(node_id).to_dto(node_id)
    }

    pub fn start_abnormality_attempt(
        &mut self,
        node_id: crate::game::map::MapNodeId,
    ) -> Option<AbnormalityAttemptState> {
        let state = self
            .abnormality_attempts
            .entry(node_id)
            .or_insert_with(|| AbnormalityAttemptState::new(ABNORMALITY_MAX_ATTEMPTS));
        state.start_next_attempt().then_some(*state)
    }

    pub fn clear_abnormality_attempt(&mut self, node_id: crate::game::map::MapNodeId) {
        self.abnormality_attempts.remove(&node_id);
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
            active_node_content: None,
            node_session: None,
            run: None,
            uuid_manager: UuidManager::new(run_seed),
            enkephalin: Enkephalin::new(0),
            qliphoth: Qliphoth::new(),
            inventory: Inventory::new(),
            roster_order: RosterOrder::new(super::ROSTER_ORDER_SLOTS),
            roster: EmployeeRoster::new(),
            starter_candidates: Vec::new(),
            skill_fragments: SkillFragmentInventory::new(),
            skill_fragment_policy: SkillFragmentPolicy::default_run_policy(),
            research_delivery_policy: ResearchDeliveryPolicy::default(),
            employee_trust_policy: EmployeeTrustPolicy::narrative_only(),
            active_battle: None,
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
