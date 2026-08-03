use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::game::abnormality_research::{
    RunAbnormalityEncounterHistory, RunAbnormalityResearchState,
};
use crate::game::battle::{
    core::{
        sim::{BattleDeployCurrentHpPolicy, BattleExecutionState, BattleLiveSignal},
        BattleCore,
    },
    event_log::BattleEventLogEntry,
    ids::UnitInstanceId,
    tile_range::FacingDirection,
    types::BattleUnitRole,
};
use crate::game::behavior::{
    AbnormalityAttemptDto, ActionDisabledReasonCode, BattlePayloadEligibilityDto,
    BattlePlaybackState, GameError, LiveBattleDeployedUnitDto, LiveBattleDeploymentDto,
    LiveBattleEventDeltaDto, LiveBattleHudBarMode, LiveBattlePlaybackEligibilityDto,
    LiveBattlePresentationEventDto, LiveBattleRedeployUnitDto, LiveBattleSetupBattlefieldDto,
    LiveBattleSetupCatalogRefsDto, LiveBattleSetupSnapshotDto, LiveBattleSetupSnapshotMessageType,
    LiveBattleSetupTacticalPointDto, LiveBattleSetupTacticalPointType,
    LiveBattleStateCheckpointDto, LiveBattleUnitCheckpointDto, LiveBattleUnitDeployCostDto,
    LiveBattleUnitHudDto, LiveBattleUpdateDto, LiveBattleUpdateMessageType,
    RunCheckpointSnapshotDto,
};
use crate::game::boss_omen::BossOmenRunState;
use crate::game::combat_preview::{CombatMissionVariant, CombatNodeType, CombatPreview};
use crate::game::data::run_policy_data::LiveBattleDeploymentPolicy;
use crate::game::employee::{EmployeeDefinition, EmployeeRoster};
use crate::game::encounter_compendium::EncounterCompendiumProfileState;
use crate::game::enums::RewardMode;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::map::{
    FacilityRunState, GameMode, MapNodeId, MapProgression, NodeSession, RunMap, RunProgression,
};
use crate::game::monolith::MonolithRunState;
use crate::game::range_preview::range_previews_for_player_runtime_unit;
use crate::game::recruitment::RecruitmentState;
use crate::game::resources::{
    ActiveNodeContent, Enkephalin, EventSessionState, GameState, Inventory, ShopSessionState,
};
use crate::game::reward::RewardOption;
use crate::game::skill_fragment::{SkillFragmentInventory, SkillFragmentPolicy};
use crate::game::visual_novel::{
    VisualNovelRuntimeState, VisualNovelTriggerConsumedKey, VisualNovelTriggerRuntimeState,
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
    pub active_node_content: Option<ActiveNodeContent>,
    pub node_session: Option<NodeSession>,
    pub run: Option<RunState>,
    pub visual_novel: VisualNovelRuntimeState,
    pub visual_novel_triggers: VisualNovelTriggerRuntimeState,
    pub uuid_manager: UuidManager,
    pub enkephalin: Enkephalin,
    pub inventory: Inventory,
    pub roster: EmployeeRoster,
    pub starter_candidates: Vec<EmployeeDefinition>,
    pub pending_game_mode: Option<GameMode>,
    pub skill_fragments: SkillFragmentInventory,
    pub skill_fragment_policy: SkillFragmentPolicy,
    pub encounter_compendium: EncounterCompendiumProfileState,
    pub active_battle: Option<ActiveBattleSession>,
    pub run_checkpoint: RunCheckpointState,
    pub run_power_authority: RunPowerAuthorityState,
}

impl GameCoreState {
    pub(super) fn clone_non_battle_transaction_state(&self) -> Option<Self> {
        if self.active_battle.is_some() {
            return None;
        }
        Some(Self {
            player: self.player.clone(),
            game_state: self.game_state.clone(),
            active_node_content: self.active_node_content.clone(),
            node_session: self.node_session.clone(),
            run: self.run.clone(),
            visual_novel: self.visual_novel.clone(),
            visual_novel_triggers: self.visual_novel_triggers.clone(),
            uuid_manager: self.uuid_manager.clone(),
            enkephalin: self.enkephalin.clone(),
            inventory: self.inventory.clone(),
            roster: self.roster.clone(),
            starter_candidates: self.starter_candidates.clone(),
            pending_game_mode: self.pending_game_mode,
            skill_fragments: self.skill_fragments.clone(),
            skill_fragment_policy: self.skill_fragment_policy.clone(),
            // Local-install profile knowledge is not part of an active-run
            // transaction rollback. The restore path preserves the live profile.
            encounter_compendium: EncounterCompendiumProfileState::default(),
            active_battle: None,
            run_checkpoint: self.run_checkpoint.clone(),
            run_power_authority: self.run_power_authority,
        })
    }
}

pub const RUN_CHECKPOINT_MAX_LOADS: u8 = 3;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunPowerAuthorityState {
    pub power_model_version: u32,
    pub run_peak_certified_power: u64,
    pub current_floor_power_anchor: u64,
}

impl RunPowerAuthorityState {
    pub fn initialize_run(&mut self, power_model_version: u32, certified_power: u64) {
        self.power_model_version = power_model_version;
        self.run_peak_certified_power = certified_power;
        self.current_floor_power_anchor = 0;
    }

    pub fn record_combat_entry(&mut self, power_model_version: u32, certified_power: u64) {
        if self.power_model_version == 0 {
            self.power_model_version = power_model_version;
        }
        debug_assert_eq!(self.power_model_version, power_model_version);
        self.run_peak_certified_power = self.run_peak_certified_power.max(certified_power);
    }

    pub fn manifest_floor(
        &mut self,
        policy: crate::game::data::run_policy_data::ResolvedFloorPowerAnchorPolicy,
    ) -> u64 {
        let previous_rise = self
            .current_floor_power_anchor
            .checked_add(policy.minimum_rise)
            .filter(|_| self.current_floor_power_anchor > 0)
            .unwrap_or(0);
        self.current_floor_power_anchor = policy
            .authored_minimum_power
            .max(previous_rise)
            .max(self.run_peak_certified_power);
        self.current_floor_power_anchor
    }

    pub fn merge_run_peak_monotonic(&mut self, other: Self) {
        if self.power_model_version == other.power_model_version {
            self.run_peak_certified_power = self
                .run_peak_certified_power
                .max(other.run_peak_certified_power);
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RunCheckpointState {
    pub payload: Option<RunCheckpointPayload>,
    pub loads_used: u8,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunCheckpointPayload {
    pub power_model_version: u32,
    pub current_floor_power_anchor: u64,
    pub map: RunMap,
    pub map_progression: MapProgression,
    pub run_progression: RunProgression,
    pub facility_run_state: FacilityRunState,
    pub abnormality_research: RunAbnormalityResearchState,
    pub abnormality_encounter_history: RunAbnormalityEncounterHistory,
    pub monolith: MonolithRunState,
    pub boss_omen: BossOmenRunState,
    pub event_sessions: HashMap<crate::game::map::MapNodeId, EventSessionState>,
    pub shop_sessions: HashMap<crate::game::map::MapNodeId, ShopSessionState>,
    pub combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    pub abnormality_attempts: HashMap<crate::game::map::MapNodeId, AbnormalityAttemptState>,
    pub recruitment: RecruitmentState,
    pub visual_novel_trigger_checkpoint_consumed:
        std::collections::HashSet<VisualNovelTriggerConsumedKey>,
    pub enkephalin: Enkephalin,
    pub inventory: Inventory,
    pub roster: EmployeeRoster,
    pub skill_fragments: SkillFragmentInventory,
    pub uuid_manager: UuidManager,
}

impl RunCheckpointState {
    #[cfg(test)]
    pub fn can_load(&self) -> bool {
        self.payload.is_some() && self.loads_used < RUN_CHECKPOINT_MAX_LOADS
    }

    pub fn to_dto(&self, loadable: bool) -> RunCheckpointSnapshotDto {
        let remaining_loads = RUN_CHECKPOINT_MAX_LOADS.saturating_sub(self.loads_used);
        let location = self.payload.as_ref().and_then(|payload| {
            let node_id = payload.map_progression.current_node_id?;
            let node = payload.map.node(node_id)?;
            Some(crate::game::behavior::RunCheckpointLocationDto {
                floor_index: payload.run_progression.floor_index(),
                node_id,
                slot_id: node.slot_id.clone(),
                kind_id: node.kind_id.clone(),
                room_display_name: node.kind_id.as_str().to_string(),
            })
        });
        RunCheckpointSnapshotDto {
            exists: self.payload.is_some(),
            loadable,
            loads_used: self.loads_used,
            max_loads: RUN_CHECKPOINT_MAX_LOADS,
            remaining_loads,
            location,
        }
    }
}

pub struct ActiveBattleSession {
    pub node_id: MapNodeId,
    pub battle_uuid: Uuid,
    pub battle_state_revision: u64,
    pub primary_abnormality_id: Option<String>,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub combat_preview: CombatPreview,
    pub battle: BattleCore,
    pub execution: BattleExecutionState,
    pub entry_snapshot: Option<BattleEntrySnapshot>,
    pub employee_attempt: BattleEmployeeAttemptState,
    pub live_deployment: Option<LiveBattleDeploymentState>,
    pub playback: BattlePlaybackState,
    pub playback_delta_remainder: u64,
}

#[derive(Clone)]
pub struct BattleEntrySnapshot {
    game_state: GameState,
    active_node_content: Option<ActiveNodeContent>,
    node_session: Option<NodeSession>,
    run: Option<RunState>,
    visual_novel: VisualNovelRuntimeState,
    visual_novel_triggers: VisualNovelTriggerRuntimeState,
    uuid_manager: UuidManager,
    enkephalin: Enkephalin,
    inventory: Inventory,
    roster: EmployeeRoster,
    starter_candidates: Vec<EmployeeDefinition>,
    pending_game_mode: Option<GameMode>,
    skill_fragments: SkillFragmentInventory,
    run_checkpoint: RunCheckpointState,
}

impl BattleEntrySnapshot {
    pub fn capture(state: &GameCoreState) -> Self {
        Self {
            game_state: state.game_state.clone(),
            active_node_content: state.active_node_content.clone(),
            node_session: state.node_session.clone(),
            run: state.run.clone(),
            visual_novel: state.visual_novel.clone(),
            visual_novel_triggers: state.visual_novel_triggers.clone(),
            uuid_manager: state.uuid_manager.clone(),
            enkephalin: state.enkephalin.clone(),
            inventory: state.inventory.clone(),
            roster: state.roster.clone(),
            starter_candidates: state.starter_candidates.clone(),
            pending_game_mode: state.pending_game_mode,
            skill_fragments: state.skill_fragments.clone(),
            run_checkpoint: state.run_checkpoint.clone(),
        }
    }

    pub fn restore(self, state: &mut GameCoreState) {
        state.game_state = self.game_state;
        state.active_node_content = self.active_node_content;
        state.node_session = self.node_session;
        state.run = self.run;
        state.visual_novel = self.visual_novel;
        state.visual_novel_triggers = self.visual_novel_triggers;
        state.uuid_manager = self.uuid_manager;
        state.enkephalin = self.enkephalin;
        state.inventory = self.inventory;
        state.roster = self.roster;
        state.starter_candidates = self.starter_candidates;
        state.pending_game_mode = self.pending_game_mode;
        state.skill_fragments = self.skill_fragments;
        state.run_checkpoint = self.run_checkpoint;
        state.active_battle = None;
    }

    pub fn as_event_combat_reentry(
        mut self,
        node_id: crate::game::map::MapNodeId,
        kind_id: crate::game::map::MapNodeKindId,
        category: crate::game::map::MapNodeCategory,
        session: NodeSession,
    ) -> Self {
        self.game_state = GameState::NodeConfirm {
            node_id,
            kind_id,
            category,
        };
        self.active_node_content = None;
        self.node_session = Some(session);
        self
    }
}

pub struct BattleEmployeeAttemptState {
    pub roster: EmployeeRoster,
    pub processed_incapacitations: HashSet<UnitInstanceId>,
}

impl BattleEmployeeAttemptState {
    pub fn new(entry_roster: &EmployeeRoster) -> Self {
        Self {
            roster: entry_roster.clone(),
            processed_incapacitations: HashSet::new(),
        }
    }
}

pub struct LiveBattleDeploymentState {
    pub current_cost: u32,
    pub max_cost: u32,
    pub cost_per_second: u32,
    pub last_cost_update_ms: u64,
    pub base_deploy_cost: u32,
    pub withdraw_redeploy_cooldown_ms: u64,
    pub defeat_redeploy_cooldown_ms: u64,
    pub redeploy_cost_multiplier_pct: u32,
    pub next_instance_salt: u32,
    pub deployed_units: HashMap<Uuid, LiveBattleDeployedUnitState>,
    pub redeploy_locks: HashMap<Uuid, LiveBattleRedeployState>,
}

pub struct LiveBattleRedeployState {
    pub ready_at_ms: u64,
    pub deploy_cost: u32,
    pub current_hp_policy: Option<BattleDeployCurrentHpPolicy>,
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
            withdraw_redeploy_cooldown_ms: policy.withdraw_redeploy_cooldown_ms,
            defeat_redeploy_cooldown_ms: policy.defeat_redeploy_cooldown_ms,
            redeploy_cost_multiplier_pct: policy.redeploy_cost_multiplier_pct,
            next_instance_salt: policy.first_instance_salt,
            deployed_units: HashMap::new(),
            redeploy_locks: HashMap::new(),
        }
    }
}

impl ActiveBattleSession {
    pub fn employee_roster(&self) -> &EmployeeRoster {
        &self.employee_attempt.roster
    }

    pub fn unprocessed_deployed_employee_incapacitations(&self) -> Vec<(Uuid, UnitInstanceId)> {
        let Some(deployment) = self.live_deployment.as_ref() else {
            return Vec::new();
        };
        let mut incapacitations = deployment
            .deployed_units
            .iter()
            .filter_map(|(employee_uuid, deployed)| {
                let unit = self.battle.units.get(&deployed.unit_instance_id)?;
                (unit.lifecycle.is_dead()
                    && unit.source_owned_uuid == *employee_uuid
                    && matches!(
                        unit.source_identity,
                        crate::game::battle::types::BattleUnitSourceIdentity::Employee { .. }
                    )
                    && !self
                        .employee_attempt
                        .processed_incapacitations
                        .contains(&deployed.unit_instance_id))
                .then_some((*employee_uuid, deployed.unit_instance_id))
            })
            .collect::<Vec<_>>();
        incapacitations.sort_by(|left, right| left.1.cmp(&right.1));
        incapacitations
    }

    pub fn playback_state(&self) -> BattlePlaybackState {
        self.playback
    }

    pub fn deployment_candidate_eligibility(
        &self,
        employee_uuid: Uuid,
        effective_deploy_cost: u32,
    ) -> BattlePayloadEligibilityDto {
        let Some(deployment) = self.live_deployment.as_ref() else {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::UnitUnavailable,
            );
        };
        if deployment.deployed_units.contains_key(&employee_uuid) {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::UnitUnavailable,
            );
        }
        if deployment
            .redeploy_locks
            .get(&employee_uuid)
            .is_some_and(|lock| self.execution.last_event_time_ms() < lock.ready_at_ms)
        {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::RedeployCooldown,
            );
        }
        if deployment.current_cost < effective_deploy_cost {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::InsufficientResources,
            );
        }
        BattlePayloadEligibilityDto::enabled()
    }

    fn playback_eligibility_dto(&self) -> LiveBattlePlaybackEligibilityDto {
        LiveBattlePlaybackEligibilityDto::for_state(self.playback)
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

    pub fn reconcile_live_deployment_with_battle_state(&mut self) {
        let Some(deployment) = self.live_deployment.as_mut() else {
            return;
        };
        let now_ms = self.execution.last_event_time_ms();
        let redeploy_cost = deployment
            .base_deploy_cost
            .saturating_mul(deployment.redeploy_cost_multiplier_pct)
            / 100;
        let stale_deployments = deployment
            .deployed_units
            .iter()
            .filter_map(|(employee_uuid, deployed)| {
                let current_hp_policy = match self.battle.units.get(&deployed.unit_instance_id) {
                    Some(unit) if unit.is_active() => return None,
                    Some(unit)
                        if matches!(
                            unit.lifecycle,
                            crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
                        ) =>
                    {
                        Some(BattleDeployCurrentHpPolicy::withdraw_redeploy(
                            unit.stats.current_health,
                            unit.stats.max_health,
                        ))
                    }
                    Some(_) | None => None,
                };
                Some((*employee_uuid, current_hp_policy))
            })
            .collect::<Vec<_>>();

        for (employee_uuid, current_hp_policy) in stale_deployments {
            deployment.deployed_units.remove(&employee_uuid);
            deployment
                .redeploy_locks
                .entry(employee_uuid)
                .or_insert_with(|| LiveBattleRedeployState {
                    ready_at_ms: now_ms.saturating_add(deployment.defeat_redeploy_cooldown_ms),
                    deploy_cost: redeploy_cost,
                    current_hp_policy,
                });
        }
    }

    pub fn battle_setup_snapshot_dto(&self) -> LiveBattleSetupSnapshotDto {
        let scenario = self.battle.scenario();
        let mut tactical_points = scenario
            .tactical_plan
            .points
            .iter()
            .map(|point| LiveBattleSetupTacticalPointDto {
                id: point.id.0.clone(),
                point_type: LiveBattleSetupTacticalPointType::TacticalPoint,
                position: point.position,
            })
            .collect::<Vec<_>>();

        for route in &self.combat_preview.routes {
            let endpoint_id = format!("{}_endpoint", route.id);
            if tactical_points.iter().any(|point| point.id == endpoint_id) {
                continue;
            }
            tactical_points.push(LiveBattleSetupTacticalPointDto {
                id: endpoint_id,
                point_type: LiveBattleSetupTacticalPointType::RouteEndpoint,
                position: route.end,
            });
        }
        tactical_points.sort_by(|left, right| left.id.cmp(&right.id));

        let is_actual_abnormality_catalog_id = |id: &str| {
            !id.trim().is_empty()
                && self
                    .battle
                    .game_data
                    .abnormality_data
                    .get_by_id(id)
                    .is_some()
        };
        let mut abnormality_ids = self
            .primary_abnormality_id
            .iter()
            .filter(|id| is_actual_abnormality_catalog_id(id))
            .cloned()
            .collect::<Vec<_>>();
        abnormality_ids.extend(
            self.combat_preview
                .enemy_briefing
                .iter()
                .filter(|briefing| {
                    briefing.kind == crate::game::combat_preview::EnemyKind::Abnormality
                        && is_actual_abnormality_catalog_id(&briefing.abnormality_id)
                })
                .map(|briefing| briefing.abnormality_id.clone()),
        );
        abnormality_ids.extend(
            self.combat_preview
                .spawn_waves
                .iter()
                .flat_map(|wave| wave.enemy_entries.iter())
                .filter(|entry| {
                    entry.kind == crate::game::combat_preview::EnemyKind::Abnormality
                        && is_actual_abnormality_catalog_id(&entry.abnormality_id)
                })
                .map(|entry| entry.abnormality_id.clone()),
        );
        abnormality_ids.sort();
        abnormality_ids.dedup();

        LiveBattleSetupSnapshotDto {
            message_type: LiveBattleSetupSnapshotMessageType::BattleSetupSnapshot,
            setup_version: LiveBattleSetupSnapshotDto::SETUP_VERSION,
            battle_uuid: self.battle_uuid,
            encounter_id: self.encounter_id.clone(),
            node_type: self.node_type,
            mission_variant: self.mission_variant,
            survive_timer_ms: self.combat_preview.survive_timer_ms,
            battlefield: LiveBattleSetupBattlefieldDto {
                width: i32::from(scenario.battlefield.width),
                height: i32::from(scenario.battlefield.height),
                tiles: self.combat_preview.tiles.clone(),
                valid_tiles: scenario.battlefield.valid_tiles.clone(),
                blocked_tiles: scenario.battlefield.obstacles.clone(),
                static_obstacles: scenario.battlefield.obstacles.clone(),
            },
            routes: self.combat_preview.routes.clone(),
            deployment_zones: self.combat_preview.deployment_zones.clone(),
            spawn_zones: self.combat_preview.spawn_zones.clone(),
            tactical_points,
            static_objects: Vec::new(),
            initial_units: Vec::new(),
            catalog_refs: LiveBattleSetupCatalogRefsDto {
                battlefield_template_id: self.combat_preview.battlefield_template_id.clone(),
                abnormality_ids,
            },
        }
    }

    pub fn live_deployment_dto(&self, roster: &EmployeeRoster) -> Option<LiveBattleDeploymentDto> {
        let deployment = self.live_deployment.as_ref()?;
        let mut deployed_units = deployment
            .deployed_units
            .iter()
            .map(|(employee_uuid, deployed)| {
                let position = self
                    .battle
                    .unit_projected_tile(deployed.unit_instance_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "live deployment references unit without runtime position: {:?}",
                            deployed.unit_instance_id
                        )
                    });
                LiveBattleDeployedUnitDto {
                    employee_uuid: *employee_uuid,
                    unit_instance_id: deployed.unit_instance_id,
                    position,
                    facing: deployed.facing,
                    withdraw_eligibility: BattlePayloadEligibilityDto::enabled(),
                    skill_readiness: self
                        .battle
                        .live_skill_readiness(
                            self.execution.last_event_time_ms(),
                            deployed.unit_instance_id,
                        )
                        .unwrap_or_else(|| {
                            panic!(
                                "live deployment references unit without skill readiness: {:?}",
                                deployed.unit_instance_id
                            )
                        }),
                }
            })
            .collect::<Vec<_>>();
        deployed_units.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        let mut redeploying_units = deployment
            .redeploy_locks
            .iter()
            .filter(|(employee_uuid, _)| {
                roster
                    .get(employee_uuid)
                    .is_some_and(|employee| employee.is_combat_eligible())
            })
            .map(|(employee_uuid, state)| LiveBattleRedeployUnitDto {
                employee_uuid: *employee_uuid,
                ready_at_ms: state.ready_at_ms,
                deploy_cost: state.deploy_cost,
            })
            .collect::<Vec<_>>();
        redeploying_units.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        let mut unit_deploy_costs = roster
            .iter()
            .filter(|employee| employee.is_combat_eligible())
            .filter(|employee| !deployment.deployed_units.contains_key(&employee.uuid))
            .map(|employee| {
                let base_deploy_cost = deployment
                    .redeploy_locks
                    .get(&employee.uuid)
                    .map(|lock| lock.deploy_cost)
                    .unwrap_or(deployment.base_deploy_cost);
                let effective_deploy_cost =
                    employee.deployment_cost_after_consumable(base_deploy_cost);
                LiveBattleUnitDeployCostDto {
                    employee_uuid: employee.uuid,
                    base_deploy_cost,
                    effective_deploy_cost,
                    eligibility: self
                        .deployment_candidate_eligibility(employee.uuid, effective_deploy_cost),
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

    pub fn commit_battle_projection(&mut self) -> Result<u64, GameError> {
        self.battle_state_revision = self.next_battle_projection_revision()?;
        Ok(self.battle_state_revision)
    }

    pub fn ensure_battle_projection_commit_available(&self) -> Result<(), GameError> {
        self.next_battle_projection_revision().map(|_| ())
    }

    fn next_battle_projection_revision(&self) -> Result<u64, GameError> {
        self.battle_state_revision
            .checked_add(1)
            .ok_or(GameError::BattleStateRevisionOverflow)
    }

    pub fn latest_event_log_seq(&self) -> u64 {
        self.battle
            .event_log
            .entries
            .last()
            .map(|entry| entry.seq)
            .unwrap_or(0)
    }

    pub fn battle_update_dto_after(
        &self,
        after_seq: u64,
        roster: &EmployeeRoster,
    ) -> Result<LiveBattleUpdateDto, GameError> {
        let latest_seq = self.latest_event_log_seq();
        if after_seq > latest_seq {
            return Err(GameError::InvalidBattleResyncSeq {
                requested: after_seq,
                latest: latest_seq,
            });
        }
        let events = self.battle.event_log_entries_after_seq(after_seq);
        let to_seq = events.last().map(|entry| entry.seq).unwrap_or(after_seq);
        Ok(self.battle_update_dto_from_events(after_seq, to_seq, events, roster))
    }

    fn battle_update_dto_from_events(
        &self,
        after_seq: u64,
        to_seq: u64,
        events: Vec<BattleEventLogEntry>,
        roster: &EmployeeRoster,
    ) -> LiveBattleUpdateDto {
        LiveBattleUpdateDto {
            message_type: LiveBattleUpdateMessageType::BattleUpdate,
            battle_uuid: self.battle_uuid,
            battle_state_revision: self.battle_state_revision,
            action_capabilities: Vec::new(),
            server_battle_time_ms: self.execution.last_event_time_ms(),
            events_delta: LiveBattleEventDeltaDto {
                after_seq,
                to_seq,
                events: events
                    .into_iter()
                    .map(LiveBattlePresentationEventDto::from)
                    .collect(),
            },
            checkpoint: self.live_checkpoint_dto(to_seq, roster),
            visual_novel: None,
        }
    }

    fn live_checkpoint_dto(
        &self,
        at_seq: u64,
        roster: &EmployeeRoster,
    ) -> LiveBattleStateCheckpointDto {
        let mut units = self
            .battle
            .units
            .values()
            .filter(|unit| unit.is_active())
            .map(|unit| {
                let position = unit.body.projected_tile();
                LiveBattleUnitCheckpointDto {
                    unit_instance_id: unit.instance_id,
                    owner: unit.owner,
                    role: unit.role,
                    hud: live_unit_hud_dto(unit),
                    range_previews: (unit.owner == crate::game::enums::Side::Player).then(|| {
                        range_previews_for_player_runtime_unit(
                            unit,
                            self.battle.game_data.as_ref(),
                            &self.battle.battlefield,
                            position,
                        )
                    }),
                    mobility_kind: unit.mobility_kind,
                    unit_source: unit.source_identity.clone(),
                    position,
                    world_position: unit.world_position(),
                    stats: unit.stats,
                }
            })
            .collect::<Vec<_>>();
        units.sort_by(|left, right| left.unit_instance_id.cmp(&right.unit_instance_id));

        LiveBattleStateCheckpointDto {
            at_seq,
            battle_time_ms: self.execution.last_event_time_ms(),
            playback: self.playback_state(),
            playback_eligibility: self.playback_eligibility_dto(),
            units,
            deployment: self.live_deployment_dto(roster),
        }
    }
}

fn live_unit_hud_dto(unit: &crate::game::battle::core::types::RuntimeUnit) -> LiveBattleUnitHudDto {
    let uses_resonance = unit.role != BattleUnitRole::DefenseObject
        && (unit.owner == crate::game::enums::Side::Player
            || unit.threat_class.uses_resonance_bar());
    let bar_mode = if uses_resonance {
        LiveBattleHudBarMode::HpAndResonance
    } else {
        LiveBattleHudBarMode::HpOnly
    };

    LiveBattleUnitHudDto {
        threat_class: unit.threat_class,
        bar_mode,
        resonance_current: uses_resonance.then_some(unit.resonance_current),
        resonance_max: uses_resonance.then_some(unit.resonance_max),
    }
}

#[derive(Clone)]
pub struct RunState {
    pub map: RunMap,
    pub map_progression: MapProgression,
    pub run_progression: RunProgression,
    pub facility_run_state: FacilityRunState,
    pub abnormality_research: RunAbnormalityResearchState,
    pub abnormality_encounter_history: RunAbnormalityEncounterHistory,
    pub monolith: MonolithRunState,
    pub boss_omen: BossOmenRunState,
    pub event_sessions: HashMap<crate::game::map::MapNodeId, EventSessionState>,
    pub shop_sessions: HashMap<crate::game::map::MapNodeId, ShopSessionState>,
    pub combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    pub abnormality_attempts: HashMap<crate::game::map::MapNodeId, AbnormalityAttemptState>,
    pub recruitment: RecruitmentState,
    pub combat_recovery_restriction: Option<CombatRecoveryRestriction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatRecoveryCapability {
    Recruitment,
    #[allow(dead_code)]
    Treatment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatRecoveryRestriction {
    pub allowed_node_ids: Vec<crate::game::map::MapNodeId>,
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
            facility_run_state: FacilityRunState::default(),
            abnormality_research: RunAbnormalityResearchState::default(),
            abnormality_encounter_history: RunAbnormalityEncounterHistory::default(),
            monolith: MonolithRunState::default(),
            boss_omen: BossOmenRunState::default(),
            event_sessions: HashMap::new(),
            shop_sessions: HashMap::new(),
            combat_previews: HashMap::new(),
            abnormality_attempts: HashMap::new(),
            recruitment: RecruitmentState::default(),
            combat_recovery_restriction: None,
        }
    }

    /// Build the next floor's run state from an existing run.
    ///
    /// Floor-local caches start empty, while run-scoped research history is preserved.
    pub fn for_next_floor_from(
        previous: &Self,
        map: RunMap,
        map_progression: MapProgression,
        run_progression: RunProgression,
        boss_omen: BossOmenRunState,
    ) -> Self {
        let mut next = Self::new(map, map_progression, run_progression)
            .with_persistent_run_state_from(previous)
            .with_boss_omen(boss_omen);
        next.monolith.clear_floor_local_remanifestations();
        next
    }

    /// Build the terminal run state from an existing run while preserving run-scoped research.
    pub fn for_run_complete_from(
        previous: &Self,
        map: RunMap,
        map_progression: MapProgression,
        run_progression: RunProgression,
    ) -> Self {
        Self::new(map, map_progression, run_progression).with_persistent_run_state_from(previous)
    }

    fn with_persistent_run_state_from(mut self, previous: &Self) -> Self {
        self.facility_run_state = previous.facility_run_state.clone();
        self.abnormality_research = previous.abnormality_research.clone();
        self.abnormality_encounter_history = previous.abnormality_encounter_history.clone();
        self.monolith = previous.monolith.clone();
        self.recruitment = previous.recruitment.clone();
        self
    }

    pub fn with_recruitment(mut self, recruitment: RecruitmentState) -> Self {
        self.recruitment = recruitment;
        self
    }

    pub fn with_abnormality_research(
        mut self,
        abnormality_research: RunAbnormalityResearchState,
    ) -> Self {
        self.abnormality_research = abnormality_research;
        self
    }

    pub fn with_abnormality_encounter_history(
        mut self,
        abnormality_encounter_history: RunAbnormalityEncounterHistory,
    ) -> Self {
        self.abnormality_encounter_history = abnormality_encounter_history;
        self
    }

    pub fn with_boss_omen(mut self, boss_omen: BossOmenRunState) -> Self {
        self.boss_omen = boss_omen;
        self
    }

    pub fn with_facility_run_state(mut self, facility_run_state: FacilityRunState) -> Self {
        self.facility_run_state = facility_run_state;
        self
    }

    pub fn abnormality_attempt_state(
        &self,
        node_id: crate::game::map::MapNodeId,
    ) -> AbnormalityAttemptState {
        let max_attempts = if self.monolith.remanifestation(node_id).is_some() {
            crate::game::monolith::MONOLITH_REMANIFESTATION_MAX_ATTEMPTS
        } else {
            ABNORMALITY_MAX_ATTEMPTS
        };
        self.abnormality_attempts
            .get(&node_id)
            .copied()
            .unwrap_or_else(|| AbnormalityAttemptState::new(max_attempts))
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
        let max_attempts = if self.monolith.remanifestation(node_id).is_some() {
            crate::game::monolith::MONOLITH_REMANIFESTATION_MAX_ATTEMPTS
        } else {
            ABNORMALITY_MAX_ATTEMPTS
        };
        let state = self
            .abnormality_attempts
            .entry(node_id)
            .or_insert_with(|| AbnormalityAttemptState::new(max_attempts));
        state.start_next_attempt().then_some(*state)
    }

    pub fn clear_abnormality_attempt(&mut self, node_id: crate::game::map::MapNodeId) {
        self.abnormality_attempts.remove(&node_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        abnormality_research::RunAbnormalityResearchEntry,
        data::event_data::{EventId, EventSceneId},
        employee::{EmployeeDefinition, EmployeeInitialLoadout, EmployeeProfile},
        map::{test_facility_map, MapNodeId},
        recruitment::RecruitmentState,
    };

    fn test_recruitment_state() -> RecruitmentState {
        let definitions = vec![EmployeeDefinition {
            employee_definition_id: "floor_candidate".to_string(),
            name: "Floor Candidate".to_string(),
            starter_eligible: false,
            initial_loadout: EmployeeInitialLoadout::default(),
            profile: EmployeeProfile::default(),
        }];
        RecruitmentState::initialize(41, &definitions, &[]).unwrap()
    }

    fn generated_run_state(seed: u64, run_progression: RunProgression) -> RunState {
        let map = test_facility_map(seed);
        let progression = MapProgression::from_map(&map);
        RunState::new(map, progression, run_progression)
    }

    #[test]
    fn run_power_anchor_uses_authored_minimum_previous_rise_and_peak_monotonically() {
        use crate::game::data::run_policy_data::ResolvedFloorPowerAnchorPolicy;

        let mut authority = RunPowerAuthorityState::default();
        authority.initialize_run(1, 80);
        assert_eq!(
            authority.manifest_floor(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 100,
                minimum_rise: 0,
            }),
            100
        );
        authority.record_combat_entry(1, 130);
        assert_eq!(
            authority.manifest_floor(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 110,
                minimum_rise: 25,
            }),
            130
        );
        authority.record_combat_entry(1, 120);
        assert_eq!(authority.run_peak_certified_power, 130);
        assert_eq!(
            authority.manifest_floor(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 140,
                minimum_rise: 25,
            }),
            155,
            "previous anchor plus minimum rise must win when it is strongest"
        );
    }

    #[test]
    fn next_floor_run_state_preserves_research_and_resets_node_scoped_state() {
        let mut previous =
            generated_run_state(0xA11CE, RunProgression::new(0xBEEF, GameMode::Standard, 3));
        previous.abnormality_research.entries.insert(
            "test_abnormality".to_string(),
            RunAbnormalityResearchEntry {
                research_points: 20,
                research_required: 100,
                response_complete: false,
                suppression_wins: 1,
                last_encountered_floor: Some(0),
                completed_at_floor: None,
                unique_fragment_granted: false,
            },
        );
        previous
            .abnormality_encounter_history
            .last_appeared_floor_by_abnormality
            .insert("test_abnormality".to_string(), 0);
        let node_id = MapNodeId::new(Uuid::from_u128(0xF100));
        previous.event_sessions.insert(
            node_id,
            EventSessionState::new(
                node_id,
                EventId::new("test_event"),
                EventSceneId::new("start"),
            ),
        );
        previous.abnormality_attempts.insert(
            node_id,
            AbnormalityAttemptState {
                max_attempts: 3,
                attempts_started: 2,
            },
        );
        previous.recruitment = test_recruitment_state();
        let mut run_progression = previous.run_progression.clone();
        assert!(run_progression.advance_floor());
        let next_map = test_facility_map(0xF100);
        let next_progression = MapProgression::from_map(&next_map);
        let next = RunState::for_next_floor_from(
            &previous,
            next_map,
            next_progression,
            run_progression,
            BossOmenRunState::default(),
        );

        assert_eq!(next.abnormality_research, previous.abnormality_research);
        assert_eq!(
            next.abnormality_encounter_history,
            previous.abnormality_encounter_history
        );
        assert!(next.event_sessions.is_empty());
        assert!(next.combat_previews.is_empty());
        assert!(next.abnormality_attempts.is_empty());
        assert_eq!(next.recruitment, previous.recruitment);
    }

    #[test]
    fn run_complete_state_preserves_run_scoped_research() {
        let mut previous =
            generated_run_state(0xC0DE, RunProgression::new(0xCAFE, GameMode::Standard, 1));
        previous
            .abnormality_encounter_history
            .last_appeared_floor_by_abnormality
            .insert("test_abnormality".to_string(), 0);
        previous.recruitment = test_recruitment_state();

        let completed = RunState::for_run_complete_from(
            &previous,
            previous.map.clone(),
            previous.map_progression.clone(),
            previous.run_progression.clone(),
        );

        assert_eq!(
            completed.abnormality_encounter_history,
            previous.abnormality_encounter_history
        );
        assert_eq!(completed.recruitment, previous.recruitment);
    }
}

impl GameCoreState {
    pub fn new(run_seed: u64) -> Self {
        let game_state = GameState::NotStarted;
        Self {
            player: None,
            game_state,
            active_node_content: None,
            node_session: None,
            run: None,
            visual_novel: VisualNovelRuntimeState::default(),
            visual_novel_triggers: VisualNovelTriggerRuntimeState::default(),
            uuid_manager: UuidManager::new(run_seed),
            enkephalin: Enkephalin::new(0),
            inventory: Inventory::new(),
            roster: EmployeeRoster::new(),
            starter_candidates: Vec::new(),
            pending_game_mode: None,
            skill_fragments: SkillFragmentInventory::new(),
            skill_fragment_policy: SkillFragmentPolicy::default_run_policy(),
            encounter_compendium: EncounterCompendiumProfileState::default(),
            active_battle: None,
            run_checkpoint: RunCheckpointState::default(),
            run_power_authority: RunPowerAuthorityState::default(),
        }
    }

    pub fn transition_to(&mut self, new_state: GameState) {
        self.game_state = new_state;
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
