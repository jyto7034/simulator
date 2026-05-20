use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use uuid::Uuid;

use crate::game::{
    battle::{
        battlefield::Battlefield,
        core::movement::{
            engine::ContinuousMovementBackend,
            types::{TimelineVec2, UnitBody, WorldVec2},
        },
        core::spatial::SpatialQueryBackend,
        enums::BattleEvent,
        ids::UnitInstanceId,
        scenario::{BattleScenario, ScenarioGroupId, ScenarioUnitRef, TacticalGroupPlanId},
        timeline::{Timeline, TimelineCause, TimelineEvent},
        types::UnitSnapshot,
    },
    data::GameDataBase,
};

pub mod build;
pub mod commands;
pub mod ids;
pub mod movement;
pub mod sim;
pub mod skill_runtime;
pub mod spatial;
pub mod targeting;
pub mod triggers;
pub mod types;

use self::types::{
    AbilityProcKey, AbilityProcState, ActiveBuff, ActiveProjectileRuntime, ActiveSkillCast,
    ActiveSkillCastDebugState, AreaRuntime, BuffInstanceKey, ProjectileRecord, RuntimeArtifact,
    RuntimeItem, RuntimeUnit, TriggerSource,
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
    melee_slot_reservations: HashMap<UnitInstanceId, MeleeSlotReservation>,
    last_continuous_movement_tick_ms: Option<u64>,
    movement_backend: ContinuousMovementBackend,
    spatial_query_backend: SpatialQueryBackend,
    pub battlefield: Battlefield,

    pub game_data: Arc<GameDataBase>,

    pub timeline: Timeline,
    pub timeline_seq: u64,
    pub projectile_seq: u64,
    pub area_seq: u64,
    pub seed: u64,
    pub recording_cause_stack: Vec<TimelineCause>,
}

#[derive(Debug, Clone, Copy)]
struct ActiveMovementSegment {
    timeline_index: usize,
    velocity: TimelineVec2,
    start: WorldVec2,
    target: WorldVec2,
    started_at_ms: u64,
    ends_at_ms: u64,
}

impl ActiveMovementSegment {
    fn sample_position_at(&self, time_ms: u64) -> WorldVec2 {
        if time_ms <= self.started_at_ms {
            return self.start;
        }
        if time_ms >= self.ends_at_ms {
            return self.target;
        }

        let duration_ms = self.ends_at_ms.saturating_sub(self.started_at_ms);
        if duration_ms == 0 {
            return self.target;
        }

        let elapsed_ms = time_ms.saturating_sub(self.started_at_ms);
        let t = elapsed_ms as f32 / duration_ms as f32;
        self.start + (self.target - self.start) * t
    }
}

#[derive(Debug, Clone, Default)]
pub(in crate::game::battle::core) struct ScenarioRuntimeState {
    spawned_groups: std::collections::HashSet<ScenarioGroupId>,
    unit_refs: HashMap<ScenarioUnitRef, UnitInstanceId>,
    tactical_groups: HashMap<TacticalGroupPlanId, Vec<UnitInstanceId>>,
    recovery_target_secured: bool,
    recovery_hold_started_at_ms: Option<u64>,
    recovery_hold_completed: bool,
    forced_winner: Option<crate::game::battle::types::BattleWinner>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::game::battle::core) struct MeleeSlotReservation {
    target_id: UnitInstanceId,
    slot_index: usize,
    position: WorldVec2,
    unit_radius: f32,
}

impl BattleCore {
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
            melee_slot_reservations: HashMap::new(),
            last_continuous_movement_tick_ms: None,
            movement_backend: ContinuousMovementBackend::default(),
            spatial_query_backend: SpatialQueryBackend::default(),
            battlefield: Battlefield::new_with_valid_tiles(field_size.0, field_size.1, valid_tiles),
            game_data,
            timeline: Timeline::new(),
            timeline_seq: 0,
            projectile_seq: 0,
            area_seq: 0,
            seed,
            recording_cause_stack: Vec::new(),
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

    pub fn live_unit_bodies(&self) -> Vec<(UnitInstanceId, UnitBody)> {
        let mut bodies: Vec<_> = self
            .units
            .values()
            .filter(|unit| !unit.is_dead())
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

    fn can_gain_resonance(unit: &RuntimeUnit, now_ms: u64) -> bool {
        unit.action_locks.can_gain_resonance(now_ms)
    }

    fn can_start_autocast(unit: &RuntimeUnit, now_ms: u64) -> bool {
        if now_ms < unit.next_action_time {
            return false;
        }
        if !unit.is_combatant() {
            return false;
        }
        if unit.is_dead() {
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
            let Some(unit) = self.units.get_mut(&unit_instance_id) else {
                return;
            };

            if unit.is_dead() {
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

            if allow_autocast_when_full && before < max && after == max {
                unit.pending_cast = true;
            }

            (before, after, max)
        };

        self.record_timeline(
            now_ms,
            TimelineEvent::ResonanceChanged {
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

            if unit.is_dead() {
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

        self.record_timeline(
            now_ms,
            TimelineEvent::ResonanceChanged {
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
                && crate::game::battle::buffs::get(key.buff_id).is_some_and(|def| def.kind == kind)
        })
    }

    fn schedule_pending_autocast_for(&mut self, caster_instance_id: UnitInstanceId, now_ms: u64) {
        let fallback = self.recording_cause().unwrap_or_default();

        if self.has_buff_kind(
            caster_instance_id,
            crate::game::battle::buffs::BuffKind::Silence,
        ) {
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
                if unit.pending_cast && !unit.is_dead() {
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
        DeliveryDef, SkillAreaAnchorSource, SkillAreaDeliveryDef, SkillAreaShapeDef,
        SkillAreaTracking, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillId, SkillKind,
        SkillPresentationDef, SkillStepDef, SkillTarget, StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::buffs::BuffId;
    use crate::game::battle::core::movement::{types::WorldVec2, ActionState, TILE_UNITS_PER_TILE};
    use crate::game::battle::core::types::RuntimeUnit;
    use crate::game::battle::damage::BattleCommand;
    use crate::game::battle::enums::BattleEvent;
    use crate::game::battle::scenario::BattleScenario;
    use crate::game::battle::timeline::{TimelineCause, TimelineEvent};
    use crate::game::battle::types::{BattleUnitDraft, BattleUnitSource, UnitCombatProfile};
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata, skill_data::SkillDatabase, GameDataBase,
        GameDataBuilder,
    };
    use crate::game::enums::Side;
    use crate::game::resources::Position;
    use crate::game::stats::UnitStats;
    use crate::game::stats::{StatId, StatModifier, StatModifierKind};

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty().build_arc()
    }

    fn new_core() -> BattleCore {
        BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), empty_game_data(), 123)
    }

    #[test]
    fn battle_runtime_reset_clears_continuous_movement_state() {
        let mut core = new_core();
        let unit_id = UnitInstanceId::from(Uuid::from_u128(101));
        let target_id = UnitInstanceId::from(Uuid::from_u128(102));

        core.active_movement_segments.insert(
            unit_id,
            ActiveMovementSegment {
                timeline_index: 7,
                velocity: TimelineVec2 {
                    x_milli: 1_000,
                    y_milli: 0,
                },
                start: WorldVec2::ZERO,
                target: WorldVec2::new(3.0, 0.0),
                started_at_ms: 10,
                ends_at_ms: 100,
            },
        );
        core.melee_slot_reservations.insert(
            unit_id,
            MeleeSlotReservation {
                target_id,
                slot_index: 0,
                position: WorldVec2::new(1.0, 0.0),
                unit_radius: 0.35,
            },
        );
        core.last_continuous_movement_tick_ms = Some(100);

        core.reset_runtime_state_for_battle();

        assert!(core.active_movement_segments.is_empty());
        assert!(core.melee_slot_reservations.is_empty());
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
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: first_enemy_ref,
                        side: Side::Opponent,
                        draft: fixture_draft(first_enemy_owned, Uuid::from_u128(20), 1, 1),
                        position: Position::new(1, 2),
                        instance_salt: 0,
                    }],
                },
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: second_wave_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: true,
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
        assert!(result.timeline.entries.iter().any(|entry| {
            entry.time_ms == 1_000 && matches!(entry.event, TimelineEvent::UnitSpawned { .. })
        }));
    }

    #[test]
    fn protect_unit_battle_loses_when_defense_object_is_destroyed() {
        let player_group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let object_group_id = crate::game::battle::scenario::ScenarioGroupId::new("defense_object");
        let enemy_group_id = crate::game::battle::scenario::ScenarioGroupId::new("enemy_wave");
        let protected_ref = crate::game::battle::scenario::ScenarioUnitRef::new("black_box");

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 6,
                height: 6,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: player_group_id.clone(),
                    side: Side::Player,
                    required_for_victory: false,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new("player_0"),
                        side: Side::Player,
                        draft: fixture_draft(Uuid::from_u128(401), Uuid::from_u128(402), 100, 1),
                        position: Position::new(0, 0),
                        instance_salt: 0,
                    }],
                },
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: object_group_id.clone(),
                    side: Side::Player,
                    required_for_victory: false,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: protected_ref.clone(),
                        side: Side::Player,
                        draft: defense_object_draft(Uuid::from_u128(411), Uuid::from_u128(412), 3),
                        position: Position::new(4, 4),
                        instance_salt: 0,
                    }],
                },
                crate::game::battle::scenario::ScenarioSpawnGroup {
                    id: enemy_group_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: false,
                    spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                        unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new("enemy_0"),
                        side: Side::Opponent,
                        draft: fixture_draft(Uuid::from_u128(421), Uuid::from_u128(422), 100, 10),
                        position: Position::new(4, 3),
                        instance_salt: 0,
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
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_object"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: object_group_id,
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
            win_condition: crate::game::battle::scenario::WinCondition::ProtectUnit {
                unit_ref: protected_ref,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let result = core.run_battle().expect("battle should run");

        assert_eq!(
            result.winner,
            crate::game::battle::types::BattleWinner::Opponent
        );
        assert!(result.timeline.entries.iter().any(|entry| matches!(
            &entry.event,
            TimelineEvent::UnitSpawned {
                role: crate::game::battle::types::BattleUnitRole::DefenseObject,
                ..
            }
        )));
    }

    #[test]
    fn recovery_battle_wins_after_target_clear_hold_and_extraction() {
        let player_group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let target_point_id =
            crate::game::battle::scenario::TacticalPointId::new("recovery_target");
        let extraction_point_id =
            crate::game::battle::scenario::TacticalPointId::new("extraction_point");

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 4,
                height: 2,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: player_group_id.clone(),
                side: Side::Player,
                required_for_victory: false,
                spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                    unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new("player_0"),
                    side: Side::Player,
                    draft: fixture_draft(Uuid::from_u128(311), Uuid::from_u128(312), 100, 100),
                    position: Position::new(0, 0),
                    instance_salt: 0,
                }],
            }],
            events: vec![crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("spawn_player"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                    group_id: player_group_id,
                },
                once: true,
            }],
            win_condition: crate::game::battle::scenario::WinCondition::RecoverHoldAndExtract {
                target_point_id: target_point_id.clone(),
                extraction_point_id: extraction_point_id.clone(),
                target_radius: 0.25,
                extraction_radius: 0.25,
                hold_duration_ms: 100,
            },
            tactical_plan: crate::game::battle::scenario::TacticalPlan {
                points: vec![
                    crate::game::battle::scenario::TacticalPoint {
                        id: target_point_id,
                        position: Position::new(0, 0),
                    },
                    crate::game::battle::scenario::TacticalPoint {
                        id: extraction_point_id,
                        position: Position::new(0, 0),
                    },
                ],
                group_plans: vec![crate::game::battle::scenario::TacticalGroupPlan {
                    id: crate::game::battle::scenario::TacticalGroupPlanId::new("player_main"),
                    side: Side::Player,
                    members: crate::game::battle::scenario::TacticalGroupMembers::SideAll(
                        Side::Player,
                    ),
                    objective: crate::game::battle::scenario::GroupObjective::AdvanceAlongPath {
                        point_ids: vec![
                            crate::game::battle::scenario::TacticalPointId::new("recovery_target"),
                            crate::game::battle::scenario::TacticalPointId::new("extraction_point"),
                        ],
                    },
                    formation: crate::game::battle::scenario::FormationKind::Loose,
                    cohesion_radius: 2.0,
                    engage_radius: 2.0,
                }],
                ..crate::game::battle::scenario::TacticalPlan::default()
            },
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let result = core.run_battle().expect("battle should run");

        assert_eq!(
            result.winner,
            crate::game::battle::types::BattleWinner::Player
        );
        assert!(result.timeline.entries.iter().any(|entry| matches!(
            &entry.event,
            TimelineEvent::RecoveryTargetSecured {
                target_point_id,
                ..
            } if target_point_id == "recovery_target"
        )));
        assert!(result.timeline.entries.iter().any(|entry| matches!(
            &entry.event,
            TimelineEvent::ExtractionCompleted {
                extraction_point_id,
                ..
            } if extraction_point_id == "extraction_point"
        )));
    }

    #[test]
    fn scenario_spawn_group_members_are_recorded_as_tactical_group_members() {
        let player_group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let tactical_group_id =
            crate::game::battle::scenario::TacticalGroupPlanId::new("player_squad");
        let player_ref = crate::game::battle::scenario::ScenarioUnitRef::new("player_0");
        let player_owned = Uuid::from_u128(11);

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 5,
                height: 5,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: player_group_id.clone(),
                side: Side::Player,
                required_for_victory: false,
                spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                    unit_ref: player_ref,
                    side: Side::Player,
                    draft: fixture_draft(player_owned, Uuid::from_u128(110), 100, 100),
                    position: Position::new(1, 1),
                    instance_salt: 0,
                }],
            }],
            events: vec![crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("spawn_player"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                    group_id: player_group_id.clone(),
                },
                once: true,
            }],
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: crate::game::battle::scenario::TacticalPlan {
                group_plans: vec![
                    crate::game::battle::scenario::TacticalGroupPlan {
                        id: crate::game::battle::scenario::TacticalGroupPlanId::new("catch_all"),
                        side: Side::Player,
                        members: crate::game::battle::scenario::TacticalGroupMembers::SideAll(
                            Side::Player,
                        ),
                        objective:
                            crate::game::battle::scenario::GroupObjective::FollowBattleObjective,
                        formation: crate::game::battle::scenario::FormationKind::Loose,
                        cohesion_radius: 4.0,
                        engage_radius: 3.0,
                    },
                    crate::game::battle::scenario::TacticalGroupPlan {
                        id: tactical_group_id.clone(),
                        side: Side::Player,
                        members: crate::game::battle::scenario::TacticalGroupMembers::SpawnGroup(
                            player_group_id,
                        ),
                        objective: crate::game::battle::scenario::GroupObjective::HoldArea {
                            point_id: crate::game::battle::scenario::TacticalPointId::new("hold"),
                        },
                        formation: crate::game::battle::scenario::FormationKind::Loose,
                        cohesion_radius: 2.0,
                        engage_radius: 1.0,
                    },
                ],
                ..crate::game::battle::scenario::TacticalPlan::default()
            },
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        core.spawn_scenario_group(&crate::game::battle::scenario::ScenarioGroupId::new(
            "player",
        ))
        .expect("group should spawn");
        let unit_id = *core
            .scenario_runtime
            .tactical_groups
            .get(&tactical_group_id)
            .expect("tactical group should be tracked")
            .first()
            .expect("spawned unit should be in tactical group");
        let unit = core.units.get(&unit_id).expect("spawned unit should exist");

        assert_eq!(unit.tactical_group_id, Some(tactical_group_id));
    }

    #[test]
    fn default_tactical_plan_records_player_side_as_main_group() {
        let player_group_id = crate::game::battle::scenario::ScenarioGroupId::new("player");
        let player_ref = crate::game::battle::scenario::ScenarioUnitRef::new("player_0");
        let player_owned = Uuid::from_u128(12);
        let default_group_id =
            crate::game::battle::scenario::TacticalPlan::default_player_main_group_id();

        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 5,
                height: 5,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: player_group_id.clone(),
                side: Side::Player,
                required_for_victory: false,
                spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                    unit_ref: player_ref,
                    side: Side::Player,
                    draft: fixture_draft(player_owned, Uuid::from_u128(120), 100, 100),
                    position: Position::new(1, 1),
                    instance_salt: 0,
                }],
            }],
            events: vec![crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("spawn_player"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                    group_id: player_group_id.clone(),
                },
                once: true,
            }],
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };

        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        core.spawn_scenario_group(&player_group_id)
            .expect("group should spawn");
        let unit_id = *core
            .scenario_runtime
            .tactical_groups
            .get(&default_group_id)
            .expect("default player main group should be tracked")
            .first()
            .expect("spawned player should be in default main group");
        let unit = core.units.get(&unit_id).expect("spawned unit should exist");

        assert_eq!(unit.tactical_group_id, Some(default_group_id));
    }

    fn runtime_unit(unit_id: UnitInstanceId, owner: Side) -> RuntimeUnit {
        RuntimeUnit {
            instance_id: unit_id,
            source_owned_uuid: unit_id.as_uuid(),
            owner,
            role: crate::game::battle::types::BattleUnitRole::Combatant,
            base_uuid: Uuid::nil(),
            stats: UnitStats::with_values(10, 10, 1, 0, 1),
            basic_attack: Default::default(),
            skill_id: None,
            body: Default::default(),
            tactical_anchor: None,
            tactical_group_id: None,
            move_epoch: 0,
            action_state: ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
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
            level: crate::game::enums::Tier::I,
            growth_stacks: crate::game::growth::GrowthStack::new(),
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
        }
    }

    fn defense_object_draft(owned_uuid: Uuid, base_uuid: Uuid, hp: u32) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.stats = UnitStats::with_values(hp, hp, 0, 0, 100);
        profile.stats.move_speed_units_per_ms = 0;
        profile.movement.speed_units_per_ms = 0;
        profile.skill_id = None;
        BattleUnitDraft {
            owned_uuid,
            source: BattleUnitSource::DefenseObject { base_uuid, profile },
            level: crate::game::enums::Tier::I,
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

    fn sync_unit_to_battlefield_tile_center(core: &mut BattleCore, unit_id: UnitInstanceId) {
        let Some(position) = core.battlefield.position_of(unit_id) else {
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
        unit
    }

    fn place_unit(core: &mut BattleCore, unit_id: UnitInstanceId, pos: Position) {
        core.battlefield.place(unit_id, pos).unwrap();
        let unit = core.units.get_mut(&unit_id).unwrap();
        unit.set_world_position(WorldVec2::new(pos.x as f32, pos.y as f32));
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
            movement: Default::default(),
            basic_attack: crate::game::data::abnormality_data::BasicAttackDef {
                range_units: f32::from(range_units),
                delivery,
                ..Default::default()
            },
            resonance: Default::default(),
            skill_id: None,
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
        SkillDef {
            id: SkillId::from(id),
            name: id.to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: SkillCastTargetingDef::FirstStepTarget,
            focus_time_ms,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "step_01".to_string(),
                delay_ms: 0,
                range_units: f32::from(range_units),
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

    fn core_with_skill_data(
        abnormalities: Vec<AbnormalityMetadata>,
        skills: Vec<SkillDef>,
    ) -> BattleCore {
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(abnormalities)
            .with_skills(SkillDatabase::new(skills))
            .build_arc();

        BattleCore::new_from_scenario(BattleScenario::empty((6, 6)), game_data, 123)
    }

    fn insert_test_active_skill_cast(
        core: &mut BattleCore,
        cast_seq: u64,
        caster_id: UnitInstanceId,
        caster_owner: Side,
        anchor_position: Position,
        cast_target_anchor_position: Option<Position>,
        cast_target_anchor_world_position: Option<WorldVec2>,
    ) {
        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
                caster_owner,
                anchor_position,
                cast_target_anchor_position,
                cast_target_anchor_world_position,
                allow_dead_caster: false,
                total_steps: 1,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: Default::default(),
                last_impact_context: None,
                active_area_ids: Vec::new(),
            },
        );
    }

    #[test]
    fn persistent_area_follow_caster_updates_geometry_each_tick() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xFA01).into();
        let skill_id = SkillId::from("follow_caster_area");
        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Circle {
                radius_units: TILE_UNITS_PER_TILE as u32,
            },
            anchor: SkillAreaAnchorSource::Caster,
            tracking: SkillAreaTracking::FollowCaster,
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 1_000,
            tick_interval_ms: Some(100),
        };
        let skill = single_step_skill(
            skill_id.as_str(),
            SkillTarget::SelfUnit,
            0,
            0,
            DeliveryDef::Area { area },
            vec![],
        );
        let mut core = core_with_skill_data(vec![], vec![skill]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        insert_test_active_skill_cast(
            &mut core,
            9001,
            caster_id,
            Side::Player,
            Position::new(1, 1),
            None,
            None,
        );

        assert!(core.register_persistent_area(
            0,
            9001,
            0,
            caster_id,
            skill_id.clone(),
            "step_01".to_string(),
            None,
            area,
        ));
        let area_id = *core.active_areas.keys().next().expect("area id");
        core.units
            .get_mut(&caster_id)
            .unwrap()
            .set_world_position(WorldVec2::new(3.25, 2.75));

        core.apply_skill_area_tick(100, area_id);

        assert_eq!(
            core.active_areas
                .get(&area_id)
                .map(|runtime| runtime.center),
            Some(WorldVec2::new(3.25, 2.75))
        );
    }

    #[test]
    fn persistent_area_follow_target_updates_geometry_each_tick() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xFA11).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xFA12).into();
        let skill_id = SkillId::from("follow_target_area");
        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Circle {
                radius_units: TILE_UNITS_PER_TILE as u32,
            },
            anchor: SkillAreaAnchorSource::CastTarget,
            tracking: SkillAreaTracking::FollowTarget,
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 1_000,
            tick_interval_ms: Some(100),
        };
        let skill = single_step_skill(
            skill_id.as_str(),
            SkillTarget::CastTarget,
            0,
            0,
            DeliveryDef::Area { area },
            vec![],
        );
        let mut core = core_with_skill_data(vec![], vec![skill]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(3, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        insert_test_active_skill_cast(
            &mut core,
            9002,
            caster_id,
            Side::Player,
            Position::new(1, 1),
            Some(Position::new(3, 1)),
            Some(WorldVec2::from_tile_center(Position::new(3, 1))),
        );
        let step_target = Some(crate::game::battle::timeline::SkillCastTarget::Unit {
            unit_instance_id: target_id,
        });

        assert!(core.register_persistent_area(
            0,
            9002,
            0,
            caster_id,
            skill_id.clone(),
            "step_01".to_string(),
            step_target,
            area,
        ));
        let area_id = *core.active_areas.keys().next().expect("area id");
        core.units
            .get_mut(&target_id)
            .unwrap()
            .set_world_position(WorldVec2::new(4.25, 1.75));

        core.apply_skill_area_tick(100, area_id);

        assert_eq!(
            core.active_areas
                .get(&area_id)
                .map(|runtime| runtime.center),
            Some(WorldVec2::new(4.25, 1.75))
        );
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

        let cause = TimelineCause::Parent { seq: 42 };
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
                cause: TimelineCause::Parent { seq: 42 },
            } if caster_instance_id == caster_small
        ));
        assert!(matches!(
            second,
            BattleEvent::AutoCastStart {
                time_ms: 100,
                caster_instance_id,
                cause: TimelineCause::Parent { seq: 42 },
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
                cause: TimelineCause::default(),
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
    fn attack_start_hint_overrides_persisted_target_when_it_is_in_range() {
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
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(hinted_target_id)
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
                cause: TimelineCause::default(),
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

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, dead_target_id, Position::new(1, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::default(),
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
    fn attack_start_prefers_straight_enemy_over_equal_range_diagonal_enemy() {
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
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(straight_enemy_id)
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
                    collision: Default::default(),
                },
                3,
            ),
            abnormality_with_basic_attack(melee_enemy_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(
                ranged_enemy_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
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
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            melee_enemy_id,
            runtime_unit_with_base(melee_enemy_id, Side::Opponent, melee_enemy_base_uuid),
        );
        core.units.insert(
            ranged_enemy_id,
            runtime_unit_with_base(ranged_enemy_id, Side::Opponent, ranged_enemy_base_uuid),
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
    fn choose_attack_target_in_range_prefers_straight_enemy_over_equal_range_diagonal_enemy() {
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
            Some(straight_enemy_id)
        );
    }

    #[test]
    fn choose_enemy_target_in_range_units_prefers_melee_enemy_when_distance_is_equal() {
        let melee_enemy_base_uuid = Uuid::from_u128(0xAA08);
        let ranged_enemy_base_uuid = Uuid::from_u128(0xAA09);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(melee_enemy_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(
                ranged_enemy_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
                    collision: Default::default(),
                },
                3,
            ),
        ]);
        let caster_id: UnitInstanceId = Uuid::from_u128(82).into();
        let melee_enemy_id: UnitInstanceId = Uuid::from_u128(83).into();
        let ranged_enemy_id: UnitInstanceId = Uuid::from_u128(84).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            melee_enemy_id,
            runtime_unit_with_base(melee_enemy_id, Side::Opponent, melee_enemy_base_uuid),
        );
        core.units.insert(
            ranged_enemy_id,
            runtime_unit_with_base(ranged_enemy_id, Side::Opponent, ranged_enemy_base_uuid),
        );

        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, melee_enemy_id, Position::new(1, 0));
        place_unit(&mut core, ranged_enemy_id, Position::new(0, 1));

        assert_eq!(
            core.choose_enemy_target_in_range_units(caster_id, Side::Player, 3.0),
            Some(melee_enemy_id)
        );
    }

    #[test]
    fn choose_enemy_target_in_range_units_prefers_nearest_enemy_by_continuous_distance() {
        let enemy_base_uuid = Uuid::from_u128(0xAA12);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            enemy_base_uuid,
            DeliveryDef::Instant,
            1,
        )]);
        let caster_id: UnitInstanceId = Uuid::from_u128(93).into();
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(94).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(95).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            diagonal_enemy_id,
            runtime_unit_with_base(diagonal_enemy_id, Side::Opponent, enemy_base_uuid),
        );
        core.units.insert(
            straight_enemy_id,
            runtime_unit_with_base(straight_enemy_id, Side::Opponent, enemy_base_uuid),
        );

        place_unit(&mut core, caster_id, Position::new(2, 2));
        place_unit(&mut core, diagonal_enemy_id, Position::new(1, 1));
        place_unit(&mut core, straight_enemy_id, Position::new(2, 1));

        assert_eq!(
            core.choose_enemy_target_in_range_units(caster_id, Side::Player, 1.0),
            Some(straight_enemy_id)
        );
    }

    #[test]
    fn instant_basic_attack_uses_continuous_range_for_selection_and_resolve() {
        let attacker_base_uuid = Uuid::from_u128(0xAA01);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            1,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(61).into();
        let target_id: UnitInstanceId = Uuid::from_u128(62).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(2, 2));

        assert_eq!(core.choose_attack_target_in_range(attacker_id), None);
        assert!(!core.resolve_basic_attack(attacker_id, target_id, 0));

        let attacker = core.units.get_mut(&attacker_id).unwrap();
        attacker.set_world_position(WorldVec2::new(1.0, 1.0));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(target_id)
        );
        assert!(core.resolve_basic_attack(attacker_id, target_id, 1));
    }

    #[test]
    fn projectile_basic_attack_uses_continuous_range_for_selection_and_resolve() {
        let attacker_base_uuid = Uuid::from_u128(0xAA02);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000,
                collision: Default::default(),
            },
            1,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(71).into();
        let target_id: UnitInstanceId = Uuid::from_u128(72).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(target_id)
        );
        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));

        let attacker = core.units.get_mut(&attacker_id).unwrap();
        attacker.set_world_position(WorldVec2::new(-1.0, -1.0));

        assert_eq!(core.choose_attack_target_in_range(attacker_id), None);
        assert!(!core.resolve_basic_attack(attacker_id, target_id, 1));
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
                cause: TimelineCause::default(),
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
                cause: TimelineCause::default(),
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
            vec![],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id })
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
            vec![],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id })
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
            vec![],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id })
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

        core.battlefield
            .place(caster_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(tank_id, Position::new(1, 0))
            .unwrap();
        core.battlefield
            .place(weak_id, Position::new(1, 1))
            .unwrap();

        let skill = single_step_skill(
            "lowest_health",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == weak_id
        ));
    }

    #[test]
    fn area_reuse_cast_target_preserves_unit_identity_for_anchor_sampling() {
        let mut core = new_core();
        let cast_seq = 26;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA151).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xA152).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(3, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        let target = core.units.get_mut(&target_id).unwrap();
        target.set_world_position(target.body.position + WorldVec2::new(0.25, 0.0));
        let stored_target_position = target.body.position;

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
                caster_owner: Side::Player,
                anchor_position: Position::new(1, 1),
                cast_target_anchor_position: Some(Position::new(3, 1)),
                cast_target_anchor_world_position: Some(stored_target_position),
                allow_dead_caster: false,
                total_steps: 1,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: Default::default(),
                last_impact_context: None,
                active_area_ids: Vec::new(),
            },
        );
        core.units.remove(&target_id);

        let step = SkillStepDef {
            id: "step_01".to_string(),
            delay_ms: 0,
            range_units: 4.0,
            target: SkillTarget::CastTarget,
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Area {
                area: SkillAreaDeliveryDef {
                    shape: SkillAreaShapeDef::Circle {
                        radius_units: TILE_UNITS_PER_TILE as u32,
                    },
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tracking: Default::default(),
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 0,
                    tick_interval_ms: None,
                },
            },
            effects: vec![],
            presentation: Default::default(),
        };

        let step_target = core.resolve_skill_step_context(
            cast_seq,
            caster_id,
            &step,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
        );
        assert!(matches!(
            step_target,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == target_id
        ));

        let DeliveryDef::Area { area } = step.delivery else {
            panic!("expected area delivery");
        };
        let anchor = core
            .resolve_area_anchor_position(0, cast_seq, 0, caster_id, step_target, &area)
            .expect("area anchor");
        assert_eq!(anchor, stored_target_position);
    }

    #[test]
    fn instant_circle_area_delivery_hits_only_units_inside_radius() {
        let mut core = new_core();
        let cast_seq = 1;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA001).into();
        let enemy_inside_id: UnitInstanceId = Uuid::from_u128(0xA002).into();
        let enemy_outside_id: UnitInstanceId = Uuid::from_u128(0xA003).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_inside_id,
            runtime_unit(enemy_inside_id, Side::Opponent),
        );
        core.units.insert(
            enemy_outside_id,
            runtime_unit(enemy_outside_id, Side::Opponent),
        );

        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_inside_id, Position::new(2, 1))
            .unwrap();
        core.battlefield
            .place(enemy_outside_id, Position::new(3, 3))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_inside_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_outside_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Circle {
                radius_units: TILE_UNITS_PER_TILE as u32,
            },
            anchor: SkillAreaAnchorSource::CastTarget,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, center, _, targets) = core
            .resolve_instant_area_targets(
                0,
                cast_seq,
                0,
                caster_id,
                Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                    position: Position::new(2, 1),
                }),
                &area,
            )
            .expect("area target resolution");

        assert_eq!(center, WorldVec2::from_tile_center(Position::new(2, 1)));
        assert!(targets.contains(&enemy_inside_id));
        assert!(!targets.contains(&enemy_outside_id));
    }

    #[test]
    fn instant_rectangle_area_delivery_uses_caster_to_anchor_direction() {
        let mut core = new_core();
        let cast_seq = 2;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA011).into();
        let enemy_front_id: UnitInstanceId = Uuid::from_u128(0xA012).into();
        let enemy_back_id: UnitInstanceId = Uuid::from_u128(0xA013).into();
        let enemy_offline_id: UnitInstanceId = Uuid::from_u128(0xA014).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(enemy_front_id, runtime_unit(enemy_front_id, Side::Opponent));
        core.units
            .insert(enemy_back_id, runtime_unit(enemy_back_id, Side::Opponent));
        core.units.insert(
            enemy_offline_id,
            runtime_unit(enemy_offline_id, Side::Opponent),
        );

        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_front_id, Position::new(2, 1))
            .unwrap();
        core.battlefield
            .place(enemy_back_id, Position::new(3, 1))
            .unwrap();
        core.battlefield
            .place(enemy_offline_id, Position::new(2, 2))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_front_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_back_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_offline_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Rectangle {
                width_units: TILE_UNITS_PER_TILE as u32,
                length_units: (TILE_UNITS_PER_TILE * 2) as u32,
            },
            anchor: SkillAreaAnchorSource::CastTarget,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, _, _, targets) = core
            .resolve_instant_area_targets(
                0,
                cast_seq,
                0,
                caster_id,
                Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                    position: Position::new(3, 1),
                }),
                &area,
            )
            .expect("area target resolution");

        assert!(targets.contains(&enemy_front_id));
        assert!(targets.contains(&enemy_back_id));
        assert!(!targets.contains(&enemy_offline_id));
    }

    #[test]
    fn instant_box_area_delivery_is_centered_on_anchor() {
        let mut core = new_core();
        let cast_seq = 23;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA121).into();
        let enemy_corner_id: UnitInstanceId = Uuid::from_u128(0xA122).into();
        let enemy_far_id: UnitInstanceId = Uuid::from_u128(0xA123).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_corner_id,
            runtime_unit(enemy_corner_id, Side::Opponent),
        );
        core.units
            .insert(enemy_far_id, runtime_unit(enemy_far_id, Side::Opponent));

        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_corner_id, Position::new(3, 3))
            .unwrap();
        core.battlefield
            .place(enemy_far_id, Position::new(0, 0))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_corner_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_far_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Box {
                width_units: (TILE_UNITS_PER_TILE * 2) as u32,
                height_units: (TILE_UNITS_PER_TILE * 2) as u32,
            },
            anchor: SkillAreaAnchorSource::CastTarget,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, _, _, targets) = core
            .resolve_instant_area_targets(
                0,
                cast_seq,
                0,
                caster_id,
                Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                    position: Position::new(2, 2),
                }),
                &area,
            )
            .expect("box area target resolution");

        assert!(targets.contains(&enemy_corner_id));
        assert!(!targets.contains(&enemy_far_id));
    }

    #[test]
    fn instant_line_area_delivery_uses_caster_to_anchor_direction() {
        let mut core = new_core();
        let cast_seq = 24;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA131).into();
        let enemy_on_line_front_id: UnitInstanceId = Uuid::from_u128(0xA132).into();
        let enemy_on_line_back_id: UnitInstanceId = Uuid::from_u128(0xA133).into();
        let enemy_off_line_id: UnitInstanceId = Uuid::from_u128(0xA134).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_on_line_front_id,
            runtime_unit(enemy_on_line_front_id, Side::Opponent),
        );
        core.units.insert(
            enemy_on_line_back_id,
            runtime_unit(enemy_on_line_back_id, Side::Opponent),
        );
        core.units.insert(
            enemy_off_line_id,
            runtime_unit(enemy_off_line_id, Side::Opponent),
        );

        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_on_line_front_id, Position::new(2, 1))
            .unwrap();
        core.battlefield
            .place(enemy_on_line_back_id, Position::new(3, 1))
            .unwrap();
        core.battlefield
            .place(enemy_off_line_id, Position::new(3, 2))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_on_line_front_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_on_line_back_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_off_line_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Line {
                length_units: (TILE_UNITS_PER_TILE * 3) as u32,
            },
            anchor: SkillAreaAnchorSource::CastTarget,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, _, _, targets) = core
            .resolve_instant_area_targets(
                0,
                cast_seq,
                0,
                caster_id,
                Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                    position: Position::new(4, 1),
                }),
                &area,
            )
            .expect("line area target resolution");

        assert!(targets.contains(&enemy_on_line_front_id));
        assert!(targets.contains(&enemy_on_line_back_id));
        assert!(!targets.contains(&enemy_off_line_id));
    }

    #[test]
    fn instant_cone_area_delivery_uses_caster_to_anchor_direction() {
        let mut core = new_core();
        let cast_seq = 22;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA111).into();
        let enemy_front_id: UnitInstanceId = Uuid::from_u128(0xA112).into();
        let enemy_diagonal_id: UnitInstanceId = Uuid::from_u128(0xA113).into();
        let enemy_off_cone_id: UnitInstanceId = Uuid::from_u128(0xA114).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(enemy_front_id, runtime_unit(enemy_front_id, Side::Opponent));
        core.units.insert(
            enemy_diagonal_id,
            runtime_unit(enemy_diagonal_id, Side::Opponent),
        );
        core.units.insert(
            enemy_off_cone_id,
            runtime_unit(enemy_off_cone_id, Side::Opponent),
        );

        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_front_id, Position::new(3, 1))
            .unwrap();
        core.battlefield
            .place(enemy_diagonal_id, Position::new(3, 2))
            .unwrap();
        core.battlefield
            .place(enemy_off_cone_id, Position::new(1, 3))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_front_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_diagonal_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_off_cone_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Cone {
                angle_degrees: 70,
                length_units: (TILE_UNITS_PER_TILE * 3) as u32,
            },
            anchor: SkillAreaAnchorSource::CastTarget,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, _, _, targets) = core
            .resolve_instant_area_targets(
                0,
                cast_seq,
                0,
                caster_id,
                Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                    position: Position::new(4, 1),
                }),
                &area,
            )
            .expect("cone area target resolution");

        assert!(targets.contains(&enemy_front_id));
        assert!(targets.contains(&enemy_diagonal_id));
        assert!(!targets.contains(&enemy_off_cone_id));
    }

    #[test]
    fn impact_context_start_line_area_begins_at_the_impact_point() {
        let mut core = new_core();
        let cast_seq = 25;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA141).into();
        let enemy_at_impact_id: UnitInstanceId = Uuid::from_u128(0xA142).into();
        let enemy_beyond_impact_id: UnitInstanceId = Uuid::from_u128(0xA143).into();
        let enemy_between_caster_and_impact_id: UnitInstanceId = Uuid::from_u128(0xA144).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_at_impact_id,
            runtime_unit(enemy_at_impact_id, Side::Opponent),
        );
        core.units.insert(
            enemy_beyond_impact_id,
            runtime_unit(enemy_beyond_impact_id, Side::Opponent),
        );
        core.units.insert(
            enemy_between_caster_and_impact_id,
            runtime_unit(enemy_between_caster_and_impact_id, Side::Opponent),
        );

        core.battlefield
            .place(caster_id, Position::new(0, 1))
            .unwrap();
        core.battlefield
            .place(enemy_between_caster_and_impact_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_at_impact_id, Position::new(2, 1))
            .unwrap();
        core.battlefield
            .place(enemy_beyond_impact_id, Position::new(3, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_between_caster_and_impact_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_at_impact_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_beyond_impact_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
                caster_owner: Side::Player,
                anchor_position: Position::new(0, 1),
                cast_target_anchor_position: None,
                cast_target_anchor_world_position: None,
                allow_dead_caster: false,
                total_steps: 1,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: std::iter::once((
                    0usize,
                    super::types::SkillImpactContext {
                        delivery_id: Uuid::from_u128(0xA145),
                        impact_time_ms: 0,
                        impact_position: WorldVec2::from_tile_center(Position::new(2, 1)),
                        direction_hint: Some(WorldVec2::from_tile_center(Position::new(4, 1))),
                        first_hit_unit_id: Some(enemy_at_impact_id),
                        hit_unit_ids: vec![enemy_at_impact_id],
                        spawned_area_id: None,
                    },
                ))
                .collect(),
                last_impact_context: Some(super::types::SkillImpactContext {
                    delivery_id: Uuid::from_u128(0xA145),
                    impact_time_ms: 0,
                    impact_position: WorldVec2::from_tile_center(Position::new(2, 1)),
                    direction_hint: Some(WorldVec2::from_tile_center(Position::new(4, 1))),
                    first_hit_unit_id: Some(enemy_at_impact_id),
                    hit_unit_ids: vec![enemy_at_impact_id],
                    spawned_area_id: None,
                }),
                active_area_ids: Vec::new(),
            },
        );

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Line {
                length_units: (TILE_UNITS_PER_TILE * 2) as u32,
            },
            anchor: SkillAreaAnchorSource::ImpactContextStart,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, _, _, targets) = core
            .resolve_instant_area_targets(0, cast_seq, 1, caster_id, None, &area)
            .expect("impact-start line area target resolution");

        assert!(targets.contains(&enemy_at_impact_id));
        assert!(targets.contains(&enemy_beyond_impact_id));
        assert!(!targets.contains(&enemy_between_caster_and_impact_id));
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
    fn impact_context_start_line_area_uses_stored_impact_direction_not_live_step_target() {
        let mut core = new_core();
        let cast_seq = 27;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA161).into();
        let enemy_at_impact_id: UnitInstanceId = Uuid::from_u128(0xA162).into();
        let enemy_beyond_impact_id: UnitInstanceId = Uuid::from_u128(0xA163).into();
        let misleading_live_target_id: UnitInstanceId = Uuid::from_u128(0xA164).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_at_impact_id,
            runtime_unit(enemy_at_impact_id, Side::Opponent),
        );
        core.units.insert(
            enemy_beyond_impact_id,
            runtime_unit(enemy_beyond_impact_id, Side::Opponent),
        );
        core.units.insert(
            misleading_live_target_id,
            runtime_unit(misleading_live_target_id, Side::Opponent),
        );

        core.battlefield
            .place(caster_id, Position::new(0, 1))
            .unwrap();
        core.battlefield
            .place(enemy_at_impact_id, Position::new(2, 1))
            .unwrap();
        core.battlefield
            .place(enemy_beyond_impact_id, Position::new(3, 1))
            .unwrap();
        core.battlefield
            .place(misleading_live_target_id, Position::new(2, 2))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_at_impact_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_beyond_impact_id);
        sync_unit_to_battlefield_tile_center(&mut core, misleading_live_target_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
                caster_owner: Side::Player,
                anchor_position: Position::new(0, 1),
                cast_target_anchor_position: None,
                cast_target_anchor_world_position: None,
                allow_dead_caster: false,
                total_steps: 2,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: std::iter::once((
                    0usize,
                    super::types::SkillImpactContext {
                        delivery_id: Uuid::from_u128(0xA165),
                        impact_time_ms: 0,
                        impact_position: WorldVec2::from_tile_center(Position::new(2, 1)),
                        direction_hint: Some(WorldVec2::from_tile_center(Position::new(4, 1))),
                        first_hit_unit_id: Some(enemy_at_impact_id),
                        hit_unit_ids: vec![enemy_at_impact_id],
                        spawned_area_id: None,
                    },
                ))
                .collect(),
                last_impact_context: None,
                active_area_ids: Vec::new(),
            },
        );

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Line {
                length_units: (TILE_UNITS_PER_TILE * 2) as u32,
            },
            anchor: SkillAreaAnchorSource::ImpactContextStart,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, _, _, targets) = core
            .resolve_instant_area_targets(
                0,
                cast_seq,
                1,
                caster_id,
                Some(crate::game::battle::timeline::SkillCastTarget::Unit {
                    unit_instance_id: misleading_live_target_id,
                }),
                &area,
            )
            .expect("impact-start line area target resolution");

        assert!(targets.contains(&enemy_at_impact_id));
        assert!(targets.contains(&enemy_beyond_impact_id));
        assert!(!targets.contains(&misleading_live_target_id));
    }

    #[test]
    fn instant_area_delivery_can_anchor_on_previous_impact_context() {
        let mut core = new_core();
        let cast_seq = 3;
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA021).into();
        let enemy_near_impact_id: UnitInstanceId = Uuid::from_u128(0xA022).into();
        let enemy_near_cast_target_id: UnitInstanceId = Uuid::from_u128(0xA023).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units.insert(
            enemy_near_impact_id,
            runtime_unit(enemy_near_impact_id, Side::Opponent),
        );
        core.units.insert(
            enemy_near_cast_target_id,
            runtime_unit(enemy_near_cast_target_id, Side::Opponent),
        );

        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_near_impact_id, Position::new(3, 1))
            .unwrap();
        core.battlefield
            .place(enemy_near_cast_target_id, Position::new(1, 3))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_near_impact_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_near_cast_target_id);

        core.active_skill_casts.insert(
            cast_seq,
            ActiveSkillCast {
                caster_instance_id: caster_id,
                caster_owner: Side::Player,
                anchor_position: Position::new(1, 1),
                cast_target_anchor_position: None,
                cast_target_anchor_world_position: None,
                allow_dead_caster: false,
                total_steps: 1,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: std::iter::once((
                    0usize,
                    super::types::SkillImpactContext {
                        delivery_id: Uuid::from_u128(0xA024),
                        impact_time_ms: 0,
                        impact_position: WorldVec2::from_tile_center(Position::new(3, 1)),
                        direction_hint: Some(WorldVec2::from_tile_center(Position::new(5, 1))),
                        first_hit_unit_id: Some(enemy_near_impact_id),
                        hit_unit_ids: vec![enemy_near_impact_id],
                        spawned_area_id: None,
                    },
                ))
                .collect(),
                last_impact_context: Some(super::types::SkillImpactContext {
                    delivery_id: Uuid::from_u128(0xA024),
                    impact_time_ms: 0,
                    impact_position: WorldVec2::from_tile_center(Position::new(3, 1)),
                    direction_hint: Some(WorldVec2::from_tile_center(Position::new(5, 1))),
                    first_hit_unit_id: Some(enemy_near_impact_id),
                    hit_unit_ids: vec![enemy_near_impact_id],
                    spawned_area_id: None,
                }),
                active_area_ids: Vec::new(),
            },
        );

        let area = SkillAreaDeliveryDef {
            shape: SkillAreaShapeDef::Circle {
                radius_units: (TILE_UNITS_PER_TILE / 2) as u32,
            },
            anchor: SkillAreaAnchorSource::ImpactContext,
            tracking: Default::default(),
            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
            include_caster: false,
            tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
            duration_ms: 0,
            tick_interval_ms: None,
        };

        let (_, center, _, targets) = core
            .resolve_instant_area_targets(
                0,
                cast_seq,
                1,
                caster_id,
                Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                    position: Position::new(1, 3),
                }),
                &area,
            )
            .expect("area target resolution");

        assert_eq!(center, WorldVec2::from_tile_center(Position::new(3, 1)));
        assert!(targets.contains(&enemy_near_impact_id));
        assert!(!targets.contains(&enemy_near_cast_target_id));
    }

    #[test]
    fn persistent_area_ticks_immediately_then_repeats_until_expire() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA101).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(0xA102).into();
        let skill = single_step_skill(
            "persistent_area",
            SkillTarget::CastTarget,
            3,
            0,
            DeliveryDef::Area {
                area: SkillAreaDeliveryDef {
                    shape: SkillAreaShapeDef::Circle {
                        radius_units: TILE_UNITS_PER_TILE as u32,
                    },
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tracking: Default::default(),
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 1_000,
                    tick_interval_ms: Some(500),
                },
            },
            vec![SkillEffectDef::Damage {
                amount: 2,
                damage_type: crate::game::battle::damage::DamageType::Magic,
            }],
        );

        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_id, Position::new(2, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_id);
        core.active_skill_casts.insert(
            10,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let DeliveryDef::Area { area } = skill.first_step().unwrap().delivery else {
            panic!("expected area delivery");
        };
        core.register_persistent_area(
            0,
            10,
            0,
            caster_id,
            skill.id.clone(),
            "step_01".to_string(),
            Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                position: Position::new(2, 1),
            }),
            area,
        );

        let area_id = *core.active_areas.keys().next().expect("area should exist");
        core.apply_skill_area_tick(0, area_id);
        assert_eq!(core.units.get(&enemy_id).unwrap().stats.current_health, 8);

        let area_runtime = core.active_areas.get(&area_id).unwrap();
        assert_eq!(area_runtime.next_tick_ms, Some(500));
        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::SkillAreaTick { time_ms: 500, area_id: queued_id, .. } if *queued_id == area_id
        )));

        core.apply_skill_area_tick(500, area_id);
        assert_eq!(core.units.get(&enemy_id).unwrap().stats.current_health, 6);
        assert!(!core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::SkillAreaTick { time_ms: 1000, area_id: queued_id, .. } if *queued_id == area_id
        )));

        core.expire_skill_area(area_id);
        assert!(!core.active_areas.contains_key(&area_id));
    }

    #[test]
    fn persistent_area_keeps_ticking_after_the_caster_dies() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA109).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(0xA10A).into();
        let skill = single_step_skill(
            "persistent_area_after_death",
            SkillTarget::CastTarget,
            3,
            0,
            DeliveryDef::Area {
                area: SkillAreaDeliveryDef {
                    shape: SkillAreaShapeDef::Circle {
                        radius_units: TILE_UNITS_PER_TILE as u32,
                    },
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tracking: Default::default(),
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 1_000,
                    tick_interval_ms: Some(500),
                },
            },
            vec![SkillEffectDef::Damage {
                amount: 2,
                damage_type: crate::game::battle::damage::DamageType::Magic,
            }],
        );

        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_id, Position::new(2, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_id);
        core.active_skill_casts.insert(
            12,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let DeliveryDef::Area { area } = skill.first_step().unwrap().delivery else {
            panic!("expected area delivery");
        };
        core.register_persistent_area(
            0,
            12,
            0,
            caster_id,
            skill.id.clone(),
            "step_01".to_string(),
            Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                position: Position::new(2, 1),
            }),
            area,
        );

        let area_id = *core.active_areas.keys().next().expect("area should exist");
        core.apply_skill_area_tick(0, area_id);
        assert_eq!(core.units.get(&enemy_id).unwrap().stats.current_health, 8);

        core.units.get_mut(&caster_id).unwrap().stats.current_health = 0;

        core.apply_skill_area_tick(500, area_id);
        assert_eq!(
            core.units.get(&enemy_id).unwrap().stats.current_health,
            6,
            "persistent area should continue ticking after the original caster dies"
        );
    }

    #[test]
    fn persistent_area_once_per_area_hits_target_only_once() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA111).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(0xA112).into();
        let skill = single_step_skill(
            "persistent_area_once",
            SkillTarget::CastTarget,
            3,
            0,
            DeliveryDef::Area {
                area: SkillAreaDeliveryDef {
                    shape: SkillAreaShapeDef::Circle {
                        radius_units: TILE_UNITS_PER_TILE as u32,
                    },
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tracking: Default::default(),
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::OncePerArea,
                    duration_ms: 1_000,
                    tick_interval_ms: Some(500),
                },
            },
            vec![SkillEffectDef::Damage {
                amount: 2,
                damage_type: crate::game::battle::damage::DamageType::Magic,
            }],
        );

        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_id, Position::new(2, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_id);
        core.active_skill_casts.insert(
            11,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let DeliveryDef::Area { area } = skill.first_step().unwrap().delivery else {
            panic!("expected area delivery");
        };
        core.register_persistent_area(
            0,
            11,
            0,
            caster_id,
            skill.id.clone(),
            "step_01".to_string(),
            Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                position: Position::new(2, 1),
            }),
            area,
        );

        let area_id = *core.active_areas.keys().next().expect("area should exist");
        core.apply_skill_area_tick(0, area_id);
        assert_eq!(core.units.get(&enemy_id).unwrap().stats.current_health, 8);

        core.apply_skill_area_tick(500, area_id);
        assert_eq!(
            core.units.get(&enemy_id).unwrap().stats.current_health,
            8,
            "OncePerArea should not re-hit the same unit on later ticks"
        );
    }

    #[test]
    fn persistent_area_on_enter_only_hits_when_unit_enters_zone() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA121).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(0xA122).into();
        let skill = single_step_skill(
            "persistent_area_on_enter",
            SkillTarget::CastTarget,
            3,
            0,
            DeliveryDef::Area {
                area: SkillAreaDeliveryDef {
                    shape: SkillAreaShapeDef::Circle {
                        radius_units: TILE_UNITS_PER_TILE as u32,
                    },
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tracking: Default::default(),
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::OnEnter,
                    duration_ms: 1_000,
                    tick_interval_ms: Some(500),
                },
            },
            vec![SkillEffectDef::Damage {
                amount: 2,
                damage_type: crate::game::battle::damage::DamageType::Magic,
            }],
        );

        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        core.battlefield
            .place(enemy_id, Position::new(4, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, enemy_id);
        core.active_skill_casts.insert(
            12,
            ActiveSkillCast {
                caster_instance_id: caster_id,
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

        let DeliveryDef::Area { area } = skill.first_step().unwrap().delivery else {
            panic!("expected area delivery");
        };
        core.register_persistent_area(
            0,
            12,
            0,
            caster_id,
            skill.id.clone(),
            "step_01".to_string(),
            Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                position: Position::new(2, 1),
            }),
            area,
        );

        let area_id = *core.active_areas.keys().next().expect("area should exist");
        core.apply_skill_area_tick(0, area_id);
        assert_eq!(
            core.units.get(&enemy_id).unwrap().stats.current_health,
            10,
            "unit outside the zone should not be hit on initial tick"
        );

        core.battlefield.remove(enemy_id);
        core.battlefield
            .place(enemy_id, Position::new(2, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, enemy_id);

        core.apply_skill_area_tick(500, area_id);
        assert_eq!(
            core.units.get(&enemy_id).unwrap().stats.current_health,
            8,
            "unit should be hit once when entering the zone"
        );

        core.apply_skill_area_tick(1000, area_id);
        assert_eq!(
            core.units.get(&enemy_id).unwrap().stats.current_health,
            8,
            "OnEnter should not re-hit while the unit remains inside the zone"
        );
    }

    #[test]
    fn targeted_execute_keeps_locked_target_when_target_leaves_range() {
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
        core.battlefield.place(caster_id, caster_pos).unwrap();
        core.battlefield
            .place(locked_target_id, Position::new(1, 0))
            .unwrap();
        core.battlefield
            .place(other_target_id, Position::new(1, 1))
            .unwrap();

        let skill = single_step_skill(
            "locked_target_execute",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            50,
            DeliveryDef::Instant,
            vec![],
        );

        let start_target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos);
        core.battlefield
            .move_unit(locked_target_id, Position::new(3, 3))
            .unwrap();

        let targets = core.resolve_skill_step_targets(
            0,
            caster_id,
            skill.first_step().unwrap(),
            start_target,
        );
        assert_eq!(targets, vec![locked_target_id]);
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
        core.battlefield.place(caster_id, caster_pos).unwrap();
        core.battlefield
            .place(locked_target_id, Position::new(1, 0))
            .unwrap();
        core.battlefield
            .place(other_target_id, Position::new(1, 1))
            .unwrap();

        let skill = single_step_skill(
            "locked_target_death",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            50,
            DeliveryDef::Instant,
            vec![],
        );

        let start_target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos);
        core.battlefield.remove(locked_target_id);
        core.units
            .get_mut(&locked_target_id)
            .unwrap()
            .stats
            .current_health = 0;

        let targets = core.resolve_skill_step_targets(
            0,
            caster_id,
            skill.first_step().unwrap(),
            start_target,
        );
        assert!(targets.is_empty());
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
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
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
                cause: TimelineCause::default(),
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
                cause: TimelineCause::default(),
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
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(82).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.battlefield
            .place(caster_id, Position::new(0, 0))
            .unwrap();

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: TimelineCause::default(),
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
                cause: TimelineCause::default(),
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
            .timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::AbilityCast { .. })));

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

        assert!(core.timeline.entries.iter().any(|entry| {
            entry.time_ms == 12
                && matches!(
                    entry.event,
                    TimelineEvent::AbilityCast {
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
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(83).into();
        let target_id: UnitInstanceId = Uuid::from_u128(84).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.stats.attack_interval_ms = 10;
        core.units.insert(caster_id, caster);
        core.battlefield
            .place(caster_id, Position::new(0, 0))
            .unwrap();

        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: TimelineCause::default(),
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
                cause: TimelineCause::Root {
                    kind: crate::game::battle::timeline::TimelineRootCause::Period,
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
        assert!(!core.timeline.entries.iter().any(|entry| {
            entry.time_ms == 5
                && matches!(
                    entry.event,
                    TimelineEvent::AttackStart {
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
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            None,
            TimelineCause::default(),
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

    #[test]
    fn persistent_area_skill_cast_is_cleaned_up_after_expire() {
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA211).into();
        let skill = single_step_skill(
            "cleanup_area",
            SkillTarget::CastTarget,
            3,
            0,
            DeliveryDef::Area {
                area: SkillAreaDeliveryDef {
                    shape: SkillAreaShapeDef::Circle {
                        radius_units: TILE_UNITS_PER_TILE as u32,
                    },
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tracking: Default::default(),
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 100,
                    tick_interval_ms: Some(50),
                },
            },
            vec![SkillEffectDef::Damage {
                amount: 2,
                damage_type: crate::game::battle::damage::DamageType::Magic,
            }],
        );

        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.battlefield
            .place(caster_id, Position::new(1, 1))
            .unwrap();
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(crate::game::battle::timeline::SkillCastTarget::Tile {
                position: Position::new(2, 1),
            }),
            TimelineCause::default(),
            false,
        );

        let step_event = core.event_queue.pop().expect("missing initial skill step");
        let cast_seq = match &step_event {
            BattleEvent::SkillStep { cast_seq, .. } => *cast_seq,
            other => panic!("expected SkillStep, got {other:?}"),
        };
        core.process_event(step_event, 0)
            .expect("process area step");

        assert!(
            core.active_skill_casts.contains_key(&cast_seq),
            "persistent area cast must remain active until the area fully expires"
        );

        let area_id = *core
            .active_areas
            .keys()
            .next()
            .expect("active area should exist");
        core.expire_skill_area(area_id);

        assert!(
            !core.active_skill_casts.contains_key(&cast_seq),
            "cast should be removed after its final active area expires"
        );
    }
}
