use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use uuid::Uuid;

use crate::game::{
    battle::{
        battlefield::BattlefieldLayout,
        core::movement::{
            engine::ContinuousMovementBackend,
            types::{ActiveMovementSegment, EventLogVec2, UnitBody, WorldVec2},
        },
        enums::BattleEvent,
        event_log::{BattleEventCause, BattleEventLog, BattleLogEvent},
        ids::UnitInstanceId,
        scenario::{BattleScenario, ScenarioGroupId, ScenarioUnitRef},
        types::UnitSnapshot,
    },
    data::GameDataBase,
    resources::Position,
};

mod basic_attack;
mod basic_attack_projectile;
pub mod build;
pub mod commands;
mod damage_runtime;
mod death_runtime;
pub mod ids;
pub mod movement;
mod projectile_math;
mod resonance_runtime;
pub mod sim;
pub mod skill_runtime;
pub mod spatial;
mod target_usefulness;
pub mod targeting;
pub mod triggers;
pub mod types;

use self::types::{
    AbilityProcKey, AbilityProcState, ActiveBuff, ActiveProjectileRuntime, ActiveSkillCast,
    ActiveSkillCastDebugState, AreaRuntime, BuffInstanceKey, ProjectileRecord, RuntimeArtifact,
    RuntimeItem, RuntimeUnit, RuntimeUnitLifecycle, TriggerSource,
};

pub struct BattleCore {
    event_queue: BinaryHeap<BattleEvent>,

    scenario: BattleScenario,
    scenario_runtime: ScenarioRuntimeState,

    pub units: HashMap<UnitInstanceId, RuntimeUnit>,
    artifacts: HashMap<Uuid, RuntimeArtifact>,
    items: HashMap<Uuid, RuntimeItem>,
    graveyard: HashMap<UnitInstanceId, UnitSnapshot>,

    buffs: HashMap<BuffInstanceKey, ActiveBuff>,
    active_skill_casts: HashMap<u64, ActiveSkillCast>,
    ability_proc_states: HashMap<AbilityProcKey, AbilityProcState>,

    projectiles: HashMap<Uuid, ProjectileRecord>,
    active_projectiles: HashMap<Uuid, ActiveProjectileRuntime>,
    active_areas: HashMap<Uuid, AreaRuntime>,
    active_movement_segments: HashMap<UnitInstanceId, ActiveMovementSegment>,
    block_state: BlockRuntimeState,
    last_continuous_movement_tick_ms: Option<u64>,
    movement_backend: ContinuousMovementBackend,
    pub battlefield: BattlefieldLayout,
    live_signals: Vec<sim::BattleLiveSignal>,

    pub game_data: Arc<GameDataBase>,

    pub event_log: BattleEventLog,
    pub event_log_seq: u64,
    pub projectile_seq: u64,
    pub area_seq: u64,
    pub seed: u64,
    pub recording_cause_stack: Vec<BattleEventCause>,
    pub recording_source_command_stack: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InactiveRuntimeUnitDebugSnapshot {
    pub unit_instance_id: UnitInstanceId,
    pub owner: crate::game::enums::Side,
    pub lifecycle: RuntimeUnitLifecycle,
    pub world_position: EventLogVec2,
    pub position: Position,
    pub current_health: u32,
    pub max_health: u32,
}

#[derive(Debug, Clone, Default)]
pub(in crate::game::battle::core) struct ScenarioRuntimeState {
    spawned_groups: std::collections::HashSet<ScenarioGroupId>,
    unit_refs: HashMap<ScenarioUnitRef, UnitInstanceId>,
    forced_winner: Option<crate::game::battle::types::BattleWinner>,
    next_spawn_order: u64,
}

#[derive(Debug, Clone, Default)]
pub(in crate::game::battle::core) struct BlockRuntimeState {
    blocker_to_enemies: HashMap<UnitInstanceId, Vec<UnitInstanceId>>,
    enemy_to_blocker: HashMap<UnitInstanceId, UnitInstanceId>,
}

impl BattleCore {
    pub fn scenario(&self) -> &BattleScenario {
        &self.scenario
    }

    pub fn new_from_scenario(
        scenario: BattleScenario,
        game_data: Arc<GameDataBase>,
        seed: u64,
    ) -> Self {
        let field_size = (scenario.battlefield.width, scenario.battlefield.height);
        let valid_tiles = scenario.battlefield.valid_tiles.clone();
        Self {
            event_queue: BinaryHeap::new(),
            scenario,
            scenario_runtime: ScenarioRuntimeState::default(),
            units: HashMap::new(),
            artifacts: HashMap::new(),
            items: HashMap::new(),
            graveyard: HashMap::new(),
            buffs: HashMap::new(),
            active_skill_casts: HashMap::new(),
            ability_proc_states: HashMap::new(),
            projectiles: HashMap::new(),
            active_projectiles: HashMap::new(),
            active_areas: HashMap::new(),
            active_movement_segments: HashMap::new(),
            block_state: BlockRuntimeState::default(),
            last_continuous_movement_tick_ms: None,
            movement_backend: ContinuousMovementBackend::default(),
            battlefield: BattlefieldLayout::new_with_valid_tiles(
                field_size.0,
                field_size.1,
                valid_tiles,
            ),
            live_signals: Vec::new(),
            game_data,
            event_log: BattleEventLog::new(),
            event_log_seq: 1,
            projectile_seq: 0,
            area_seq: 0,
            seed,
            recording_cause_stack: Vec::new(),
            recording_source_command_stack: Vec::new(),
        }
    }

    /// Enqueue a deterministic battle event before the simulation loop starts.
    ///
    /// This is primarily used by battle setup harnesses that need to seed
    /// future state changes, such as scripted buff application during
    /// integration tests.
    pub fn enqueue_event(&mut self, event: BattleEvent) {
        self.event_queue.push(event);
    }

    pub fn unit_world_position(&self, unit_instance_id: UnitInstanceId) -> Option<WorldVec2> {
        self.units
            .get(&unit_instance_id)
            .map(RuntimeUnit::world_position)
    }

    pub fn unit_projected_tile(&self, unit_instance_id: UnitInstanceId) -> Option<Position> {
        self.units
            .get(&unit_instance_id)
            .map(|unit| unit.body.projected_tile())
    }

    pub(in crate::game::battle::core) fn live_unit_projected_tile(
        &self,
        unit_instance_id: UnitInstanceId,
    ) -> Option<Position> {
        self.units
            .get(&unit_instance_id)
            .filter(|unit| unit.is_active())
            .map(|unit| unit.body.projected_tile())
    }

    pub fn live_unit_bodies(&self) -> Vec<(UnitInstanceId, UnitBody)> {
        let mut bodies: Vec<_> = self
            .units
            .values()
            .filter(|unit| unit.is_active())
            .map(|unit| (unit.instance_id, unit.movement_body_view()))
            .collect();
        bodies.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        bodies
    }

    pub fn active_basic_projectile_count(&self) -> usize {
        self.projectiles.len()
    }

    pub fn active_skill_projectile_count(&self) -> usize {
        self.active_projectiles.len()
    }

    pub fn active_area_count(&self) -> usize {
        self.active_areas.len()
    }

    pub(in crate::game::battle::core) fn unit_body_view(
        &self,
        unit_instance_id: UnitInstanceId,
    ) -> Option<UnitBody> {
        self.units
            .get(&unit_instance_id)
            .map(RuntimeUnit::movement_body_view)
    }

    pub fn apply_unit_body_position(
        &mut self,
        unit_instance_id: UnitInstanceId,
        body: &UnitBody,
    ) -> bool {
        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return false;
        };
        unit.apply_movement_body_position(body);
        true
    }

    /// Read-only debug snapshot of one active skill cast.
    ///
    /// This is intended for integration harnesses and deterministic runtime
    /// debugging, so tests can assert whether async skill steps reached a
    /// terminal state without exposing mutable internal state.
    pub fn debug_skill_cast_state(&self, cast_seq: u64) -> Option<ActiveSkillCastDebugState> {
        self.active_skill_casts
            .get(&cast_seq)
            .map(ActiveSkillCastDebugState::from)
    }

    /// Read-only debug/replay snapshot of retained inactive runtime units.
    ///
    /// Official gameplay checkpoints intentionally expose Active units only.
    /// This query exists for debug/admin/replay inspection of retained
    /// `Withdrawn` and `Dead` runtime entities without making them gameplay
    /// presentation sources.
    pub fn debug_inactive_units(&self) -> Vec<InactiveRuntimeUnitDebugSnapshot> {
        let mut units = self
            .units
            .values()
            .filter(|unit| !unit.is_active())
            .map(|unit| InactiveRuntimeUnitDebugSnapshot {
                unit_instance_id: unit.instance_id,
                owner: unit.owner,
                lifecycle: unit.lifecycle,
                world_position: unit.body.position.quantized_milli(),
                position: unit.body.projected_tile(),
                current_health: unit.stats.current_health,
                max_health: unit.stats.max_health,
            })
            .collect::<Vec<_>>();
        units.sort_by(|left, right| left.unit_instance_id.cmp(&right.unit_instance_id));
        units
    }

    fn can_gain_resonance(unit: &RuntimeUnit, now_ms: u64) -> bool {
        unit.action_locks.can_gain_resonance(now_ms)
    }

    fn can_start_autocast(unit: &RuntimeUnit, now_ms: u64) -> bool {
        if unit.skill_activation_mode != crate::game::ability::SkillActivationMode::Auto {
            return false;
        }
        if now_ms < unit.next_action_time {
            return false;
        }
        if !unit.is_combatant() {
            return false;
        }
        if !unit.is_active() {
            return false;
        }
        true
    }

    fn add_resonance(
        &mut self,
        unit_instance_id: UnitInstanceId,
        amount: u32,
        now_ms: u64,
        allow_autocast_when_full: bool,
    ) {
        if amount == 0 {
            return;
        }

        let (before, after, max) = {
            if self
                .active_hard_cc_release_time_ms(unit_instance_id, now_ms)
                .is_some()
            {
                return;
            }
            let Some(unit) = self.units.get_mut(&unit_instance_id) else {
                return;
            };

            if !unit.is_active() {
                return;
            }

            if !Self::can_gain_resonance(unit, now_ms) {
                return;
            }

            let max = unit.resonance_max.max(1);
            let before = unit.resonance_current.min(max);
            let after = before.saturating_add(amount).min(max);
            if before == after {
                return;
            }

            unit.resonance_current = after;

            if allow_autocast_when_full
                && unit.skill_activation_mode == crate::game::ability::SkillActivationMode::Auto
                && before < max
                && after == max
            {
                unit.pending_cast = true;
            }

            (before, after, max)
        };

        self.record_event_log(
            now_ms,
            BattleLogEvent::ResonanceChanged {
                unit_instance_id,
                before,
                after,
                max,
            },
        );
    }

    fn modify_resonance(
        &mut self,
        unit_instance_id: UnitInstanceId,
        delta: i32,
        now_ms: u64,
        allow_autocast_when_full: bool,
    ) {
        if delta == 0 {
            return;
        }

        if delta > 0 {
            self.add_resonance(
                unit_instance_id,
                delta as u32,
                now_ms,
                allow_autocast_when_full,
            );
            return;
        }

        let (before, after, max) = {
            let Some(unit) = self.units.get_mut(&unit_instance_id) else {
                return;
            };

            if !unit.is_active() {
                return;
            }

            let max = unit.resonance_max.max(1);
            let before = unit.resonance_current.min(max);
            let dec = delta.unsigned_abs().min(before);
            let after = before.saturating_sub(dec);
            if before == after {
                return;
            }

            unit.resonance_current = after;
            (before, after, max)
        };

        self.record_event_log(
            now_ms,
            BattleLogEvent::ResonanceChanged {
                unit_instance_id,
                before,
                after,
                max,
            },
        );
    }

    fn has_buff_kind(
        &self,
        unit_instance_id: UnitInstanceId,
        kind: crate::game::battle::buffs::BuffKind,
    ) -> bool {
        self.buffs.iter().any(|(key, _)| {
            key.target_instance_id == unit_instance_id
                && self
                    .game_data
                    .buff_data
                    .get(key.buff_id)
                    .is_some_and(|def| def.kind == kind)
        })
    }

    fn is_hard_cc_kind(kind: crate::game::battle::buffs::BuffKind) -> bool {
        matches!(
            kind,
            crate::game::battle::buffs::BuffKind::Stun
                | crate::game::battle::buffs::BuffKind::Freeze
        )
    }

    fn active_hard_cc_expires_at(&self, unit_instance_id: UnitInstanceId) -> Option<u64> {
        self.buffs
            .iter()
            .filter_map(|(key, active)| {
                if key.target_instance_id != unit_instance_id {
                    return None;
                }
                let def = self.game_data.buff_data.get(key.buff_id)?;
                Self::is_hard_cc_kind(def.kind).then_some(active.expires_at_ms)
            })
            .max()
    }

    pub(in crate::game::battle::core) fn active_hard_cc_release_time_ms(
        &self,
        unit_instance_id: UnitInstanceId,
        now_ms: u64,
    ) -> Option<u64> {
        self.active_hard_cc_expires_at(unit_instance_id)
            .filter(|expires_at_ms| now_ms <= *expires_at_ms)
            .map(|expires_at_ms| expires_at_ms.saturating_add(1))
    }

    fn schedule_pending_autocast_for(&mut self, caster_instance_id: UnitInstanceId, now_ms: u64) {
        let fallback = self.recording_cause().unwrap_or_default();

        if self.has_buff_kind(
            caster_instance_id,
            crate::game::battle::buffs::BuffKind::Silence,
        ) {
            return;
        }
        if self
            .active_hard_cc_release_time_ms(caster_instance_id, now_ms)
            .is_some()
        {
            return;
        }

        let Some((cause, should_schedule)) = self.units.get_mut(&caster_instance_id).map(|unit| {
            if !unit.pending_cast {
                return (fallback, false);
            }
            if !Self::can_start_autocast(unit, now_ms) {
                return (fallback, false);
            }

            unit.pending_cast = false;
            let cause = unit.pending_cast_cause.take().unwrap_or(fallback);
            (cause, true)
        }) else {
            return;
        };

        if !should_schedule {
            return;
        }

        self.event_queue.push(BattleEvent::AutoCastStart {
            time_ms: now_ms,
            caster_instance_id,
            cause,
        });
    }

    fn schedule_pending_autocasts(&mut self, now_ms: u64) {
        let mut casters: Vec<UnitInstanceId> = self
            .units
            .iter()
            .filter_map(|(id, unit)| {
                if unit.pending_cast && unit.is_active() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        casters.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let cause = self.recording_cause().unwrap_or_default();

        for caster_instance_id in casters {
            if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                if unit.pending_cast && unit.pending_cast_cause.is_none() {
                    unit.pending_cast_cause = Some(cause);
                }
            }
            self.schedule_pending_autocast_for(caster_instance_id, now_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ability::{
        DeliveryDef, SkillActivationMode, SkillAreaAnchorSource, SkillAreaTracking,
        SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillId, SkillKind, SkillPresentationDef,
        SkillStepDef, SkillTarget, SkillTileAreaDeliveryDef, SkillTileAreaOrigin,
        StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::buffs::BuffId;
    use crate::game::battle::core::movement::{types::WorldVec2, ActionState};
    use crate::game::battle::core::types::{PendingSkillCast, RuntimeUnit, RuntimeUnitLifecycle};
    use crate::game::battle::damage::{BattleCommand, DamageSource, DamageType};
    use crate::game::battle::enums::BattleEvent;
    use crate::game::battle::event_log::{
        AttackKind, BattleEventCause, BattleEventRootCause, BattleLogEvent, BuffExpireReason,
        SkillCastCancelReason, SkillCastTarget,
    };
    use crate::game::battle::scenario::BattleScenario;
    use crate::game::battle::tile_range::{FacingDirection, TileRangePattern, TileRangePolicy};
    use crate::game::battle::types::{
        BattleUnitDraft, BattleUnitSource, DeploymentAffinity, MobilityKind, UnitCombatProfile,
    };
    use crate::game::data::run_policy_data::RunPolicyData;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata, skill_data::SkillDatabase, GameDataBase,
        GameDataBuilder,
    };
    use crate::game::enums::Side;
    use crate::game::resources::Position;
    use crate::game::stats::UnitStats;
    use crate::game::stats::{StatId, StatModifier, StatModifierKind};

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::live_defaults().build_arc()
    }

    fn new_core() -> BattleCore {
        BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), empty_game_data(), 123)
    }

    #[test]
    fn battle_runtime_reset_clears_continuous_movement_state() {
        let mut core = new_core();
        let unit_id = UnitInstanceId::from(Uuid::from_u128(101));

        core.active_movement_segments.insert(
            unit_id,
            ActiveMovementSegment {
                start: WorldVec2::ZERO,
                target: WorldVec2::new(3.0, 0.0),
                started_at_ms: 10,
                ends_at_ms: 100,
            },
        );
        core.last_continuous_movement_tick_ms = Some(100);

        core.reset_runtime_state_for_battle();

        assert!(core.active_movement_segments.is_empty());
        assert_eq!(core.last_continuous_movement_tick_ms, None);
    }

    #[test]
    fn scenario_delayed_required_group_prevents_early_victory_and_spawns_later() {
        let player_group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let first_wave_id = crate::game::battle::scenario::ScenarioGroupId::new("wave_1");
        let second_wave_id = crate::game::battle::scenario::ScenarioGroupId::new("wave_2");

        let player_owned = Uuid::from_u128(1);
        let first_enemy_owned = Uuid::from_u128(2);
        let second_enemy_owned = Uuid::from_u128(3);

        let player_ref = crate::game::battle::scenario::ScenarioUnitRef::new("player_0");
        let first_enemy_ref = crate::game::battle::scenario::ScenarioUnitRef::new("wave_1_0");
        let second_enemy_ref = crate::game::battle::scenario::ScenarioUnitRef::new("wave_2_0");

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 5,
                height: 5,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: player_group_id.clone(),
                    side: Side::Player,
                    required_for_victory: false,
                    enemy_movement_plan: None,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: player_ref,
                        side: Side::Player,
                        draft: fixture_draft(player_owned, Uuid::from_u128(10), 100, 100),
                        position: Position::new(1, 1),
                        instance_salt: 0,
                    }],
                },
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: first_wave_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: true,
                    enemy_movement_plan: None,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: first_enemy_ref,
                        side: Side::Opponent,
                        draft: fixture_draft(first_enemy_owned, Uuid::from_u128(20), 1, 1),
                        position: Position::new(2, 1),
                        instance_salt: 0,
                    }],
                },
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: second_wave_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: true,
                    enemy_movement_plan: None,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: second_enemy_ref,
                        side: Side::Opponent,
                        draft: fixture_draft(second_enemy_owned, Uuid::from_u128(30), 1, 1),
                        position: Position::new(2, 1),
                        instance_salt: 1,
                    }],
                },
            ],
            events: vec![
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_player"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: player_group_id,
                    },
                    once: true,
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_wave_1"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: first_wave_id,
                    },
                    once: true,
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_wave_2"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtTimeMs(1_000),
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: second_wave_id,
                    },
                    once: true,
                },
            ],
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let result = core.run_battle().expect("battle should run");

        assert_eq!(
            result.winner,
            crate::game::battle::types::BattleWinner::Player
        );
        assert!(result.event_log.entries.iter().any(|entry| {
            entry.time_ms == 1_000 && matches!(entry.event, BattleLogEvent::UnitSpawned { .. })
        }));
    }

    #[test]
    fn step_battle_execution_matches_full_battle_run() {
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: vec![crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("force_end"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtTimeMs(100),
                action: crate::game::battle::scenario::ScenarioAction::EndBattle {
                    winner: crate::game::battle::types::BattleWinner::Player,
                },
                once: true,
            }],
            win_condition: crate::game::battle::scenario::WinCondition::SurviveUntil {
                time_ms: 1_000,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let game_data = empty_game_data();

        let mut full_core = BattleCore::new_from_scenario(scenario.clone(), game_data.clone(), 123);
        let full_result = full_core
            .run_battle()
            .expect("full event-log battle run should complete");

        let mut stepped_core = BattleCore::new_from_scenario(scenario, game_data, 123);
        let mut execution = stepped_core
            .start_battle_execution()
            .expect("battle execution should start");
        let stepped_result = loop {
            match stepped_core
                .step_battle_execution(&mut execution)
                .expect("battle step should run")
            {
                crate::game::battle::core::sim::BattleStepOutcome::Running => {}
                crate::game::battle::core::sim::BattleStepOutcome::Finished(finish) => {
                    break stepped_core
                        .finalize_battle_execution(&mut execution, finish)
                        .expect("battle execution should finalize")
                }
            }
        };

        assert!(execution.is_finished());
        assert!(execution.is_finalized());
        assert_eq!(stepped_result.winner, full_result.winner);
        assert_eq!(
            stepped_result.event_log.to_json_string().unwrap(),
            full_result.event_log.to_json_string().unwrap()
        );
    }

    #[test]
    fn protect_unit_until_wins_at_timer_even_with_enemy_alive() {
        let protected_group_id = crate::game::battle::scenario::ScenarioGroupId::new("protected");
        let enemy_group_id = crate::game::battle::scenario::ScenarioGroupId::new("enemy");
        let protected_ref = crate::game::battle::scenario::ScenarioUnitRef::new("black_box");
        let enemy_ref = crate::game::battle::scenario::ScenarioUnitRef::new("enemy_0");

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: protected_group_id.clone(),
                    side: Side::Player,
                    required_for_victory: false,
                    enemy_movement_plan: None,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: protected_ref.clone(),
                        side: Side::Player,
                        draft: fixture_draft(
                            Uuid::from_u128(0x901),
                            Uuid::from_u128(0x902),
                            100,
                            0,
                        ),
                        position: Position::new(0, 0),
                        instance_salt: 0,
                    }],
                },
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: enemy_group_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: true,
                    enemy_movement_plan: None,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: enemy_ref,
                        side: Side::Opponent,
                        draft: fixture_draft(
                            Uuid::from_u128(0x903),
                            Uuid::from_u128(0x904),
                            100,
                            0,
                        ),
                        position: Position::new(3, 3),
                        instance_salt: 0,
                    }],
                },
            ],
            events: vec![
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_protected"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: protected_group_id,
                    },
                    once: true,
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_enemy"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: enemy_group_id,
                    },
                    once: true,
                },
            ],
            win_condition: crate::game::battle::scenario::WinCondition::ProtectUnitUntil {
                unit_ref: protected_ref,
                time_ms: 1_000,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let mut execution = core
            .start_battle_execution()
            .expect("battle execution should start");

        let outcome = core
            .step_battle_execution_by(&mut execution, 1_000)
            .expect("timer step should finish");

        assert!(matches!(
            outcome,
            crate::game::battle::core::sim::BattleStepOutcome::Finished(finish)
                if finish.winner == crate::game::battle::types::BattleWinner::Player
        ));
        assert!(core
            .units
            .values()
            .any(|unit| unit.owner == Side::Opponent && unit.is_active()));
    }

    #[test]
    fn protect_unit_until_loses_when_objective_is_destroyed_before_timer() {
        let protected_group_id = crate::game::battle::scenario::ScenarioGroupId::new("protected");
        let protected_ref = crate::game::battle::scenario::ScenarioUnitRef::new("black_box");

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: protected_group_id.clone(),
                side: Side::Player,
                required_for_victory: false,
                enemy_movement_plan: None,
                spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                    unit_ref: protected_ref.clone(),
                    side: Side::Player,
                    draft: fixture_draft(Uuid::from_u128(0x911), Uuid::from_u128(0x912), 100, 0),
                    position: Position::new(0, 0),
                    instance_salt: 0,
                }],
            }],
            events: vec![crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("spawn_protected"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                    group_id: protected_group_id,
                },
                once: true,
            }],
            win_condition: crate::game::battle::scenario::WinCondition::ProtectUnitUntil {
                unit_ref: protected_ref.clone(),
                time_ms: 1_000,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let mut execution = core
            .start_battle_execution()
            .expect("battle execution should start");
        let initial_outcome = core
            .step_battle_execution_by(&mut execution, 0)
            .expect("battle start spawn should process");
        assert!(matches!(
            initial_outcome,
            crate::game::battle::core::sim::BattleStepOutcome::Running
        ));
        let protected_id = core.scenario_runtime.unit_refs[&protected_ref];
        core.units
            .get_mut(&protected_id)
            .expect("protected unit should exist")
            .stats
            .current_health = 0;
        core.units
            .get_mut(&protected_id)
            .expect("protected unit should exist")
            .lifecycle = RuntimeUnitLifecycle::Dead;

        let outcome = core
            .step_battle_execution_by(&mut execution, 100)
            .expect("objective destruction should finish");

        assert!(matches!(
            outcome,
            crate::game::battle::core::sim::BattleStepOutcome::Finished(finish)
                if finish.winner == crate::game::battle::types::BattleWinner::Opponent
        ));
    }

    #[test]
    fn apply_live_command_deploys_and_withdraws_player_unit() {
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: Vec::new(),
            win_condition: crate::game::battle::scenario::WinCondition::SurviveUntil {
                time_ms: 1_000,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let draft = fixture_draft(Uuid::from_u128(10), Uuid::from_u128(11), 100, 10);

        let deployed = core
            .apply_live_command(
                crate::game::battle::core::sim::BattleLiveCommand::DeployPlayerUnit {
                    draft,
                    position: Position::new(1, 1),
                    facing: crate::game::battle::tile_range::FacingDirection::Right,
                    instance_salt: 7,
                    time_ms: 250,
                    current_hp_policy: None,
                },
            )
            .expect("live deploy command should succeed");
        let unit_id = match deployed {
            crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id } => {
                unit_id
            }
            crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitWithdrawn { .. } => {
                panic!("deploy command must report deployed unit")
            }
            crate::game::battle::core::sim::BattleLiveCommandOutcome::SkillActivated { .. } => {
                panic!("deploy command must report deployed unit")
            }
        };

        assert!(core.units.contains_key(&unit_id));
        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 250
                && matches!(
                    entry.event,
                    BattleLogEvent::UnitSpawned {
                        unit_instance_id,
                        ..
                    } if unit_instance_id == unit_id
                )
        }));
        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 250
                && matches!(
                    entry.event,
                    BattleLogEvent::UnitDeployed {
                        employee_uuid,
                        unit_instance_id,
                        position,
                        facing,
                    } if employee_uuid == Uuid::from_u128(10)
                        && unit_instance_id == unit_id
                        && position == Position::new(1, 1)
                        && facing == crate::game::battle::tile_range::FacingDirection::Right
                )
        }));

        let withdrawn = core
            .apply_live_command(
                crate::game::battle::core::sim::BattleLiveCommand::WithdrawUnit {
                    unit_id,
                    time_ms: 25,
                },
            )
            .expect("live withdraw command should succeed");

        assert_eq!(
            withdrawn,
            crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitWithdrawn { unit_id }
        );
        assert_eq!(
            core.units.get(&unit_id).map(|unit| unit.lifecycle),
            Some(RuntimeUnitLifecycle::Withdrawn)
        );
        assert_eq!(core.unit_projected_tile(unit_id), Some(Position::new(1, 1)));
        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 25
                && matches!(
                    entry.event,
                    BattleLogEvent::UnitWithdrawn {
                        unit_instance_id,
                        world_position,
                        position,
                    } if unit_instance_id == unit_id
                        && world_position.x_milli == 1500
                        && world_position.y_milli == 1500
                        && position == Position::new(1, 1)
                )
        }));
    }

    #[test]
    fn debug_inactive_units_lists_retained_non_active_units_with_lifecycle_and_position() {
        let mut core = new_core();
        let active_id: UnitInstanceId = Uuid::from_u128(0xDD01).into();
        let withdrawn_id: UnitInstanceId = Uuid::from_u128(0xDD02).into();
        let dead_id: UnitInstanceId = Uuid::from_u128(0xDD03).into();

        let mut active = runtime_unit(active_id, Side::Player);
        active.set_world_position(WorldVec2::from_tile_center(Position::new(0, 0)));
        core.units.insert(active_id, active);

        let mut withdrawn = runtime_unit(withdrawn_id, Side::Player);
        withdrawn.lifecycle = RuntimeUnitLifecycle::Withdrawn;
        withdrawn.set_world_position(WorldVec2::from_tile_center(Position::new(1, 2)));
        core.units.insert(withdrawn_id, withdrawn);

        let mut dead = runtime_unit(dead_id, Side::Opponent);
        dead.lifecycle = RuntimeUnitLifecycle::Dead;
        dead.stats.current_health = 0;
        dead.set_world_position(WorldVec2::from_tile_center(Position::new(2, 3)));
        core.units.insert(dead_id, dead);

        let inactive = core.debug_inactive_units();

        assert_eq!(inactive.len(), 2);
        assert!(inactive
            .iter()
            .all(|unit| unit.unit_instance_id != active_id));
        assert!(inactive.iter().any(|unit| {
            unit.unit_instance_id == withdrawn_id
                && unit.lifecycle == RuntimeUnitLifecycle::Withdrawn
                && unit.position == Position::new(1, 2)
                && unit.world_position.x_milli == 1500
                && unit.world_position.y_milli == 2500
                && unit.current_health == 10
        }));
        assert!(inactive.iter().any(|unit| {
            unit.unit_instance_id == dead_id
                && unit.lifecycle == RuntimeUnitLifecycle::Dead
                && unit.position == Position::new(2, 3)
                && unit.world_position.x_milli == 2500
                && unit.world_position.y_milli == 3500
                && unit.current_health == 0
        }));
    }

    #[test]
    fn withdraw_clears_runtime_buffs_with_withdrawn_reasons() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD11).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD12).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        let poison_id = BuffId::from_name("poison");

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: caster_id,
                target_instance_id: target_id,
                buff_id: poison_id,
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: target_id,
                target_instance_id: caster_id,
                buff_id: poison_id,
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        core.withdraw_unit(target_id, 10).unwrap();

        assert!(core.buffs.is_empty());
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
                reason: BuffExpireReason::TargetWithdrawn,
            } if entry.time_ms == 10
                && caster_instance_id == caster_id
                && target_instance_id == target_id
                && buff_id == poison_id
        )));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
                reason: BuffExpireReason::CasterWithdrawn,
            } if entry.time_ms == 10
                && caster_instance_id == target_id
                && target_instance_id == caster_id
                && buff_id == poison_id
        )));
    }

    #[test]
    fn withdraw_cancels_pending_skill_cast_with_cancelled_event() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD21).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let skill_id = SkillId::from("pending_withdraw_skill");
        let start_seq = core.record_event_log(
            0,
            BattleLogEvent::AutoCastStart {
                caster_instance_id: caster_id,
                skill_id: Some(skill_id.clone()),
                target: None,
            },
        );
        core.units.get_mut(&caster_id).unwrap().pending_skill_cast = Some(PendingSkillCast {
            skill_id: skill_id.clone(),
            cast_target: None,
            start_seq,
        });

        core.withdraw_unit(caster_id, 10).unwrap();

        assert!(core
            .units
            .get(&caster_id)
            .unwrap()
            .pending_skill_cast
            .is_none());
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::SkillCastCancelled {
                caster_instance_id,
                interrupted_skill_id,
                interrupted_cast_seq,
                reason: SkillCastCancelReason::Withdrawn,
            } if entry.time_ms == 10
                && *caster_instance_id == caster_id
                && interrupted_skill_id == &skill_id
                && *interrupted_cast_seq == start_seq
        )));
    }

    #[test]
    fn withdraw_cancels_active_skill_cast_with_cancelled_event() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD31).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let skill_id = SkillId::from("active_withdraw_skill");
        let cast_seq = core.record_event_log(
            0,
            BattleLogEvent::AbilityCast {
                skill_id: skill_id.clone(),
                caster_instance_id: caster_id,
                target_instance_id: None,
            },
        );
        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
                skill_id: skill_id.clone(),
                caster_owner: Side::Player,
                anchor_position: Position::new(1, 1),
                cast_target_anchor_position: None,
                cast_target_anchor_world_position: None,
                allow_dead_caster: false,
                total_steps: 1,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: Default::default(),
                last_impact_context: None,
                active_area_ids: Vec::new(),
            },
        );

        core.withdraw_unit(caster_id, 10).unwrap();

        assert!(!core.active_skill_casts.contains_key(&cast_seq));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::SkillCastCancelled {
                caster_instance_id,
                interrupted_skill_id,
                interrupted_cast_seq,
                reason: SkillCastCancelReason::Withdrawn,
            } if entry.time_ms == 10
                && *caster_instance_id == caster_id
                && interrupted_skill_id == &skill_id
                && *interrupted_cast_seq == cast_seq
        )));
    }

    #[test]
    fn battle_runtime_policy_controls_max_battle_time() {
        let mut policy = RunPolicyData::builtin();
        policy.battle_runtime.movement_tick_ms = 5;
        policy.battle_runtime.max_battle_time_ms = 10;
        let game_data = GameDataBuilder::live_defaults()
            .with_run_policy(policy)
            .build_arc();
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: vec![crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("late_force_end"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtTimeMs(100),
                action: crate::game::battle::scenario::ScenarioAction::EndBattle {
                    winner: crate::game::battle::types::BattleWinner::Player,
                },
                once: true,
            }],
            win_condition: crate::game::battle::scenario::WinCondition::SurviveUntil {
                time_ms: 1_000,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, game_data, 123);

        let result = core
            .run_battle()
            .expect("battle should finish at runtime cap");

        assert_eq!(
            result.winner,
            crate::game::battle::types::BattleWinner::Draw
        );
        assert!(result.event_log.entries.iter().any(|entry| {
            entry.time_ms == 10 && matches!(entry.event, BattleLogEvent::BattleEnd { .. })
        }));
    }

    #[test]
    fn movement_tick_schedule_uses_run_policy_interval() {
        let mut policy = RunPolicyData::builtin();
        policy.battle_runtime.movement_tick_ms = 25;
        let game_data = GameDataBuilder::live_defaults()
            .with_run_policy(policy)
            .build_arc();
        let mut core = BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), game_data, 123);

        core.last_continuous_movement_tick_ms = Some(100);
        core.schedule_continuous_movement_tick(124);
        assert!(!core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::ContinuousMovementTick { time_ms } if *time_ms == 124
        )));

        core.schedule_continuous_movement_tick(125);
        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::ContinuousMovementTick { time_ms } if *time_ms == 125
        )));
    }

    #[test]
    fn same_timestamp_attack_resolve_is_drained_before_battle_end() {
        let mut core = new_core();
        let mut execution = core
            .start_battle_execution()
            .expect("battle execution should start");
        let attacker_id = UnitInstanceId::from(Uuid::from_u128(0xA55A));
        let target_id = UnitInstanceId::from(Uuid::from_u128(0xB55B));
        core.units
            .insert(attacker_id, runtime_unit(attacker_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        let cause = BattleEventCause::Root {
            kind: BattleEventRootCause::System,
        };

        core.event_queue.push(BattleEvent::EndBattle {
            time_ms: 10,
            winner: crate::game::battle::types::BattleWinner::Player,
            cause,
        });
        core.event_queue.push(BattleEvent::AttackResolve {
            time_ms: 10,
            attacker_instance_id: attacker_id,
            target_instance_id: target_id,
            source_snapshot: core
                .damage_source_snapshot_for_unit(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    1,
                    Default::default(),
                    1,
                    10,
                    true,
                )
                .expect("source snapshot"),
            kind: AttackKind::Auto,
            delivery: crate::game::battle::event_log::AttackDelivery::Instant,
            cause,
        });

        let finish = match core
            .step_battle_execution_until(&mut execution, 10)
            .expect("battle step should process same-time bucket")
        {
            crate::game::battle::core::sim::BattleStepOutcome::Finished(finish) => finish,
            crate::game::battle::core::sim::BattleStepOutcome::Running => {
                panic!("forced winner should finish after same-time bucket")
            }
        };
        let result = core
            .finalize_battle_execution(&mut execution, finish)
            .expect("finished battle should finalize");

        let attack_resolve_index = result
            .event_log
            .entries
            .iter()
            .position(|entry| {
                entry.time_ms == 10
                    && matches!(
                        entry.event,
                        BattleLogEvent::AttackResolve {
                            attacker_instance_id,
                            target_instance_id,
                            ..
                        } if attacker_instance_id == attacker_id && target_instance_id == target_id
                    )
            })
            .expect("same-time AttackResolve should be recorded");
        let battle_end_index = result
            .event_log
            .entries
            .iter()
            .position(|entry| {
                entry.time_ms == 10 && matches!(entry.event, BattleLogEvent::BattleEnd { .. })
            })
            .expect("BattleEnd should be recorded at the same timestamp");

        assert!(attack_resolve_index < battle_end_index);
    }

    #[test]
    fn live_manual_skill_command_starts_cast_and_reuses_skill_runtime() {
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: Vec::new(),
            win_condition: crate::game::battle::scenario::WinCondition::SurviveUntil {
                time_ms: 1_000,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let skill = SkillDef {
            id: SkillId::from("manual_guard"),
            name: "Manual Guard".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: explicit_cast_targeting(
                SkillTarget::SelfUnit,
                Default::default(),
                None,
            ),
            focus_time_ms: 50,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "step".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: Default::default(),
                effects: vec![],
                presentation: Default::default(),
            }],
        };
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill.clone()]))
            .build_arc();
        let mut core = BattleCore::new_from_scenario(scenario, game_data, 123);
        let mut execution = core
            .start_battle_execution()
            .expect("live battle execution starts before commands");
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(skill.id.clone());
        profile.skill_activation_mode = SkillActivationMode::Manual;
        profile.resonance.start = 100;
        profile.resonance.max = 100;
        let draft = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(20),
            source: BattleUnitSource::Employee(profile),
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            level: crate::game::enums::Tier::I,
            stat_scale: Default::default(),
            growth_stacks: Default::default(),
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
        };

        let deployed = core
            .apply_live_command(
                crate::game::battle::core::sim::BattleLiveCommand::DeployPlayerUnit {
                    draft,
                    position: Position::new(1, 1),
                    facing: crate::game::battle::tile_range::FacingDirection::Right,
                    instance_salt: 8,
                    time_ms: 250,
                    current_hp_policy: None,
                },
            )
            .expect("live deploy command should succeed");
        let unit_id = match deployed {
            crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id } => {
                unit_id
            }
            other => panic!("unexpected deploy outcome: {other:?}"),
        };

        let activated = core
            .apply_live_command(
                crate::game::battle::core::sim::BattleLiveCommand::ActivateSkill {
                    unit_id,
                    skill_id: skill.id.clone(),
                    target: None,
                    time_ms: 300,
                },
            )
            .expect("manual skill command should start cast");
        assert_eq!(
            activated,
            crate::game::battle::core::sim::BattleLiveCommandOutcome::SkillActivated { unit_id }
        );
        assert!(core.event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::ManualCastStart {
                    caster_instance_id,
                    skill_id,
                    ..
                } if *caster_instance_id == unit_id && skill_id.as_str() == "manual_guard"
            )
        }));

        core.step_battle_execution_until(&mut execution, 351)
            .expect("manual cast end should be processed");
        assert!(core.event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AbilityCast {
                    caster_instance_id,
                    skill_id,
                    ..
                } if *caster_instance_id == unit_id && skill_id.as_str() == "manual_guard"
            )
        }));
        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 0);
    }

    #[test]
    fn step_battle_execution_by_does_not_jump_to_future_wave_time() {
        let player_group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let enemy_group_id = crate::game::battle::scenario::ScenarioGroupId::new("enemy_wave");
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: player_group_id.clone(),
                    side: Side::Player,
                    required_for_victory: false,
                    enemy_movement_plan: None,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new("player_0"),
                        side: Side::Player,
                        draft: fixture_draft(Uuid::from_u128(10), Uuid::from_u128(11), 100, 1),
                        position: Position::new(0, 0),
                        instance_salt: 0,
                    }],
                },
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: enemy_group_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: true,
                    enemy_movement_plan: None,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new("enemy_0"),
                        side: Side::Opponent,
                        draft: fixture_draft(Uuid::from_u128(1), Uuid::from_u128(2), 10, 1),
                        position: Position::new(1, 1),
                        instance_salt: 0,
                    }],
                },
            ],
            events: vec![
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("player_start"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: player_group_id,
                    },
                    once: true,
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("delayed_wave"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtTimeMs(1_000),
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: enemy_group_id,
                    },
                    once: true,
                },
            ],
            win_condition: crate::game::battle::scenario::WinCondition::SurviveUntil {
                time_ms: 2_000,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let mut execution = core
            .start_battle_execution()
            .expect("battle execution should start");

        let outcome = core
            .step_battle_execution_by(&mut execution, 100)
            .expect("wall-clock step should run");
        assert!(matches!(
            outcome,
            crate::game::battle::core::sim::BattleStepOutcome::Running
        ));
        assert_eq!(execution.last_event_time_ms(), 100);
        assert!(core.event_log.entries.iter().all(|entry| {
            !matches!(entry.event, BattleLogEvent::UnitSpawned { .. }) || entry.time_ms != 1_000
        }));

        let outcome = core
            .step_battle_execution_until(&mut execution, 1_000)
            .expect("step until delayed wave should run");
        assert!(matches!(
            outcome,
            crate::game::battle::core::sim::BattleStepOutcome::Running
        ));
        assert_eq!(execution.last_event_time_ms(), 1_000);
        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 1_000 && matches!(entry.event, BattleLogEvent::UnitSpawned { .. })
        }));
    }

    fn runtime_unit(unit_id: UnitInstanceId, owner: Side) -> RuntimeUnit {
        let basic_attack = crate::game::data::abnormality_data::BasicAttackDef {
            defense_tile_range: Some(TileRangePattern {
                include_anchor_tile: false,
                rows: vec![
                    "XXXXX".to_string(),
                    "XXXXX".to_string(),
                    "XX@XX".to_string(),
                    "XXXXX".to_string(),
                    "XXXXX".to_string(),
                ],
            }),
            ..Default::default()
        };
        RuntimeUnit {
            instance_id: unit_id,
            lifecycle: RuntimeUnitLifecycle::Active,
            spawn_order: u64::from(unit_id.as_bytes()[15]),
            source_owned_uuid: unit_id.as_uuid(),
            owner,
            role: crate::game::battle::types::BattleUnitRole::Combatant,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            base_uuid: Uuid::nil(),
            source_identity: crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                base_uuid: Uuid::nil(),
            },
            stats: UnitStats::with_values(10, 10, 1, 0, 1),
            incoming_damage_modifiers: Default::default(),
            basic_attack,
            skill_id: None,
            skill_activation_mode: SkillActivationMode::Auto,
            body: Default::default(),
            tactical_anchor: None,
            enemy_movement_plan: None,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: true,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
            facing_direction: Some(FacingDirection::Right),
            move_epoch: 0,
            action_state: ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
            ranged_reposition_until_ms: 0,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    fn fixture_draft(owned_uuid: Uuid, base_uuid: Uuid, hp: u32, attack: u32) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.stats = UnitStats::with_values(hp, hp, attack, 0, 100);
        profile.basic_attack.interval_ms = 100;
        profile.basic_attack.windup_ms = 1;
        profile.basic_attack.range_units = 10.0;
        BattleUnitDraft {
            owned_uuid,
            source: BattleUnitSource::TestFixture { base_uuid, profile },
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            level: crate::game::enums::Tier::I,
            stat_scale: Default::default(),
            growth_stacks: crate::game::growth::GrowthStack::new(),
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
        }
    }

    #[test]
    fn spawned_unit_uses_authored_body_radius() {
        let group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let unit_ref = crate::game::battle::scenario::ScenarioUnitRef::new("wide_unit");
        let owned_uuid = Uuid::from_u128(0xA0A0);
        let base_uuid = Uuid::from_u128(0xB0B0);
        let mut draft = fixture_draft(owned_uuid, base_uuid, 100, 10);
        if let BattleUnitSource::TestFixture { profile, .. } = &mut draft.source {
            profile.movement.radius_units = 800_000;
        }

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: group_id.clone(),
                side: Side::Player,
                required_for_victory: true,
                enemy_movement_plan: None,
                spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                    unit_ref,
                    side: Side::Player,
                    draft,
                    position: Position::new(1, 1),
                    instance_salt: 0,
                }],
            }],
            events: Vec::new(),
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: Default::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);

        let spawned = core
            .spawn_scenario_group(&group_id)
            .expect("spawn group with custom radius");
        let body = core.unit_body_view(spawned[0]).expect("spawned body");

        assert!((body.radius - 0.8).abs() <= f32::EPSILON);
    }

    #[test]
    fn opponent_spawn_uses_authored_position_even_when_occupied() {
        let group_id = crate::game::battle::scenario::ScenarioGroupId::new("enemy");
        let position = Position::new(2, 2);
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: group_id.clone(),
                side: Side::Opponent,
                required_for_victory: true,
                enemy_movement_plan: None,
                spawns: vec![
                    crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new("enemy_a"),
                        side: Side::Opponent,
                        draft: fixture_draft(
                            Uuid::from_u128(0xA100),
                            Uuid::from_u128(0xB100),
                            100,
                            10,
                        ),
                        position,
                        instance_salt: 0,
                    },
                    crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new("enemy_b"),
                        side: Side::Opponent,
                        draft: fixture_draft(
                            Uuid::from_u128(0xA101),
                            Uuid::from_u128(0xB101),
                            100,
                            10,
                        ),
                        position,
                        instance_salt: 1,
                    },
                ],
            }],
            events: Vec::new(),
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: Default::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);

        let spawned = core
            .spawn_scenario_group(&group_id)
            .expect("opponent overlap should use authored spawn positions");

        assert_eq!(spawned.len(), 2);
        assert_eq!(core.unit_projected_tile(spawned[0]), Some(position));
        assert_eq!(core.unit_projected_tile(spawned[1]), Some(position));
    }

    #[test]
    fn platform_only_spawned_unit_cannot_block_even_if_profile_has_capacity() {
        let group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let unit_ref = crate::game::battle::scenario::ScenarioUnitRef::new("platform_unit");
        let owned_uuid = Uuid::from_u128(0xA0A1);
        let base_uuid = Uuid::from_u128(0xB0B1);
        let mut draft = fixture_draft(owned_uuid, base_uuid, 100, 10);
        if let BattleUnitSource::TestFixture { profile, .. } = &mut draft.source {
            profile.deployment_affinity = DeploymentAffinity::PlatformOnly;
            profile.block_capacity = 3;
            profile.block_radius_units = 2.0;
        }

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: group_id.clone(),
                side: Side::Player,
                required_for_victory: true,
                enemy_movement_plan: None,
                spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                    unit_ref,
                    side: Side::Player,
                    draft,
                    position: Position::new(1, 1),
                    instance_salt: 0,
                }],
            }],
            events: Vec::new(),
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: Default::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);

        let spawned = core
            .spawn_scenario_group(&group_id)
            .expect("spawn group with platform-only profile");
        let unit = core.units.get(&spawned[0]).expect("spawned unit");

        assert_eq!(unit.block_capacity, 0);
        assert_eq!(unit.block_radius_units, 0.0);
    }

    fn sync_unit_to_battlefield_tile_center(core: &mut BattleCore, unit_id: UnitInstanceId) {
        let Some(position) = core.unit_projected_tile(unit_id) else {
            return;
        };
        if let Some(unit) = core.units.get_mut(&unit_id) {
            unit.set_world_position(WorldVec2::from_tile_center(position));
        }
    }

    fn runtime_unit_with_base(
        unit_id: UnitInstanceId,
        owner: Side,
        base_uuid: Uuid,
    ) -> RuntimeUnit {
        let mut unit = runtime_unit(unit_id, owner);
        unit.base_uuid = base_uuid;
        unit.source_identity =
            crate::game::battle::types::BattleUnitSourceIdentity::TestFixture { base_uuid };
        unit
    }

    fn place_unit(core: &mut BattleCore, unit_id: UnitInstanceId, pos: Position) {
        core.battlefield.ensure_walkable_tile(pos).unwrap();
        let unit = core.units.get_mut(&unit_id).unwrap();
        unit.set_world_position(WorldVec2::new(pos.x as f32, pos.y as f32));
    }

    fn explicit_cast_targeting(
        target: SkillTarget,
        range_policy: TileRangePolicy,
        defense_tile_range: Option<TileRangePattern>,
    ) -> SkillCastTargetingDef {
        SkillCastTargetingDef::explicit(target, range_policy, defense_tile_range, false)
    }

    fn abnormality_with_basic_attack(
        base_uuid: Uuid,
        delivery: DeliveryDef,
        range_units: u8,
    ) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: format!("abnormality-{base_uuid}"),
            uuid: base_uuid,
            name: "test".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 1,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: crate::game::data::abnormality_data::BasicAttackDef {
                range_units: f32::from(range_units),
                delivery,
                ..Default::default()
            },
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        }
    }

    fn core_with_abnormalities(abnormalities: Vec<AbnormalityMetadata>) -> BattleCore {
        core_with_skill_data(abnormalities, vec![])
    }

    fn single_step_skill(
        id: &str,
        target: SkillTarget,
        range_units: u8,
        focus_time_ms: u32,
        delivery: DeliveryDef,
        effects: Vec<SkillEffectDef>,
    ) -> SkillDef {
        let radius = range_units as usize;
        let width = radius.saturating_mul(2).saturating_add(1);
        let anchor_row = radius;
        let anchor_col = radius;
        let defense_tile_rows = (0..width)
            .map(|row| {
                (0..width)
                    .map(|col| {
                        if row == anchor_row && col == anchor_col {
                            '@'
                        } else {
                            'X'
                        }
                    })
                    .collect::<String>()
            })
            .collect();
        let include_anchor_tile = radius == 0;
        let defense_tile_range = TileRangePattern {
            include_anchor_tile,
            rows: defense_tile_rows,
        };
        SkillDef {
            id: SkillId::from(id),
            name: id.to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: explicit_cast_targeting(
                target.clone(),
                Default::default(),
                Some(defense_tile_range.clone()),
            ),
            focus_time_ms,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "step_01".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: Some(defense_tile_range),
                air_capable: false,
                target,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery,
                effects,
                presentation: SkillPresentationDef::default(),
            }],
        }
    }

    fn damage_effect() -> SkillEffectDef {
        SkillEffectDef::Damage {
            amount: 1,
            damage_type: DamageType::Magic,
        }
    }

    fn zero_damage_effect() -> SkillEffectDef {
        SkillEffectDef::Damage {
            amount: 0,
            damage_type: DamageType::Magic,
        }
    }

    fn enemy_tile_area_skill(id: &str, effects: Vec<SkillEffectDef>) -> SkillDef {
        single_step_skill(
            id,
            SkillTarget::SelfUnit,
            1,
            0,
            DeliveryDef::TileArea {
                area: SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::Caster,
                    tile_origin: SkillTileAreaOrigin::Caster,
                    tracking: SkillAreaTracking::GroundFixed,
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 0,
                    tick_interval_ms: None,
                },
            },
            effects,
        )
    }

    fn core_with_skill_data(
        abnormalities: Vec<AbnormalityMetadata>,
        skills: Vec<SkillDef>,
    ) -> BattleCore {
        let game_data = GameDataBuilder::live_defaults()
            .with_abnormalities(abnormalities)
            .with_skills(SkillDatabase::new(skills))
            .build_arc();

        BattleCore::new_from_scenario(BattleScenario::empty((6, 6)), game_data, 123)
    }

    #[test]
    fn add_resonance_clamps_and_sets_pending_cast_only_when_allowed() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(1).into();
        let mut unit = runtime_unit(unit_id, Side::Player);
        unit.resonance_current = 90;
        unit.resonance_max = 100;
        core.units.insert(unit_id, unit);

        core.add_resonance(unit_id, 10, 0, false);
        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 100);
        assert!(!core.units.get(&unit_id).unwrap().pending_cast);

        core.units.get_mut(&unit_id).unwrap().resonance_current = 90;
        core.add_resonance(unit_id, 10, 0, true);
        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 100);
        assert!(core.units.get(&unit_id).unwrap().pending_cast);

        // Locked resonance gain should prevent any change.
        core.units
            .get_mut(&unit_id)
            .unwrap()
            .action_locks
            .lock_resonance_gain_until(10);
        core.units.get_mut(&unit_id).unwrap().resonance_current = 0;
        core.add_resonance(unit_id, 10, 0, true);
        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 0);
    }

    #[test]
    fn manual_activation_mode_does_not_schedule_autocast_when_resonance_fills() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(41).into();
        let mut unit = runtime_unit(unit_id, Side::Player);
        unit.skill_activation_mode = SkillActivationMode::Manual;
        unit.skill_id = Some(SkillId::from("manual_only"));
        unit.resonance_current = 90;
        unit.resonance_max = 100;
        core.units.insert(unit_id, unit);

        core.add_resonance(unit_id, 10, 0, true);
        core.schedule_pending_autocasts(0);

        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 100);
        assert!(!core.units.get(&unit_id).unwrap().pending_cast);
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn schedule_pending_autocasts_is_deterministic_and_consumes_pending_cause() {
        let mut core = new_core();
        let caster_small: UnitInstanceId = Uuid::from_u128(1).into();
        let caster_large: UnitInstanceId = Uuid::from_u128(2).into();
        assert!(caster_small.as_bytes() < caster_large.as_bytes());

        let mut u1 = runtime_unit(caster_small, Side::Player);
        u1.pending_cast = true;
        let mut u2 = runtime_unit(caster_large, Side::Player);
        u2.pending_cast = true;
        core.units.insert(caster_small, u1);
        core.units.insert(caster_large, u2);

        let cause = BattleEventCause::Parent { seq: 42 };
        core.recording_cause_stack.push(cause);

        core.schedule_pending_autocasts(100);

        // Pending flags are consumed and causes are taken.
        assert!(!core.units.get(&caster_small).unwrap().pending_cast);
        assert!(core
            .units
            .get(&caster_small)
            .unwrap()
            .pending_cast_cause
            .is_none());
        assert!(!core.units.get(&caster_large).unwrap().pending_cast);
        assert!(core
            .units
            .get(&caster_large)
            .unwrap()
            .pending_cast_cause
            .is_none());

        let first = core.event_queue.pop().unwrap();
        let second = core.event_queue.pop().unwrap();

        assert!(matches!(
            first,
            BattleEvent::AutoCastStart {
                time_ms: 100,
                caster_instance_id,
                cause: BattleEventCause::Parent { seq: 42 },
            } if caster_instance_id == caster_small
        ));
        assert!(matches!(
            second,
            BattleEvent::AutoCastStart {
                time_ms: 100,
                caster_instance_id,
                cause: BattleEventCause::Parent { seq: 42 },
            } if caster_instance_id == caster_large
        ));
    }

    #[test]
    fn schedule_pending_autocast_respects_next_action_time_and_dead_units() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(1).into();
        let mut unit = runtime_unit(unit_id, Side::Player);
        unit.pending_cast = true;
        unit.next_action_time = 200;
        core.units.insert(unit_id, unit);

        core.schedule_pending_autocasts(100);
        assert!(core.event_queue.is_empty());
        assert!(core.units.get(&unit_id).unwrap().pending_cast);

        // Dead unit never schedules.
        core.units.get_mut(&unit_id).unwrap().next_action_time = 0;
        core.units.get_mut(&unit_id).unwrap().stats.current_health = 0;
        core.units.get_mut(&unit_id).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;
        core.schedule_pending_autocasts(300);
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn pending_basic_attack_retargets_when_persisted_target_is_out_of_range() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(1).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(2).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(3).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.pending_basic_attack = true;
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(3, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 0));

        core.try_start_pending_basic_attacks(0);

        assert!(!core.units.get(&attacker_id).unwrap().pending_basic_attack);

        let event = core.event_queue.pop().expect("expected attack start");
        assert!(matches!(
            event,
            BattleEvent::AttackStart {
                attacker_instance_id,
                target_instance_id: Some(target_instance_id),
                schedule_next: true,
                ..
            } if attacker_instance_id == attacker_id && target_instance_id == nearer_enemy_id
        ));
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn pending_basic_attack_retargets_when_persisted_target_is_dead() {
        let attacker_base_uuid = Uuid::from_u128(0xAA03);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            2,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(4).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(5).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(6).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.pending_basic_attack = true;
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        core.units
            .get_mut(&locked_target_id)
            .unwrap()
            .stats
            .current_health = 0;
        core.units.get_mut(&locked_target_id).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 1));

        core.try_start_pending_basic_attacks(0);

        assert!(!core.units.get(&attacker_id).unwrap().pending_basic_attack);

        let event = core.event_queue.pop().expect("expected attack start");
        assert!(matches!(
            event,
            BattleEvent::AttackStart {
                attacker_instance_id,
                target_instance_id: Some(target_instance_id),
                schedule_next: true,
                ..
            } if attacker_instance_id == attacker_id && target_instance_id == nearer_enemy_id
        ));
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn attack_start_keeps_persisted_target_when_it_is_still_in_range() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(31).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(32).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(33).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(locked_target_id)
        );
    }

    #[test]
    fn attack_start_player_weapon_profile_overrides_hinted_target_when_it_can_select() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(34).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(35).into();
        let hinted_target_id: UnitInstanceId = Uuid::from_u128(36).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            hinted_target_id,
            runtime_unit(hinted_target_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, hinted_target_id, Position::new(0, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: Some(hinted_target_id),
                schedule_next: false,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(locked_target_id)
        );
    }

    #[test]
    fn attack_start_invalid_hint_falls_back_to_persisted_target() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(0x3401).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(0x3501).into();
        let hinted_target_id: UnitInstanceId = Uuid::from_u128(0x3601).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            hinted_target_id,
            runtime_unit(hinted_target_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, hinted_target_id, Position::new(3, 3));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: Some(hinted_target_id),
                schedule_next: false,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(locked_target_id)
        );
    }

    #[test]
    fn attack_start_retargets_to_nearest_in_range_when_persisted_target_is_dead() {
        let attacker_base_uuid = Uuid::from_u128(0xAA04);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            2,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(37).into();
        let dead_target_id: UnitInstanceId = Uuid::from_u128(38).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(39).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.current_target = Some(dead_target_id);

        core.units.insert(attacker_id, attacker);
        core.units
            .insert(dead_target_id, runtime_unit(dead_target_id, Side::Opponent));
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        core.units
            .get_mut(&dead_target_id)
            .unwrap()
            .stats
            .current_health = 0;
        core.units.get_mut(&dead_target_id).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, dead_target_id, Position::new(1, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(nearer_enemy_id)
        );
    }

    #[test]
    fn attack_start_fixed_defense_player_retargets_by_uuid_order() {
        let attacker_base_uuid = Uuid::from_u128(0xAA13);
        let enemy_base_uuid = Uuid::from_u128(0xAA14);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(enemy_base_uuid, DeliveryDef::Instant, 1),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(40).into();
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(41).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(42).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            diagonal_enemy_id,
            runtime_unit_with_base(diagonal_enemy_id, Side::Opponent, enemy_base_uuid),
        );
        core.units.insert(
            straight_enemy_id,
            runtime_unit_with_base(straight_enemy_id, Side::Opponent, enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(2, 2));
        place_unit(&mut core, diagonal_enemy_id, Position::new(1, 1));
        place_unit(&mut core, straight_enemy_id, Position::new(2, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(diagonal_enemy_id)
        );
    }

    #[test]
    fn choose_attack_target_in_range_prefers_melee_enemy_when_distance_is_equal() {
        let attacker_base_uuid = Uuid::from_u128(0xAA05);
        let melee_enemy_base_uuid = Uuid::from_u128(0xAA06);
        let ranged_enemy_base_uuid = Uuid::from_u128(0xAA07);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(
                attacker_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
                    hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                    allow_targetless_cast: false,
                    max_range_tiles: None,
                    max_lifetime_ms: None,
                    max_kills: None,
                    max_pierces: None,
                    collision: Default::default(),
                },
                3,
            ),
            abnormality_with_basic_attack(melee_enemy_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(
                ranged_enemy_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
                    hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                    allow_targetless_cast: false,
                    max_range_tiles: None,
                    max_lifetime_ms: None,
                    max_kills: None,
                    max_pierces: None,
                    collision: Default::default(),
                },
                3,
            ),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(80).into();
        let melee_enemy_id: UnitInstanceId = Uuid::from_u128(81).into();
        let ranged_enemy_id: UnitInstanceId = Uuid::from_u128(82).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Opponent, attacker_base_uuid),
        );
        core.units.insert(
            melee_enemy_id,
            runtime_unit_with_base(melee_enemy_id, Side::Player, melee_enemy_base_uuid),
        );
        core.units.insert(
            ranged_enemy_id,
            runtime_unit_with_base(ranged_enemy_id, Side::Player, ranged_enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, melee_enemy_id, Position::new(1, 0));
        place_unit(&mut core, ranged_enemy_id, Position::new(0, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(melee_enemy_id)
        );
    }

    #[test]
    fn choose_attack_target_in_range_fixed_defense_player_uses_uuid_order() {
        let attacker_base_uuid = Uuid::from_u128(0xAA10);
        let enemy_base_uuid = Uuid::from_u128(0xAA11);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(enemy_base_uuid, DeliveryDef::Instant, 1),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(91).into();
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(92).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(93).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            diagonal_enemy_id,
            runtime_unit_with_base(diagonal_enemy_id, Side::Opponent, enemy_base_uuid),
        );
        core.units.insert(
            straight_enemy_id,
            runtime_unit_with_base(straight_enemy_id, Side::Opponent, enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(2, 2));
        place_unit(&mut core, diagonal_enemy_id, Position::new(1, 1));
        place_unit(&mut core, straight_enemy_id, Position::new(2, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(diagonal_enemy_id)
        );
    }

    #[test]
    fn fixed_defense_player_basic_attack_uses_facing_tile_pattern_not_continuous_radius() {
        let attacker_base_uuid = Uuid::from_u128(0xAA03);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            99,
        )]);
        core.scenario.tactical_plan.player_plan =
            crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;

        let attacker_id: UnitInstanceId = Uuid::from_u128(81).into();
        let forward_target_id: UnitInstanceId = Uuid::from_u128(82).into();
        let side_target_id: UnitInstanceId = Uuid::from_u128(83).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.facing_direction = Some(FacingDirection::Up);
        attacker.basic_attack.range_units = 99.0;
        attacker.basic_attack.defense_tile_range = Some(TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
        });
        core.units.insert(attacker_id, attacker);
        core.units.insert(
            forward_target_id,
            runtime_unit(forward_target_id, Side::Opponent),
        );
        core.units
            .insert(side_target_id, runtime_unit(side_target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(1, 1));
        place_unit(&mut core, forward_target_id, Position::new(1, 0));
        place_unit(&mut core, side_target_id, Position::new(2, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(forward_target_id)
        );
        assert!(core.resolve_basic_attack(attacker_id, forward_target_id, 0));
        assert!(!core.resolve_basic_attack(attacker_id, side_target_id, 0));
    }

    #[test]
    fn basic_attack_uses_tile_range_not_range_units() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xAA30).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xAA31).into();
        let mut core = new_core();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.facing_direction = Some(FacingDirection::Right);
        attacker.basic_attack.range_units = 99.0;
        attacker.basic_attack.defense_tile_range = Some(TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
        });
        core.units.insert(attacker_id, attacker);
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(1, 0));

        assert_eq!(core.choose_attack_target_in_range(attacker_id), None);
        assert!(!core.resolve_basic_attack(attacker_id, target_id, 0));
    }

    #[test]
    fn fixed_defense_player_basic_attack_selects_lowest_uuid_target_in_tile_range() {
        let attacker_base_uuid = Uuid::from_u128(0xAA15);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            99,
        )]);
        core.scenario.tactical_plan.player_plan =
            crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;

        let attacker_id: UnitInstanceId = Uuid::from_u128(0xAA16).into();
        let lower_uuid_far_target_id: UnitInstanceId = Uuid::from_u128(0xAA17).into();
        let higher_uuid_near_target_id: UnitInstanceId = Uuid::from_u128(0xAA18).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.facing_direction = Some(FacingDirection::Up);
        attacker.basic_attack.range_units = 99.0;
        attacker.basic_attack.defense_tile_range = Some(TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".X.".to_string(), ".X.".to_string(), ".@.".to_string()],
        });
        core.units.insert(attacker_id, attacker);
        core.units.insert(
            lower_uuid_far_target_id,
            runtime_unit(lower_uuid_far_target_id, Side::Opponent),
        );
        core.units.insert(
            higher_uuid_near_target_id,
            runtime_unit(higher_uuid_near_target_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(1, 2));
        place_unit(&mut core, lower_uuid_far_target_id, Position::new(1, 0));
        place_unit(&mut core, higher_uuid_near_target_id, Position::new(1, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(lower_uuid_far_target_id)
        );
    }

    #[test]
    fn fixed_defense_player_basic_attack_uses_default_pattern_but_requires_facing() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(84).into();
        let target_id: UnitInstanceId = Uuid::from_u128(85).into();
        let mut core = new_core();
        core.scenario.tactical_plan.player_plan =
            crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.basic_attack.range_units = 99.0;
        attacker.basic_attack.defense_tile_range = None;
        attacker.facing_direction = Some(FacingDirection::Right);
        core.units.insert(attacker_id, attacker);
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, attacker_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(target_id)
        );

        core.units
            .get_mut(&attacker_id)
            .unwrap()
            .basic_attack
            .defense_tile_range = Some(TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
        });
        core.units.get_mut(&attacker_id).unwrap().facing_direction = None;

        assert_eq!(core.choose_attack_target_in_range(attacker_id), None);
    }

    #[test]
    fn fixed_defense_player_skill_target_uses_facing_tile_pattern() {
        let tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
        };
        let skill = SkillDef {
            id: SkillId::from("tile_skill"),
            name: "tile_skill".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: explicit_cast_targeting(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                Default::default(),
                Some(tile_range.clone()),
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: Some(tile_range),
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![damage_effect()],
                presentation: Default::default(),
            }],
        };
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        core.scenario.tactical_plan.player_plan =
            crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;

        let caster_id: UnitInstanceId = Uuid::from_u128(86).into();
        let right_target_id: UnitInstanceId = Uuid::from_u128(87).into();
        let up_target_id: UnitInstanceId = Uuid::from_u128(88).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.facing_direction = Some(FacingDirection::Right);
        core.units.insert(caster_id, caster);
        core.units.insert(
            right_target_id,
            runtime_unit(right_target_id, Side::Opponent),
        );
        core.units
            .insert(up_target_id, runtime_unit(up_target_id, Side::Opponent));

        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, right_target_id, Position::new(2, 1));
        place_unit(&mut core, up_target_id, Position::new(1, 0));

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(1, 1)),
            Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: right_target_id
            })
        );

        let mut missing_pattern_skill = skill.clone();
        missing_pattern_skill.cast_targeting = explicit_cast_targeting(
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            Default::default(),
            None,
        );
        missing_pattern_skill.steps[0].defense_tile_range = None;
        assert_eq!(
            core.resolve_skill_cast_target(
                &missing_pattern_skill,
                caster_id,
                Side::Player,
                Position::new(1, 1)
            ),
            None
        );
    }

    #[test]
    fn skill_enemy_single_uses_tile_range_not_range_units() {
        let tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
        };
        let skill = SkillDef {
            id: SkillId::from("tile_skill_without_continuous_fallback"),
            name: "tile_skill_without_continuous_fallback".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: explicit_cast_targeting(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                Default::default(),
                Some(tile_range.clone()),
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: Some(tile_range),
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![damage_effect()],
                presentation: Default::default(),
            }],
        };
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);

        let caster_id: UnitInstanceId = Uuid::from_u128(0xAA40).into();
        let outside_tile_target_id: UnitInstanceId = Uuid::from_u128(0xAA41).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.facing_direction = Some(FacingDirection::Right);
        core.units.insert(caster_id, caster);
        core.units.insert(
            outside_tile_target_id,
            runtime_unit(outside_tile_target_id, Side::Opponent),
        );

        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, outside_tile_target_id, Position::new(1, 0));

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(1, 1)),
            None
        );
    }

    #[test]
    fn skill_enemy_single_whole_field_uses_valid_tiles_without_facing() {
        let skill = SkillDef {
            id: SkillId::from("whole_field_target"),
            name: "whole_field_target".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: explicit_cast_targeting(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                TileRangePolicy::WholeFieldValidTiles,
                None,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                range_policy: TileRangePolicy::WholeFieldValidTiles,
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![damage_effect()],
                presentation: Default::default(),
            }],
        };
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);

        let caster_id: UnitInstanceId = Uuid::from_u128(0xAA50).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xAA51).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.facing_direction = None;
        core.units.insert(caster_id, caster);
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(4, 2));

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(1, 1)),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id
            })
        );
    }

    #[test]
    fn hard_cc_clears_locked_target_and_reacquires_after_release() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(21).into();
        let old_target_id: UnitInstanceId = Uuid::from_u128(22).into();
        let new_target_id: UnitInstanceId = Uuid::from_u128(23).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(old_target_id);

        core.units.insert(attacker_id, attacker);
        core.units
            .insert(old_target_id, runtime_unit(old_target_id, Side::Opponent));
        core.units
            .insert(new_target_id, runtime_unit(new_target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, old_target_id, Position::new(3, 0));
        place_unit(&mut core, new_target_id, Position::new(1, 0));

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: new_target_id,
                target_instance_id: attacker_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 50,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(core.units.get(&attacker_id).unwrap().current_target, None);

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 51,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: BattleEventCause::default(),
            },
            51,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(new_target_id)
        );
    }

    #[test]
    fn hard_cc_replacement_records_explicit_buff_expired_reason() {
        let mut core = new_core();
        let first_caster_id: UnitInstanceId = Uuid::from_u128(0xCC01).into();
        let second_caster_id: UnitInstanceId = Uuid::from_u128(0xCC02).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC03).into();
        core.units.insert(
            first_caster_id,
            runtime_unit(first_caster_id, Side::Opponent),
        );
        core.units.insert(
            second_caster_id,
            runtime_unit(second_caster_id, Side::Opponent),
        );
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Player));

        let stun_id = BuffId::from_name("stun");
        let freeze_id = BuffId::from_name("freeze");
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: first_caster_id,
                target_instance_id: target_id,
                buff_id: stun_id,
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 10,
                caster_instance_id: second_caster_id,
                target_instance_id: target_id,
                buff_id: freeze_id,
                duration_ms: 20,
                cause: BattleEventCause::default(),
            },
            10,
        )
        .unwrap();

        let hard_cc_events = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::BuffApplied { buff_id, .. }
                    if buff_id == stun_id || buff_id == freeze_id =>
                {
                    Some((entry.time_ms, "applied", buff_id, None))
                }
                BattleLogEvent::BuffExpired {
                    buff_id, reason, ..
                } if buff_id == stun_id || buff_id == freeze_id => {
                    Some((entry.time_ms, "expired", buff_id, Some(reason)))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            hard_cc_events,
            vec![
                (0, "applied", stun_id, None),
                (10, "expired", stun_id, Some(BuffExpireReason::Replaced)),
                (10, "applied", freeze_id, None),
            ]
        );
        assert!(!core.has_buff_kind(target_id, crate::game::battle::buffs::BuffKind::Stun));
        assert!(core.has_buff_kind(target_id, crate::game::battle::buffs::BuffKind::Freeze));
        assert_eq!(
            core.units.get(&target_id).unwrap().action_locks,
            Default::default()
        );
    }

    #[test]
    fn unit_death_clears_target_and_caster_active_buffs_with_reasons() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD01).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD02).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        let poison_id = BuffId::from_name("poison");

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: caster_id,
                target_instance_id: target_id,
                buff_id: poison_id,
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: target_id,
                target_instance_id: caster_id,
                buff_id: poison_id,
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        core.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id,
                source_snapshot: core
                    .damage_source_snapshot_for_unit(
                        caster_id,
                        target_id,
                        DamageSource::Ability,
                        DamageType::True,
                        99,
                        Default::default(),
                        0,
                        10,
                        false,
                    )
                    .expect("source snapshot"),
            }],
            10,
        );

        assert!(core.buffs.is_empty());
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
                reason: BuffExpireReason::TargetDied,
            } if entry.time_ms == 10
                && caster_instance_id == caster_id
                && target_instance_id == target_id
                && buff_id == poison_id
        )));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
                reason: BuffExpireReason::CasterDied,
            } if entry.time_ms == 10
                && caster_instance_id == target_id
                && target_instance_id == caster_id
                && buff_id == poison_id
        )));
    }

    #[test]
    fn move_speed_modifier_updates_runtime_body_speed_for_next_movement_tick() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(0xCE01).into();
        let mut unit = runtime_unit(unit_id, Side::Player);
        unit.stats.move_speed_units_per_ms = 1;
        unit.body.move_speed =
            1_000.0 / crate::game::battle::core::movement::types::DATA_UNITS_PER_WORLD;
        core.units.insert(unit_id, unit);

        core.process_commands(
            vec![BattleCommand::ApplyModifier {
                target_id: unit_id,
                modifier: StatModifier {
                    stat: StatId::MoveSpeedUnitsPerMs,
                    kind: StatModifierKind::Flat,
                    value: 2,
                },
            }],
            0,
        );

        let unit = core.units.get(&unit_id).unwrap();
        assert_eq!(unit.stats.move_speed_units_per_ms, 3);
        assert_eq!(
            unit.body.move_speed,
            3_000.0 / crate::game::battle::core::movement::types::DATA_UNITS_PER_WORLD
        );
    }

    #[test]
    fn current_target_skill_rule_prefers_locked_target() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(41).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(42).into();
        let other_target_id: UnitInstanceId = Uuid::from_u128(43).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            other_target_id,
            runtime_unit(other_target_id, Side::Opponent),
        );

        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, other_target_id, Position::new(1, 1));

        let skill = single_step_skill(
            "current_target",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::event_log::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == locked_target_id
        ));
    }

    #[test]
    fn nearest_skill_rule_prefers_current_attack_target_when_in_range() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(44).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(45).into();
        let nearer_target_id: UnitInstanceId = Uuid::from_u128(46).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_target_id,
            runtime_unit(nearer_target_id, Side::Opponent),
        );

        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(2, 0));
        place_unit(&mut core, nearer_target_id, Position::new(1, 0));

        let skill = single_step_skill(
            "nearest_prefers_current_target",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::event_log::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == locked_target_id
        ));
    }

    #[test]
    fn nearest_skill_rule_falls_back_when_current_attack_target_is_out_of_range() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(47).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(48).into();
        let nearer_target_id: UnitInstanceId = Uuid::from_u128(49).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_target_id,
            runtime_unit(nearer_target_id, Side::Opponent),
        );

        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(3, 0));
        place_unit(&mut core, nearer_target_id, Position::new(1, 0));

        let skill = single_step_skill(
            "nearest_falls_back_from_out_of_range_current_target",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            1,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::event_log::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == nearer_target_id
        ));
    }

    #[test]
    fn lowest_health_enemy_skill_rule_prefers_weaker_target() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(51).into();
        let tank_id: UnitInstanceId = Uuid::from_u128(52).into();
        let weak_id: UnitInstanceId = Uuid::from_u128(53).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut tank = runtime_unit(tank_id, Side::Opponent);
        tank.stats.current_health = 20;
        let mut weak = runtime_unit(weak_id, Side::Opponent);
        weak.stats.current_health = 5;
        core.units.insert(tank_id, tank);
        core.units.insert(weak_id, weak);

        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, tank_id, Position::new(1, 0));
        place_unit(&mut core, weak_id, Position::new(1, 1));

        let skill = single_step_skill(
            "lowest_health",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::event_log::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == weak_id
        ));
    }

    #[test]
    fn hostile_skill_targeting_excludes_zero_damage_without_hostile_effect() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5101).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x5102).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 0));

        let skill = single_step_skill(
            "zero_damage_no_effect",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![zero_damage_effect()],
        );

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0)),
            None
        );
    }

    #[test]
    fn hostile_skill_targeting_allows_zero_damage_hostile_buff() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5111).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x5112).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 0));

        let skill = single_step_skill(
            "zero_damage_stun",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![
                zero_damage_effect(),
                SkillEffectDef::ApplyBuff {
                    buff_id: "stun".to_string(),
                    duration_ms: 500,
                },
            ],
        );

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0)),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id
            })
        );
    }

    #[test]
    fn hostile_skill_targeting_excludes_enemy_heal_without_damage_or_hostile_effect() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5121).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x5122).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 0));

        let skill = single_step_skill(
            "enemy_heal_is_not_hostile",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 5 }],
        );

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0)),
            None
        );
    }

    #[test]
    fn automatic_enemy_tile_area_cast_waits_without_useful_hostile_target() {
        let caster_base_uuid = Uuid::from_u128(0x5130);
        let skill = enemy_tile_area_skill("area_no_useful_target", vec![zero_damage_effect()]);
        let abnormality = AbnormalityMetadata {
            id: "area_caster".to_string(),
            uuid: caster_base_uuid,
            name: "area caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5131).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x5132).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        core.units.insert(caster_id, caster);
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        assert!(core.units.get(&caster_id).unwrap().pending_cast);
        assert!(!core
            .event_queue
            .iter()
            .any(|event| matches!(event, BattleEvent::AutoCastEnd { .. })));
        assert!(!core.event_log.entries.iter().any(|entry| {
            matches!(entry.event, BattleLogEvent::AutoCastStart { .. })
                || matches!(entry.event, BattleLogEvent::SkillAreaDeclared { .. })
        }));
    }

    #[test]
    fn automatic_enemy_tile_area_cast_allows_zero_damage_hostile_buff() {
        let caster_base_uuid = Uuid::from_u128(0x5140);
        let skill = enemy_tile_area_skill(
            "area_hostile_buff",
            vec![SkillEffectDef::ApplyBuff {
                buff_id: "stun".to_string(),
                duration_ms: 500,
            }],
        );
        let abnormality = AbnormalityMetadata {
            id: "area_caster".to_string(),
            uuid: caster_base_uuid,
            name: "area caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        let mut core = core_with_skill_data(vec![abnormality], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5141).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x5142).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        core.units.insert(caster_id, caster);
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();
        let end = core.event_queue.pop().expect("missing AutoCastEnd");
        core.process_event(end, 1).unwrap();
        let step = core.event_queue.pop().expect("missing SkillStep");
        core.process_event(step, 1).unwrap();

        assert!(core.event_log.entries.iter().any(|entry| {
            matches!(entry.event, BattleLogEvent::SkillAreaDeclared { ref skill_id, .. }
                if skill_id == &skill.id)
        }));
    }

    #[test]
    fn manual_pending_cast_can_be_interrupted_and_stale_end_is_ignored() {
        let skill = single_step_skill(
            "manual_interrupt_victim",
            SkillTarget::SelfUnit,
            0,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 1 }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5151).into();
        let interrupter_id: UnitInstanceId = Uuid::from_u128(0x5152).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(interrupter_id, runtime_unit(interrupter_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, interrupter_id, Position::new(2, 1));

        core.start_manual_skill_cast(0, caster_id, skill.id.clone(), None)
            .expect("manual cast should start");
        let scheduled_end = core.event_queue.pop().expect("manual end scheduled");
        let BattleEvent::ManualCastEnd { cause, .. } = scheduled_end else {
            panic!("expected ManualCastEnd, got {scheduled_end:?}");
        };
        let start_seq = match cause {
            BattleEventCause::Parent { seq } => seq,
            other => panic!("expected parent cause, got {other:?}"),
        };

        core.process_commands(
            vec![BattleCommand::InterruptCast {
                source_id: interrupter_id,
                target_id: caster_id,
            }],
            1,
        );

        assert!(core
            .units
            .get(&caster_id)
            .unwrap()
            .pending_skill_cast
            .is_none());
        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 1
                && matches!(
                    &entry.event,
                    BattleLogEvent::SkillCastInterrupted {
                        interrupter_instance_id,
                        caster_instance_id,
                        interrupted_skill_id,
                        interrupted_cast_seq,
                    } if *interrupter_instance_id == interrupter_id
                        && *caster_instance_id == caster_id
                        && interrupted_skill_id == &skill.id
                        && *interrupted_cast_seq == start_seq
                )
        }));

        core.process_event(
            BattleEvent::ManualCastEnd {
                time_ms: 5,
                caster_instance_id: caster_id,
                cause,
            },
            5,
        )
        .expect("stale manual end should be ignored");

        assert!(!core.event_log.entries.iter().any(|entry| {
            matches!(
                entry.event,
                BattleLogEvent::AbilityCast {
                    caster_instance_id,
                    ..
                } if caster_instance_id == caster_id
            ) || matches!(
                entry.event,
                BattleLogEvent::ManualCastEnd {
                    caster_instance_id,
                } if caster_instance_id == caster_id
            )
        }));
    }

    #[test]
    fn auto_pending_cast_can_be_interrupted_and_stale_end_is_ignored() {
        let skill = single_step_skill(
            "auto_interrupt_victim",
            SkillTarget::SelfUnit,
            0,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::ModifyResonance { amount: -10 }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5161).into();
        let interrupter_id: UnitInstanceId = Uuid::from_u128(0x5162).into();
        let mut caster = runtime_unit(caster_id, Side::Opponent);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Auto;
        core.units.insert(caster_id, caster);
        core.units
            .insert(interrupter_id, runtime_unit(interrupter_id, Side::Player));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, interrupter_id, Position::new(2, 1));

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .expect("auto cast should start");
        let scheduled_end = core.event_queue.pop().expect("auto end scheduled");
        let BattleEvent::AutoCastEnd { cause, .. } = scheduled_end else {
            panic!("expected AutoCastEnd, got {scheduled_end:?}");
        };
        let start_seq = match cause {
            BattleEventCause::Parent { seq } => seq,
            other => panic!("expected parent cause, got {other:?}"),
        };

        core.process_commands(
            vec![BattleCommand::InterruptCast {
                source_id: interrupter_id,
                target_id: caster_id,
            }],
            1,
        );
        core.process_event(
            BattleEvent::AutoCastEnd {
                time_ms: 5,
                caster_instance_id: caster_id,
                cause,
            },
            5,
        )
        .expect("stale auto end should be ignored");

        assert!(core.event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::SkillCastInterrupted {
                    interrupted_cast_seq,
                    ..
                } if *interrupted_cast_seq == start_seq
            )
        }));
        assert!(!core.event_log.entries.iter().any(|entry| {
            matches!(
                entry.event,
                BattleLogEvent::AbilityCast {
                    caster_instance_id,
                    ..
                } if caster_instance_id == caster_id
            ) || matches!(
                entry.event,
                BattleLogEvent::AutoCastEnd {
                    caster_instance_id,
                } if caster_instance_id == caster_id
            )
        }));
    }

    #[test]
    fn stale_cast_end_does_not_invoke_newer_cast_on_same_caster() {
        let old_skill = single_step_skill(
            "old_cast",
            SkillTarget::SelfUnit,
            0,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 1 }],
        );
        let new_skill = single_step_skill(
            "new_cast",
            SkillTarget::SelfUnit,
            0,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 2 }],
        );
        let mut core = core_with_skill_data(vec![], vec![old_skill.clone(), new_skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5171).into();
        let interrupter_id: UnitInstanceId = Uuid::from_u128(0x5172).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.skill_id = Some(old_skill.id.clone());
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(interrupter_id, runtime_unit(interrupter_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, interrupter_id, Position::new(2, 1));

        core.start_manual_skill_cast(0, caster_id, old_skill.id.clone(), None)
            .expect("old manual cast should start");
        let old_end = core.event_queue.pop().expect("old manual end scheduled");
        let BattleEvent::ManualCastEnd {
            cause: old_cause, ..
        } = old_end
        else {
            panic!("expected old ManualCastEnd, got {old_end:?}");
        };
        core.process_commands(
            vec![BattleCommand::InterruptCast {
                source_id: interrupter_id,
                target_id: caster_id,
            }],
            1,
        );

        {
            let caster = core.units.get_mut(&caster_id).unwrap();
            caster.skill_id = Some(new_skill.id.clone());
            caster.resonance_current = 100;
        }
        core.start_manual_skill_cast(5, caster_id, new_skill.id.clone(), None)
            .expect("new manual cast should start");
        let new_start_seq = core
            .units
            .get(&caster_id)
            .and_then(|unit| unit.pending_skill_cast.as_ref())
            .map(|pending| pending.start_seq)
            .expect("new cast should be pending");

        core.process_event(
            BattleEvent::ManualCastEnd {
                time_ms: 5,
                caster_instance_id: caster_id,
                cause: old_cause,
            },
            5,
        )
        .expect("old stale end should not affect the new cast");

        let pending = core
            .units
            .get(&caster_id)
            .and_then(|unit| unit.pending_skill_cast.as_ref())
            .expect("new cast should still be pending");
        assert_eq!(pending.start_seq, new_start_seq);
        assert_eq!(pending.skill_id, new_skill.id);
        assert!(!core.event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AbilityCast {
                    skill_id,
                    caster_instance_id,
                    ..
                } if *caster_instance_id == caster_id && skill_id == &new_skill.id
            )
        }));
    }

    #[test]
    fn silence_blocks_new_casts_but_does_not_interrupt_pending_casts() {
        let skill = single_step_skill(
            "silence_pending_cast",
            SkillTarget::SelfUnit,
            0,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 1 }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5181).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        place_unit(&mut core, caster_id, Position::new(1, 1));

        core.start_manual_skill_cast(0, caster_id, skill.id.clone(), None)
            .expect("manual cast should start before silence");
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("silence"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            1,
        )
        .expect("silence should apply");

        let scheduled_end = core.event_queue.pop().expect("manual end scheduled");
        core.process_event(scheduled_end, 5)
            .expect("silence must not cancel already pending cast");

        assert!(core.event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AbilityCast {
                    skill_id,
                    caster_instance_id,
                    ..
                } if *caster_instance_id == caster_id && skill_id == &skill.id
            )
        }));
    }

    #[test]
    fn interrupt_only_skill_targets_only_currently_casting_enemies() {
        let skill = single_step_skill(
            "interrupt_only",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::InterruptCast],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5191).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x5192).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(1, 1)),
            None
        );

        core.units.get_mut(&target_id).unwrap().pending_skill_cast = Some(PendingSkillCast {
            skill_id: SkillId::from("target_cast"),
            cast_target: None,
            start_seq: 42,
        });

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(1, 1)),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id
            })
        );
    }

    #[test]
    fn interrupt_cast_skill_effect_cancels_pending_cast() {
        let skill = single_step_skill(
            "interrupt_effect",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::InterruptCast],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x51A1).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x51A2).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut target = runtime_unit(target_id, Side::Opponent);
        target.pending_skill_cast = Some(PendingSkillCast {
            skill_id: SkillId::from("target_pending_skill"),
            cast_target: None,
            start_seq: 77,
        });
        core.units.insert(target_id, target);

        let (commands, _result) =
            core.build_skill_step_commands(caster_id, &skill.steps[0], &[target_id], 0, None);
        assert!(matches!(
            commands.as_slice(),
            [BattleCommand::InterruptCast {
                source_id,
                target_id: command_target_id,
            }] if *source_id == caster_id && *command_target_id == target_id
        ));

        core.process_commands(commands, 10);

        assert!(core
            .units
            .get(&target_id)
            .unwrap()
            .pending_skill_cast
            .is_none());
        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 10
                && matches!(
                    &entry.event,
                    BattleLogEvent::SkillCastInterrupted {
                        interrupter_instance_id,
                        caster_instance_id,
                        interrupted_skill_id,
                        interrupted_cast_seq,
                    } if *interrupter_instance_id == caster_id
                        && *caster_instance_id == target_id
                        && interrupted_skill_id.as_str() == "target_pending_skill"
                        && *interrupted_cast_seq == 77
                )
        }));
    }

    #[test]
    fn instant_tile_area_delivery_hits_airborne_units_on_defense_tile_range() {
        let mut core = new_core();
        let cast_seq = 101;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xAA01).into();
        let enemy_on_tile_id: UnitInstanceId = Uuid::from_u128(0xAA02).into();
        let enemy_near_but_outside_id: UnitInstanceId = Uuid::from_u128(0xAA03).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_on_tile_id,
            runtime_unit(enemy_on_tile_id, Side::Opponent),
        );
        core.units.get_mut(&enemy_on_tile_id).unwrap().mobility_kind = MobilityKind::Airborne;
        core.units.insert(
            enemy_near_but_outside_id,
            runtime_unit(enemy_near_but_outside_id, Side::Opponent),
        );

        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, enemy_on_tile_id, Position::new(2, 1));
        place_unit(&mut core, enemy_near_but_outside_id, Position::new(1, 0));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_on_tile_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_near_but_outside_id);
        core.units.get_mut(&caster_id).unwrap().facing_direction = Some(FacingDirection::Up);

        let cast_target_tile = Position::new(3, 3);
        let tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["...".to_string(), ".@X".to_string(), "...".to_string()],
        };
        let area = SkillTileAreaDeliveryDef {
            anchor: SkillAreaAnchorSource::CastTarget,
            tile_origin: SkillTileAreaOrigin::Caster,
            tracking: SkillAreaTracking::GroundFixed,
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, center, _, affected_tiles, targets) = core
            .resolve_instant_tile_area_targets(
                0,
                cast_seq,
                0,
                caster_id,
                Some(crate::game::battle::event_log::SkillCastTarget::Tile {
                    position: cast_target_tile,
                }),
                &tile_range,
                &area,
            )
            .expect("tile area target resolution");

        assert_eq!(center, WorldVec2::from_tile_center(cast_target_tile));
        assert_eq!(affected_tiles, vec![Position::new(2, 1)]);
        assert!(targets.contains(&enemy_on_tile_id));
        assert!(!targets.contains(&enemy_near_but_outside_id));
    }

    #[test]
    fn instant_tile_area_affected_tiles_clip_to_valid_battlefield_tiles() {
        let scenario = BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 3,
                valid_tiles: vec![Position::new(1, 1), Position::new(2, 1)],
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: Vec::new(),
            win_condition: crate::game::battle::scenario::WinCondition::SurviveUntil {
                time_ms: 1_000,
            },
            tactical_plan: Default::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        core.battlefield =
            crate::game::battle::battlefield::BattlefieldLayout::new_with_valid_tiles(
                4,
                3,
                vec![Position::new(1, 1), Position::new(2, 1)],
            );
        let cast_seq = 103;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xAD01).into();
        let enemy_on_valid_tile_id: UnitInstanceId = Uuid::from_u128(0xAD02).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_on_valid_tile_id,
            runtime_unit(enemy_on_valid_tile_id, Side::Opponent),
        );
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, enemy_on_valid_tile_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_on_valid_tile_id);
        core.units.get_mut(&caster_id).unwrap().facing_direction = Some(FacingDirection::Up);

        let tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["X@X".to_string()],
        };
        let area = SkillTileAreaDeliveryDef {
            anchor: SkillAreaAnchorSource::Caster,
            tile_origin: SkillTileAreaOrigin::Caster,
            tracking: SkillAreaTracking::GroundFixed,
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, _, _, affected_tiles, targets) = core
            .resolve_instant_tile_area_targets(0, cast_seq, 0, caster_id, None, &tile_range, &area)
            .expect("tile area target resolution");

        assert_eq!(affected_tiles, vec![Position::new(2, 1)]);
        assert_eq!(targets, vec![enemy_on_valid_tile_id]);
    }

    #[test]
    fn instant_tile_area_origin_anchor_projects_range_from_cast_target_tile() {
        let mut core = new_core();
        let cast_seq = 102;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xAB01).into();
        let enemy_on_anchor_projected_tile_id: UnitInstanceId = Uuid::from_u128(0xAB02).into();
        let enemy_near_caster_id: UnitInstanceId = Uuid::from_u128(0xAB03).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_on_anchor_projected_tile_id,
            runtime_unit(enemy_on_anchor_projected_tile_id, Side::Opponent),
        );
        core.units.insert(
            enemy_near_caster_id,
            runtime_unit(enemy_near_caster_id, Side::Opponent),
        );
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, enemy_near_caster_id, Position::new(2, 1));
        place_unit(
            &mut core,
            enemy_on_anchor_projected_tile_id,
            Position::new(3, 2),
        );
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_near_caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_on_anchor_projected_tile_id);
        core.units.get_mut(&caster_id).unwrap().facing_direction = Some(FacingDirection::Up);

        let tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["...".to_string(), ".@X".to_string(), "...".to_string()],
        };
        let area = SkillTileAreaDeliveryDef {
            anchor: SkillAreaAnchorSource::CastTarget,
            tile_origin: SkillTileAreaOrigin::Anchor,
            tracking: SkillAreaTracking::GroundFixed,
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, center, _, affected_tiles, targets) = core
            .resolve_instant_tile_area_targets(
                0,
                cast_seq,
                0,
                caster_id,
                Some(crate::game::battle::event_log::SkillCastTarget::Tile {
                    position: Position::new(2, 2),
                }),
                &tile_range,
                &area,
            )
            .expect("tile area target resolution");

        assert_eq!(center, WorldVec2::from_tile_center(Position::new(2, 2)));
        assert_eq!(affected_tiles, vec![Position::new(3, 2)]);
        assert!(targets.contains(&enemy_on_anchor_projected_tile_id));
        assert!(!targets.contains(&enemy_near_caster_id));
    }

    #[test]
    fn impact_context_before_step_does_not_fall_back_to_later_step_context() {
        let mut core = new_core();
        let cast_seq = 26;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA151).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(0xA152).into();
        let later_context = super::types::SkillImpactContext {
            delivery_id: Uuid::from_u128(0xA153),
            impact_time_ms: 25,
            impact_position: WorldVec2::from_tile_center(Position::new(4, 1)),
            direction_hint: Some(WorldVec2::from_tile_center(Position::new(5, 1))),
            first_hit_unit_id: Some(enemy_id),
            hit_unit_ids: vec![enemy_id],
            spawned_area_id: None,
        };

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
                skill_id: SkillId::from("impact_context_fixture"),
                caster_owner: Side::Player,
                anchor_position: Position::new(1, 1),
                cast_target_anchor_position: None,
                cast_target_anchor_world_position: None,
                allow_dead_caster: false,
                total_steps: 3,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: std::iter::once((2usize, later_context.clone())).collect(),
                last_impact_context: Some(later_context),
                active_area_ids: Vec::new(),
            },
        );

        assert!(core.impact_context_before_step(cast_seq, 1).is_none());
    }

    #[test]
    fn targeted_execute_drops_locked_target_when_target_leaves_range() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(65).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(66).into();
        let other_target_id: UnitInstanceId = Uuid::from_u128(67).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            other_target_id,
            runtime_unit(other_target_id, Side::Opponent),
        );

        let caster_pos = Position::new(0, 0);
        place_unit(&mut core, caster_id, caster_pos);
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, other_target_id, Position::new(1, 1));

        let skill = single_step_skill(
            "locked_target_execute",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            50,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );

        let start_target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos);
        core.units
            .get_mut(&locked_target_id)
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(3, 3)));

        let targets = core.resolve_skill_step_targets(
            0,
            caster_id,
            skill.first_step().unwrap(),
            start_target,
        );
        assert_eq!(targets, Vec::<UnitInstanceId>::new());
    }

    #[test]
    fn targeted_execute_does_not_reacquire_when_locked_target_dies() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(68).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(69).into();
        let other_target_id: UnitInstanceId = Uuid::from_u128(70).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            other_target_id,
            runtime_unit(other_target_id, Side::Opponent),
        );

        let caster_pos = Position::new(0, 0);
        place_unit(&mut core, caster_id, caster_pos);
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, other_target_id, Position::new(1, 1));

        let skill = single_step_skill(
            "locked_target_death",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            50,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );

        let start_target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos);
        core.units
            .get_mut(&locked_target_id)
            .unwrap()
            .stats
            .current_health = 0;
        core.units.get_mut(&locked_target_id).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;

        let targets = core.resolve_skill_step_targets(
            0,
            caster_id,
            skill.first_step().unwrap(),
            start_target,
        );
        assert!(targets.is_empty());
    }

    #[test]
    fn skill_cast_target_requires_air_capable_for_airborne_enemy() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(71).into();
        let airborne_target_id: UnitInstanceId = Uuid::from_u128(72).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut airborne_target = runtime_unit(airborne_target_id, Side::Opponent);
        airborne_target.mobility_kind = MobilityKind::Airborne;
        core.units.insert(airborne_target_id, airborne_target);

        let caster_pos = Position::new(0, 0);
        place_unit(&mut core, caster_id, caster_pos);
        place_unit(&mut core, airborne_target_id, Position::new(1, 0));

        let mut skill = single_step_skill(
            "anti_air_gate",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos),
            None
        );

        skill.cast_targeting = explicit_cast_targeting(
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            Default::default(),
            skill.steps[0].defense_tile_range.clone(),
        );
        let SkillCastTargetingDef::Explicit { air_capable, .. } = &mut skill.cast_targeting;
        *air_capable = true;
        skill.steps[0].air_capable = true;

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos),
            Some(SkillCastTarget::Unit {
                unit_instance_id: airborne_target_id
            })
        );
    }

    #[test]
    fn reused_enemy_single_skill_step_revalidates_air_capable() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(73).into();
        let airborne_target_id: UnitInstanceId = Uuid::from_u128(74).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut airborne_target = runtime_unit(airborne_target_id, Side::Opponent);
        airborne_target.mobility_kind = MobilityKind::Airborne;
        core.units.insert(airborne_target_id, airborne_target);
        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, airborne_target_id, Position::new(1, 0));

        let mut skill = single_step_skill(
            "anti_air_step_gate",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );
        let cast_target = Some(SkillCastTarget::Unit {
            unit_instance_id: airborne_target_id,
        });

        assert!(core
            .resolve_skill_step_targets(0, caster_id, skill.first_step().unwrap(), cast_target)
            .is_empty());

        skill.steps[0].air_capable = true;

        assert_eq!(
            core.resolve_skill_step_targets(0, caster_id, skill.first_step().unwrap(), cast_target),
            vec![airborne_target_id]
        );
    }

    #[test]
    fn process_commands_applies_stat_modifier_and_resonance_delta() {
        let mut core = new_core();
        let target_id: UnitInstanceId = Uuid::from_u128(71).into();
        let mut unit = runtime_unit(target_id, Side::Player);
        unit.resonance_current = 50;
        core.units.insert(target_id, unit);

        core.process_commands(
            vec![
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 7,
                    },
                },
                BattleCommand::ModifyResonance {
                    target_id,
                    amount: -20,
                    allow_autocast_when_full: false,
                },
            ],
            100,
        );

        let unit = core.units.get(&target_id).unwrap();
        assert_eq!(unit.stats.attack, 8);
        assert_eq!(unit.resonance_current, 30);
    }

    #[test]
    fn silence_blocks_pending_autocast_until_expire() {
        let caster_base_uuid = Uuid::from_u128(0x9001);
        let skill = single_step_skill(
            "silence_test_skill",
            SkillTarget::SelfUnit,
            1,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::ModifyResonance { amount: -10 }],
        );
        let abnormality = AbnormalityMetadata {
            id: "caster".to_string(),
            uuid: caster_base_uuid,
            name: "caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(81).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.pending_cast = true;
        core.units.insert(caster_id, caster);

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("silence"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        core.schedule_pending_autocasts(0);
        assert!(!core
            .event_queue
            .iter()
            .any(|event| matches!(event, BattleEvent::AutoCastStart { .. })));

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 10,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("silence"),
                cause: BattleEventCause::default(),
            },
            10,
        )
        .unwrap();

        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::AutoCastStart {
                time_ms: 11,
                caster_instance_id,
                ..
            } if *caster_instance_id == caster_id
        )));
    }

    #[test]
    fn hard_cc_delays_autocast_completion_until_lock_release() {
        let caster_base_uuid = Uuid::from_u128(0x9002);
        let skill = single_step_skill(
            "delayed_cast_skill",
            SkillTarget::SelfUnit,
            1,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::ModifyResonance { amount: -10 }],
        );
        let abnormality = AbnormalityMetadata {
            id: "caster".to_string(),
            uuid: caster_base_uuid,
            name: "caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(82).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        place_unit(&mut core, caster_id, Position::new(0, 0));

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        let scheduled_end = core
            .event_queue
            .pop()
            .expect("missing scheduled AutoCastEnd");
        let end_cause = match scheduled_end {
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                assert_eq!(time_ms, 5);
                assert_eq!(caster_instance_id, caster_id);
                cause
            }
            other => panic!("expected AutoCastEnd, got {other:?}"),
        };

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            1,
        )
        .unwrap();

        core.process_event(
            BattleEvent::AutoCastEnd {
                time_ms: 5,
                caster_instance_id: caster_id,
                cause: end_cause,
            },
            5,
        )
        .unwrap();

        assert!(!core
            .event_log
            .entries
            .iter()
            .any(|entry| matches!(entry.event, BattleLogEvent::AbilityCast { .. })));

        let buff_expire = core.event_queue.pop().expect("missing BuffExpire");
        assert!(matches!(
            buff_expire,
            BattleEvent::BuffExpire { time_ms: 11, .. }
        ));
        core.process_event(buff_expire, 11).unwrap();

        let delayed_end = core.event_queue.pop().expect("missing delayed AutoCastEnd");
        assert!(matches!(
            delayed_end,
            BattleEvent::AutoCastEnd { time_ms: 12, .. }
        ));
        core.process_event(delayed_end, 12).unwrap();

        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 12
                && matches!(
                    entry.event,
                    BattleLogEvent::AbilityCast {
                        caster_instance_id,
                        ..
                    } if caster_instance_id == caster_id
                )
        }));
    }

    #[test]
    fn autocast_completion_resets_auto_attack_cycle_to_full_interval() {
        let caster_base_uuid = Uuid::from_u128(0x9003);
        let skill = single_step_skill(
            "recovery_cast_skill",
            SkillTarget::SelfUnit,
            1,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::ModifyResonance { amount: -10 }],
        );
        let abnormality = AbnormalityMetadata {
            id: "caster".to_string(),
            uuid: caster_base_uuid,
            name: "caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(83).into();
        let target_id: UnitInstanceId = Uuid::from_u128(84).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.stats.attack_interval_ms = 10;
        core.units.insert(caster_id, caster);
        place_unit(&mut core, caster_id, Position::new(0, 0));

        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, target_id, Position::new(1, 0));

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();

        let scheduled_end = core
            .event_queue
            .pop()
            .expect("missing scheduled AutoCastEnd");
        let end_cause = match scheduled_end {
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                assert_eq!(time_ms, 5);
                assert_eq!(caster_instance_id, caster_id);
                cause
            }
            other => panic!("expected AutoCastEnd, got {other:?}"),
        };

        core.process_event(
            BattleEvent::AutoCastEnd {
                time_ms: 5,
                caster_instance_id: caster_id,
                cause: end_cause,
            },
            5,
        )
        .unwrap();

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 5,
                attacker_instance_id: caster_id,
                target_instance_id: Some(target_id),
                schedule_next: true,
                cause: BattleEventCause::Root {
                    kind: crate::game::battle::event_log::BattleEventRootCause::Period,
                },
            },
            5,
        )
        .unwrap();

        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::AttackStart {
                time_ms: 15,
                attacker_instance_id,
                schedule_next: true,
                ..
            } if *attacker_instance_id == caster_id
        )));
        assert!(!core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 5
                && matches!(
                    entry.event,
                    BattleLogEvent::AttackStart {
                        attacker_instance_id,
                        ..
                    } if attacker_instance_id == caster_id
                )
        }));
    }

    #[test]
    fn completed_instant_skill_cast_is_cleaned_up_immediately() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA201).into();
        let skill = single_step_skill(
            "cleanup_instant",
            SkillTarget::SelfUnit,
            1,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 1 }],
        );

        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            None,
            BattleEventCause::default(),
            false,
        );

        let event = core.event_queue.pop().expect("missing skill step");
        let cast_seq = match &event {
            BattleEvent::SkillStep { cast_seq, .. } => *cast_seq,
            other => panic!("expected SkillStep, got {other:?}"),
        };

        core.process_event(event, 0).expect("process skill step");

        assert!(
            !core.active_skill_casts.contains_key(&cast_seq),
            "single-step instant cast should be removed once its step is terminal"
        );
    }
}
