use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufWriter, path::PathBuf};
use uuid::Uuid;

use crate::game::abnormality_research::{
    RunAbnormalityEncounterHistory, RunAbnormalityResearchState,
};
use crate::game::battle::{
    core::{
        sim::{BattleDeployCurrentHpPolicy, BattleExecutionState, BattleLiveSignal},
        BattleCore,
    },
    event_log::{BattleEventLog, BattleEventLogEntry},
    ids::UnitInstanceId,
    result_stats::BattleResultStatsDto,
    tile_range::FacingDirection,
    types::{BattleUnitRole, BattleWinner, ParticipantBattleResult},
};
use crate::game::behavior::{
    AbnormalityAttemptDto, ActionKind, BattlePlaybackState, GameError, LiveBattleDeployedUnitDto,
    LiveBattleDeploymentDto, LiveBattleEventDeltaDto, LiveBattleHudBarMode,
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
use crate::game::employee::{EmployeeRoster, StarterEmployeeCandidate};
use crate::game::employee_trust::EmployeeTrustPolicy;
use crate::game::enums::RewardMode;
use crate::game::managers::action_scheduler::ActionScheduler;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::map::{GameMode, MapProgression, NodeSession, RunMap, RunProgression};
use crate::game::range_preview::range_previews_for_runtime_unit;
use crate::game::resources::{
    ActionValidator, ActiveNodeContent, CombatBattleState, Enkephalin, EventSessionState,
    GameState, Inventory, RosterOrder,
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
    pub inventory: Inventory,
    pub roster_order: RosterOrder,
    pub roster: EmployeeRoster,
    pub starter_candidates: Vec<StarterEmployeeCandidate>,
    pub pending_game_mode: Option<GameMode>,
    pub skill_fragments: SkillFragmentInventory,
    pub skill_fragment_policy: SkillFragmentPolicy,
    pub research_delivery_policy: ResearchDeliveryPolicy,
    pub employee_trust_policy: EmployeeTrustPolicy,
    pub active_battle: Option<ActiveBattleSession>,
    pub run_checkpoint: RunCheckpointState,
}

pub const RUN_CHECKPOINT_MAX_LOADS: u8 = 3;

#[derive(Clone, Default)]
pub struct RunCheckpointState {
    pub payload: Option<RunCheckpointPayload>,
    pub loads_used: u8,
}

#[derive(Clone)]
pub struct RunCheckpointPayload {
    pub map: RunMap,
    pub map_progression: MapProgression,
    pub run_progression: RunProgression,
    pub abnormality_research: RunAbnormalityResearchState,
    pub abnormality_encounter_history: RunAbnormalityEncounterHistory,
    pub boss_omen: BossOmenRunState,
    pub event_sessions: HashMap<crate::game::map::MapNodeId, EventSessionState>,
    pub combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    pub abnormality_attempts: HashMap<crate::game::map::MapNodeId, AbnormalityAttemptState>,
    pub enkephalin: Enkephalin,
    pub inventory: Inventory,
    pub roster_order: RosterOrder,
    pub roster: EmployeeRoster,
    pub skill_fragments: SkillFragmentInventory,
    pub uuid_manager: UuidManager,
}

impl RunCheckpointState {
    pub fn can_load(&self) -> bool {
        self.payload.is_some() && self.loads_used < RUN_CHECKPOINT_MAX_LOADS
    }

    pub fn to_dto(&self) -> RunCheckpointSnapshotDto {
        let remaining_loads = RUN_CHECKPOINT_MAX_LOADS.saturating_sub(self.loads_used);
        RunCheckpointSnapshotDto {
            exists: self.payload.is_some(),
            loads_used: self.loads_used,
            max_loads: RUN_CHECKPOINT_MAX_LOADS,
            remaining_loads,
            can_load: self.can_load(),
        }
    }
}

pub struct ActiveBattleSession {
    pub battle_uuid: Uuid,
    pub primary_abnormality_id: Option<String>,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub combat_preview: CombatPreview,
    pub battle: BattleCore,
    pub execution: BattleExecutionState,
    pub last_pushed_event_log_seq: Option<u64>,
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
    pub current_hp_policy: BattleDeployCurrentHpPolicy,
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
                        BattleDeployCurrentHpPolicy::withdraw_redeploy(
                            unit.stats.current_health,
                            unit.stats.max_health,
                        )
                    }
                    Some(_) | None => BattleDeployCurrentHpPolicy::death_redeploy(),
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

        let mut abnormality_ids = self
            .primary_abnormality_id
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        abnormality_ids.extend(
            self.combat_preview
                .enemy_briefing
                .iter()
                .map(|briefing| briefing.abnormality_id.clone()),
        );
        abnormality_ids.extend(
            self.combat_preview
                .spawn_waves
                .iter()
                .flat_map(|wave| wave.enemy_entries.iter())
                .filter(|entry| {
                    entry.kind == crate::game::combat_preview::EnemyKind::Abnormality
                        && !entry.abnormality_id.trim().is_empty()
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
                    skill_readiness: self.battle.live_skill_readiness(
                        self.execution.last_event_time_ms(),
                        deployed.unit_instance_id,
                    ),
                }
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

    pub fn drain_battle_update_dto(&mut self, roster: &EmployeeRoster) -> LiveBattleUpdateDto {
        self.reconcile_live_deployment_with_battle_state();
        let after_seq = self.last_pushed_event_log_seq.unwrap_or(0);
        let events = self.battle.event_log_entries_after_seq(after_seq);
        let to_seq = events.last().map(|entry| entry.seq).unwrap_or(after_seq);
        if !events.is_empty() {
            self.last_pushed_event_log_seq = Some(to_seq);
        }
        self.battle_update_dto_from_events(after_seq, to_seq, events, roster)
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
        last_seen_seq: Option<u64>,
        roster: &EmployeeRoster,
    ) -> Result<LiveBattleUpdateDto, GameError> {
        let after_seq = last_seen_seq.unwrap_or(0);
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
                    range_previews: range_previews_for_runtime_unit(
                        unit,
                        self.battle.game_data.as_ref(),
                        &self.battle.battlefield,
                        position,
                    ),
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

pub struct RunState {
    pub map: RunMap,
    pub map_progression: MapProgression,
    pub run_progression: RunProgression,
    pub abnormality_research: RunAbnormalityResearchState,
    pub abnormality_encounter_history: RunAbnormalityEncounterHistory,
    pub boss_omen: BossOmenRunState,
    pub event_sessions: HashMap<crate::game::map::MapNodeId, EventSessionState>,
    pub combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    pub abnormality_attempts: HashMap<crate::game::map::MapNodeId, AbnormalityAttemptState>,
    /// Run-local abnormality codex/observation records.
    ///
    /// This intentionally stores one representative combat record per
    /// `abnormality_uuid`. It is not a per-battle timeline archive; individual
    /// timeline exports must use a separate battle-uuid keyed store.
    pub battle_records: Vec<CombatBattleState>,
}

#[derive(Debug, Serialize)]
struct AbnormalityBattleRecordDebugExport<'a> {
    version: u32,
    run_seed: u64,
    abnormality_uuid: Uuid,
    primary_abnormality_id: Option<&'a str>,
    encounter_id: &'a str,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    winner: BattleWinner,
    participant_results: &'a [ParticipantBattleResult],
    result_stats: &'a BattleResultStatsDto,
    bonus_objectives: &'a [crate::game::pve_bonus_objectives::PveBonusObjectiveOutcomeDto],
    event_log: &'a BattleEventLog,
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
            abnormality_research: RunAbnormalityResearchState::default(),
            abnormality_encounter_history: RunAbnormalityEncounterHistory::default(),
            boss_omen: BossOmenRunState::default(),
            event_sessions: HashMap::new(),
            combat_previews: HashMap::new(),
            abnormality_attempts: HashMap::new(),
            battle_records: Vec::new(),
        }
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

    pub fn battle_record_debug_export_root_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("battle_records")
            .join(format!("run_{:016x}", self.run_progression.run_seed))
    }

    pub fn battle_record_debug_export_path(&self, abnormality_uuid: Uuid) -> PathBuf {
        self.battle_record_debug_export_root_dir()
            .join(format!("{abnormality_uuid}.json"))
    }

    pub fn store_abnormality_battle_record(&mut self, battle: &CombatBattleState) -> bool {
        // Codex records are keyed by abnormality, not by battle UUID. Repeated
        // fights against the same abnormality keep the first representative
        // observation record.
        if self
            .battle_records
            .iter()
            .any(|record| record.abnormality_uuid == battle.abnormality_uuid)
        {
            return false;
        }
        self.battle_records.push(battle.clone());
        true
    }

    pub fn export_abnormality_battle_record_debug_json(
        &self,
        battle: &CombatBattleState,
    ) -> Result<PathBuf, GameError> {
        let out_dir = self.battle_record_debug_export_root_dir();
        std::fs::create_dir_all(&out_dir).map_err(|error| {
            GameError::InvalidStaticData(format!(
                "failed to create battle record debug export directory '{}': {error}",
                out_dir.display()
            ))
        })?;
        let out_path = self.battle_record_debug_export_path(battle.abnormality_uuid);
        let file = File::create(&out_path).map_err(|error| {
            GameError::InvalidStaticData(format!(
                "failed to create battle record debug export file '{}': {error}",
                out_path.display()
            ))
        })?;
        let writer = BufWriter::new(file);
        let export = AbnormalityBattleRecordDebugExport {
            version: 1,
            run_seed: self.run_progression.run_seed,
            abnormality_uuid: battle.abnormality_uuid,
            primary_abnormality_id: battle.primary_abnormality_id.as_deref(),
            encounter_id: &battle.encounter_id,
            node_type: battle.node_type,
            mission_variant: battle.mission_variant,
            winner: battle.winner,
            participant_results: &battle.participant_results,
            result_stats: &battle.result_stats,
            bonus_objectives: &battle.bonus_objectives,
            event_log: &battle.event_log,
        };
        serde_json::to_writer_pretty(writer, &export).map_err(|error| {
            GameError::InvalidStaticData(format!(
                "failed to write battle record debug export file '{}': {error}",
                out_path.display()
            ))
        })?;
        Ok(out_path)
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
            inventory: Inventory::new(),
            roster_order: RosterOrder::new(super::ROSTER_ORDER_SLOTS),
            roster: EmployeeRoster::new(),
            starter_candidates: Vec::new(),
            pending_game_mode: None,
            skill_fragments: SkillFragmentInventory::new(),
            skill_fragment_policy: SkillFragmentPolicy::default_run_policy(),
            research_delivery_policy: ResearchDeliveryPolicy::default(),
            employee_trust_policy: EmployeeTrustPolicy::narrative_only(),
            active_battle: None,
            run_checkpoint: RunCheckpointState::default(),
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
