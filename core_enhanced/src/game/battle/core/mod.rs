use crate::owned_collections::{MapCloneHistory, OwnedHashMap};
use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::game::{
    battle::{
        battlefield::BattlefieldLayout,
        buffs::BuffControlCapability,
        core::movement::{
            types::{ActiveMovementSegment, UnitBody},
            RapierMovementWorld,
        },
        enums::BattleEvent,
        event_log::{BattleEventCause, BattleEventLog, BattleLogEvent},
        ids::UnitInstanceId,
        result_stats::BattleResultStatsAccumulator,
        scenario::{BattleScenario, ScenarioGroupId, ScenarioUnitRef},
        types::{BattleUnitSource, UnitSnapshot},
    },
    behavior::ActionDisabledReasonCode,
    data::GameDataBase,
    resources::Position,
};

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
use crate::game::{
    battle::core::movement::types::{EventLogVec2, WorldVec2},
    data::ValidatedGameData,
};

pub(crate) mod amount_observation;
pub(crate) mod authoring_observation;
mod basic_attack;
mod basic_attack_projectile;
mod basic_heal;
pub(crate) mod boss_runtime;
pub mod build;
pub mod commands;
mod corrosion;
mod damage_runtime;
mod death_runtime;
mod effective_delivery;
pub(crate) mod effective_skill;
mod field_work;
pub mod ids;
mod independent_objects;
pub mod movement;
mod object_attack;
mod object_creation;
mod object_evolution;
mod object_query;
mod projectile_math;
mod queue;
pub(crate) mod relic_numeric;
mod resonance_runtime;
mod resource_runtime;
pub(crate) mod runtime_facts;
pub mod sim;
pub mod skill_runtime;
pub mod spatial;
mod spawn_kernel;
mod summon_runtime;
mod sustained;
mod tactical_relocation;
mod tactical_stance;
mod tactical_swap;
mod target_usefulness;
pub mod targeting;
pub mod triggers;
pub mod types;
pub(crate) mod value_expr_runtime;
mod wave_runtime;

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
use self::types::ActiveSkillCastDebugState;
use self::types::{
    AbilityProcKey, AbilityProcState, ActiveBuff, ActiveProjectileRuntime, ActiveSkillCast,
    AreaRuntime, BuffInstanceKey, ProjectileRecord, RuntimeItem, RuntimeUnit, RuntimeUnitLifecycle,
    UnitControlState, UnitTacticalState,
};

pub use self::field_work::FieldWorkObjectStatus;
#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
pub use self::field_work::{FieldWorkCellSnapshot, FieldWorkObjectSnapshot};
pub use self::spawn_kernel::{
    CompendiumCandidate, CompendiumCandidateKind, RemovalEventId, SpawnCancellationReason,
    SpawnKernelError, SpawnOccupancyClass, SpawnOccupancyPolicy, SpawnOccupantKind, SpawnPlacement,
    SpawnRejection, SpawnRequest, SpawnRequestId, SpawnRequestSettlement, SpawnRequestSource,
    SpawnTriggerId,
};
#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
pub use self::spawn_kernel::{CompendiumCandidateError, SpawnPolicyError};
pub use self::tactical_stance::{TacticalTileEffect, UnitTacticalSnapshot};
pub use independent_objects::{ObjectRemovalReason, ObjectRemovalReceipt};
pub use object_evolution::ObjectEvolutionReceipt;
pub use object_query::{
    IndependentObjectView, IndependentObjectsSnapshot, ObjectCreationReceipt, ObjectKindView,
    ObjectPlacementPreview, ObjectSourceView, SegmentWallCostPreview,
};

#[derive(Clone)]
pub struct BattleCore {
    sustained: sustained::SustainedState,
    corrosion_freeze: Option<corrosion::CorrosionFreeze>,
    runtime_facts: runtime_facts::RuntimeFactState,
    event_queue: queue::EventQueue,

    scenario: Arc<BattleScenario>,
    scenario_fact_binding_extent: Option<usize>,
    relic_numeric_owner: Option<Arc<relic_numeric::RelicNumericOwner>>,
    effective_owner: Option<Arc<effective_skill::EffectiveBattleOwner>>,
    scenario_runtime: ScenarioRuntimeState,
    spawn_kernel: spawn_kernel::SpawnKernelState,
    summon_runtime: summon_runtime::SummonRuntimeState,
    independent_objects: independent_objects::IndependentObjectState,
    boss_runtime: Option<boss_runtime::BossEncounterRuntime>,

    pub units: HashMap<UnitInstanceId, RuntimeUnit>,
    // Covers the closed application-owned construction path. The existing
    // public fixture map remains unwrapped; arbitrary external mutations are
    // not admissible provenance for an opaque authoring session.
    units_clone_history: MapCloneHistory,
    tactical_states: OwnedHashMap<UnitInstanceId, UnitTacticalState>,
    items: OwnedHashMap<Uuid, RuntimeItem>,
    graveyard: OwnedHashMap<UnitInstanceId, UnitSnapshot>,

    buffs: OwnedHashMap<BuffInstanceKey, ActiveBuff>,
    active_skill_casts: OwnedHashMap<u64, ActiveSkillCast>,
    ability_proc_states: OwnedHashMap<AbilityProcKey, AbilityProcState>,

    projectiles: OwnedHashMap<Uuid, ProjectileRecord>,
    active_projectiles: OwnedHashMap<Uuid, ActiveProjectileRuntime>,
    active_areas: OwnedHashMap<Uuid, AreaRuntime>,
    active_movement_segments: OwnedHashMap<UnitInstanceId, ActiveMovementSegment>,
    field_work_objects: std::collections::BTreeMap<
        crate::game::battle::scenario::FieldWorkObjectId,
        field_work::FieldWorkObjectRuntime,
    >,
    block_state: BlockRuntimeState,
    last_continuous_movement_tick_ms: Option<u64>,
    movement_backend: Arc<RapierMovementWorld>,
    pub battlefield: BattlefieldLayout,
    live_signals: Vec<sim::BattleLiveSignal>,

    pub(crate) game_data: Arc<GameDataBase>,
    /// Exact immutable publication generation. Pre-publication feasibility
    /// candidates intentionally have no generation authority and cannot create
    /// a runtime value-expression capture.
    content_identity: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,

    pub event_log: BattleEventLog,
    pub event_log_seq: u64,
    pub(super) result_stats_accumulator: BattleResultStatsAccumulator,
    pub skill_cast_seq: u64,
    pub projectile_seq: u64,
    pub area_seq: u64,
    pub damage_source_seq: u64,
    pub seed: u64,
    pub recording_cause_stack: Vec<BattleEventCause>,
    pub recording_source_command_stack: Vec<String>,
    pub(super) authoring_observation: Option<authoring_observation::MandatoryObservation>,
    pub(crate) authoring_trace: crate::authoring::trace::OptionalTrace,
    pub(super) authoring_direct_source:
        Option<Arc<crate::game::battle::skill_source::SkillEffectExecutionSource>>,
    runtime_strike_scope: Option<runtime_facts::StrikeScope>,
    runtime_effect_projection: Option<value_expr_runtime::EffectResultAccumulator>,
    pub(super) runtime_fact_source: crate::game::battle::runtime_facts::RuntimeFactSource,
    pub(super) recording_skill_effect_source:
        Option<Arc<crate::game::battle::skill_source::SkillEffectExecutionSource>>,
    pub(super) recording_skill_proc_origin:
        Option<crate::game::battle::skill_source::SkillProcOrigin>,
}

impl crate::authoring::clone_shape::CloneShape for BattleCore {
    fn preflight_native_clone(&self) -> Result<(), crate::game::behavior::GameError> {
        self.preflight_runtime_fact_clone()
    }
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        let Self {
            sustained,
            corrosion_freeze,
            runtime_strike_scope,
            runtime_effect_projection,
            runtime_facts,
            event_queue,
            scenario,
            scenario_fact_binding_extent,
            relic_numeric_owner,
            effective_owner,
            scenario_runtime,
            spawn_kernel,
            summon_runtime,
            independent_objects,
            boss_runtime,
            units,
            units_clone_history,
            tactical_states,
            items,
            graveyard,
            buffs,
            active_skill_casts,
            ability_proc_states,
            projectiles,
            active_projectiles,
            active_areas,
            active_movement_segments,
            field_work_objects,
            block_state,
            last_continuous_movement_tick_ms,
            movement_backend,
            battlefield,
            live_signals,
            game_data,
            content_identity,
            event_log,
            event_log_seq,
            result_stats_accumulator,
            skill_cast_seq,
            projectile_seq,
            area_seq,
            damage_source_seq,
            seed,
            recording_cause_stack,
            recording_source_command_stack,
            authoring_observation,
            authoring_trace,
            authoring_direct_source,
            runtime_fact_source,
            recording_skill_effect_source,
            recording_skill_proc_origin,
        } = self;
        meter.scan_storage(Some(1))?;
        sustained.visit_clone_shape(meter)?;
        corrosion_freeze.visit_clone_shape(meter)?;
        runtime_strike_scope.visit_clone_shape(meter)?;
        runtime_effect_projection.visit_clone_shape(meter)?;
        runtime_facts.visit_clone_shape(meter)?;
        event_queue.visit_clone_shape(meter)?;
        scenario.visit_clone_shape(meter)?;
        scenario_fact_binding_extent.visit_clone_shape(meter)?;
        if let Some(owner) = relic_numeric_owner {
            owner.visit_clone_shape(meter)?;
        }
        effective_owner.visit_clone_shape(meter)?;
        scenario_runtime.visit_clone_shape(meter)?;
        spawn_kernel.visit_clone_shape(meter)?;
        summon_runtime.visit_clone_shape(meter)?;
        independent_objects.visit_clone_shape(meter)?;
        boss_runtime.visit_clone_shape(meter)?;
        // Public fixture units retain their concrete native API; only the closed
        // application construction/mutation history qualifies for authoring.
        meter.scan_storage(
            units_clone_history
                .bucket_bound()
                .and_then(|n| n.checked_add(2)),
        )?;
        for (id, unit) in units {
            id.visit_clone_shape(meter)?;
            unit.visit_clone_shape(meter)?;
        }
        tactical_states.visit_clone_shape(meter)?;
        items.visit_clone_shape(meter)?;
        graveyard.visit_clone_shape(meter)?;
        buffs.visit_clone_shape(meter)?;
        active_skill_casts.visit_clone_shape(meter)?;
        ability_proc_states.visit_clone_shape(meter)?;
        projectiles.visit_clone_shape(meter)?;
        active_projectiles.visit_clone_shape(meter)?;
        active_areas.visit_clone_shape(meter)?;
        active_movement_segments.visit_clone_shape(meter)?;
        field_work_objects.visit_clone_shape(meter)?;
        block_state.visit_clone_shape(meter)?;
        last_continuous_movement_tick_ms.visit_clone_shape(meter)?;
        movement_backend.visit_clone_shape(meter)?;
        battlefield.visit_clone_shape(meter)?;
        live_signals.visit_clone_shape(meter)?;
        game_data.visit_clone_shape(meter)?;
        content_identity.visit_clone_shape(meter)?;
        event_log.visit_clone_shape(meter)?;
        event_log_seq.visit_clone_shape(meter)?;
        result_stats_accumulator.visit_clone_shape(meter)?;
        skill_cast_seq.visit_clone_shape(meter)?;
        projectile_seq.visit_clone_shape(meter)?;
        area_seq.visit_clone_shape(meter)?;
        damage_source_seq.visit_clone_shape(meter)?;
        seed.visit_clone_shape(meter)?;
        recording_cause_stack.visit_clone_shape(meter)?;
        recording_source_command_stack.visit_clone_shape(meter)?;
        authoring_observation.visit_clone_shape(meter)?;
        authoring_trace.visit_clone_shape(meter)?;
        authoring_direct_source.visit_clone_shape(meter)?;
        runtime_fact_source.visit_clone_shape(meter)?;
        recording_skill_effect_source.visit_clone_shape(meter)?;
        recording_skill_proc_origin.visit_clone_shape(meter)
    }
}

#[cfg(test)]
impl BattleCore {
    pub(crate) fn event_queue_debug_snapshot(&self) -> Vec<String> {
        let mut events = self.event_queue.clone().into_vec();
        events.sort_by(|left, right| right.cmp(left));
        events
            .into_iter()
            .map(|event| format!("{event:?}"))
            .collect()
    }

    pub(crate) fn transaction_debug_snapshot(&self) -> String {
        format!(
            "sustained={:?};corrosion_freeze={:?};scenario_runtime={:?};spawn_kernel={:?};independent_objects={:?};boss_runtime={:?};units={:?};tactical_states={:?};items={:?};graveyard={:?};buffs={:?};active_skill_casts={:?};ability_proc_states={:?};projectiles={:?};active_projectiles={:?};active_areas={:?};active_movement_segments={:?};field_work_objects={:?};block_state={:?};last_movement_tick={:?};result_stats={:?};event_log_seq={};skill_cast_seq={};projectile_seq={};area_seq={};damage_source_seq={};recording_causes={:?};recording_commands={:?}",
            self.sustained,
            self.corrosion_freeze,
            self.scenario_runtime,
            self.spawn_kernel,
            self.independent_objects,
            self.boss_runtime,
            self.units,
            self.tactical_states,
            self.items,
            self.graveyard,
            self.buffs,
            self.active_skill_casts,
            self.ability_proc_states,
            self.projectiles,
            self.active_projectiles,
            self.active_areas,
            self.active_movement_segments,
            self.field_work_objects,
            self.block_state,
            self.last_continuous_movement_tick_ms,
            self.result_stats_accumulator,
            self.event_log_seq,
            self.skill_cast_seq,
            self.projectile_seq,
            self.area_seq,
            self.damage_source_seq,
            self.recording_cause_stack,
            self.recording_source_command_stack,
        )
    }
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
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
    spawned: Arc<ScenarioSpawnRegistry>,
    next_spawn_order: u64,
    wave_runtime: wave_runtime::WaveRuntimeState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(in crate::game::battle::core) struct BlockRuntimeState {
    blocker_to_enemies: OwnedHashMap<UnitInstanceId, Vec<UnitInstanceId>>,
    enemy_to_blocker: OwnedHashMap<UnitInstanceId, UnitInstanceId>,
}

#[derive(Debug, Clone, Default)]
struct ScenarioSpawnRegistry {
    spawned_groups: std::collections::HashSet<ScenarioGroupId>,
    unit_refs: OwnedHashMap<ScenarioUnitRef, UnitInstanceId>,
}
// Spawned groups are fresh/insert-only within the closed application owner.
crate::authoring::clone_shape::struct_shape_with_sets!(ScenarioSpawnRegistry {
    unit_refs,
} sets { spawned_groups });
crate::authoring::clone_shape::struct_shape!(ScenarioRuntimeState {
    spawned,
    next_spawn_order,
    wave_runtime
});
impl ScenarioRuntimeState {
    fn spawned_for_write(
        &mut self,
    ) -> Result<&mut ScenarioSpawnRegistry, crate::game::behavior::GameError> {
        if Arc::get_mut(&mut self.spawned).is_none() {
            self.spawned = Arc::new(crate::authoring::clone_shape::controlled_clone(
                self.spawned.as_ref(),
            )?);
        }
        Ok(Arc::get_mut(&mut self.spawned).expect("spawn registry has a unique writer"))
    }
}
crate::authoring::clone_shape::struct_shape!(BlockRuntimeState {
    blocker_to_enemies,
    enemy_to_blocker
});

impl BattleCore {
    /// Caller has admitted this copy and proved unchanged bucket lineage.
    /// Other fields retain their ordinary native Clone semantics.
    fn clone_reusing_timestamp_units(
        &self,
        mut reused: HashMap<UnitInstanceId, RuntimeUnit>,
        history: MapCloneHistory,
    ) -> Self {
        let Self {
            sustained,
            corrosion_freeze,
            runtime_strike_scope,
            runtime_effect_projection,
            runtime_facts,
            event_queue,
            scenario,
            scenario_fact_binding_extent,
            relic_numeric_owner,
            effective_owner,
            scenario_runtime,
            spawn_kernel,
            summon_runtime,
            independent_objects,
            boss_runtime,
            units: source_units,
            units_clone_history: source_units_clone_history,
            tactical_states,
            items,
            graveyard,
            buffs,
            active_skill_casts,
            ability_proc_states,
            projectiles,
            active_projectiles,
            active_areas,
            active_movement_segments,
            field_work_objects,
            block_state,
            last_continuous_movement_tick_ms,
            movement_backend,
            battlefield,
            live_signals,
            game_data,
            content_identity,
            event_log,
            event_log_seq,
            result_stats_accumulator,
            skill_cast_seq,
            projectile_seq,
            area_seq,
            damage_source_seq,
            seed,
            recording_cause_stack,
            recording_source_command_stack,
            authoring_observation,
            authoring_trace,
            authoring_direct_source,
            runtime_fact_source,
            recording_skill_effect_source,
            recording_skill_proc_origin,
        } = self;
        debug_assert!(history.same_unsaturated_generation(source_units_clone_history));
        for ((target_id, target), (source_id, source)) in reused.iter_mut().zip(source_units.iter())
        {
            debug_assert_eq!(target_id, source_id);
            target.reuse_timestamp_value(source);
        }
        Self {
            sustained: sustained.clone(),
            corrosion_freeze: *corrosion_freeze,
            runtime_strike_scope: runtime_strike_scope.clone(),
            runtime_effect_projection: runtime_effect_projection.clone(),
            runtime_facts: runtime_facts.clone(),
            event_queue: event_queue.clone(),
            scenario: scenario.clone(),
            scenario_fact_binding_extent: scenario_fact_binding_extent.clone(),
            relic_numeric_owner: relic_numeric_owner.clone(),
            effective_owner: effective_owner.clone(),
            scenario_runtime: scenario_runtime.clone(),
            spawn_kernel: spawn_kernel.clone(),
            summon_runtime: summon_runtime.clone(),
            independent_objects: independent_objects.clone(),
            boss_runtime: boss_runtime.clone(),
            units: reused,
            units_clone_history: history,
            tactical_states: tactical_states.clone(),
            items: items.clone(),
            graveyard: graveyard.clone(),
            buffs: buffs.clone(),
            active_skill_casts: active_skill_casts.clone(),
            ability_proc_states: ability_proc_states.clone(),
            projectiles: projectiles.clone(),
            active_projectiles: active_projectiles.clone(),
            active_areas: active_areas.clone(),
            active_movement_segments: active_movement_segments.clone(),
            field_work_objects: field_work_objects.clone(),
            block_state: block_state.clone(),
            last_continuous_movement_tick_ms: last_continuous_movement_tick_ms.clone(),
            movement_backend: movement_backend.clone(),
            battlefield: battlefield.clone(),
            live_signals: live_signals.clone(),
            game_data: game_data.clone(),
            content_identity: content_identity.clone(),
            event_log: event_log.clone(),
            event_log_seq: event_log_seq.clone(),
            result_stats_accumulator: result_stats_accumulator.clone(),
            skill_cast_seq: skill_cast_seq.clone(),
            projectile_seq: projectile_seq.clone(),
            area_seq: area_seq.clone(),
            damage_source_seq: damage_source_seq.clone(),
            seed: seed.clone(),
            recording_cause_stack: recording_cause_stack.clone(),
            recording_source_command_stack: recording_source_command_stack.clone(),
            authoring_observation: authoring_observation.clone(),
            authoring_trace: authoring_trace.clone(),
            authoring_direct_source: authoring_direct_source.clone(),
            runtime_fact_source: runtime_fact_source.clone(),
            recording_skill_effect_source: recording_skill_effect_source.clone(),
            recording_skill_proc_origin: recording_skill_proc_origin.clone(),
        }
    }

    pub(super) fn preflight_authoring_clone(&self) -> Result<(), crate::game::behavior::GameError> {
        use crate::authoring::{
            clone_shape::CloneShape,
            control::{self, CloneMeter},
        };
        if crate::game::battle::runtime_control::is_active() {
            self.preflight_runtime_fact_clone()?;
        }
        if !control::is_active() {
            return Ok(());
        }
        control::poll("battle_core_clone_entry")?;
        let mut meter = CloneMeter::default();
        self.visit_clone_shape(&mut meter)?;
        meter.finish("battle_core_clone_storage", "battle_core_clone_copy")
    }

    pub(super) fn cloned_skill_definition(
        &self,
        id: impl AsRef<str>,
    ) -> Result<Option<crate::game::ability::SkillDef>, crate::game::behavior::GameError> {
        let Some(skill) = self.game_data.skill_data.get_by_id(id) else {
            return Ok(None);
        };
        crate::authoring::control::charge_serialized(skill, "skill_definition_copy")?;
        Ok(Some(skill.clone()))
    }

    /// Materialized gameplay object/record counts, including retained delivery
    /// hit/context vectors. Byte sizes and backend storage require separate bounds.
    pub(super) fn admit_authoring_runtime_growth(
        &self,
        additional: usize,
        site: &'static str,
    ) -> Result<(), crate::game::behavior::GameError> {
        self.admit_authoring_runtime_replacement(0, additional, site)
    }

    pub(super) fn admit_authoring_runtime_replacement(
        &self,
        removed: usize,
        additional: usize,
        site: &'static str,
    ) -> Result<(), crate::game::behavior::GameError> {
        use crate::authoring::control::{self, Resource};
        if !control::is_active() {
            return Ok(());
        }
        let mut current = 0usize;
        let mut count = |items| {
            control::charge(1, "runtime_shape_scan")?;
            // Saturation is always over the finite hard ceiling; final admission
            // rejects it. Totalling first avoids HashMap traversal-order receipts.
            current = current.saturating_add(items);
            Ok::<(), crate::game::behavior::GameError>(())
        };
        for items in [
            self.units.len(),
            self.tactical_states.len(),
            self.items.len(),
            self.graveyard.len(),
            self.buffs.len(),
            self.sustained.entity_count(),
            self.independent_objects.entity_count(),
            self.spawn_kernel.independent_entity_count()?,
            self.active_skill_casts.len(),
            self.ability_proc_states.len(),
            self.projectiles.len(),
            self.active_projectiles.len(),
            self.active_areas.len(),
            self.active_movement_segments.len(),
            self.field_work_objects.len(),
            self.live_signals.len(),
        ] {
            count(items)?;
        }
        for cast in self.active_skill_casts.values() {
            count(cast.flow.as_ref().map_or(0, |flow| flow.nodes.len()))?;
            count(cast.step_progress.len())?;
            count(cast.deferred_steps.len())?;
            count(cast.active_area_ids.len())?;
            count(cast.impact_contexts_by_step.len())?;
            for context in cast
                .impact_contexts_by_step
                .values()
                .chain(cast.last_impact_context.iter())
            {
                count(context.hit_unit_ids.len())?;
            }
            count(usize::from(cast.last_impact_context.is_some()))?;
        }
        for projectile in self.projectiles.values() {
            if let Some(spatial) = &projectile.spatial {
                count(spatial.hit_unit_ids.len())?;
            }
        }
        for projectile in self.active_projectiles.values() {
            count(projectile.hit_unit_ids.len())?;
        }
        for area in self.active_areas.values() {
            count(area.hit_unit_ids.len())?;
            count(area.previous_tick_unit_ids.len())?;
        }
        control::admit(
            Resource::RetainedEntities,
            current.checked_sub(removed).unwrap_or(usize::MAX),
            additional,
            site,
        )
    }

    #[cfg(test)]
    pub(crate) fn process_event_for_test(
        &mut self,
        event: crate::game::battle::enums::BattleEvent,
        current_time_ms: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        self.process_event(event, current_time_ms)
    }

    pub fn scenario(&self) -> &BattleScenario {
        &self.scenario
    }

    fn insert_runtime_unit(
        &mut self,
        id: UnitInstanceId,
        unit: RuntimeUnit,
    ) -> Option<RuntimeUnit> {
        if crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(RuntimeUnit::fact_slot_extent()),
            "unit_fact_slots",
        )
        .is_err()
        {
            return None;
        }
        self.units_clone_history.inserted();
        let previous = self.units.insert(id, unit);
        #[cfg(feature = "runtime-observation")]
        crate::runtime_observation::peak(
            crate::runtime_observation::UNITS,
            self.units.len() as u64,
        );
        previous
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn new_from_scenario(
        scenario: BattleScenario,
        game_data: impl Into<ValidatedGameData>,
        seed: u64,
    ) -> Self {
        let game_data = game_data.into();
        let content_identity = game_data.identity_arc();
        Self::new_from_database(
            scenario,
            game_data.into_database_arc(),
            Some(content_identity),
            seed,
        )
    }

    pub(crate) fn new_from_database(
        scenario: BattleScenario,
        game_data: Arc<GameDataBase>,
        content_identity: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
        seed: u64,
    ) -> Self {
        let binding = Self::scenario_fact_binding_extent(&scenario);
        Self::new_from_database_with_binding(scenario, game_data, content_identity, seed, binding)
    }

    fn scenario_fact_binding_extent(scenario: &BattleScenario) -> Option<usize> {
        let binding_delta =
            std::mem::size_of::<crate::game::battle::types::BattleEquipmentBinding>()
                .checked_sub(std::mem::size_of::<Uuid>());
        scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .try_fold(64_usize, |sum, spawn| {
                binding_delta
                    .and_then(|size| spawn.draft.equipped_items.len().checked_mul(size))
                    .and_then(|bytes| {
                        bytes.checked_add(std::mem::size_of::<
                            Option<crate::game::resources::item_slot::SlotLayoutKind>,
                        >())
                    })
                    .and_then(|bytes| sum.checked_add(bytes))
            })
            .and_then(|bytes| {
                let domain_bytes =
                    scenario
                        .battlefield
                        .object_placement
                        .as_ref()
                        .map_or(Some(0), |domain| {
                            domain
                                .ground
                                .capacity()
                                .checked_add(domain.reserved.capacity())?
                                .checked_mul(std::mem::size_of::<Position>())
                        })?;
                bytes
                    .checked_add(std::mem::size_of::<
                        Option<crate::game::battle::scenario::IndependentPlacementDomain>,
                    >())?
                    .checked_add(domain_bytes)
            })
    }

    /// Additional R04 owner registration for a World relic operation whose
    /// outer budget is already seeded. Ordinary constructors keep first-seed
    /// semantics and never enter this opt-in allocation boundary.
    pub(crate) fn new_for_relic_owner(
        scenario: BattleScenario,
        game_data: Arc<GameDataBase>,
        content_identity: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
        seed: u64,
        _owner: crate::game::events::combat::RelicBattleRuntimeOwner,
    ) -> Result<Self, crate::game::behavior::GameError> {
        let binding = Self::scenario_fact_binding_extent(&scenario);
        let initial = Self::initial_runtime_fact_extent(binding);
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            initial,
            "relic_battle_initial_owner",
        )?;
        Ok(Self::new_from_database_with_binding(
            scenario,
            game_data,
            content_identity,
            seed,
            binding,
        ))
    }

    fn initial_runtime_fact_extent(binding: Option<usize>) -> Option<usize> {
        runtime_facts::RuntimeFactState::initial_retained_bytes_bound()?
            .checked_add(std::mem::size_of::<runtime_facts::RuntimeFactState>())?
            .checked_add(binding?)?
            .checked_add(sustained::SustainedState::default().retained_extent()?)?
            .checked_add(independent_objects::IndependentObjectState::default().retained_extent()?)?
            .checked_add(spawn_kernel::SpawnKernelState::default().independent_retained_extent()?)?
            .checked_add(128)?
            .checked_add(
                crate::game::battle::runtime_facts::RuntimeFactSource::default()
                    .retained_extent()?,
            )
    }

    fn new_from_database_with_binding(
        scenario: BattleScenario,
        game_data: Arc<GameDataBase>,
        content_identity: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
        seed: u64,
        scenario_fact_binding_extent: Option<usize>,
    ) -> Self {
        let runtime_safety = Arc::clone(game_data.runtime_safety());
        let boss_runtime = scenario
            .boss_runtime
            .as_ref()
            .map(boss_runtime::BossEncounterRuntime::from_setup);
        let field_size = (scenario.battlefield.width, scenario.battlefield.height);
        let valid_tiles = scenario.battlefield.valid_tiles.clone();
        let field_work_objects = field_work::runtime_from_scenario(&scenario);
        let employee_participants_possible = scenario.groups.iter().any(|group| {
            group
                .spawns
                .iter()
                .any(|spawn| matches!(&spawn.draft.source, BattleUnitSource::Employee(_)))
        });
        Self {
            sustained: Default::default(),
            corrosion_freeze: None,
            event_queue: queue::EventQueue::new(),
            scenario: Arc::new(scenario),
            scenario_fact_binding_extent,
            relic_numeric_owner: None,
            effective_owner: None,
            scenario_runtime: ScenarioRuntimeState::default(),
            spawn_kernel: spawn_kernel::SpawnKernelState::default(),
            summon_runtime: summon_runtime::SummonRuntimeState::default(),
            independent_objects: independent_objects::IndependentObjectState::default(),
            boss_runtime,
            units: HashMap::new(),
            units_clone_history: MapCloneHistory::default(),
            tactical_states: OwnedHashMap::new(),
            items: OwnedHashMap::new(),
            graveyard: OwnedHashMap::new(),
            buffs: OwnedHashMap::new(),
            active_skill_casts: OwnedHashMap::new(),
            ability_proc_states: OwnedHashMap::new(),
            projectiles: OwnedHashMap::new(),
            active_projectiles: OwnedHashMap::new(),
            active_areas: OwnedHashMap::new(),
            active_movement_segments: OwnedHashMap::new(),
            field_work_objects,
            block_state: BlockRuntimeState::default(),
            last_continuous_movement_tick_ms: None,
            movement_backend: Arc::new(RapierMovementWorld::default()),
            battlefield: BattlefieldLayout::new_with_valid_tiles(
                field_size.0,
                field_size.1,
                valid_tiles,
            ),
            live_signals: Vec::new(),
            game_data,
            content_identity,
            event_log: BattleEventLog::with_runtime_safety(&runtime_safety),
            event_log_seq: 1,
            result_stats_accumulator: BattleResultStatsAccumulator::new(
                employee_participants_possible,
            ),
            skill_cast_seq: 0,
            projectile_seq: 0,
            area_seq: 0,
            damage_source_seq: 0,
            seed,
            recording_cause_stack: Vec::new(),
            recording_source_command_stack: Vec::new(),
            authoring_observation: None,
            authoring_trace: Default::default(),
            authoring_direct_source: None,
            runtime_strike_scope: None,
            runtime_effect_projection: None,
            runtime_fact_source: Default::default(),
            runtime_facts: runtime_facts::RuntimeFactState::new(runtime_safety.native_runtime()),
            recording_skill_effect_source: None,
            recording_skill_proc_origin: None,
        }
    }

    fn admit_scenario_binding_growth(
        &mut self,
        bindings: usize,
    ) -> Result<(), crate::game::behavior::GameError> {
        use crate::game::battle::runtime_control;
        let additional = std::mem::size_of::<crate::game::battle::types::BattleEquipmentBinding>()
            .checked_sub(std::mem::size_of::<Uuid>())
            .and_then(|size| size.checked_mul(bindings))
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<
                    Option<crate::game::resources::item_slot::SlotLayoutKind>,
                >())
            });
        let current = self.retained_runtime_fact_extent()?;
        runtime_control::admit_retained_growth(
            current,
            additional.and_then(|n| current.checked_add(n)),
            "scenario_fact_binding",
        )?;
        self.scenario_fact_binding_extent = self
            .scenario_fact_binding_extent
            .and_then(|old| additional.and_then(|n| old.checked_add(n)));
        Ok(())
    }

    /// Scenario definitions are shared across gameplay candidates. Deployment
    /// appends a definition only after admitting the actual owned copy.
    fn scenario_for_deployment(
        &mut self,
    ) -> Result<&mut BattleScenario, crate::game::behavior::GameError> {
        use crate::authoring::clone_shape::CloneShape;
        if Arc::get_mut(&mut self.scenario).is_none() {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.scenario_fact_binding_extent,
                "scenario_fact_copy",
            )?;
            if crate::game::battle::runtime_control::is_active() {
                let mut meter = crate::authoring::control::CloneMeter::for_native_scan()?;
                self.scenario.as_ref().visit_clone_shape(&mut meter)?;
                crate::game::battle::runtime_control::charge(
                    meter.native_scan_slots() as u64,
                    "scenario_copy_scan",
                )?;
                let slots = meter.native_copy_slots().ok_or_else(||
                    crate::game::battle::runtime_control::reject(crate::game::battle::runtime_control::BattleRuntimeFault {
                        code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                        site: "scenario_copy_slots", limit: None, requested: None,
                    }))?;
                crate::game::battle::runtime_control::charge_clone(slots, "scenario_copy")?;
            }
            let owned = crate::authoring::clone_shape::controlled_clone(self.scenario.as_ref())?;
            self.scenario = Arc::new(owned);
        }
        Ok(Arc::get_mut(&mut self.scenario).expect("scenario copy is exclusively owned"))
    }

    /// Enqueue a deterministic battle event before the simulation loop starts.
    ///
    /// This capability is compiled only for test/debug support. Default release
    /// callers cannot name the kernel module or inject a raw event.
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn enqueue_event(&mut self, event: BattleEvent) {
        self.event_queue.push(event);
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
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

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
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

    #[cfg(feature = "runtime-observation")]
    fn observe_projectile_peak(&self) {
        crate::runtime_observation::peak(
            crate::runtime_observation::PROJECTILES,
            self.projectiles
                .len()
                .saturating_add(self.active_projectiles.len()) as u64,
        );
    }

    #[cfg(feature = "runtime-observation")]
    pub(crate) fn observation_usage(&self) -> crate::runtime_observation::Usage {
        // Lengths and cached accounting only. No unit/event/owner traversal.
        let history = self.event_log.observation_usage();
        crate::runtime_observation::Usage([
            0,
            self.event_queue.len() as u64,
            self.runtime_facts.usage.active_actions as u64,
            self.projectiles
                .len()
                .saturating_add(self.active_projectiles.len()) as u64,
            self.units.len() as u64,
            history[0],
            history[1],
            history[2],
            0,
        ])
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn active_basic_projectile_count(&self) -> usize {
        self.projectiles.len()
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn active_skill_projectile_count(&self) -> usize {
        self.active_projectiles.len()
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
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

    /// Read-only debug snapshot of one active skill cast.
    ///
    /// This is intended for integration harnesses and deterministic runtime
    /// debugging, so tests can assert whether async skill steps reached a
    /// terminal state without exposing mutable internal state.
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn debug_skill_cast_state(
        &self,
        ability_event_seq: u64,
    ) -> Option<ActiveSkillCastDebugState> {
        self.active_skill_casts
            .values()
            .find(|cast| cast.ability_event_seq == ability_event_seq)
            .map(ActiveSkillCastDebugState::from)
    }

    /// Read-only debug/replay snapshot of retained inactive runtime units.
    ///
    /// Official gameplay checkpoints intentionally expose Active units only.
    /// This query exists for debug/admin/replay inspection of retained
    /// `Withdrawn` and `Dead` runtime entities without making them gameplay
    /// presentation sources.
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
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

    pub(in crate::game::battle::core) fn add_resonance(
        &mut self,
        unit_instance_id: UnitInstanceId,
        amount: u32,
        now_ms: u64,
        allow_autocast_when_full: bool,
    ) {
        if amount == 0 {
            return;
        }

        self.reconcile_resonance_use(unit_instance_id);
        if self
            .resonance_gain_blocks(unit_instance_id, now_ms)
            .blocked()
        {
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
        let before = self
            .units
            .get(&unit_instance_id)
            .filter(|unit| unit.is_active())
            .map(|unit| unit.resonance_current);
        self.modify_resonance_inner(unit_instance_id, delta, now_ms, allow_autocast_when_full);
        if let (Some(before), Some(unit)) = (before, self.units.get(&unit_instance_id)) {
            let after = unit.resonance_current;
            let _ = self.record_resource_fact(
                crate::game::battle::runtime_facts::ResourceOwner::Unit(unit_instance_id),
                crate::game::battle::runtime_facts::FactResource::Resonance,
                None,
                self.runtime_fact_source.clone(),
                i64::from(delta),
                before,
                after,
                now_ms,
            );
        }
    }

    fn modify_resonance_inner(
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

    fn derive_control_state(
        &self,
        unit_instance_id: UnitInstanceId,
        now_ms: Option<u64>,
    ) -> UnitControlState {
        let mut state = UnitControlState::default();
        for (key, active) in &self.buffs {
            if key.target_instance_id != unit_instance_id
                || now_ms.is_some_and(|now_ms| now_ms > active.expires_at_ms)
            {
                continue;
            }
            let Some(capability) = self
                .game_data
                .buff_data
                .get(key.buff_id)
                .and_then(|def| def.kind.control_capability())
            else {
                continue;
            };
            match capability {
                BuffControlCapability::HardCrowdControl => {
                    state.hard_cc_release_time_ms = Some(
                        state
                            .hard_cc_release_time_ms
                            .unwrap_or(0)
                            .max(active.expires_at_ms),
                    );
                }
                BuffControlCapability::Silence => state.silenced = true,
            }
        }
        state
    }

    pub fn unit_control_state(
        &self,
        unit_instance_id: UnitInstanceId,
        now_ms: u64,
    ) -> Option<UnitControlState> {
        self.units
            .contains_key(&unit_instance_id)
            .then(|| self.derive_control_state(unit_instance_id, Some(now_ms)))
    }

    pub(in crate::game::battle::core) fn current_control_state(
        &self,
        unit_instance_id: UnitInstanceId,
    ) -> UnitControlState {
        self.derive_control_state(unit_instance_id, None)
    }

    pub(in crate::game::battle::core) fn active_hard_cc_release_time_ms(
        &self,
        unit_instance_id: UnitInstanceId,
        now_ms: u64,
    ) -> Option<u64> {
        self.unit_control_state(unit_instance_id, now_ms)
            .and_then(|state| state.hard_cc_release_time_ms)
            .map(|expires_at_ms| expires_at_ms.saturating_add(1))
    }

    pub fn skill_start_disabled_reason(
        &self,
        unit_instance_id: UnitInstanceId,
        now_ms: u64,
    ) -> Option<ActionDisabledReasonCode> {
        if self.channel_active(unit_instance_id) {
            return Some(ActionDisabledReasonCode::ActionLocked);
        }
        let state = self.unit_control_state(unit_instance_id, now_ms)?;
        if state.hard_cc_active() {
            Some(ActionDisabledReasonCode::HardCrowdControl)
        } else if state.silenced {
            Some(ActionDisabledReasonCode::Silenced)
        } else if self.unit_is_field_working(unit_instance_id) {
            Some(ActionDisabledReasonCode::FieldWorking)
        } else if self.unit_is_in_defense_stance(unit_instance_id) {
            Some(ActionDisabledReasonCode::DefenseStance)
        } else {
            None
        }
    }

    fn cancel_unreleased_actions_for_hard_cc(
        &mut self,
        time_ms: u64,
        interrupter_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
    ) {
        self.cancel_resonance_use_control(target_instance_id);
        let _ = self.stop_channel_with_reason(
            target_instance_id,
            time_ms,
            crate::game::ability::sustained::SustainedEndReason::HardCrowdControl,
        );
        self.cancel_structured_flows(
            target_instance_id,
            time_ms,
            crate::game::battle::runtime_facts::ActionCancelReason::HardCrowdControl,
        );
        let (pending, cast_lock_until) = self
            .units
            .get_mut(&target_instance_id)
            .map(|unit| {
                unit.basic_attack_epoch = unit.basic_attack_epoch.wrapping_add(1);
                if let Some(windup_until_ms) = unit.basic_attack_windup_until_ms.take() {
                    if unit.action_locks.basic_attack_until_ms == windup_until_ms {
                        unit.action_locks.basic_attack_until_ms = time_ms;
                    }
                    if unit.action_locks.movement_until_ms == windup_until_ms {
                        unit.action_locks.movement_until_ms = time_ms;
                    }
                }
                unit.current_target = None;
                let pending = unit.pending_skill_cast.take();
                let cast_lock_until = pending.as_ref().map(|_| unit.next_action_time);
                if pending.is_some() {
                    unit.pending_cast = unit.skill_activation_mode
                        == crate::game::ability::SkillActivationMode::Auto;
                    unit.pending_cast_cause = None;
                    unit.next_action_time = 0;
                }
                (pending, cast_lock_until)
            })
            .unwrap_or((None, None));

        let Some(pending) = pending else {
            let _ = self.runtime_facts.cancel_unreleased_basic(
                target_instance_id,
                crate::game::battle::runtime_facts::ActionCancelReason::HardCrowdControl,
                time_ms,
            );
            return;
        };
        let _ = self.runtime_facts.cancel_unreleased_basic(
            target_instance_id,
            crate::game::battle::runtime_facts::ActionCancelReason::HardCrowdControl,
            time_ms,
        );
        self.cancel_pending_action(
            pending.action.as_ref(),
            time_ms,
            crate::game::battle::runtime_facts::ActionCancelReason::HardCrowdControl,
        );
        if let Some(cast_lock_until) = cast_lock_until {
            let focus_permissions = self
                .game_data
                .skill_data
                .get_by_id(pending.skill_id.as_str())
                .map(|skill| skill.focus_permissions)
                .unwrap_or_default();
            if let Some(unit) = self.units.get_mut(&target_instance_id) {
                if unit.action_locks.resonance_gain_until_ms == cast_lock_until {
                    unit.action_locks.resonance_gain_until_ms = time_ms;
                }
                if !focus_permissions.allows_basic_attack
                    && unit.action_locks.basic_attack_until_ms == cast_lock_until
                {
                    unit.action_locks.basic_attack_until_ms = time_ms;
                }
                if !focus_permissions.allows_move
                    && unit.action_locks.movement_until_ms == cast_lock_until
                {
                    unit.action_locks.movement_until_ms = time_ms;
                }
            }
        }
        self.record_event_log(
            time_ms,
            BattleLogEvent::SkillCastInterrupted {
                interrupter_instance_id,
                caster_instance_id: target_instance_id,
                interrupted_skill_id: pending.skill_id,
                interrupted_cast_seq: pending.start_seq,
            },
        );
    }

    fn schedule_pending_autocast_for(&mut self, caster_instance_id: UnitInstanceId, now_ms: u64) {
        let fallback = self.recording_cause().unwrap_or_default();

        if !self.boss_runtime_allows_combat_action(caster_instance_id) {
            return;
        }

        if self
            .skill_start_disabled_reason(caster_instance_id, now_ms)
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

    pub(in crate::game::battle::core) fn schedule_pending_autocasts(&mut self, now_ms: u64) {
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
    mod amount_order;
    use super::*;
    use crate::game::ability::{
        DeliveryDef, SkillActivationMode, SkillAreaAnchorSource, SkillAreaTracking,
        SkillCastSelection, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillId, SkillKind,
        SkillPresentationDef, SkillStepDef, SkillTarget, SkillTileAreaDeliveryDef,
        SkillTileAreaOrigin, StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::buffs::{BuffId, BuffReapplyPolicy};
    use crate::game::battle::cooldown::CooldownSource;
    use crate::game::battle::core::movement::{types::WorldVec2, ActionState};
    use crate::game::battle::core::types::{PendingSkillCast, RuntimeUnit, RuntimeUnitLifecycle};
    use crate::game::battle::damage::{
        BattleCommand, CombatRollIdentity, CombatRollKind, DamageModifiers, DamageSource,
        DamageSourceSnapshot, DamageType, ProcRollIdentity,
    };
    use crate::game::battle::enums::BattleEvent;
    use crate::game::battle::event_log::{
        AttackKind, BattleEventCause, BattleEventLog, BattleEventRootCause, BattleLogEvent,
        BuffExpireReason, SkillCastCancelReason, SkillCastTarget, BATTLE_EVENT_LOG_VERSION,
    };
    use crate::game::battle::scenario::BattleScenario;
    use crate::game::battle::tile_range::{FacingDirection, TileRangePattern, TileRangePolicy};
    use crate::game::battle::types::{
        BattleUnitDraft, BattleUnitSource, DeploymentAffinity, EngagementStance, MobilityKind,
        TacticalTileKind, UnitCombatProfile,
    };
    use crate::game::behavior::GameError;
    use crate::game::data::run_policy_data::RunPolicyData;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata, skill_data::SkillDatabase, GameDataBase,
        GameDataBuilder,
    };
    use crate::game::enums::Side;
    use crate::game::resources::Position;
    use crate::game::stats::TriggerType;
    use crate::game::stats::UnitStats;
    use crate::game::stats::{StatId, StatModifier, StatModifierKind};

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::live_defaults().build_arc()
    }

    fn new_core() -> BattleCore {
        BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), empty_game_data(), 123)
    }

    // These tests exercise behavior after actual start, not admission while paused.
    fn process_manual_start_gate_fixture(core: &mut BattleCore, time_ms: u64) {
        let event = core.event_queue.pop().expect("manual actual start gate");
        assert!(
            matches!(event.event.view().as_ref(), BattleEvent::SkillStartGate { time_ms: gate_time, .. } if *gate_time == time_ms)
        );
        core.process_event(event, time_ms).unwrap();
    }

    fn finite_boundary_authority_snapshot(core: &BattleCore) -> String {
        format!(
            "{};queue={:?};event_log={};field={}x{}:{:?}:{:?};seq={}:{}:{}:{}:{};seed={};causes={:?};commands={:?}",
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_string(&core.event_log).unwrap(),
            core.battlefield.width(),
            core.battlefield.height(),
            core.battlefield.valid_positions(),
            core.battlefield.static_obstacles(),
            core.event_log_seq,
            core.skill_cast_seq,
            core.projectile_seq,
            core.area_seq,
            core.damage_source_seq,
            core.seed,
            core.recording_cause_stack,
            core.recording_source_command_stack,
        )
    }

    fn try_replace_unit_body(
        core: &mut BattleCore,
        unit_id: UnitInstanceId,
        x: f32,
        y: f32,
        radius: f32,
        speed: f32,
    ) -> Result<(), movement::types::FiniteBattleValueError> {
        let position = WorldVec2::try_new(x, y)?;
        let body = UnitBody::try_new_at(position, radius, speed)?;
        core.units
            .get_mut(&unit_id)
            .expect("finite boundary fixture unit must exist")
            .body = body;
        Ok(())
    }

    #[test]
    fn non_finite_position_radius_speed_and_time_reject_before_kernel_mutation_and_retry() {
        use movement::types::{FiniteBattleSeconds, FiniteBattleValueError, FiniteBattleValueKind};

        let mut core = new_core();
        let unit_id = UnitInstanceId::from(Uuid::from_u128(0xF1A1_0001));
        core.units
            .insert(unit_id, runtime_unit(unit_id, Side::Player));
        let before = finite_boundary_authority_snapshot(&core);

        let cases = [
            (f32::NAN, 0.0, 0.35, 1.0, FiniteBattleValueKind::PositionX),
            (
                f32::INFINITY,
                0.0,
                0.35,
                1.0,
                FiniteBattleValueKind::PositionX,
            ),
            (
                f32::NEG_INFINITY,
                0.0,
                0.35,
                1.0,
                FiniteBattleValueKind::PositionX,
            ),
            (0.0, f32::NAN, 0.35, 1.0, FiniteBattleValueKind::PositionY),
            (
                0.0,
                f32::INFINITY,
                0.35,
                1.0,
                FiniteBattleValueKind::PositionY,
            ),
            (
                0.0,
                f32::NEG_INFINITY,
                0.35,
                1.0,
                FiniteBattleValueKind::PositionY,
            ),
            (0.0, 0.0, f32::NAN, 1.0, FiniteBattleValueKind::Radius),
            (0.0, 0.0, f32::INFINITY, 1.0, FiniteBattleValueKind::Radius),
            (
                0.0,
                0.0,
                f32::NEG_INFINITY,
                1.0,
                FiniteBattleValueKind::Radius,
            ),
            (0.0, 0.0, 0.35, f32::NAN, FiniteBattleValueKind::Speed),
            (0.0, 0.0, 0.35, f32::INFINITY, FiniteBattleValueKind::Speed),
            (
                0.0,
                0.0,
                0.35,
                f32::NEG_INFINITY,
                FiniteBattleValueKind::Speed,
            ),
        ];

        for (x, y, radius, speed, kind) in cases {
            let first = try_replace_unit_body(&mut core, unit_id, x, y, radius, speed)
                .expect_err("non-finite kernel input must reject");
            assert_eq!(first, FiniteBattleValueError::NonFinite { kind });
            assert_eq!(finite_boundary_authority_snapshot(&core), before);

            let retry = try_replace_unit_body(&mut core, unit_id, x, y, radius, speed)
                .expect_err("same-basis retry must reject");
            assert_eq!(retry, first);
            assert_eq!(finite_boundary_authority_snapshot(&core), before);
        }

        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let first = FiniteBattleSeconds::try_new(value)
                .expect_err("non-finite battle time must reject");
            assert_eq!(
                first,
                FiniteBattleValueError::NonFinite {
                    kind: FiniteBattleValueKind::TimeSeconds,
                }
            );
            assert_eq!(finite_boundary_authority_snapshot(&core), before);
            assert_eq!(FiniteBattleSeconds::try_new(value).unwrap_err(), first);
            assert_eq!(finite_boundary_authority_snapshot(&core), before);
        }
    }

    #[test]
    fn finite_world_position_round_trips_and_non_finite_ron_is_rejected() {
        use serde::{de::value, de::IntoDeserializer, Deserialize};

        let position = WorldVec2::try_new(-0.0, 12.5).unwrap();
        assert_eq!(position.x().to_bits(), (-0.0_f32).to_bits());
        assert_eq!(position.y(), 12.5);
        let encoded = ron::ser::to_string(&position).unwrap();
        let decoded: WorldVec2 = ron::de::from_str(&encoded).unwrap();
        assert_eq!(decoded, position);

        for raw in ["(x:NaN,y:0.0)", "(x:inf,y:0.0)", "(x:0.0,y:-inf)"] {
            assert!(ron::de::from_str::<WorldVec2>(raw).is_err(), "{raw}");
        }

        for (x, y, component) in [
            (f32::NAN, 0.0, "position.x"),
            (f32::INFINITY, 0.0, "position.x"),
            (0.0, f32::NEG_INFINITY, "position.y"),
        ] {
            let entries = vec![
                ("x".into_deserializer(), x.into_deserializer()),
                ("y".into_deserializer(), y.into_deserializer()),
            ];
            let deserializer = value::MapDeserializer::<_, value::Error>::new(entries.into_iter());
            let error = WorldVec2::deserialize(deserializer)
                .expect_err("non-finite deserialization must reach the finite gate");
            assert!(error.to_string().contains(component), "{error}");
        }
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
    fn observation_log_insertion_does_not_change_basic_projectile_identity() {
        let attacker_id = UnitInstanceId::from(Uuid::from_u128(0xD170_01));
        let target_id = UnitInstanceId::from(Uuid::from_u128(0xD170_02));
        let launch = super::basic_attack_projectile::ProjectileLaunch {
            spatial: None,
            fired_at_ms: 10,
            attacker_instance_id: attacker_id,
            attacker_owner_at_launch: Side::Player,
            air_capable_at_launch: false,
            target_instance_id: target_id,
            attacker_origin: WorldVec2::ZERO,
            target_aim: WorldVec2::new(1.0, 0.0),
            speed_units_per_ms: 1,
            guidance: super::types::ProjectileGuidance::Fixed,
            damage_type: DamageType::Physical,
            source_snapshot: DamageSourceSnapshot {
                fact_source: Default::default(),
                skill_cast_source: None,
                value_expr_capture: None,
                source_id: attacker_id,
                source_side: Side::Player,
                source_attack: 10,
                source: DamageSource::BasicAttack,
                damage_type: DamageType::Physical,
                base_damage: 10,
                modifiers: DamageModifiers::default(),
                crit_roll_percent: None,
                crit_roll_identity: CombatRollIdentity::new(
                    CombatRollKind::BasicAttackCrit,
                    Some(attacker_id),
                    Uuid::from_u128(0xD170_03),
                ),
                minimum_damage: 1,
                committed_at_ms: 10,
                on_attack_modifiers: DamageModifiers::default(),
                on_attack_bonus_damage: vec![],
                on_attack_modifier_sources: vec![],
                basic_lifesteal: Vec::new(),
            },
        };
        let mut control = new_core();
        let mut with_observation = new_core();

        let control_parent = control.record_event_log(
            10,
            BattleLogEvent::BattleStart {
                width: 4,
                height: 4,
            },
        );
        with_observation.record_event_log(
            1,
            BattleLogEvent::BattleStart {
                width: 4,
                height: 4,
            },
        );
        let observed_parent = with_observation.record_event_log(
            10,
            BattleLogEvent::BattleStart {
                width: 4,
                height: 4,
            },
        );
        control.with_recording_cause(control_parent, |core| {
            core.spawn_basic_attack_projectile(launch.clone());
        });
        with_observation.with_recording_cause(observed_parent, |core| {
            core.spawn_basic_attack_projectile(launch);
        });

        let control_id = *control.projectiles.keys().next().unwrap();
        let observed_id = *with_observation.projectiles.keys().next().unwrap();
        assert_ne!(control.event_log_seq, with_observation.event_log_seq);
        assert_eq!(
            control_id, observed_id,
            "observation-only log insertion must not alter delivery identity"
        );
    }

    #[test]
    fn observation_log_insertion_does_not_change_skill_cast_delivery_or_queue_order() {
        let skill = single_step_skill(
            "identity_projectile",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            vec![damage_effect()],
        );
        let caster_id = UnitInstanceId::from(Uuid::from_u128(0xD171_01));
        let target_id = UnitInstanceId::from(Uuid::from_u128(0xD171_02));
        let target = Some(SkillCastTarget::Unit {
            unit_instance_id: target_id,
        });
        let mut control = core_with_skill_data(vec![], vec![skill.clone()]);
        let mut with_observation = core_with_skill_data(vec![], vec![skill.clone()]);
        for core in [&mut control, &mut with_observation] {
            core.units
                .insert(caster_id, runtime_unit(caster_id, Side::Player));
            core.units
                .insert(target_id, runtime_unit(target_id, Side::Opponent));
            place_unit(core, caster_id, Position::new(1, 1));
            place_unit(core, target_id, Position::new(2, 1));
            sync_unit_to_battlefield_tile_center(core, caster_id);
            sync_unit_to_battlefield_tile_center(core, target_id);
        }
        with_observation.record_event_log(
            1,
            BattleLogEvent::BattleStart {
                width: 6,
                height: 6,
            },
        );

        control.invoke_ability(
            10,
            caster_id,
            skill.id.as_str(),
            target,
            BattleEventCause::default(),
            false,
        );
        with_observation.invoke_ability(
            10,
            caster_id,
            skill.id.as_str(),
            target,
            BattleEventCause::default(),
            false,
        );

        let skill_step = |core: &BattleCore| {
            core.event_queue
                .iter()
                .find_map(|event| match event.as_ref() {
                    BattleEvent::SkillStep {
                        cast_seq, cause, ..
                    } => Some((*cast_seq, *cause)),
                    _ => None,
                })
        };
        let (control_cast_seq, control_cause) = skill_step(&control).unwrap();
        let (observed_cast_seq, observed_cause) = skill_step(&with_observation).unwrap();
        assert_eq!(control_cast_seq, observed_cast_seq);
        assert_ne!(control_cause.parent_seq(), observed_cause.parent_seq());

        for (core, cast_seq) in [
            (&mut control, control_cast_seq),
            (&mut with_observation, observed_cast_seq),
        ] {
            assert_eq!(
                core.dispatch_skill_projectile_delivery(
                    10,
                    cast_seq,
                    0,
                    caster_id,
                    &skill,
                    &skill.steps[0],
                    target,
                    &skill.steps[0].delivery,
                    1,
                )
                .unwrap(),
                1
            );
        }

        let control_delivery_id = *control.active_projectiles.keys().next().unwrap();
        let observed_delivery_id = *with_observation.active_projectiles.keys().next().unwrap();
        assert_eq!(control_delivery_id, observed_delivery_id);

        let gameplay_queue = |core: &BattleCore| {
            let mut queue = core.event_queue.clone();
            let mut order = Vec::new();
            while let Some(event) = queue.pop() {
                match event.event.view().into_owned() {
                    BattleEvent::SkillStep {
                        time_ms,
                        cast_seq,
                        step_index,
                        ..
                    } => order.push(format!("step:{time_ms}:{cast_seq}:{step_index}")),
                    BattleEvent::SkillProjectileAdvance {
                        time_ms,
                        delivery_id,
                        ..
                    } => order.push(format!("projectile:{time_ms}:{delivery_id}")),
                    _ => {}
                }
            }
            order
        };
        assert_eq!(gameplay_queue(&control), gameplay_queue(&with_observation));
    }

    #[test]
    fn observation_log_insertion_does_not_change_skill_area_identity_or_outcome() {
        let skill = enemy_tile_area_skill("identity_area", vec![damage_effect()]);
        let caster_id = UnitInstanceId::from(Uuid::from_u128(0xD172_01));
        let target_id = UnitInstanceId::from(Uuid::from_u128(0xD172_02));
        let mut control = core_with_skill_data(vec![], vec![skill.clone()]);
        let mut with_observation = core_with_skill_data(vec![], vec![skill.clone()]);
        for core in [&mut control, &mut with_observation] {
            core.units
                .insert(caster_id, runtime_unit(caster_id, Side::Player));
            core.units
                .insert(target_id, runtime_unit(target_id, Side::Opponent));
            place_unit(core, caster_id, Position::new(1, 1));
            place_unit(core, target_id, Position::new(2, 1));
            sync_unit_to_battlefield_tile_center(core, caster_id);
            sync_unit_to_battlefield_tile_center(core, target_id);
        }
        with_observation.record_event_log(
            1,
            BattleLogEvent::BattleStart {
                width: 6,
                height: 6,
            },
        );

        for core in [&mut control, &mut with_observation] {
            core.invoke_ability(
                10,
                caster_id,
                skill.id.as_str(),
                None,
                BattleEventCause::default(),
                false,
            );
            let event = core.event_queue.pop().expect("queued skill step");
            core.process_event(event, 10).expect("resolve area step");
        }

        let area_id = |core: &BattleCore| {
            core.event_log.entries.iter().find_map(|entry| {
                if let BattleLogEvent::SkillAreaDeclared { area_id, .. } = entry.event {
                    Some(area_id)
                } else {
                    None
                }
            })
        };
        assert_eq!(area_id(&control), area_id(&with_observation));
        assert_eq!(
            control.units.get(&target_id).unwrap().stats.current_health,
            with_observation
                .units
                .get(&target_id)
                .unwrap()
                .stats
                .current_health
        );
        assert_eq!(control.skill_cast_seq, with_observation.skill_cast_seq);
        assert_ne!(control.event_log_seq, with_observation.event_log_seq);
    }

    #[test]
    fn effect_preview_matches_runtime_area_union_at_edges_and_all_facings_without_mutation() {
        let first_footprint = TileRangePattern {
            include_anchor_tile: true,
            rows: vec![".XX".to_string(), ".@.".to_string(), "X..".to_string()],
        };
        let second_footprint = TileRangePattern {
            include_anchor_tile: true,
            rows: vec![".X.".to_string(), "X@X".to_string(), ".X.".to_string()],
        };
        let area = |footprint, anchor, tile_origin| DeliveryDef::TileArea {
            footprint,
            area: SkillTileAreaDeliveryDef {
                anchor,
                tile_origin,
                tracking: SkillAreaTracking::GroundFixed,
                hit_targets: crate::game::ability::SkillHitTargetFilter::Allies,
                include_caster: true,
                tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                duration_ms: 0,
                tick_interval_ms: None,
            },
        };
        let mut skill = single_step_skill(
            "effect_preview_runtime_parity",
            SkillTarget::SelfUnit,
            0,
            0,
            area(
                first_footprint,
                SkillAreaAnchorSource::Caster,
                SkillTileAreaOrigin::Caster,
            ),
            vec![SkillEffectDef::Heal { amount: 1 }],
        );
        skill.steps.push(SkillStepDef {
            id: "second_area".to_string(),
            delay_ms: 1,
            target: SkillTarget::CastTarget,
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: area(
                second_footprint,
                SkillAreaAnchorSource::CastTarget,
                SkillTileAreaOrigin::Anchor,
            ),
            effects: vec![SkillEffectDef::Heal { amount: 1 }.authored("effect_01")],
            presentation: SkillPresentationDef::default(),
        });
        skill.steps.push(SkillStepDef {
            id: "instant_no_cells".to_string(),
            delay_ms: 2,
            target: SkillTarget::SelfUnit,
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::Heal { amount: 1 }.authored("effect_01")],
            presentation: SkillPresentationDef::default(),
        });

        let cases = [
            (Position::new(2, 2), FacingDirection::Up),
            (Position::new(2, 2), FacingDirection::Right),
            (Position::new(2, 2), FacingDirection::Down),
            (Position::new(2, 2), FacingDirection::Left),
            (Position::new(0, 0), FacingDirection::Up),
            (Position::new(0, 0), FacingDirection::Left),
            (Position::new(5, 5), FacingDirection::Right),
            (Position::new(5, 5), FacingDirection::Down),
        ];

        for (position, facing) in cases {
            let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
            let caster_id = UnitInstanceId::from(Uuid::from_u128(
                0xE180_0000 + position.x as u128 * 100 + position.y as u128 * 10 + facing as u128,
            ));
            let mut caster = runtime_unit(caster_id, Side::Player);
            caster.skill_id = Some(skill.id.clone());
            caster.facing_direction = Some(facing);
            core.units.insert(caster_id, caster);
            place_unit(&mut core, caster_id, position);
            sync_unit_to_battlefield_tile_center(&mut core, caster_id);

            let state_before = core.transaction_debug_snapshot();
            let preview = crate::game::range_preview::range_previews_for_player_runtime_unit(
                core.units.get(&caster_id).unwrap(),
                core.game_data.as_ref(),
                &core.battlefield,
                position,
            );
            assert_eq!(
                core.transaction_debug_snapshot(),
                state_before,
                "preview must be observational at {position:?} facing {facing:?}"
            );

            core.invoke_ability(
                0,
                caster_id,
                skill.id.as_str(),
                None,
                BattleEventCause::default(),
                false,
            );
            while let Some(event) = core.event_queue.pop() {
                let time_ms = event.time_ms();
                core.process_event(event, time_ms).unwrap();
            }
            let mut runtime_union = core
                .event_log
                .entries
                .iter()
                .filter_map(|entry| match &entry.event {
                    BattleLogEvent::SkillAreaDeclared {
                        skill_id,
                        shape:
                            crate::game::battle::event_log::BattleSkillAreaShape::TilePattern {
                                affected_tiles,
                            },
                        ..
                    } if skill_id == &skill.id => Some(affected_tiles.as_slice()),
                    _ => None,
                })
                .flatten()
                .copied()
                .collect::<Vec<_>>();
            runtime_union.sort_by_key(|cell| (cell.y, cell.x));
            runtime_union.dedup();

            assert_eq!(
                preview.active_skill.effect_preview_cells, runtime_union,
                "runtime parity at {position:?} facing {facing:?}"
            );
        }
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
                object_placement: None,
                width: 5,
                height: 5,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
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
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_wave_1"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: first_wave_id,
                    },
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_wave_2"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtTimeMs(1_000),
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: second_wave_id,
                    },
                },
            ],
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            boss_runtime: None,
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

        let producer_envelope_is_current = |event_log: &BattleEventLog| {
            event_log.version == BATTLE_EVENT_LOG_VERSION
                && matches!(
                    event_log.entries.first().map(|entry| &entry.event),
                    Some(BattleLogEvent::BattleStart { .. })
                )
                && matches!(
                    event_log.entries.last().map(|entry| &entry.event),
                    Some(BattleLogEvent::BattleEnd { .. })
                )
                && event_log.entries.iter().enumerate().all(|(index, entry)| {
                    entry.seq == index as u64 + 1
                        && (index == 0 || entry.time_ms >= event_log.entries[index - 1].time_ms)
                })
        };
        let parent_links_are_prior_and_present = |event_log: &BattleEventLog| {
            event_log.entries.iter().all(|entry| match entry.cause {
                BattleEventCause::Parent { seq } => {
                    seq < entry.seq
                        && event_log
                            .entries
                            .iter()
                            .any(|candidate| candidate.seq == seq)
                }
                BattleEventCause::Root { .. } => true,
            })
        };
        let references_follow_spawn = |event_log: &BattleEventLog| {
            let mut spawned = std::collections::HashSet::new();
            for entry in &event_log.entries {
                let references = match &entry.event {
                    BattleLogEvent::UnitSpawned {
                        unit_instance_id, ..
                    } => {
                        spawned.insert(*unit_instance_id);
                        Vec::new()
                    }
                    BattleLogEvent::AttackStart {
                        attacker_instance_id,
                        target_instance_id,
                        ..
                    }
                    | BattleLogEvent::AttackResolve {
                        attacker_instance_id,
                        target_instance_id,
                        ..
                    }
                    | BattleLogEvent::AttackMiss {
                        attacker_instance_id,
                        target_instance_id,
                        ..
                    } => vec![*attacker_instance_id, *target_instance_id],
                    BattleLogEvent::HpChanged {
                        source_instance_id,
                        target_instance_id,
                        ..
                    } => source_instance_id
                        .iter()
                        .copied()
                        .chain(std::iter::once(*target_instance_id))
                        .collect(),
                    BattleLogEvent::ResonanceChanged {
                        unit_instance_id, ..
                    }
                    | BattleLogEvent::MovementStopped {
                        unit_instance_id, ..
                    } => vec![*unit_instance_id],
                    BattleLogEvent::UnitDied {
                        unit_instance_id,
                        killer_instance_id,
                        ..
                    } => killer_instance_id
                        .iter()
                        .copied()
                        .chain(std::iter::once(*unit_instance_id))
                        .collect(),
                    BattleLogEvent::BattleStart { .. } | BattleLogEvent::BattleEnd { .. } => {
                        Vec::new()
                    }
                    _ => return false,
                };
                if references
                    .into_iter()
                    .any(|unit_instance_id| !spawned.contains(&unit_instance_id))
                {
                    return false;
                }
            }
            true
        };
        let hp_and_death_history_is_consistent = |event_log: &BattleEventLog| {
            let mut dead_units = std::collections::HashSet::new();
            for entry in &event_log.entries {
                match &entry.event {
                    BattleLogEvent::HpChanged {
                        delta,
                        hp_before,
                        hp_after,
                        ..
                    } if i64::from(*hp_after) - i64::from(*hp_before) != i64::from(*delta) => {
                        return false;
                    }
                    BattleLogEvent::AttackStart {
                        attacker_instance_id,
                        target_instance_id,
                        ..
                    }
                    | BattleLogEvent::AttackResolve {
                        attacker_instance_id,
                        target_instance_id,
                        ..
                    }
                    | BattleLogEvent::AttackMiss {
                        attacker_instance_id,
                        target_instance_id,
                        ..
                    } if dead_units.contains(attacker_instance_id)
                        || dead_units.contains(target_instance_id) =>
                    {
                        return false;
                    }
                    BattleLogEvent::UnitDied {
                        unit_instance_id, ..
                    } if !dead_units.insert(*unit_instance_id) => return false,
                    _ => {}
                }
            }
            true
        };
        let auto_attack_cadence_is_respected = |event_log: &BattleEventLog| {
            let mut intervals = std::collections::HashMap::new();
            let mut last_attack = std::collections::HashMap::new();
            for entry in &event_log.entries {
                match &entry.event {
                    BattleLogEvent::UnitSpawned {
                        unit_instance_id,
                        stats,
                        ..
                    } => {
                        intervals.insert(*unit_instance_id, stats.attack_interval_ms);
                    }
                    BattleLogEvent::AttackStart {
                        attacker_instance_id,
                        kind: Some(AttackKind::Auto),
                        ..
                    } => {
                        let Some(interval_ms) = intervals.get(attacker_instance_id) else {
                            return false;
                        };
                        if last_attack
                            .insert(*attacker_instance_id, entry.time_ms)
                            .is_some_and(|previous| {
                                entry.time_ms < previous.saturating_add(u64::from(*interval_ms))
                            })
                        {
                            return false;
                        }
                    }
                    BattleLogEvent::AttackStart { kind: None, .. }
                    | BattleLogEvent::AttackResolve { kind: None, .. }
                    | BattleLogEvent::AttackMiss { kind: None, .. } => return false,
                    _ => {}
                }
            }
            true
        };

        assert!(producer_envelope_is_current(&result.event_log));
        assert!(parent_links_are_prior_and_present(&result.event_log));
        assert!(references_follow_spawn(&result.event_log));
        assert!(hp_and_death_history_is_consistent(&result.event_log));
        assert!(auto_attack_cadence_is_respected(&result.event_log));

        let mut malformed_version = result.event_log.clone();
        malformed_version.version = BATTLE_EVENT_LOG_VERSION.saturating_sub(1);
        assert!(!producer_envelope_is_current(&malformed_version));

        let mut malformed_seq = result.event_log.clone();
        malformed_seq.entries[4].seq = malformed_seq.entries[4].seq.saturating_add(1);
        assert!(!producer_envelope_is_current(&malformed_seq));

        let mut malformed_parent = result.event_log.clone();
        malformed_parent.entries[4].cause = BattleEventCause::Parent { seq: u64::MAX };
        assert!(!parent_links_are_prior_and_present(&malformed_parent));

        let mut malformed_reference = result.event_log.clone();
        if let BattleLogEvent::AttackStart {
            target_instance_id, ..
        } = &mut malformed_reference.entries[3].event
        {
            *target_instance_id = UnitInstanceId::from(Uuid::from_u128(0xBAD0_0001));
        }
        assert!(!references_follow_spawn(&malformed_reference));

        let mut malformed_hp = result.event_log.clone();
        if let BattleLogEvent::HpChanged { delta, .. } = &mut malformed_hp.entries[6].event {
            *delta = 0;
        }
        assert!(!hp_and_death_history_is_consistent(&malformed_hp));

        let first_dead_unit_id = result.event_log.entries.iter().find_map(|entry| {
            if let BattleLogEvent::UnitDied {
                unit_instance_id, ..
            } = entry.event
            {
                Some(unit_instance_id)
            } else {
                None
            }
        });
        let mut malformed_death = result.event_log.clone();
        if let (
            Some(dead_unit_id),
            BattleLogEvent::AttackStart {
                attacker_instance_id,
                ..
            },
        ) = (first_dead_unit_id, &mut malformed_death.entries[10].event)
        {
            *attacker_instance_id = dead_unit_id;
        }
        assert!(!hp_and_death_history_is_consistent(&malformed_death));

        let mut malformed_cadence = result.event_log.clone();
        malformed_cadence.entries[10].time_ms = 50;
        assert!(!auto_attack_cadence_is_respected(&malformed_cadence));
    }

    #[test]
    fn step_battle_execution_matches_full_battle_run() {
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups: vec![crate::game::battle::scenario::ScenarioSpawnGroup {
                id: crate::game::battle::scenario::ScenarioGroupId::new("clock"),
                side: Side::Opponent,
                required_for_victory: false,
                enemy_movement_plan: None,
                spawns: Vec::new(),
            }],
            events: vec![crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("clock_100"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtTimeMs(100),
                action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                    group_id: crate::game::battle::scenario::ScenarioGroupId::new("clock"),
                },
            }],
            terminal_outcomes: BattleScenario::fixture_time_victory_outcomes(100),
            boss_runtime: None,
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
                crate::game::battle::core::sim::BattleStepOutcome::Finished(_) => {
                    break stepped_core
                        .finalize_battle_execution(&mut execution)
                        .expect("battle execution should finalize");
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
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
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("spawn_enemy"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: enemy_group_id,
                    },
                },
            ],
            terminal_outcomes:
                crate::game::battle::scenario::BattleScenario::fixture_protected_outcomes(
                    protected_ref,
                    Some(1_000),
                ),
            boss_runtime: None,
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
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
            }],
            terminal_outcomes:
                crate::game::battle::scenario::BattleScenario::fixture_protected_outcomes(
                    protected_ref.clone(),
                    Some(1_000),
                ),
            boss_runtime: None,
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
        let protected_id = core.scenario_runtime.spawned.unit_refs[&protected_ref];
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups: Vec::new(),
            events: Vec::new(),
            terminal_outcomes: BattleScenario::fixture_survival_outcomes(1_000),
            boss_runtime: None,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let draft = fixture_draft(Uuid::from_u128(10), Uuid::from_u128(11), 100, 10);

        let deployed = core
            .apply_live_command(
                crate::game::battle::core::sim::BattleLiveCommand::DeployPlayerUnit {
                    draft,
                    position: Position::new(1, 1),
                    tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
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
            crate::game::battle::core::sim::BattleLiveCommandOutcome::EngagementStanceChanged {
                ..
            } => panic!("deploy command must report deployed unit"),
            crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitTacticallyRelocated {
                ..
            } => panic!("deploy command must report deployed unit"),
            crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitsTacticallySwapped {
                ..
            } => panic!("deploy command must report deployed unit"),
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
    fn withdraw_eligibility_ignores_hard_cc_and_silence() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD15).into();
        let withdrawing_id: UnitInstanceId = Uuid::from_u128(0xCD16).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Opponent));
        core.units
            .insert(withdrawing_id, runtime_unit(withdrawing_id, Side::Player));
        for buff_id in [BuffId::from_name("stun"), BuffId::from_name("silence")] {
            core.process_event(
                BattleEvent::ApplyBuff {
                    time_ms: 0,
                    caster_instance_id: caster_id,
                    target_instance_id: withdrawing_id,
                    buff_id,
                    duration_ms: 100,
                    cause: BattleEventCause::default(),
                },
                0,
            )
            .unwrap();
        }

        core.withdraw_unit(withdrawing_id, 10).unwrap();

        assert_eq!(
            core.units.get(&withdrawing_id).unwrap().lifecycle,
            RuntimeUnitLifecycle::Withdrawn
        );
        assert!(core.buffs.is_empty());
    }

    #[test]
    fn withdraw_rejects_dead_and_already_withdrawn_units_without_mutation() {
        for lifecycle in [RuntimeUnitLifecycle::Dead, RuntimeUnitLifecycle::Withdrawn] {
            let mut core = new_core();
            let unit_id: UnitInstanceId = Uuid::from_u128(0xCD18 + lifecycle as u128).into();
            let mut unit = runtime_unit(unit_id, Side::Player);
            unit.lifecycle = lifecycle;
            if lifecycle == RuntimeUnitLifecycle::Dead {
                unit.stats.current_health = 0;
            }
            core.units.insert(unit_id, unit);
            let event_count = core.event_log.entries.len();

            assert!(matches!(
                core.withdraw_unit(unit_id, 10),
                Err(crate::game::behavior::GameError::InvalidAction)
            ));
            assert_eq!(core.units.get(&unit_id).unwrap().lifecycle, lifecycle);
            assert_eq!(core.event_log.entries.len(), event_count);
        }
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
            start_gate_pending: false,
            reset_resonance: true,
            action: None,
            skill_id: skill_id.clone(),
            cast_target: None,
            accepted_target_tile: None,
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
                control_cancelled: false,
                flow: None,
                fact_source: Default::default(),
                execution_source: None,
                caster_instance_id: caster_id,
                ability_event_seq: cast_seq,
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

        let cancelled_cast_references_current_ability = |event_log: &BattleEventLog| {
            event_log.entries.iter().all(|entry| {
                let BattleLogEvent::SkillCastCancelled {
                    caster_instance_id,
                    interrupted_skill_id,
                    interrupted_cast_seq,
                    ..
                } = &entry.event
                else {
                    return true;
                };
                event_log.entries.iter().any(|candidate| {
                    candidate.seq == *interrupted_cast_seq
                        && matches!(
                            &candidate.event,
                            BattleLogEvent::AbilityCast {
                                skill_id,
                                caster_instance_id: ability_caster_id,
                                ..
                            } if skill_id == interrupted_skill_id
                                && ability_caster_id == caster_instance_id
                        )
                })
            })
        };
        assert!(cancelled_cast_references_current_ability(&core.event_log));

        let mut malformed_cancel = core.event_log.clone();
        let cancelled = malformed_cancel
            .entries
            .iter_mut()
            .find_map(|entry| match &mut entry.event {
                BattleLogEvent::SkillCastCancelled {
                    interrupted_cast_seq,
                    ..
                } => Some(interrupted_cast_seq),
                _ => None,
            })
            .expect("produced cancelled-cast entry");
        *cancelled = u64::MAX;
        assert!(!cancelled_cast_references_current_ability(
            &malformed_cancel
        ));
    }

    #[test]
    fn withdraw_cancels_unreleased_instant_basic_attack() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xCD41).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD42).into();
        core.units
            .insert(attacker_id, runtime_unit(attacker_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, attacker_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;
        let source_snapshot = core
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                5,
                DamageModifiers::default(),
                1,
                0,
                true,
            )
            .expect("windup source snapshot");

        core.withdraw_unit(attacker_id, 5).unwrap();
        core.handle_basic_attack_resolve_event(
            10,
            attacker_id,
            target_id,
            0,
            source_snapshot,
            AttackKind::Auto,
            crate::game::battle::event_log::AttackDelivery::Instant,
            BattleEventCause::default(),
        );

        assert_eq!(
            core.units.get(&target_id).unwrap().stats.current_health,
            target_hp
        );
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::AttackResolve {
                attacker_instance_id,
                ..
            } if attacker_instance_id == attacker_id
        )));
    }

    #[test]
    fn withdraw_preserves_released_basic_attack_projectile() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xCD51).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD52).into();
        core.units
            .insert(attacker_id, runtime_unit(attacker_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, attacker_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, attacker_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;
        let source_snapshot = core
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                5,
                DamageModifiers::default(),
                1,
                0,
                true,
            )
            .expect("released projectile source snapshot");
        core.spawn_basic_attack_projectile(super::basic_attack_projectile::ProjectileLaunch {
            spatial: None,
            fired_at_ms: 0,
            attacker_instance_id: attacker_id,
            attacker_owner_at_launch: Side::Player,
            air_capable_at_launch: true,
            target_instance_id: target_id,
            attacker_origin: core
                .unit_world_position_or_tile_center(attacker_id)
                .unwrap(),
            target_aim: core.unit_world_position_or_tile_center(target_id).unwrap(),
            speed_units_per_ms: 1_000_000,
            guidance: super::types::ProjectileGuidance::Homing,
            damage_type: DamageType::Physical,
            source_snapshot,
        });
        let projectile_id = *core.projectiles.keys().next().unwrap();

        core.withdraw_unit(attacker_id, 0).unwrap();
        assert!(core.projectiles.contains_key(&projectile_id));
        core.advance_basic_attack_projectile(1, projectile_id);

        assert!(core.units.get(&target_id).unwrap().stats.current_health < target_hp);
    }

    #[test]
    fn relocation_preserves_released_basic_projectiles_and_guidance_meaning() {
        let mut core = new_core();
        let target_id = match core
            .apply_live_command(
                crate::game::battle::core::sim::BattleLiveCommand::DeployPlayerUnit {
                    draft: fixture_draft(Uuid::from_u128(0xCE01), Uuid::from_u128(0xCE02), 100, 10),
                    position: Position::new(1, 1),
                    tile_kind: TacticalTileKind::Ground,
                    facing: FacingDirection::Right,
                    instance_salt: 1,
                    time_ms: 0,
                    current_hp_policy: None,
                },
            )
            .unwrap()
        {
            crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id } => {
                unit_id
            }
            other => panic!("unexpected deploy outcome: {other:?}"),
        };
        core.set_engagement_stance(target_id, EngagementStance::Defense, 1_500)
            .unwrap();

        let attacker_id: UnitInstanceId = Uuid::from_u128(0xCE03).into();
        core.units
            .insert(attacker_id, runtime_unit(attacker_id, Side::Opponent));
        place_unit(&mut core, attacker_id, Position::new(0, 1));
        sync_unit_to_battlefield_tile_center(&mut core, attacker_id);
        let initial_target_aim = core.units[&target_id].body.position;
        let source_snapshot = core
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                5,
                DamageModifiers::default(),
                1,
                1_500,
                true,
            )
            .unwrap();
        for guidance in [
            super::types::ProjectileGuidance::Homing,
            super::types::ProjectileGuidance::Fixed,
        ] {
            core.spawn_basic_attack_projectile(super::basic_attack_projectile::ProjectileLaunch {
                spatial: None,
                fired_at_ms: 1_500,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Opponent,
                air_capable_at_launch: true,
                target_instance_id: target_id,
                attacker_origin: core.units[&attacker_id].body.position,
                target_aim: initial_target_aim,
                speed_units_per_ms: 100_000,
                guidance,
                damage_type: DamageType::Physical,
                source_snapshot: source_snapshot.clone(),
            });
        }
        let projectile_ids = core.projectiles.keys().copied().collect::<Vec<_>>();
        assert_eq!(projectile_ids.len(), 2);

        core.tactically_relocate_unit(
            target_id,
            Position::new(3, 1),
            TacticalTileKind::Ground,
            FacingDirection::Left,
            1_500,
        )
        .unwrap();
        let relocated_aim = WorldVec2::from_tile_center(Position::new(3, 1));
        for projectile_id in projectile_ids {
            let guidance = core.projectiles[&projectile_id].guidance;
            core.advance_basic_attack_projectile(1_501, projectile_id);
            let projectile = core
                .projectiles
                .get(&projectile_id)
                .expect("released projectile should remain active after relocation");
            match guidance {
                super::types::ProjectileGuidance::Homing => {
                    assert_eq!(projectile.aim, relocated_aim)
                }
                super::types::ProjectileGuidance::Fixed => {
                    assert_eq!(projectile.aim, initial_target_aim)
                }
            }
        }
    }

    #[test]
    fn relocation_preserves_ground_fixed_and_reanchors_follow_areas() {
        for tracking in [
            SkillAreaTracking::GroundFixed,
            SkillAreaTracking::FollowCaster,
            SkillAreaTracking::FollowTarget,
        ] {
            let follows_target = tracking == SkillAreaTracking::FollowTarget;
            let skill_target = if follows_target {
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                }
            } else {
                SkillTarget::SelfUnit
            };
            let area = SkillTileAreaDeliveryDef {
                anchor: if follows_target {
                    SkillAreaAnchorSource::CastTarget
                } else {
                    SkillAreaAnchorSource::Caster
                },
                tile_origin: if follows_target {
                    SkillTileAreaOrigin::Anchor
                } else {
                    SkillTileAreaOrigin::Caster
                },
                tracking,
                hit_targets: if follows_target {
                    crate::game::ability::SkillHitTargetFilter::Enemies
                } else {
                    crate::game::ability::SkillHitTargetFilter::Allies
                },
                include_caster: !follows_target,
                tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                duration_ms: 10_000,
                tick_interval_ms: Some(1_000),
            };
            let skill = single_step_skill(
                &format!("relocation_area_{tracking:?}"),
                skill_target,
                1,
                0,
                DeliveryDef::TileArea {
                    footprint: TileRangePattern {
                        include_anchor_tile: true,
                        rows: vec!["@".to_string()],
                    },
                    area: area.clone(),
                },
                if follows_target {
                    vec![damage_effect()]
                } else {
                    vec![SkillEffectDef::Heal { amount: 1 }]
                },
            );
            let mut core = core_with_skill_data(Vec::new(), vec![skill.clone()]);
            let relocated_id: UnitInstanceId = Uuid::from_u128(0xCF01).into();
            let caster_id: UnitInstanceId = if follows_target {
                Uuid::from_u128(0xCF02).into()
            } else {
                relocated_id
            };
            core.units
                .insert(relocated_id, runtime_unit(relocated_id, Side::Player));
            if follows_target {
                core.units
                    .insert(caster_id, runtime_unit(caster_id, Side::Opponent));
                place_unit(&mut core, caster_id, Position::new(0, 1));
                sync_unit_to_battlefield_tile_center(&mut core, caster_id);
            }
            place_unit(&mut core, relocated_id, Position::new(1, 1));
            sync_unit_to_battlefield_tile_center(&mut core, relocated_id);
            core.initialize_deployed_tactical_state(
                relocated_id,
                TacticalTileKind::Ground,
                crate::game::data::equipment_data::WeaponCombatProfile::default(),
                0,
            )
            .unwrap();

            let step_target = follows_target.then_some(SkillCastTarget::Unit {
                unit_instance_id: relocated_id,
            });
            assert!(core
                .register_persistent_tile_area(
                    1_000,
                    77,
                    0,
                    caster_id,
                    skill.id.clone(),
                    skill.steps[0].id.clone(),
                    &skill.steps[0],
                    step_target,
                    TileRangePattern {
                        include_anchor_tile: true,
                        rows: vec!["@".to_string()],
                    },
                    area,
                )
                .unwrap());
            let area_id = *core.active_areas.keys().next().unwrap();
            let initial_center = core.active_areas[&area_id].center;
            core.set_engagement_stance(relocated_id, EngagementStance::Defense, 1_500)
                .unwrap();
            core.tactically_relocate_unit(
                relocated_id,
                Position::new(3, 1),
                TacticalTileKind::Ground,
                FacingDirection::Left,
                1_500,
            )
            .unwrap();
            assert!(core.active_areas.contains_key(&area_id));
            core.apply_skill_area_tick(1_501, area_id).unwrap();
            let center_after = core.active_areas[&area_id].center;
            if tracking == SkillAreaTracking::GroundFixed {
                assert_eq!(center_after, initial_center);
            } else {
                assert_eq!(
                    center_after,
                    WorldVec2::from_tile_center(Position::new(3, 1))
                );
            }
        }
    }

    #[test]
    fn withdraw_preserves_released_skill_projectile_with_launch_snapshot() {
        let skill = single_step_skill(
            "withdraw_released_projectile",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            vec![SkillEffectDef::Damage {
                amount: 5,
                damage_type: DamageType::Magic,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD61).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD62).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("queued projectile step");
        core.process_event(step, 0).unwrap();
        let delivery_id = *core.active_projectiles.keys().next().unwrap();

        core.withdraw_unit(caster_id, 0).unwrap();
        assert!(core.active_projectiles.contains_key(&delivery_id));
        assert!(core.active_skill_casts.is_empty());
        // Provenance must survive both cast removal and bounded ancestor eviction.
        core.event_log.entries.clear();

        for _ in 0..8 {
            let Some(event) = core.event_queue.pop() else {
                break;
            };
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).unwrap();
            if core.units.get(&target_id).unwrap().stats.current_health < target_hp {
                break;
            }
        }

        assert!(core.units.get(&target_id).unwrap().stats.current_health < target_hp);
        let source = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| {
                matches!(entry.event, BattleLogEvent::HpChanged { .. })
                    .then_some(entry.skill_effect_source.as_ref())
                    .flatten()
            })
            .expect("released projectile retains authored source after withdrawal");
        assert_eq!(source.cast.caster_instance_id, caster_id);
        assert_eq!(source.authored.site.action.definition_id, skill.id);
        assert_eq!(source.authored.site.effect_id, "effect_00");
    }

    #[test]
    fn released_projectile_keeps_snapshot_expression_after_source_withdrawal() {
        let skill = single_step_skill(
            "withdraw_snapshot_expr_projectile",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            vec![SkillEffectDef::DamageExpr {
                amount: crate::game::value_expr::AmountExpr::authored(
                    crate::game::value_expr::EvalMoment::Snapshot,
                    crate::game::value_expr::ValueExpr::Read(
                        crate::game::value_expr::ValueRef::Stat {
                            subject: crate::game::value_expr::ValueSubject::Source,
                            stat: crate::game::value_expr::StatSelector::Attack,
                        },
                    ),
                ),
                damage_type: DamageType::True,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD63).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD64).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        let expected_damage = core.units[&caster_id].stats.attack;
        let target_hp = core.units[&target_id].stats.current_health;

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("queued projectile step");
        core.process_event(step, 0).unwrap();
        let delivery_id = *core.active_projectiles.keys().next().unwrap();
        assert!(core.active_projectiles[&delivery_id]
            .source_snapshot
            .value_expr_capture
            .is_some());

        core.withdraw_unit(caster_id, 0).unwrap();
        for _ in 0..8 {
            let Some(event) = core.event_queue.pop() else {
                break;
            };
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).unwrap();
            if core.units[&target_id].stats.current_health < target_hp {
                break;
            }
        }
        assert_eq!(
            core.units[&target_id].stats.current_health,
            target_hp.saturating_sub(expected_damage)
        );
    }

    fn resolved_attack_amount(
        moment: crate::game::value_expr::EvalMoment,
    ) -> crate::game::value_expr::AmountExpr {
        use crate::game::value_expr::{
            FiniteSource, Grouping, Reduction, StatSelector, ValueExpr, ValueRef, ValueSubject,
        };
        crate::game::value_expr::AmountExpr::authored(
            moment,
            ValueExpr::Aggregate {
                source: FiniteSource::ResolvedTargets,
                grouping: Grouping::WholeSet,
                reduction: Reduction::Sum,
                outer_reduction: None,
                member: Box::new(ValueExpr::Read(ValueRef::Stat {
                    subject: ValueSubject::AggregateMember,
                    stat: StatSelector::Attack,
                })),
            },
        )
    }

    #[test]
    fn target_locked_projectile_uses_release_members_for_snapshot_and_impact_members_for_live() {
        for (moment, expected_damage) in [
            (crate::game::value_expr::EvalMoment::Snapshot, 5_u32),
            (crate::game::value_expr::EvalMoment::Live, 9_u32),
        ] {
            let skill = single_step_skill(
                "resolved_attack_projectile",
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                3,
                0,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000_000,
                    hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                    allow_targetless_cast: false,
                    max_range_tiles: None,
                    max_lifetime_ms: None,
                    max_kills: None,
                    max_pierces: None,
                    collision: Default::default(),
                },
                vec![SkillEffectDef::DamageExpr {
                    amount: resolved_attack_amount(moment),
                    damage_type: DamageType::True,
                }],
            );
            let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
            let caster_id: UnitInstanceId = Uuid::from_u128(0xCD65).into();
            let target_id: UnitInstanceId = Uuid::from_u128(0xCD66).into();
            core.units
                .insert(caster_id, runtime_unit(caster_id, Side::Player));
            core.units
                .insert(target_id, runtime_unit(target_id, Side::Opponent));
            place_unit(&mut core, caster_id, Position::new(1, 1));
            place_unit(&mut core, target_id, Position::new(2, 1));
            sync_unit_to_battlefield_tile_center(&mut core, caster_id);
            sync_unit_to_battlefield_tile_center(&mut core, target_id);
            core.units.get_mut(&target_id).unwrap().stats.attack = 5;
            let initial_hp = core.units[&target_id].stats.current_health;
            core.invoke_ability(
                0,
                caster_id,
                &skill.id,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: target_id,
                }),
                BattleEventCause::default(),
                false,
            );
            let step = core.event_queue.pop().expect("queued projectile step");
            core.process_event(step, 0).unwrap();
            core.units.get_mut(&target_id).unwrap().stats.attack = 9;
            for _ in 0..8 {
                let Some(event) = core.event_queue.pop() else {
                    break;
                };
                let time_ms = event.time_ms();
                core.process_event(event, time_ms).unwrap();
                if core.units[&target_id].stats.current_health < initial_hp {
                    break;
                }
            }
            assert_eq!(
                core.units[&target_id].stats.current_health,
                initial_hp - expected_damage
            );
        }
    }

    #[derive(Clone, Copy)]
    enum D04F01TargetDisposition {
        Removed,
        Inactive,
        EscapedBeforeTimeout,
    }

    fn d04_f01_lost_target_projectile(
        moment: crate::game::value_expr::EvalMoment,
        aggregate: bool,
        disposition: D04F01TargetDisposition,
    ) {
        use crate::game::value_expr::{AmountExpr, TypedInteger, ValueExpr, ValueUnit};
        let amount = if aggregate {
            resolved_attack_amount(moment)
        } else {
            AmountExpr::authored(
                moment,
                ValueExpr::Const(TypedInteger::new(1, ValueUnit::Magnitude)),
            )
        };
        let skill = single_step_skill(
            "d04_f01_lost_target",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            vec![SkillEffectDef::DamageExpr {
                amount,
                damage_type: DamageType::True,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD65).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD66).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("queued projectile step");
        core.process_event(step, 0).unwrap();
        let projectile = core
            .active_projectiles
            .values()
            .next()
            .expect("released projectile");
        assert!(projectile.source_snapshot.value_expr_capture.is_some());
        let action = projectile
            .source_snapshot
            .fact_source
            .action
            .clone()
            .expect("action occurrence");
        match disposition {
            D04F01TargetDisposition::Removed => {
                core.units.remove(&target_id);
            }
            D04F01TargetDisposition::Inactive => {
                core.units.get_mut(&target_id).unwrap().lifecycle = RuntimeUnitLifecycle::Withdrawn;
            }
            D04F01TargetDisposition::EscapedBeforeTimeout => {
                place_unit(&mut core, target_id, Position::new(5, 1));
                sync_unit_to_battlefield_tile_center(&mut core, target_id);
            }
        }
        let mut processed = 0;
        for _ in 0..16 {
            let Some(event) = core.event_queue.pop() else {
                break;
            };
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).unwrap();
            processed += 1;
        }
        assert!(processed > 0);
        assert!(core.active_projectiles.is_empty());
        assert!(core.active_skill_casts.is_empty());
        assert!(core.event_queue.is_empty());
        if !matches!(disposition, D04F01TargetDisposition::Removed) {
            assert_eq!(core.units[&target_id].stats.current_health, 10);
        }
        let snapshot = core.runtime_fact_snapshot().unwrap();
        let settled = snapshot.latest_settled_action.as_ref().unwrap();
        assert_eq!(settled.ordinal, action.ordinal());
        assert_eq!(settled.outstanding_deliveries, 0);
        assert_eq!(settled.pending_control_steps, 0);
        assert!(settled.effects_settled_at_ms.is_some());
        assert_eq!(settled.facts.damage_mutations, 0);
        assert_eq!(settled.facts.damage_hp_lost, 0);
        assert_eq!(settled.facts.deaths, 0);
        assert_eq!(core.runtime_fact_usage().active_actions, 0);
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::SkillProjectileImpacted {
                        first_hit_unit_id: None,
                        terminal: true,
                        ..
                    }
                ))
                .count(),
            1
        );
    }

    #[test]
    fn d04_f01_scalar_snapshot_lost_target_completes_empty_impact() {
        d04_f01_lost_target_projectile(
            crate::game::value_expr::EvalMoment::Snapshot,
            false,
            D04F01TargetDisposition::Removed,
        );
    }

    #[test]
    fn d04_f01_scalar_live_lost_target_completes_empty_impact() {
        d04_f01_lost_target_projectile(
            crate::game::value_expr::EvalMoment::Live,
            false,
            D04F01TargetDisposition::Removed,
        );
    }

    #[test]
    fn d04_f01_resolved_snapshot_lost_target_completes_empty_impact() {
        d04_f01_lost_target_projectile(
            crate::game::value_expr::EvalMoment::Snapshot,
            true,
            D04F01TargetDisposition::Removed,
        );
    }

    #[test]
    fn d04_f01_resolved_live_lost_target_completes_empty_impact() {
        d04_f01_lost_target_projectile(
            crate::game::value_expr::EvalMoment::Live,
            true,
            D04F01TargetDisposition::Removed,
        );
    }

    #[test]
    fn d04_f01_inactive_target_completes_empty_impact() {
        d04_f01_lost_target_projectile(
            crate::game::value_expr::EvalMoment::Live,
            true,
            D04F01TargetDisposition::Inactive,
        );
    }

    #[test]
    fn d04_f01_escaped_target_timeout_completes_empty_impact() {
        d04_f01_lost_target_projectile(
            crate::game::value_expr::EvalMoment::Snapshot,
            true,
            D04F01TargetDisposition::EscapedBeforeTimeout,
        );
    }

    fn d04_f01_resource_projectile(
        failing_target_owner: bool,
    ) -> (
        BattleCore,
        UnitInstanceId,
        crate::game::resource::ResourceId,
        DamageSourceSnapshot,
    ) {
        use crate::game::resource::{
            ResourceCatalog, ResourceDefinition, ResourceId, ResourceLifetime, ResourceMutation,
            ResourceOperation, ResourceOwnerKind, ResourceOwnerSelector,
        };
        use crate::game::value_expr::{AmountExpr, EvalMoment, TypedInteger, ValueExpr, ValueUnit};
        let resource = ResourceId::new("d04_f01_meter".to_string()).unwrap();
        let mut effects = vec![
            SkillEffectDef::DamageExpr {
                amount: AmountExpr::authored(
                    EvalMoment::Live,
                    ValueExpr::Const(TypedInteger::new(1, ValueUnit::Magnitude)),
                ),
                damage_type: DamageType::True,
            },
            SkillEffectDef::HealExpr {
                amount: AmountExpr::authored(
                    EvalMoment::Live,
                    ValueExpr::Const(TypedInteger::new(1, ValueUnit::Magnitude)),
                ),
            },
            SkillEffectDef::Resources {
                operations: vec![ResourceOperation {
                    resource: resource.clone(),
                    owner: ResourceOwnerSelector::SourceEmployee,
                    operation: ResourceMutation::Grant(3),
                }],
            },
        ];
        if failing_target_owner {
            effects.push(SkillEffectDef::Resources {
                operations: vec![ResourceOperation {
                    resource: resource.clone(),
                    owner: ResourceOwnerSelector::TargetEmployee,
                    operation: ResourceMutation::Grant(1),
                }],
            });
        }
        let skill = single_step_skill(
            "d04_f01_resource_after_empty_expr",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            effects,
        );
        let database = GameDataBuilder::live_defaults()
            .with_skills(SkillDatabase::new(vec![skill.clone()]))
            .with_resources(ResourceCatalog {
                resources: vec![ResourceDefinition {
                    id: resource.clone(),
                    owner: ResourceOwnerKind::Employee,
                    lifetime: ResourceLifetime::Attempt,
                    initial: 0,
                    cap: 20,
                }],
            })
            .build_arc();
        let mut core = BattleCore::new_from_scenario(BattleScenario::empty((6, 6)), database, 123);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD75).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD76).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        core.prepare_runtime_resources().unwrap();
        core.runtime_facts
            .resources
            .as_mut()
            .unwrap()
            .store
            .reconcile_owners(&std::collections::BTreeSet::from([
                crate::game::resource::ResourceOwner::Employee(caster_id.as_uuid()),
            ]))
            .unwrap();
        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("queued projectile step");
        core.process_event(step, 0).unwrap();
        assert_eq!(core.active_projectiles.len(), 1);
        let projectile_snapshot = core
            .active_projectiles
            .values()
            .next()
            .unwrap()
            .source_snapshot
            .clone();
        core.units.remove(&target_id);
        let advance = core.event_queue.pop().expect("projectile advance");
        let time_ms = advance.time_ms();
        core.process_event(advance, time_ms).unwrap();
        assert!(matches!(
            core.event_queue.iter().next().as_deref(),
            Some(BattleEvent::SkillProjectileImpact { .. })
        ));
        (core, caster_id, resource, projectile_snapshot)
    }

    fn d04_f01_source_balance(
        core: &BattleCore,
        caster: UnitInstanceId,
        resource: &crate::game::resource::ResourceId,
    ) -> u32 {
        core.typed_resources()
            .unwrap()
            .entries()
            .find(|(key, _)| {
                key.owner == crate::game::resource::ResourceOwner::Employee(caster.as_uuid())
                    && key.resource == *resource
            })
            .map(|(_, value)| value.balance)
            .unwrap_or(0)
    }

    #[test]
    fn d04_f01_empty_expression_still_grants_source_resource_once() {
        let (mut core, caster, resource, _) = d04_f01_resource_projectile(false);
        let impact = core.event_queue.pop().unwrap();
        let time_ms = impact.time_ms();
        core.process_event(impact, time_ms).unwrap();
        assert_eq!(d04_f01_source_balance(&core, caster, &resource), 3);
        assert_eq!(core.runtime_fact_summary().resource_change_mutations, 1);
        assert_eq!(core.runtime_fact_summary().resource_change_delta, 3);
        assert_eq!(core.runtime_fact_summary().damage_mutations, 0);
        assert_eq!(core.runtime_fact_summary().heal_mutations, 0);
        assert!(core.active_skill_casts.is_empty());
        assert!(core.active_projectiles.is_empty());
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn d04_f01_empty_effect_result_counts_and_capture_identity_stay_exact() {
        let (mut core, caster, resource, snapshot) = d04_f01_resource_projectile(false);
        let capture = snapshot.value_expr_capture.as_deref().unwrap();
        let release_target: UnitInstanceId = Uuid::from_u128(0xCD76).into();
        assert!(capture.capture_for(0, Some(release_target)).is_some());
        assert!(capture.capture_for(0, Some(caster)).is_none());
        assert!(capture.capture_for(0, None).is_none());
        let skill = core
            .game_data
            .skill_data
            .get_by_id(&SkillId::from("d04_f01_resource_after_empty_expr"))
            .unwrap()
            .clone();
        let result = core
            .execute_ordered_skill_effects(
                caster,
                &skill.id,
                &skill.steps[0],
                &[],
                core.event_queue.peek_time_ms().unwrap(),
                snapshot.committed_at_ms,
                Some(&snapshot),
                snapshot.skill_cast_source.clone(),
            )
            .unwrap();
        assert_eq!(result.resolved_target_count, 0);
        assert_eq!(result.actual_damage_target_count, 0);
        assert_eq!(result.killed_target_count, 0);
        assert_eq!(result.applied_effect_count, 1);
        assert_eq!(d04_f01_source_balance(&core, caster, &resource), 3);
    }

    #[test]
    fn d04_f01_actual_target_resource_error_restores_candidate() {
        use crate::game::battle::runtime_control::RuntimeFaultCode;
        use crate::game::resource::store::ResourceFaultCode;
        use crate::game::resource::{ResourceMutation, ResourceOperation, ResourceOwnerSelector};
        let (mut core, caster, resource, projectile_snapshot) = d04_f01_resource_projectile(true);
        let source = core.runtime_fact_source.clone();
        let projectile_source = projectile_snapshot.fact_source.clone();
        core.runtime_transaction(|candidate| {
            candidate.runtime_fact_source = projectile_source;
            let result = candidate.apply_typed_resource_operations(
                &[ResourceOperation {
                    resource: resource.clone(),
                    owner: ResourceOwnerSelector::SourceEmployee,
                    operation: ResourceMutation::Grant(2),
                }],
                &[],
                0,
            );
            candidate.runtime_fact_source = source;
            result
        })
        .unwrap();
        assert_eq!(d04_f01_source_balance(&core, caster, &resource), 2);
        let before = finite_boundary_authority_snapshot(&core);
        let error = core
            .runtime_transaction(|candidate| {
                let impact = candidate.event_queue.pop().expect("queued empty impact");
                let time_ms = impact.time_ms();
                candidate.process_event(impact, time_ms)
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(fault)
                if fault.code == RuntimeFaultCode::Resource(ResourceFaultCode::InvalidOwner)
        ));
        assert_eq!(finite_boundary_authority_snapshot(&core), before);
        assert_eq!(d04_f01_source_balance(&core, caster, &resource), 2);
    }

    #[test]
    fn immediate_expression_error_propagates_through_event_and_restores_queue() {
        let skill = single_step_skill(
            "immediate_expr_candidate_rollback",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Instant,
            vec![
                SkillEffectDef::Damage {
                    amount: 200,
                    damage_type: DamageType::True,
                },
                SkillEffectDef::HealExpr {
                    amount: crate::game::value_expr::AmountExpr::authored(
                        crate::game::value_expr::EvalMoment::Live,
                        crate::game::value_expr::ValueExpr::Read(
                            crate::game::value_expr::ValueRef::Stat {
                                subject: crate::game::value_expr::ValueSubject::Target,
                                stat: crate::game::value_expr::StatSelector::Attack,
                            },
                        ),
                    ),
                },
            ],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD67).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD68).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let before = finite_boundary_authority_snapshot(&core);
        let error = core
            .runtime_transaction(|candidate| {
                let event = candidate.event_queue.pop().expect("queued immediate step");
                let time_ms = event.time_ms();
                candidate.process_event(event, time_ms)
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(crate::game::value_expr::ValueExprFault {
                code: crate::game::value_expr::ValueExprFaultCode::MissingContext,
                ..
            })
        ));
        assert_eq!(finite_boundary_authority_snapshot(&core), before);
        assert!(core.units[&target_id].is_active());
    }

    #[test]
    fn live_projectile_expression_error_propagates_and_restores_impact_event() {
        let skill = single_step_skill(
            "withdraw_live_expr_projectile",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            vec![SkillEffectDef::DamageExpr {
                amount: crate::game::value_expr::AmountExpr::authored(
                    crate::game::value_expr::EvalMoment::Live,
                    crate::game::value_expr::ValueExpr::Read(
                        crate::game::value_expr::ValueRef::Stat {
                            subject: crate::game::value_expr::ValueSubject::Source,
                            stat: crate::game::value_expr::StatSelector::Attack,
                        },
                    ),
                ),
                damage_type: DamageType::True,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD69).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD6A).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("queued projectile step");
        core.process_event(step, 0).unwrap();
        core.withdraw_unit(caster_id, 1).unwrap();

        let mut observed = false;
        for _ in 0..8 {
            let before = finite_boundary_authority_snapshot(&core);
            let result = core.runtime_transaction(|candidate| {
                let event = candidate
                    .event_queue
                    .pop()
                    .expect("queued projectile advance or impact");
                let time_ms = event.time_ms();
                candidate.process_event(event, time_ms)
            });
            match result {
                Ok(()) => {}
                Err(error) => {
                    assert!(matches!(
                        error,
                        GameError::ValueExpressionFault(crate::game::value_expr::ValueExprFault {
                            code: crate::game::value_expr::ValueExprFaultCode::MissingContext,
                            ..
                        })
                    ));
                    assert_eq!(finite_boundary_authority_snapshot(&core), before);
                    observed = true;
                    break;
                }
            }
        }
        assert!(
            observed,
            "projectile impact did not surface the Live expression error"
        );
    }

    #[test]
    fn withdraw_cancels_queued_unreleased_skill_step() {
        let mut skill = single_step_skill(
            "withdraw_queued_skill_step",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Damage {
                amount: 5,
                damage_type: DamageType::Magic,
            }],
        );
        skill.steps[0].delay_ms = 10;
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD65).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD66).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let queued_step = core.event_queue.pop().expect("queued delayed step");
        assert_eq!(queued_step.time_ms(), 10);
        core.withdraw_unit(caster_id, 1).unwrap();
        core.process_event(queued_step, 10).unwrap();

        assert_eq!(
            core.units.get(&target_id).unwrap().stats.current_health,
            target_hp
        );
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::AbilityStepTriggered { skill_id, .. }
                if skill_id == &skill.id
        )));
    }

    #[test]
    fn withdraw_preserves_ground_fixed_area_and_ends_follow_areas() {
        for (tracking, expected_to_survive) in [
            (SkillAreaTracking::GroundFixed, true),
            (SkillAreaTracking::FollowCaster, false),
            (SkillAreaTracking::FollowTarget, false),
        ] {
            let follows_target = tracking == SkillAreaTracking::FollowTarget;
            let skill_target = if follows_target {
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                }
            } else {
                SkillTarget::SelfUnit
            };
            let (anchor, tile_origin, footprint) = if follows_target {
                (
                    SkillAreaAnchorSource::CastTarget,
                    SkillTileAreaOrigin::Anchor,
                    TileRangePattern {
                        include_anchor_tile: true,
                        rows: vec!["@".to_string()],
                    },
                )
            } else {
                (
                    SkillAreaAnchorSource::Caster,
                    SkillTileAreaOrigin::Caster,
                    TileRangePattern {
                        include_anchor_tile: false,
                        rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
                    },
                )
            };
            let skill = single_step_skill(
                &format!("withdraw_area_{tracking:?}"),
                skill_target,
                3,
                0,
                DeliveryDef::TileArea {
                    footprint,
                    area: SkillTileAreaDeliveryDef {
                        anchor,
                        tile_origin,
                        tracking,
                        hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                        include_caster: false,
                        tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                        duration_ms: 30,
                        tick_interval_ms: Some(10),
                    },
                },
                vec![SkillEffectDef::Damage {
                    amount: 2,
                    damage_type: DamageType::Magic,
                }],
            );
            let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
            let caster_id: UnitInstanceId =
                Uuid::from_u128(0xCD70 + tracking as u128 * 0x10).into();
            let target_id: UnitInstanceId =
                Uuid::from_u128(0xCD71 + tracking as u128 * 0x10).into();
            core.units
                .insert(caster_id, runtime_unit(caster_id, Side::Player));
            core.units
                .insert(target_id, runtime_unit(target_id, Side::Opponent));
            place_unit(&mut core, caster_id, Position::new(1, 1));
            place_unit(&mut core, target_id, Position::new(2, 1));
            sync_unit_to_battlefield_tile_center(&mut core, caster_id);
            sync_unit_to_battlefield_tile_center(&mut core, target_id);
            core.units.get_mut(&caster_id).unwrap().facing_direction = Some(FacingDirection::Up);
            let cast_target = follows_target.then_some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            });
            let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

            core.invoke_ability(
                0,
                caster_id,
                &skill.id,
                cast_target,
                BattleEventCause::default(),
                false,
            );
            let step = core.event_queue.pop().expect("queued persistent area step");
            core.process_event(step, 0).unwrap();
            let area_id = *core.active_areas.keys().next().unwrap();

            core.withdraw_unit(caster_id, 1).unwrap();
            assert_eq!(
                core.active_areas.contains_key(&area_id),
                expected_to_survive,
                "unexpected withdrawal lifetime for {tracking:?}"
            );
            core.apply_skill_area_tick(10, area_id).unwrap();
            let target_hp_after = core.units.get(&target_id).unwrap().stats.current_health;
            if expected_to_survive {
                assert!(target_hp_after < target_hp);
                let source = core
                    .event_log
                    .entries
                    .iter()
                    .rev()
                    .find_map(|entry| {
                        (entry.time_ms == 10
                            && matches!(entry.event, BattleLogEvent::HpChanged { .. }))
                        .then_some(entry.skill_effect_source.as_ref())
                        .flatten()
                    })
                    .expect("fixed area retains authored source after withdrawal");
                assert_eq!(source.cast.caster_instance_id, caster_id);
                assert_eq!(source.authored.site.action.definition_id, skill.id);
            } else {
                assert_eq!(target_hp_after, target_hp);
            }
        }
    }

    #[test]
    fn live_area_expression_error_propagates_and_rolls_back_candidate() {
        let skill = single_step_skill(
            "withdraw_live_expr_area",
            SkillTarget::SelfUnit,
            3,
            0,
            DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
                },
                area: SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::Caster,
                    tile_origin: SkillTileAreaOrigin::Caster,
                    tracking: SkillAreaTracking::GroundFixed,
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 30,
                    tick_interval_ms: Some(10),
                },
            },
            vec![SkillEffectDef::DamageExpr {
                amount: crate::game::value_expr::AmountExpr::authored(
                    crate::game::value_expr::EvalMoment::Live,
                    crate::game::value_expr::ValueExpr::Read(
                        crate::game::value_expr::ValueRef::Stat {
                            subject: crate::game::value_expr::ValueSubject::Source,
                            stat: crate::game::value_expr::StatSelector::Attack,
                        },
                    ),
                ),
                damage_type: DamageType::True,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD90).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD91).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        core.units.get_mut(&caster_id).unwrap().facing_direction = Some(FacingDirection::Up);

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            None,
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("queued persistent area step");
        core.process_event(step, 0).unwrap();
        let area_id = *core.active_areas.keys().next().unwrap();
        core.withdraw_unit(caster_id, 1).unwrap();
        assert!(core.active_areas.contains_key(&area_id));
        let before = finite_boundary_authority_snapshot(&core);
        let error = core
            .runtime_transaction(|candidate| candidate.apply_skill_area_tick(10, area_id))
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(crate::game::value_expr::ValueExprFault {
                code: crate::game::value_expr::ValueExprFaultCode::MissingContext,
                ..
            })
        ));
        assert_eq!(finite_boundary_authority_snapshot(&core), before);
    }

    #[test]
    fn resolved_area_tick_rebuilds_live_members_for_each_occurrence() {
        let skill = single_step_skill(
            "resolved_attack_area",
            SkillTarget::SelfUnit,
            3,
            0,
            DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
                },
                area: SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::Caster,
                    tile_origin: SkillTileAreaOrigin::Caster,
                    tracking: SkillAreaTracking::GroundFixed,
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 30,
                    tick_interval_ms: Some(10),
                },
            },
            vec![SkillEffectDef::DamageExpr {
                amount: resolved_attack_amount(crate::game::value_expr::EvalMoment::Live),
                damage_type: DamageType::True,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCD92).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCD93).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        core.units.get_mut(&caster_id).unwrap().facing_direction = Some(FacingDirection::Up);
        let target = core.units.get_mut(&target_id).unwrap();
        target.stats.attack = 5;
        target.stats.max_health = 100;
        target.stats.current_health = 100;
        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            None,
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("queued area step");
        core.process_event(step, 0).unwrap();
        let area_id = *core.active_areas.keys().next().unwrap();
        let initial_hp = core.units[&target_id].stats.current_health;
        core.apply_skill_area_tick(10, area_id).unwrap();
        assert_eq!(core.units[&target_id].stats.current_health, initial_hp - 5);
        core.units.get_mut(&target_id).unwrap().stats.attack = 9;
        core.apply_skill_area_tick(20, area_id).unwrap();
        assert_eq!(core.units[&target_id].stats.current_health, initial_hp - 14);
    }

    #[test]
    fn battle_without_satisfied_authored_terminal_does_not_implicitly_draw() {
        let mut policy = RunPolicyData::builtin();
        policy.battle_runtime.movement_tick_ms = 5;
        let game_data = GameDataBuilder::live_defaults()
            .with_run_policy(policy)
            .build_arc();
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups: Vec::new(),
            events: Vec::new(),
            terminal_outcomes: BattleScenario::fixture_time_victory_outcomes(1_000),
            boss_runtime: None,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, game_data, 123);
        let mut execution = core.start_battle_execution().unwrap();
        let outcome = core
            .step_battle_execution_until(&mut execution, 10)
            .expect("bounded step should succeed");

        assert_eq!(
            outcome,
            crate::game::battle::core::sim::BattleStepOutcome::Running
        );
        assert_eq!(execution.last_event_time_ms(), 10);
        assert!(!core
            .event_log
            .entries
            .iter()
            .any(|entry| { matches!(entry.event, BattleLogEvent::BattleEnd { .. }) }));
    }

    fn authored_time_outcome_core(win_time_ms: u64) -> BattleCore {
        let mut policy = RunPolicyData::builtin();
        policy.battle_runtime.movement_tick_ms = 5;
        let game_data = GameDataBuilder::live_defaults()
            .with_run_policy(policy)
            .build_arc();
        let scenario = crate::game::battle::scenario::BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups: Vec::new(),
            events: Vec::new(),
            terminal_outcomes: BattleScenario::fixture_time_victory_outcomes(win_time_ms),
            boss_runtime: None,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        BattleCore::new_from_scenario(scenario, game_data, 123)
    }

    #[test]
    fn elapsed_entrypoints_share_authored_time_outcome() {
        for entrypoint in ["until", "by"] {
            let mut core = authored_time_outcome_core(10);
            let mut execution = core
                .start_battle_execution()
                .expect("battle execution should start");
            let outcome = match entrypoint {
                "until" => core.step_battle_execution_until(&mut execution, 10),
                "by" => core.step_battle_execution_by(&mut execution, 10),
                _ => unreachable!(),
            }
            .expect("authored time step should succeed");

            assert_eq!(
                outcome,
                crate::game::battle::core::sim::BattleStepOutcome::Finished(
                    crate::game::battle::core::sim::BattleFinishSignal {
                        time_ms: 10,
                        outcome_id: crate::game::battle::types::BattleOutcomeId::new("victory"),
                        verdict: crate::game::battle::types::BattleVerdict::Victory,
                        winner: crate::game::battle::types::BattleWinner::Player,
                    }
                ),
                "{entrypoint} must expose the same terminal outcome"
            );
            assert_eq!(
                execution.last_event_time_ms(),
                10,
                "{entrypoint} must commit the authoritative clock"
            );
            assert!(execution.is_finished());
        }
    }

    #[test]
    fn time_before_authored_terminal_remains_running() {
        for entrypoint in ["until", "by"] {
            let mut core = authored_time_outcome_core(11);
            let mut execution = core.start_battle_execution().unwrap();
            let outcome = match entrypoint {
                "until" => core.step_battle_execution_until(&mut execution, 10),
                "by" => core.step_battle_execution_by(&mut execution, 10),
                _ => unreachable!(),
            }
            .unwrap();
            assert_eq!(
                outcome,
                crate::game::battle::core::sim::BattleStepOutcome::Running
            );
            assert_eq!(execution.last_event_time_ms(), 10);
        }
    }

    #[test]
    fn terminal_selection_is_defeat_first_then_priority_desc_then_id_asc() {
        use crate::game::battle::{
            scenario::{TerminalCondition, TerminalOutcome},
            types::{BattleOutcomeId, BattleVerdict},
        };

        let outcome = |id: &str, verdict, priority| TerminalOutcome {
            id: BattleOutcomeId::new(id),
            verdict,
            condition: TerminalCondition::TimeReached { time_ms: 10 },
            priority,
        };
        let mut scenario = BattleScenario::empty((4, 4));
        scenario.terminal_outcomes = vec![
            outcome("victory_high", BattleVerdict::Victory, 100),
            outcome("defeat_low", BattleVerdict::Defeat, -100),
        ];
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let mut execution = core.start_battle_execution().unwrap();
        let crate::game::battle::core::sim::BattleStepOutcome::Finished(selected) = core
            .step_battle_execution_until(&mut execution, 10)
            .expect("both outcomes should match")
        else {
            panic!("matching terminal outcomes must finish");
        };
        assert_eq!(selected.outcome_id.as_str(), "defeat_low");

        let mut scenario = BattleScenario::empty((4, 4));
        scenario.terminal_outcomes = vec![
            TerminalOutcome {
                id: BattleOutcomeId::new("victory_later"),
                verdict: BattleVerdict::Victory,
                condition: TerminalCondition::TimeReached { time_ms: 100 },
                priority: 0,
            },
            outcome("defeat_low_priority", BattleVerdict::Defeat, 1),
            outcome("defeat_z", BattleVerdict::Defeat, 5),
            outcome("defeat_a", BattleVerdict::Defeat, 5),
        ];
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let mut execution = core.start_battle_execution().unwrap();
        let crate::game::battle::core::sim::BattleStepOutcome::Finished(selected) = core
            .step_battle_execution_until(&mut execution, 10)
            .expect("all Defeat outcomes should match")
        else {
            panic!("matching Defeat outcomes must finish");
        };
        assert_eq!(selected.outcome_id.as_str(), "defeat_a");
    }

    #[test]
    fn all_player_combatants_defeated_requires_existing_dead_combatants() {
        use crate::game::battle::{
            scenario::{TerminalCondition, TerminalOutcome},
            types::{BattleOutcomeId, BattleVerdict},
        };

        let mut scenario = BattleScenario::empty((4, 4));
        scenario.terminal_outcomes = vec![
            TerminalOutcome {
                id: BattleOutcomeId::new("victory_later"),
                verdict: BattleVerdict::Victory,
                condition: TerminalCondition::TimeReached { time_ms: 100 },
                priority: 0,
            },
            TerminalOutcome {
                id: BattleOutcomeId::new("defeat"),
                verdict: BattleVerdict::Defeat,
                condition: TerminalCondition::AllPlayerCombatantsDefeated,
                priority: 0,
            },
        ];
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let mut execution = core.start_battle_execution().unwrap();
        assert_eq!(
            core.step_battle_execution_until(&mut execution, 0).unwrap(),
            crate::game::battle::core::sim::BattleStepOutcome::Running
        );

        let player_id = UnitInstanceId::from(Uuid::from_u128(0xD34D));
        let mut withdrawn = runtime_unit(player_id, Side::Player);
        withdrawn.lifecycle = RuntimeUnitLifecycle::Withdrawn;
        core.units.insert(player_id, withdrawn);
        assert_eq!(
            core.step_battle_execution_until(&mut execution, 0).unwrap(),
            crate::game::battle::core::sim::BattleStepOutcome::Running
        );

        core.units.get_mut(&player_id).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;
        let crate::game::battle::core::sim::BattleStepOutcome::Finished(signal) =
            core.step_battle_execution_until(&mut execution, 0).unwrap()
        else {
            panic!("an existing dead player combatant must satisfy Defeat");
        };
        assert_eq!(signal.outcome_id.as_str(), "defeat");
        assert_eq!(signal.verdict, BattleVerdict::Defeat);
    }

    #[test]
    fn terminal_state_clamps_clock_skips_late_event_and_finalizes_once() {
        let mut core = authored_time_outcome_core(10);
        let mut execution = core
            .start_battle_execution()
            .expect("battle execution should start");
        match core
            .step_battle_execution_until(&mut execution, 10)
            .expect("battle should reach its authored terminal")
        {
            crate::game::battle::core::sim::BattleStepOutcome::Finished(_) => {}
            crate::game::battle::core::sim::BattleStepOutcome::Running => {
                panic!("battle must finish at its authored terminal")
            }
        }

        assert_eq!(execution.last_event_time_ms(), 10);
        assert!(core.event_queue.iter().all(|event| event.time_ms() > 10));
        let result = core
            .finalize_battle_execution(&mut execution)
            .expect("terminal state should finalize once");
        assert_eq!(
            result.winner,
            crate::game::battle::types::BattleWinner::Player
        );
        assert_eq!(result.outcome_id.as_str(), "victory");
        assert_eq!(
            result.verdict,
            crate::game::battle::types::BattleVerdict::Victory
        );
        assert_eq!(
            result
                .event_log
                .entries
                .iter()
                .filter(|entry| matches!(entry.event, BattleLogEvent::BattleEnd { .. }))
                .count(),
            1
        );
        assert!(result
            .event_log
            .entries
            .iter()
            .all(|entry| entry.time_ms <= 10));
        assert!(matches!(
            core.finalize_battle_execution(&mut execution),
            Err(GameError::InvalidAction)
        ));
        assert!(matches!(
            core.step_battle_execution(&mut execution),
            Err(GameError::InvalidAction)
        ));
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
            event.as_ref(),
            BattleEvent::ContinuousMovementTick { time_ms } if *time_ms == 124
        )));

        core.schedule_continuous_movement_tick(125);
        assert!(core.event_queue.iter().any(|event| matches!(
            event.as_ref(),
            BattleEvent::ContinuousMovementTick { time_ms } if *time_ms == 125
        )));
    }

    #[test]
    fn same_timestamp_attack_resolve_is_drained_before_battle_end() {
        let mut core = new_core();
        Arc::make_mut(&mut core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(10);
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

        let source_snapshot = core
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
            .expect("source snapshot");
        core.event_queue.push(BattleEvent::AttackResolve {
            time_ms: 10,
            attacker_instance_id: attacker_id,
            target_instance_id: target_id,
            attack_epoch: 0,
            source_snapshot,
            kind: AttackKind::Auto,
            delivery: crate::game::battle::event_log::AttackDelivery::Instant,
            cause,
        });

        match core
            .step_battle_execution_until(&mut execution, 10)
            .expect("battle step should process same-time bucket")
        {
            crate::game::battle::core::sim::BattleStepOutcome::Finished(_) => {}
            crate::game::battle::core::sim::BattleStepOutcome::Running => {
                panic!("authored time outcome should finish after same-time bucket")
            }
        }
        let result = core
            .finalize_battle_execution(&mut execution)
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups: Vec::new(),
            events: Vec::new(),
            terminal_outcomes: BattleScenario::fixture_survival_outcomes(1_000),
            boss_runtime: None,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let skill = SkillDef {
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
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
        profile.weapon_profile = Some(Default::default());
        profile.skill_id = Some(skill.id.clone());
        profile.skill_activation_mode = SkillActivationMode::Manual;
        profile.resonance.start = 100;
        profile.resonance.max = 100;
        let draft = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(20),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(20),
                profile,
            },
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            level: crate::game::enums::Tier::I,
            stat_scale: Default::default(),
            growth_stacks: Default::default(),
            equipment_layout: None,
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
            employee_start_condition: None,
        };

        let deployed = core
            .apply_live_command(
                crate::game::battle::core::sim::BattleLiveCommand::DeployPlayerUnit {
                    draft,
                    position: Position::new(1, 1),
                    tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
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
                },
                crate::game::battle::scenario::ScenarioEvent {
                    id: crate::game::battle::scenario::ScenarioEventId::new("delayed_wave"),
                    trigger: crate::game::battle::scenario::ScenarioTrigger::AtTimeMs(1_000),
                    action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                        group_id: enemy_group_id,
                    },
                },
            ],
            terminal_outcomes: BattleScenario::fixture_survival_outcomes(2_000),
            boss_runtime: None,
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

    #[test]
    fn owned_clone_shape_runtime_unit_preserves_pending_skill_and_path() {
        use crate::authoring::{
            clone_shape::CloneShape,
            control::{self, CancellationToken, CloneMeter, ExecutionLimits, ExecutionPhase},
        };
        let mut unit = runtime_unit(Uuid::from_u128(19).into(), Side::Player);
        unit.pending_skill_cast = Some(types::PendingSkillCast {
            start_gate_pending: false,
            reset_resonance: true,
            action: None,
            skill_id: "pending_skill".into(),
            cast_target: None,
            accepted_target_tile: Some(Position::new(2, 3)),
            start_seq: 23,
        });
        unit.enemy_movement_plan = Some(
            crate::game::battle::scenario::EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(2, 3), Position::new(3, 3)],
            },
        );
        let before = format!("{unit:?}");
        let (candidate, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                unit.visit_clone_shape(&mut meter).unwrap();
                meter.finish("unit_storage", "unit_copy").unwrap();
                unit.clone()
            },
        )
        .unwrap();
        assert_eq!(format!("{candidate:?}"), before);
        let mut copied = false;
        let failure = control::run(
            &ExecutionLimits {
                max_clone_slots: usage.clone_slots_peak - 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                unit.visit_clone_shape(&mut meter)?;
                meter.finish("unit_storage", "unit_copy")?;
                let _candidate = unit.clone();
                copied = true;
                Ok::<_, crate::game::behavior::GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert!(!copied);
        assert_eq!(format!("{unit:?}"), before);
    }

    #[test]
    fn owned_clone_shape_complete_battle_admission_precedes_native_copy_and_keeps_continuation() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut core = new_core();
        let unit_id = Uuid::from_u128(17).into();
        core.insert_runtime_unit(unit_id, runtime_unit(unit_id, Side::Player));
        core.record_event_log(
            0,
            BattleLogEvent::BattleStart {
                width: 6,
                height: 6,
            },
        );
        core.event_queue.push(BattleEvent::RestTick {
            time_ms: 10,
            unit_instance_id: unit_id,
            rest_epoch: 0,
        });
        let before = core.transaction_debug_snapshot();
        let queue_before = core.event_queue_debug_snapshot();
        let log_before = core.event_log.to_json_string().unwrap();
        let (mut candidate, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                core.preflight_authoring_clone().unwrap();
                core.clone()
            },
        )
        .unwrap();
        assert_eq!(candidate.transaction_debug_snapshot(), before);
        assert_eq!(candidate.event_queue_debug_snapshot(), queue_before);
        assert!(Arc::ptr_eq(&candidate.game_data, &core.game_data));
        let mut copied = false;
        let failure = control::run(
            &ExecutionLimits {
                max_work: usage.work - 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                core.preflight_authoring_clone()?;
                let _candidate = core.clone();
                copied = true;
                Ok::<_, crate::game::behavior::GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_work_limit");
        assert_eq!(failure.site, "battle_core_clone_copy");
        assert!(!copied);
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.event_log.to_json_string().unwrap(), log_before);
        let (exact, exact_usage) = control::run(
            &ExecutionLimits {
                max_work: usage.work,
                max_clone_slots: usage.clone_slots_peak,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                core.preflight_authoring_clone().unwrap();
                core.clone()
            },
        )
        .unwrap();
        assert_eq!(exact_usage, usage);
        assert_eq!(exact.transaction_debug_snapshot(), before);
        let mut expected = core.clone();
        let event = expected.event_queue.pop().unwrap();
        expected
            .process_event(event.clone(), event.time_ms())
            .unwrap();
        let event = candidate.event_queue.pop().unwrap();
        candidate
            .process_event(event.clone(), event.time_ms())
            .unwrap();
        assert_eq!(
            candidate.transaction_debug_snapshot(),
            expected.transaction_debug_snapshot()
        );
        assert_eq!(
            candidate.event_log.to_json_string().unwrap(),
            expected.event_log.to_json_string().unwrap()
        );
        assert_eq!(core.event_queue_debug_snapshot(), queue_before);
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
            employee_trauma: None,
            corroded: false,
            fragment_source: None,
            basic_attack_speed_percent: 100,
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
            equipment_slots: None,
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
            basic_attack_epoch: 0,
            basic_attack_windup_until_ms: None,
            ranged_reposition_until_ms: 0,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            resonance_use: None,
            next_resonance_pulse_ms: None,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    fn fixture_draft(owned_uuid: Uuid, base_uuid: Uuid, hp: u32, attack: u32) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(Default::default());
        profile.stats = UnitStats::with_values(hp, hp, attack, 0, 100);
        profile.basic_attack.interval_ms = 100;
        profile.basic_attack.windup_ms = 1;
        BattleUnitDraft {
            owned_uuid,
            source: BattleUnitSource::TestFixture { base_uuid, profile },
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            level: crate::game::enums::Tier::I,
            stat_scale: Default::default(),
            growth_stacks: crate::game::growth::GrowthStack::new(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    #[test]
    fn scenario_group_identity_is_authoritative_one_shot_guard() {
        let group_id = crate::game::battle::scenario::ScenarioGroupId::new("reinforcements");
        let unit_ref = crate::game::battle::scenario::ScenarioUnitRef::new("reinforcement_0");
        let mut scenario = BattleScenario::empty((4, 4));
        scenario
            .groups
            .push(crate::game::battle::scenario::ScenarioSpawnGroup {
                id: group_id.clone(),
                side: Side::Player,
                required_for_victory: false,
                enemy_movement_plan: None,
                spawns: vec![crate::game::battle::scenario::ScenarioUnitSpawn {
                    unit_ref: unit_ref.clone(),
                    side: Side::Player,
                    draft: fixture_draft(Uuid::from_u128(0x5100), Uuid::from_u128(0x5101), 100, 10),
                    position: Position::new(1, 1),
                    instance_salt: 0,
                }],
            });
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);

        let first_spawn = core.spawn_scenario_group(&group_id).unwrap();
        assert_eq!(first_spawn.len(), 1);
        assert_eq!(core.units.len(), 1);
        assert_eq!(core.scenario_runtime.next_spawn_order, 1);
        assert_eq!(
            core.scenario_runtime.spawned.unit_refs.get(&unit_ref),
            first_spawn.first()
        );

        let duplicate_spawn = core.spawn_scenario_group(&group_id).unwrap();
        assert!(duplicate_spawn.is_empty());
        assert_eq!(core.units.len(), 1);
        assert_eq!(core.scenario_runtime.next_spawn_order, 1);
        assert_eq!(
            core.scenario_runtime.spawned.unit_refs.get(&unit_ref),
            first_spawn.first()
        );
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
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
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            boss_runtime: None,
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
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
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            boss_runtime: None,
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
                object_placement: None,
                width: 4,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
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
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            boss_runtime: None,
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
        SkillCastTargetingDef::explicit(
            target,
            match range_policy {
                TileRangePolicy::Pattern => crate::game::ability::SkillTargetReach::Pattern,
                TileRangePolicy::WholeFieldValidTiles => {
                    crate::game::ability::SkillTargetReach::Global
                }
            },
            defense_tile_range,
            false,
        )
    }

    fn abnormality_with_basic_attack(
        base_uuid: Uuid,
        delivery: DeliveryDef,
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

    pub(super) fn single_step_skill(
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
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
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
                target,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery,
                effects: effects
                    .into_iter()
                    .enumerate()
                    .map(|(i, op)| op.authored(format!("effect_{i:02}")))
                    .collect(),
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
                footprint: TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
                },
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

    fn enemy_anchor_tile_area_skill(
        id: &str,
        effects: Vec<SkillEffectDef>,
        hit_targets: crate::game::ability::SkillHitTargetFilter,
        footprint: TileRangePattern,
    ) -> SkillDef {
        let mut skill = single_step_skill(
            id,
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            1,
            0,
            DeliveryDef::Instant,
            effects,
        );
        skill.steps[0].delivery = DeliveryDef::TileArea {
            footprint,
            area: SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                tile_origin: SkillTileAreaOrigin::Anchor,
                tracking: SkillAreaTracking::GroundFixed,
                hit_targets,
                include_caster: false,
                tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                duration_ms: 0,
                tick_interval_ms: None,
            },
        };
        let (_, reach, target_range, air_capable) = skill.cast_target_definition().unwrap();
        skill.cast_targeting = SkillCastTargetingDef::explicit_with_selection(
            SkillCastSelection::PlayerUnit,
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            reach,
            target_range.cloned(),
            air_capable,
        );
        skill
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
            first.event.view().into_owned(),
            BattleEvent::AutoCastStart {
                time_ms: 100,
                caster_instance_id,
                cause: BattleEventCause::Parent { seq: 42 },
            } if caster_instance_id == caster_small
        ));
        assert!(matches!(
            second.event.view().into_owned(),
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
            event.event.view().into_owned(),
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
            event.event.view().into_owned(),
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
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant),
            abnormality_with_basic_attack(enemy_base_uuid, DeliveryDef::Instant),
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
            ),
            abnormality_with_basic_attack(melee_enemy_base_uuid, DeliveryDef::Instant),
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
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant),
            abnormality_with_basic_attack(enemy_base_uuid, DeliveryDef::Instant),
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
        )]);
        Arc::make_mut(&mut core.scenario).tactical_plan.player_plan =
            crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;

        let attacker_id: UnitInstanceId = Uuid::from_u128(81).into();
        let forward_target_id: UnitInstanceId = Uuid::from_u128(82).into();
        let side_target_id: UnitInstanceId = Uuid::from_u128(83).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.facing_direction = Some(FacingDirection::Up);
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
    fn basic_attack_rejects_target_outside_tile_range() {
        let attacker_id: UnitInstanceId = Uuid::from_u128(0xAA30).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xAA31).into();
        let mut core = new_core();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.facing_direction = Some(FacingDirection::Right);
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
        )]);
        Arc::make_mut(&mut core.scenario).tactical_plan.player_plan =
            crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;

        let attacker_id: UnitInstanceId = Uuid::from_u128(0xAA16).into();
        let lower_uuid_far_target_id: UnitInstanceId = Uuid::from_u128(0xAA17).into();
        let higher_uuid_near_target_id: UnitInstanceId = Uuid::from_u128(0xAA18).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.facing_direction = Some(FacingDirection::Up);
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
        Arc::make_mut(&mut core.scenario).tactical_plan.player_plan =
            crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;

        let mut attacker = runtime_unit(attacker_id, Side::Player);
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
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
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
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![damage_effect().authored("damage")],
                presentation: Default::default(),
            }],
        };
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        Arc::make_mut(&mut core.scenario).tactical_plan.player_plan =
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
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
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
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![damage_effect().authored("damage")],
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
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
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
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![damage_effect().authored("damage")],
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
    fn global_target_area_uses_captured_tile_after_target_disappearance_without_facing() {
        let footprint = TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
        };
        let skill = SkillDef {
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("global_captured_area"),
            name: "global_captured_area".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                crate::game::ability::SkillTargetReach::Global,
                None,
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "impact".to_string(),
                delay_ms: 10,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::TileArea {
                    footprint,
                    area: SkillTileAreaDeliveryDef {
                        anchor: SkillAreaAnchorSource::CastTarget,
                        tile_origin: SkillTileAreaOrigin::Anchor,
                        hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                        ..Default::default()
                    },
                },
                effects: vec![damage_effect().authored("damage")],
                presentation: Default::default(),
            }],
        };
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xAA60).into();
        let selected_id: UnitInstanceId = Uuid::from_u128(0xAA61).into();
        let bystander_id: UnitInstanceId = Uuid::from_u128(0xAA62).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.facing_direction = None;
        core.units.insert(caster_id, caster);
        core.units
            .insert(selected_id, runtime_unit(selected_id, Side::Opponent));
        core.units
            .insert(bystander_id, runtime_unit(bystander_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, selected_id, Position::new(4, 2));
        place_unit(&mut core, bystander_id, Position::new(4, 3));
        for unit_id in [caster_id, selected_id, bystander_id] {
            sync_unit_to_battlefield_tile_center(&mut core, unit_id);
        }
        let bystander_health = core.units.get(&bystander_id).unwrap().stats.current_health;

        core.invoke_ability(
            0,
            caster_id,
            skill.id.as_str(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: selected_id,
            }),
            BattleEventCause::default(),
            false,
        );
        core.units.get_mut(&selected_id).unwrap().lifecycle = RuntimeUnitLifecycle::Withdrawn;
        let event = core.event_queue.pop().expect("queued area step");
        core.process_event(event, 10)
            .expect("resolve captured area");

        assert!(
            core.units.get(&bystander_id).unwrap().stats.current_health < bystander_health,
            "another eligible unit in the captured footprint must be hit"
        );
        let declared_tiles =
            core.event_log.entries.iter().find_map(|entry| {
                if let BattleLogEvent::SkillAreaDeclared {
                    shape:
                        crate::game::battle::event_log::BattleSkillAreaShape::TilePattern {
                            affected_tiles,
                        },
                    ..
                } = &entry.event
                {
                    Some(affected_tiles)
                } else {
                    None
                }
            });
        assert!(declared_tiles.is_some_and(|tiles| {
            tiles.contains(&Position::new(4, 2)) && tiles.contains(&Position::new(4, 3))
        }));

        let health_after_withdrawal = core.units.get(&bystander_id).unwrap().stats.current_health;
        core.units.get_mut(&selected_id).unwrap().lifecycle = RuntimeUnitLifecycle::Active;
        core.invoke_ability(
            20,
            caster_id,
            skill.id.as_str(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: selected_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let selected = core.units.get_mut(&selected_id).unwrap();
        selected.lifecycle = RuntimeUnitLifecycle::Dead;
        selected.stats.current_health = 0;
        let event = core
            .event_queue
            .pop()
            .expect("queued area step after death");
        core.process_event(event, 30)
            .expect("resolve captured area after death");
        assert!(
            core.units.get(&bystander_id).unwrap().stats.current_health < health_after_withdrawal,
            "captured footprint must also survive selected-target death"
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
    fn hard_cc_overlap_keeps_both_instances_and_uses_latest_release() {
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

        let control = core
            .unit_control_state(target_id, 10)
            .expect("target control state");
        assert!(control.hard_cc_active());
        assert_eq!(control.hard_cc_release_time_ms, Some(100));
        assert_eq!(core.buffs.len(), 2);

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 30,
                caster_instance_id: second_caster_id,
                target_instance_id: target_id,
                buff_id: freeze_id,
                cause: BattleEventCause::default(),
            },
            30,
        )
        .unwrap();
        assert_eq!(
            core.unit_control_state(target_id, 30)
                .expect("target control state")
                .hard_cc_release_time_ms,
            Some(100)
        );

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 100,
                caster_instance_id: first_caster_id,
                target_instance_id: target_id,
                buff_id: stun_id,
                cause: BattleEventCause::default(),
            },
            100,
        )
        .unwrap();
        assert_eq!(
            core.unit_control_state(target_id, 100),
            Some(Default::default())
        );

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
                (10, "applied", freeze_id, None),
                (30, "expired", freeze_id, Some(BuffExpireReason::Natural)),
                (100, "expired", stun_id, Some(BuffExpireReason::Natural)),
            ]
        );
        assert_eq!(
            core.units.get(&target_id).unwrap().action_locks,
            Default::default()
        );
    }

    #[test]
    fn refresh_duration_buff_producer_uses_latest_expiration_and_rejects_unknown_ids() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC04).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC05).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        let unknown_id = BuffId::from_name("phase_37_unknown_buff");
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: caster_id,
                target_instance_id: target_id,
                buff_id: unknown_id,
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .expect("unknown buff input is a producer no-op");
        assert!(core.buffs.is_empty());
        assert!(core.event_log.entries.is_empty());
        assert!(core.event_queue.is_empty());

        let silence_id = BuffId::from_name("silence");
        assert_eq!(
            core.game_data
                .buff_data
                .get(silence_id)
                .expect("live silence buff")
                .reapply_policy,
            BuffReapplyPolicy::RefreshDuration
        );
        for (time_ms, duration_ms) in [(0, 100), (50, 10)] {
            core.process_event(
                BattleEvent::ApplyBuff {
                    time_ms,
                    caster_instance_id: caster_id,
                    target_instance_id: target_id,
                    buff_id: silence_id,
                    duration_ms,
                    cause: BattleEventCause::default(),
                },
                time_ms,
            )
            .expect("known refresh-duration buff should apply");
        }
        while let Some(event) = core.event_queue.pop() {
            let time_ms = event.time_ms();
            core.process_event(event, time_ms)
                .expect("scheduled buff expiration should process");
        }

        let produced_buff_lifecycle_is_current = |event_log: &BattleEventLog| {
            let mut active = std::collections::HashMap::new();
            for entry in &event_log.entries {
                match entry.event {
                    BattleLogEvent::BuffApplied {
                        caster_instance_id,
                        target_instance_id,
                        buff_id,
                        duration_ms,
                    } => {
                        let Some(definition) = core.game_data.buff_data.get(buff_id) else {
                            return false;
                        };
                        if duration_ms == 0 {
                            return false;
                        }
                        let key = (caster_instance_id, target_instance_id, buff_id);
                        let expires_at_ms = entry.time_ms.saturating_add(duration_ms);
                        match definition.reapply_policy {
                            BuffReapplyPolicy::StackRefreshDurationKeepCadence => {
                                active
                                    .entry(key)
                                    .and_modify(|current: &mut u64| {
                                        *current = (*current).max(expires_at_ms)
                                    })
                                    .or_insert(expires_at_ms);
                            }
                            BuffReapplyPolicy::RefreshDuration => {
                                active.insert(key, expires_at_ms);
                            }
                        }
                    }
                    BattleLogEvent::BuffExpired {
                        caster_instance_id,
                        target_instance_id,
                        buff_id,
                        reason: BuffExpireReason::Natural,
                    } => {
                        let key = (caster_instance_id, target_instance_id, buff_id);
                        if active.remove(&key) != Some(entry.time_ms) {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
            active.is_empty()
        };

        assert!(produced_buff_lifecycle_is_current(&core.event_log));
        let produced_events = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::BuffApplied {
                    buff_id,
                    duration_ms,
                    ..
                } if buff_id == silence_id => Some((entry.time_ms, duration_ms, false)),
                BattleLogEvent::BuffExpired {
                    buff_id,
                    reason: BuffExpireReason::Natural,
                    ..
                } if buff_id == silence_id => Some((entry.time_ms, 0, true)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            produced_events,
            vec![(0, 100, false), (50, 10, false), (60, 0, true)]
        );

        let mut malformed_unknown = core.event_log.clone();
        if let BattleLogEvent::BuffApplied { buff_id, .. } = &mut malformed_unknown.entries[0].event
        {
            *buff_id = unknown_id;
        }
        assert!(!produced_buff_lifecycle_is_current(&malformed_unknown));

        let mut malformed_expiration = core.event_log.clone();
        malformed_expiration.entries[2].time_ms = 100;
        assert!(!produced_buff_lifecycle_is_current(&malformed_expiration));
    }

    #[test]
    fn control_status_hard_cc_cancels_basic_windup_while_silence_preserves_it() {
        for (buff_name, should_resolve) in [("stun", false), ("silence", true)] {
            let mut core = new_core();
            let attacker_id: UnitInstanceId =
                Uuid::from_u128(if should_resolve { 0xCC11 } else { 0xCC12 }).into();
            let target_id: UnitInstanceId =
                Uuid::from_u128(if should_resolve { 0xCC13 } else { 0xCC14 }).into();
            let mut attacker = runtime_unit(attacker_id, Side::Player);
            attacker.basic_attack.windup_ms = 10;
            core.units.insert(attacker_id, attacker);
            core.units
                .insert(target_id, runtime_unit(target_id, Side::Opponent));
            place_unit(&mut core, attacker_id, Position::new(1, 1));
            place_unit(&mut core, target_id, Position::new(2, 1));
            let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

            core.process_event(
                BattleEvent::AttackStart {
                    time_ms: 0,
                    attacker_instance_id: attacker_id,
                    target_instance_id: Some(target_id),
                    schedule_next: true,
                    cause: BattleEventCause::default(),
                },
                0,
            )
            .unwrap();
            let resolve = core
                .event_queue
                .iter()
                .find(|event| {
                    matches!(
                        event.as_ref(),
                        BattleEvent::AttackResolve { time_ms: 10, .. }
                    )
                })
                .map(std::borrow::Cow::into_owned)
                .expect("basic attack resolve should be queued");

            core.process_event(
                BattleEvent::ApplyBuff {
                    time_ms: 1,
                    caster_instance_id: target_id,
                    target_instance_id: attacker_id,
                    buff_id: BuffId::from_name(buff_name),
                    duration_ms: 1,
                    cause: BattleEventCause::default(),
                },
                1,
            )
            .unwrap();

            if should_resolve {
                assert_eq!(
                    core.units
                        .get(&attacker_id)
                        .unwrap()
                        .basic_attack_windup_until_ms,
                    Some(10)
                );
            } else {
                let attacker = core.units.get(&attacker_id).unwrap();
                assert_eq!(attacker.basic_attack_windup_until_ms, None);
                assert_eq!(attacker.action_locks.basic_attack_until_ms, 1);
                assert_eq!(attacker.action_locks.movement_until_ms, 1);
            }

            core.process_event(resolve, 10).unwrap();
            let resolved_hp = core.units.get(&target_id).unwrap().stats.current_health;
            assert_eq!(resolved_hp < target_hp, should_resolve, "buff={buff_name}");
        }
    }

    #[test]
    fn control_status_hard_cc_cancels_manual_focus_and_releases_owned_locks() {
        let skill = single_step_skill(
            "hard_cc_manual_focus",
            SkillTarget::SelfUnit,
            0,
            100,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 1 }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC21).into();
        let interrupter_id: UnitInstanceId = Uuid::from_u128(0xCC22).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(interrupter_id, runtime_unit(interrupter_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));

        core.start_manual_skill_cast(0, caster_id, skill.id.clone(), None)
            .expect("manual focus should start");
        let stale_end = core
            .event_queue
            .iter()
            .find(|event| {
                matches!(
                    event.as_ref(),
                    BattleEvent::ManualCastEnd { time_ms: 100, .. }
                )
            })
            .map(std::borrow::Cow::into_owned)
            .expect("manual completion should be queued");

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: interrupter_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("freeze"),
                duration_ms: 5,
                cause: BattleEventCause::default(),
            },
            1,
        )
        .unwrap();

        let caster = core.units.get(&caster_id).unwrap();
        assert!(caster.pending_skill_cast.is_none());
        assert_eq!(caster.next_action_time, 0);
        assert_eq!(caster.action_locks.basic_attack_until_ms, 1);
        assert_eq!(caster.action_locks.movement_until_ms, 1);
        assert_eq!(caster.action_locks.resonance_gain_until_ms, 1);
        assert_eq!(caster.resonance_current, 100);
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::SkillCastInterrupted {
                interrupter_instance_id,
                caster_instance_id,
                interrupted_skill_id,
                ..
            } if *interrupter_instance_id == interrupter_id
                && *caster_instance_id == caster_id
                && interrupted_skill_id == &skill.id
        )));

        core.process_event(stale_end, 100).unwrap();
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::AbilityCast { skill_id, .. } if skill_id == &skill.id
        )));
        assert_eq!(core.units.get(&caster_id).unwrap().resonance_current, 100);
    }

    #[test]
    fn control_status_manual_skill_reason_prefers_hard_cc_and_rejection_is_immutable() {
        let skill = single_step_skill(
            "control_reason_skill",
            SkillTarget::SelfUnit,
            0,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 1 }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC31).into();
        let source_id: UnitInstanceId = Uuid::from_u128(0xCC32).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(source_id, runtime_unit(source_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: source_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("silence"),
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();
        assert!(matches!(
            core.validate_manual_skill_activation(1, caster_id, &skill.id, None),
            Err(crate::game::behavior::GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::Silenced,
                ..
            })
        ));
        let event_count = core.event_log.entries.len();
        assert!(matches!(
            core.start_manual_skill_cast(1, caster_id, skill.id.clone(), None),
            Err(crate::game::behavior::GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::Silenced,
                ..
            })
        ));
        let caster = core.units.get(&caster_id).unwrap();
        assert_eq!(caster.resonance_current, 100);
        assert!(caster.pending_skill_cast.is_none());
        assert_eq!(core.event_log.entries.len(), event_count);

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 2,
                caster_instance_id: source_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            2,
        )
        .unwrap();
        assert!(matches!(
            core.validate_manual_skill_activation(3, caster_id, &skill.id, None),
            Err(crate::game::behavior::GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::HardCrowdControl,
                ..
            })
        ));

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 12,
                caster_instance_id: source_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("stun"),
                cause: BattleEventCause::default(),
            },
            12,
        )
        .unwrap();
        assert!(matches!(
            core.validate_manual_skill_activation(13, caster_id, &skill.id, None),
            Err(crate::game::behavior::GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::Silenced,
                ..
            })
        ));
    }

    #[test]
    fn manual_player_selection_requires_exact_payload_and_captures_unit_or_tile() {
        let mut unit_skill = single_step_skill(
            "manual_player_unit",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            2,
            100,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Damage {
                amount: 10,
                damage_type: DamageType::Magic,
            }],
        );
        let (_, reach, target_range, air_capable) = unit_skill.cast_target_definition().unwrap();
        unit_skill.cast_targeting = SkillCastTargetingDef::explicit_with_selection(
            SkillCastSelection::PlayerUnit,
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            reach,
            target_range.cloned(),
            air_capable,
        );
        let mut tile_skill = single_step_skill(
            "manual_player_tile",
            SkillTarget::CastTarget,
            1,
            100,
            DeliveryDef::Instant,
            vec![],
        );
        let (_, reach, target_range, air_capable) = tile_skill.cast_target_definition().unwrap();
        tile_skill.cast_targeting = SkillCastTargetingDef::explicit_with_selection(
            SkillCastSelection::PlayerTile,
            SkillTarget::CastTarget,
            reach,
            target_range.cloned(),
            air_capable,
        );
        let mut useless_unit_skill = unit_skill.clone();
        useless_unit_skill.id = SkillId::from("manual_player_unit_useless");
        useless_unit_skill.name = useless_unit_skill.id.to_string();
        useless_unit_skill.steps[0].effects =
            vec![SkillEffectDef::Heal { amount: 10 }.authored("heal")];

        let mut core = core_with_skill_data(
            vec![],
            vec![
                unit_skill.clone(),
                tile_skill.clone(),
                useless_unit_skill.clone(),
            ],
        );
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC35).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC36).into();
        let ally_id: UnitInstanceId = Uuid::from_u128(0xCC37).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(unit_skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        core.units
            .insert(ally_id, runtime_unit(ally_id, Side::Player));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        place_unit(&mut core, ally_id, Position::new(1, 2));

        assert_eq!(
            core.validate_manual_skill_targeting_focus(0, caster_id, &unit_skill.id)
                .unwrap(),
            SkillCastSelection::PlayerUnit
        );
        assert!(matches!(
            core.validate_manual_skill_activation(0, caster_id, &unit_skill.id, None),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::SkillTargetRequired,
                ..
            })
        ));
        assert!(matches!(
            core.validate_manual_skill_activation(
                0,
                caster_id,
                &unit_skill.id,
                Some(SkillCastTarget::Tile {
                    position: Position::new(2, 1),
                }),
            ),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::SkillTargetTypeMismatch,
                ..
            })
        ));
        assert!(matches!(
            core.validate_manual_skill_activation(
                0,
                caster_id,
                &unit_skill.id,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: ally_id,
                }),
            ),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::NoValidTarget,
                ..
            })
        ));
        core.units.get_mut(&target_id).unwrap().mobility_kind = MobilityKind::Airborne;
        assert!(matches!(
            core.validate_manual_skill_activation(
                0,
                caster_id,
                &unit_skill.id,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: target_id,
                }),
            ),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::NoValidTarget,
                ..
            })
        ));
        core.units.get_mut(&target_id).unwrap().mobility_kind = MobilityKind::Ground;
        core.units.get_mut(&caster_id).unwrap().skill_id = Some(useless_unit_skill.id.clone());
        assert!(matches!(
            core.validate_manual_skill_activation(
                0,
                caster_id,
                &useless_unit_skill.id,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: target_id,
                }),
            ),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::NoValidTarget,
                ..
            })
        ));
        core.units.get_mut(&caster_id).unwrap().skill_id = Some(unit_skill.id.clone());
        core.start_manual_skill_cast(
            0,
            caster_id,
            unit_skill.id.clone(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
        )
        .unwrap();
        assert_eq!(
            core.units[&caster_id]
                .pending_skill_cast
                .as_ref()
                .and_then(|pending| pending.cast_target),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            })
        );

        {
            let caster = core.units.get_mut(&caster_id).unwrap();
            caster.pending_skill_cast = None;
            caster.action_locks = Default::default();
            caster.skill_id = Some(tile_skill.id.clone());
            caster.resonance_current = 100;
            caster.next_action_time = 0;
        }
        assert_eq!(
            core.validate_manual_skill_targeting_focus(1, caster_id, &tile_skill.id)
                .unwrap(),
            SkillCastSelection::PlayerTile
        );
        assert!(matches!(
            core.validate_manual_skill_activation(
                1,
                caster_id,
                &tile_skill.id,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: target_id,
                }),
            ),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::SkillTargetTypeMismatch,
                ..
            })
        ));
        assert!(core
            .validate_manual_skill_activation(
                1,
                caster_id,
                &tile_skill.id,
                Some(SkillCastTarget::Tile {
                    position: Position::new(2, 2),
                }),
            )
            .is_ok());
        assert!(matches!(
            core.validate_manual_skill_activation(
                1,
                caster_id,
                &tile_skill.id,
                Some(SkillCastTarget::Tile {
                    position: Position::new(3, 3),
                }),
            ),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::NoValidTarget,
                ..
            })
        ));
    }

    #[test]
    fn manual_player_unit_invalidation_noops_without_retarget_or_refund() {
        let mut skill = single_step_skill(
            "manual_player_unit_invalidation",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Damage {
                amount: 10,
                damage_type: DamageType::Magic,
            }],
        );
        let (_, reach, target_range, air_capable) = skill.cast_target_definition().unwrap();
        skill.cast_targeting = SkillCastTargetingDef::explicit_with_selection(
            SkillCastSelection::PlayerUnit,
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            reach,
            target_range.cloned(),
            air_capable,
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC38).into();
        let selected_id: UnitInstanceId = Uuid::from_u128(0xCC39).into();
        let fallback_id: UnitInstanceId = Uuid::from_u128(0xCC3A).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.source_identity = crate::game::battle::types::BattleUnitSourceIdentity::Employee {
            employee_uuid: caster_id.as_uuid(),
            base_uuid: Uuid::nil(),
        };
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(selected_id, runtime_unit(selected_id, Side::Opponent));
        core.units
            .insert(fallback_id, runtime_unit(fallback_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(0, 1));
        place_unit(&mut core, selected_id, Position::new(1, 1));
        place_unit(&mut core, fallback_id, Position::new(1, 2));
        let fallback_health = core.units[&fallback_id].stats.current_health;

        core.start_manual_skill_cast(
            0,
            caster_id,
            skill.id.clone(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: selected_id,
            }),
        )
        .unwrap();
        process_manual_start_gate_fixture(&mut core, 0);
        {
            let selected = core.units.get_mut(&selected_id).unwrap();
            selected.lifecycle = RuntimeUnitLifecycle::Dead;
            selected.stats.current_health = 0;
        }
        let cast_end = core
            .event_queue
            .pop()
            .expect("manual cast end should be queued");
        core.process_event(cast_end, 5).unwrap();
        assert!(core.event_queue.is_empty());

        assert_eq!(
            core.units[&fallback_id].stats.current_health,
            fallback_health
        );
        assert_eq!(core.units[&caster_id].resonance_current, 0);
        assert!(core.units[&caster_id].pending_skill_cast.is_none());
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::AbilityCast {
                skill_id,
                target_instance_id: Some(target_id),
                ..
            } if skill_id == &skill.id && *target_id == selected_id
        )));
    }

    #[test]
    fn manual_player_unit_tile_area_uses_acceptance_tile_after_target_moves_or_withdraws() {
        for withdrawn in [false, true] {
            let footprint = TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
            };
            let skill = SkillDef {
                use_lifetime: crate::game::ability::SkillUseLifetime::Action,
                flow: None,
                id: SkillId::from("manual_player_unit_fixed_area"),
                name: "manual_player_unit_fixed_area".to_string(),
                kind: SkillKind::Targeted,
                cast_targeting: SkillCastTargetingDef::explicit_with_selection(
                    SkillCastSelection::PlayerUnit,
                    SkillTarget::EnemySingle {
                        rule: UnitTargetRule::Nearest,
                    },
                    crate::game::ability::SkillTargetReach::Global,
                    None,
                    false,
                ),
                focus_time_ms: 5,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "fixed_area".to_string(),
                    delay_ms: 0,
                    target: SkillTarget::EnemySingle {
                        rule: UnitTargetRule::Nearest,
                    },
                    targeting: StepTargetingMode::ReuseCastTarget,
                    when: Default::default(),
                    repeat: Default::default(),
                    delivery: DeliveryDef::TileArea {
                        footprint,
                        area: SkillTileAreaDeliveryDef {
                            anchor: SkillAreaAnchorSource::CastTarget,
                            tile_origin: SkillTileAreaOrigin::Anchor,
                            hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                            ..Default::default()
                        },
                    },
                    effects: vec![SkillEffectDef::Damage {
                        amount: 10,
                        damage_type: DamageType::Magic,
                    }
                    .authored("effect_01")],
                    presentation: Default::default(),
                }],
            };
            let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
            let caster_id: UnitInstanceId = Uuid::from_u128(0xCC3B).into();
            let selected_id: UnitInstanceId = Uuid::from_u128(0xCC3C).into();
            let bystander_id: UnitInstanceId = Uuid::from_u128(0xCC3D).into();
            let mut caster = runtime_unit(caster_id, Side::Player);
            caster.source_identity =
                crate::game::battle::types::BattleUnitSourceIdentity::Employee {
                    employee_uuid: caster_id.as_uuid(),
                    base_uuid: Uuid::nil(),
                };
            caster.skill_id = Some(skill.id.clone());
            caster.skill_activation_mode = SkillActivationMode::Manual;
            caster.resonance_current = 100;
            caster.facing_direction = None;
            core.units.insert(caster_id, caster);
            core.units
                .insert(selected_id, runtime_unit(selected_id, Side::Opponent));
            core.units
                .insert(bystander_id, runtime_unit(bystander_id, Side::Opponent));
            place_unit(&mut core, caster_id, Position::new(0, 1));
            place_unit(&mut core, selected_id, Position::new(1, 1));
            place_unit(&mut core, bystander_id, Position::new(1, 2));
            for unit_id in [caster_id, selected_id, bystander_id] {
                sync_unit_to_battlefield_tile_center(&mut core, unit_id);
            }
            let bystander_health = core.units[&bystander_id].stats.current_health;

            core.start_manual_skill_cast(
                0,
                caster_id,
                skill.id.clone(),
                Some(SkillCastTarget::Unit {
                    unit_instance_id: selected_id,
                }),
            )
            .unwrap();
            process_manual_start_gate_fixture(&mut core, 0);
            assert_eq!(
                core.units[&caster_id]
                    .pending_skill_cast
                    .as_ref()
                    .and_then(|pending| pending.accepted_target_tile),
                Some(Position::new(1, 1))
            );
            core.units
                .get_mut(&selected_id)
                .unwrap()
                .set_world_position(WorldVec2::from_tile_center(Position::new(4, 3)));
            if withdrawn {
                core.withdraw_unit(selected_id, 1).unwrap();
                assert!(core.units[&caster_id].pending_skill_cast.is_some());
            }
            let cast_end = core
                .event_queue
                .pop()
                .expect("manual cast end should be queued");
            core.process_event(cast_end, 5).unwrap();
            let step = core
                .event_queue
                .pop()
                .expect("captured area step should be queued");
            core.process_event(step, 5).unwrap();

            assert!(
                core.units[&bystander_id].stats.current_health < bystander_health,
                "captured area must hit the bystander; events={:?}, bystander_body={:?}",
                core.event_log.entries,
                core.sample_unit_body_at(bystander_id, 5)
            );
            assert!(core.event_log.entries.iter().any(|entry| matches!(
                &entry.event,
                BattleLogEvent::SkillAreaDeclared {
                    shape: crate::game::battle::event_log::BattleSkillAreaShape::TilePattern {
                        affected_tiles,
                    },
                    ..
                } if affected_tiles.contains(&Position::new(1, 1))
                    && affected_tiles.contains(&Position::new(1, 2))
                    && !affected_tiles.contains(&Position::new(4, 3))
            )));
        }
    }

    #[test]
    fn control_status_overlapping_silence_clears_only_after_last_expiry() {
        let mut core = new_core();
        let first_source: UnitInstanceId = Uuid::from_u128(0xCC41).into();
        let second_source: UnitInstanceId = Uuid::from_u128(0xCC42).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC43).into();
        core.units
            .insert(first_source, runtime_unit(first_source, Side::Opponent));
        core.units
            .insert(second_source, runtime_unit(second_source, Side::Opponent));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Player));

        for (source, duration_ms) in [(first_source, 10), (second_source, 20)] {
            core.process_event(
                BattleEvent::ApplyBuff {
                    time_ms: 0,
                    caster_instance_id: source,
                    target_instance_id: target_id,
                    buff_id: BuffId::from_name("silence"),
                    duration_ms,
                    cause: BattleEventCause::default(),
                },
                0,
            )
            .unwrap();
        }
        assert!(core.unit_control_state(target_id, 0).unwrap().silenced);

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 10,
                caster_instance_id: first_source,
                target_instance_id: target_id,
                buff_id: BuffId::from_name("silence"),
                cause: BattleEventCause::default(),
            },
            10,
        )
        .unwrap();
        assert!(core.unit_control_state(target_id, 10).unwrap().silenced);

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 20,
                caster_instance_id: second_source,
                target_instance_id: target_id,
                buff_id: BuffId::from_name("silence"),
                cause: BattleEventCause::default(),
            },
            20,
        )
        .unwrap();
        assert!(!core.unit_control_state(target_id, 20).unwrap().silenced);
    }

    #[test]
    fn control_status_hard_cc_cancels_unreleased_delayed_skill_step_without_refund() {
        let mut skill = single_step_skill(
            "hard_cc_released_step",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Instant,
            vec![
                SkillEffectDef::Damage {
                    amount: 2,
                    damage_type: DamageType::Magic,
                },
                SkillEffectDef::ApplyBuff {
                    buff_id: "poison".to_string(),
                    duration_ms: 2_000,
                },
            ],
        );
        skill.steps[0].delay_ms = 10;
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC51).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC52).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();
        let cast_end = core
            .event_queue
            .pop()
            .expect("automatic cast completion should be queued");
        assert!(matches!(
            cast_end.event.view().into_owned(),
            BattleEvent::AutoCastEnd { time_ms: 1, .. }
        ));
        core.process_event(cast_end, 1).unwrap();
        assert_eq!(core.units.get(&caster_id).unwrap().resonance_current, 0);
        assert_eq!(core.active_skill_casts.len(), 1);
        let released_step = core
            .event_queue
            .iter()
            .find(|event| matches!(event.as_ref(), BattleEvent::SkillStep { time_ms: 11, .. }))
            .map(std::borrow::Cow::into_owned)
            .expect("released delayed step should be queued");

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 2,
                caster_instance_id: target_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            2,
        )
        .unwrap();
        assert!(core.active_skill_casts.is_empty());
        core.process_event(released_step, 11).unwrap();
        assert_eq!(core.units[&target_id].stats.current_health, target_hp);
        assert!(!core
            .buffs
            .keys()
            .any(|key| key.target_instance_id == target_id));
        assert_eq!(core.units[&caster_id].resonance_current, 0);
    }

    #[test]
    fn control_status_hard_cc_preserves_released_skill_projectile() {
        let skill = single_step_skill(
            "hard_cc_released_projectile",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            vec![SkillEffectDef::Damage {
                amount: 5,
                damage_type: DamageType::Magic,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC61).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC62).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        let step = core
            .event_queue
            .pop()
            .expect("projectile step should be queued");
        core.process_event(step, 0).unwrap();
        let delivery_id = *core.active_projectiles.keys().next().unwrap();

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: target_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("freeze"),
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            1,
        )
        .unwrap();
        assert!(core.active_projectiles.contains_key(&delivery_id));

        for _ in 0..8 {
            let Some(event) = core.event_queue.pop() else {
                break;
            };
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).unwrap();
            if core.units.get(&target_id).unwrap().stats.current_health < target_hp {
                break;
            }
        }
        assert!(core.units.get(&target_id).unwrap().stats.current_health < target_hp);
    }

    #[test]
    fn control_status_hard_cc_preserves_released_area_runtime() {
        let skill = single_step_skill(
            "hard_cc_released_area",
            SkillTarget::SelfUnit,
            2,
            0,
            DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
                },
                area: SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::Caster,
                    tile_origin: SkillTileAreaOrigin::Caster,
                    tracking: SkillAreaTracking::GroundFixed,
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 30,
                    tick_interval_ms: Some(10),
                },
            },
            vec![SkillEffectDef::Damage {
                amount: 2,
                damage_type: DamageType::Magic,
            }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xCC71).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC72).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(1, 1));
        place_unit(&mut core, target_id, Position::new(2, 1));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

        core.invoke_ability(
            0,
            caster_id,
            &skill.id,
            None,
            BattleEventCause::default(),
            false,
        );
        let step = core.event_queue.pop().expect("area step should be queued");
        core.process_event(step, 0).unwrap();
        let area_id = *core.active_areas.keys().next().unwrap();

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: target_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            1,
        )
        .unwrap();
        assert!(core.active_areas.contains_key(&area_id));
        core.apply_skill_area_tick(10, area_id).unwrap();
        assert!(core.units.get(&target_id).unwrap().stats.current_health < target_hp);
    }

    #[test]
    fn control_status_hard_cc_allows_damage_trigger_cooldown_and_stops_resonance_gain() {
        let mut core = new_core();
        let source_id: UnitInstanceId = Uuid::from_u128(0xCC81).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xCC82).into();
        core.units
            .insert(source_id, runtime_unit(source_id, Side::Opponent));
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Player));
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;

        let proc_identity = ProcRollIdentity {
            trigger_type: TriggerType::OnHit,
            activation_source: CooldownSource::Unit {
                unit_instance_id: target_id,
            },
            ability_id: SkillId::from("hard_cc_proc"),
            binding_index: 0,
            caster_id: target_id,
            trigger_unit_id: target_id,
            counterpart_unit_id: Some(source_id),
            target_id: Some(source_id),
            occurrence_id: Uuid::from_u128(0xCC83),
            occurrence_index: 0,
        };
        assert!(core.should_fire_triggered_ability(
            crate::game::battle::core::sim::TriggeredAbilityProcContext {
                proc_roll_identity: &proc_identity,
                current_time_ms: 0,
                proc_chance_percent: 100,
                internal_cooldown_ms: 1_000,
                max_triggers_per_battle: None,
            }
        ));

        for (buff_name, duration_ms) in [("stun", 2_000), ("silence", 3_000), ("poison", 3_000)] {
            core.process_event(
                BattleEvent::ApplyBuff {
                    time_ms: 1,
                    caster_instance_id: source_id,
                    target_instance_id: target_id,
                    buff_id: BuffId::from_name(buff_name),
                    duration_ms,
                    cause: BattleEventCause::default(),
                },
                1,
            )
            .unwrap();
        }

        assert!(!core.should_fire_triggered_ability(
            crate::game::battle::core::sim::TriggeredAbilityProcContext {
                proc_roll_identity: &proc_identity,
                current_time_ms: 999,
                proc_chance_percent: 100,
                internal_cooldown_ms: 1_000,
                max_triggers_per_battle: None,
            }
        ));
        assert!(core.should_fire_triggered_ability(
            crate::game::battle::core::sim::TriggeredAbilityProcContext {
                proc_roll_identity: &proc_identity,
                current_time_ms: 1_000,
                proc_chance_percent: 100,
                internal_cooldown_ms: 1_000,
                max_triggers_per_battle: None,
            }
        ));

        core.process_event(
            BattleEvent::BuffTick {
                time_ms: 1_001,
                caster_instance_id: source_id,
                target_instance_id: target_id,
                buff_id: BuffId::from_name("poison"),
                cause: BattleEventCause::default(),
            },
            1_001,
        )
        .unwrap();
        assert!(core.units.get(&target_id).unwrap().stats.current_health < target_hp);

        core.add_resonance(target_id, 10, 1_001, true);
        assert_eq!(core.units.get(&target_id).unwrap().resonance_current, 0);
        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 2_001,
                caster_instance_id: source_id,
                target_instance_id: target_id,
                buff_id: BuffId::from_name("stun"),
                cause: BattleEventCause::default(),
            },
            2_001,
        )
        .unwrap();
        assert!(core.unit_control_state(target_id, 2_002).unwrap().silenced);
        core.add_resonance(target_id, 10, 2_002, true);
        assert_eq!(core.units.get(&target_id).unwrap().resonance_current, 10);
    }

    #[test]
    fn control_status_hard_cc_releases_block_and_silence_preserves_engagement() {
        let mut core = new_core();
        let blocker_id: UnitInstanceId = Uuid::from_u128(0xCC91).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(0xCC92).into();
        let mut blocker = runtime_unit(blocker_id, Side::Player);
        blocker.block_capacity = 1;
        blocker.block_radius_units = 2.0;
        core.units.insert(blocker_id, blocker);
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));
        place_unit(&mut core, blocker_id, Position::new(1, 1));
        place_unit(&mut core, enemy_id, Position::new(2, 1));

        core.refresh_block_state();
        assert_eq!(core.blocked_by(enemy_id), Some(blocker_id));
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: enemy_id,
                target_instance_id: blocker_id,
                buff_id: BuffId::from_name("silence"),
                duration_ms: 100,
                cause: BattleEventCause::default(),
            },
            0,
        )
        .unwrap();
        core.refresh_block_state();
        assert_eq!(core.blocked_by(enemy_id), Some(blocker_id));

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: enemy_id,
                target_instance_id: blocker_id,
                buff_id: BuffId::from_name("freeze"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            1,
        )
        .unwrap();
        assert_eq!(core.blocked_by(enemy_id), None);

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 11,
                caster_instance_id: enemy_id,
                target_instance_id: blocker_id,
                buff_id: BuffId::from_name("freeze"),
                cause: BattleEventCause::default(),
            },
            11,
        )
        .unwrap();
        assert_eq!(core.blocked_by(enemy_id), Some(blocker_id));
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

        let source_snapshot = core
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
            .expect("source snapshot");
        core.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id,
                source_snapshot,
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
    fn skill_target_selection_is_independent_from_basic_attack_targeting_profile() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5401).into();
        let healthy_low_defense_id: UnitInstanceId = Uuid::from_u128(0x5402).into();
        let weak_high_defense_id: UnitInstanceId = Uuid::from_u128(0x5403).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut healthy_low_defense = runtime_unit(healthy_low_defense_id, Side::Opponent);
        healthy_low_defense.stats.current_health = 20;
        healthy_low_defense.stats.defense = -100;
        let mut weak_high_defense = runtime_unit(weak_high_defense_id, Side::Opponent);
        weak_high_defense.stats.current_health = 5;
        weak_high_defense.stats.defense = 100;
        core.units
            .insert(healthy_low_defense_id, healthy_low_defense);
        core.units.insert(weak_high_defense_id, weak_high_defense);

        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, healthy_low_defense_id, Position::new(1, 0));
        place_unit(&mut core, weak_high_defense_id, Position::new(1, 1));

        let skill = single_step_skill(
            "explicit_lowest_health",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );
        let resolve = |core: &BattleCore| {
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0))
        };

        core.units
            .get_mut(&caster_id)
            .unwrap()
            .basic_attack
            .targeting_profile =
            crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
        let low_defense_weapon_result = resolve(&core);
        core.units
            .get_mut(&caster_id)
            .unwrap()
            .basic_attack
            .targeting_profile =
            crate::game::data::equipment_data::TargetingProfile::DefaultForward;
        let default_weapon_result = resolve(&core);

        for target in [low_defense_weapon_result, default_weapon_result] {
            assert!(matches!(
                target,
                Some(crate::game::battle::event_log::SkillCastTarget::Unit { unit_instance_id })
                    if unit_instance_id == weak_high_defense_id
            ));
        }
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
    fn cast_usefulness_uses_actual_area_targets_for_automatic_and_manual_selection() {
        let skill = enemy_anchor_tile_area_skill(
            "area_actual_targets",
            vec![SkillEffectDef::Damage {
                amount: 100,
                damage_type: DamageType::Magic,
            }],
            crate::game::ability::SkillHitTargetFilter::Enemies,
            TileRangePattern {
                include_anchor_tile: false,
                rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
            },
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5125).into();
        let anchor_id: UnitInstanceId = Uuid::from_u128(0x5126).into();
        let affected_id: UnitInstanceId = Uuid::from_u128(0x5127).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        let mut resistant_anchor = runtime_unit(anchor_id, Side::Opponent);
        resistant_anchor.stats.magic_resist = 1_000_000;
        core.units.insert(anchor_id, resistant_anchor);
        core.units
            .insert(affected_id, runtime_unit(affected_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(0, 1));
        place_unit(&mut core, anchor_id, Position::new(1, 1));
        place_unit(&mut core, affected_id, Position::new(2, 1));

        assert!(!core.is_skill_step_useful_hostile_target(caster_id, &skill.steps[0], anchor_id,));
        assert!(core.is_skill_step_useful_hostile_target(caster_id, &skill.steps[0], affected_id,));
        let target = SkillCastTarget::Unit {
            unit_instance_id: anchor_id,
        };
        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 1)),
            Some(target)
        );
        assert!(core.skill_has_useful_automatic_cast_target(0, &skill, caster_id, Some(target),));
        let manual_validation =
            core.validate_manual_skill_activation(0, caster_id, &skill.id, Some(target));
        assert!(manual_validation.is_ok(), "{manual_validation:?}");
    }

    #[test]
    fn cast_usefulness_rejects_useful_anchor_outside_filtered_area() {
        let skill = enemy_anchor_tile_area_skill(
            "area_excludes_anchor",
            vec![damage_effect()],
            crate::game::ability::SkillHitTargetFilter::Enemies,
            TileRangePattern {
                include_anchor_tile: false,
                rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
            },
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5128).into();
        let anchor_id: UnitInstanceId = Uuid::from_u128(0x5129).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.units
            .insert(anchor_id, runtime_unit(anchor_id, Side::Opponent));
        place_unit(&mut core, caster_id, Position::new(0, 1));
        place_unit(&mut core, anchor_id, Position::new(1, 1));

        assert!(core.is_skill_step_useful_hostile_target(caster_id, &skill.steps[0], anchor_id,));
        let target = SkillCastTarget::Unit {
            unit_instance_id: anchor_id,
        };
        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 1)),
            None
        );
        assert!(!core.skill_has_useful_automatic_cast_target(0, &skill, caster_id, Some(target),));
        let manual_validation =
            core.validate_manual_skill_activation(0, caster_id, &skill.id, Some(target));
        assert!(
            matches!(
                &manual_validation,
                Err(GameError::ActionUnavailable {
                    reason: ActionDisabledReasonCode::NoValidTarget,
                    ..
                })
            ),
            "{manual_validation:?}"
        );
    }

    #[test]
    fn manual_tile_cast_accepts_valid_empty_geometry() {
        let mut skill = single_step_skill(
            "manual_tile_actual_targets",
            SkillTarget::CastTarget,
            1,
            0,
            DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
                },
                area: SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tile_origin: SkillTileAreaOrigin::Anchor,
                    tracking: SkillAreaTracking::GroundFixed,
                    hit_targets: crate::game::ability::SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: crate::game::ability::SkillAreaTickPolicy::EveryTick,
                    duration_ms: 0,
                    tick_interval_ms: None,
                },
            },
            vec![damage_effect()],
        );
        let (_, reach, target_range, air_capable) = skill.cast_target_definition().unwrap();
        skill.cast_targeting = SkillCastTargetingDef::explicit_with_selection(
            SkillCastSelection::PlayerTile,
            SkillTarget::CastTarget,
            reach,
            target_range.cloned(),
            air_capable,
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x513A).into();
        let affected_id: UnitInstanceId = Uuid::from_u128(0x513B).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        place_unit(&mut core, caster_id, Position::new(0, 1));
        let target = Some(SkillCastTarget::Tile {
            position: Position::new(1, 1),
        });

        assert!(core
            .validate_manual_skill_activation(0, caster_id, &skill.id, target)
            .is_ok());

        core.units
            .insert(affected_id, runtime_unit(affected_id, Side::Opponent));
        place_unit(&mut core, affected_id, Position::new(2, 1));
        assert!(core
            .validate_manual_skill_activation(0, caster_id, &skill.id, target)
            .is_ok());
    }

    #[test]
    fn cast_usefulness_accepts_a_later_useful_delivery_step_in_either_order() {
        let mut later_useful = single_step_skill(
            "later_useful",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            1,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Damage {
                amount: 1,
                damage_type: DamageType::Physical,
            }],
        );
        let mut useful_step = later_useful.steps[0].clone();
        useful_step.id = "step_02".to_string();
        useful_step.effects = vec![SkillEffectDef::ApplyBuff {
            buff_id: "stun".to_string(),
            duration_ms: 100,
        }
        .authored("stun")];
        later_useful.steps.push(useful_step);

        let mut first_useful = later_useful.clone();
        first_useful.id = SkillId::from("first_useful");
        first_useful.name = first_useful.id.to_string();
        first_useful.steps.swap(0, 1);

        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0x512A).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x512B).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut armored_target = runtime_unit(target_id, Side::Opponent);
        armored_target.stats.defense = 1_000_000;
        core.units.insert(target_id, armored_target);
        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 0));

        let expected = Some(SkillCastTarget::Unit {
            unit_instance_id: target_id,
        });
        assert_eq!(
            core.resolve_skill_cast_target(
                &later_useful,
                caster_id,
                Side::Player,
                Position::new(0, 0),
            ),
            expected
        );
        assert_eq!(
            core.resolve_skill_cast_target(
                &first_useful,
                caster_id,
                Side::Player,
                Position::new(0, 0),
            ),
            expected
        );
        assert!(core.skill_has_useful_automatic_cast_target(0, &later_useful, caster_id, expected,));

        let mut runtime = core_with_skill_data(vec![], vec![later_useful.clone()]);
        runtime
            .units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut armored_target = runtime_unit(target_id, Side::Opponent);
        armored_target.stats.defense = 1_000_000;
        let target_health = armored_target.stats.current_health;
        runtime.units.insert(target_id, armored_target);
        place_unit(&mut runtime, caster_id, Position::new(0, 0));
        place_unit(&mut runtime, target_id, Position::new(1, 0));
        sync_unit_to_battlefield_tile_center(&mut runtime, caster_id);
        sync_unit_to_battlefield_tile_center(&mut runtime, target_id);
        runtime.invoke_ability(
            0,
            caster_id,
            later_useful.id.as_str(),
            expected,
            BattleEventCause::default(),
            false,
        );
        for _ in 0..8 {
            let Some(event) = runtime.event_queue.pop() else {
                break;
            };
            let time_ms = event.time_ms();
            runtime.process_event(event, time_ms).unwrap();
            if runtime
                .unit_control_state(target_id, time_ms)
                .is_some_and(|state| state.hard_cc_active())
            {
                break;
            }
        }
        assert_eq!(
            runtime.units.get(&target_id).unwrap().stats.current_health,
            target_health,
            "the ineffective current step must be a no-op"
        );
        assert!(
            runtime
                .unit_control_state(target_id, 0)
                .is_some_and(|state| state.hard_cc_active()),
            "a no-op first step must not cancel the later useful step"
        );
    }

    #[test]
    fn cast_usefulness_applies_delivery_filter_and_board_clipping_before_admission() {
        let footprint = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
        };
        let enemy_filtered = enemy_anchor_tile_area_skill(
            "area_filter_enemy",
            vec![damage_effect()],
            crate::game::ability::SkillHitTargetFilter::Enemies,
            footprint.clone(),
        );
        let ally_filtered = enemy_anchor_tile_area_skill(
            "area_filter_ally",
            vec![damage_effect()],
            crate::game::ability::SkillHitTargetFilter::Allies,
            footprint.clone(),
        );
        let any_filtered = enemy_anchor_tile_area_skill(
            "area_filter_any",
            vec![damage_effect()],
            crate::game::ability::SkillHitTargetFilter::Any,
            footprint.clone(),
        );
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(0x512C).into();
        let anchor_id: UnitInstanceId = Uuid::from_u128(0x512D).into();
        let ally_id: UnitInstanceId = Uuid::from_u128(0x512E).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(anchor_id, runtime_unit(anchor_id, Side::Opponent));
        core.units
            .insert(ally_id, runtime_unit(ally_id, Side::Player));
        place_unit(&mut core, caster_id, Position::new(0, 1));
        place_unit(&mut core, anchor_id, Position::new(1, 1));
        place_unit(&mut core, ally_id, Position::new(2, 1));
        let target = Some(SkillCastTarget::Unit {
            unit_instance_id: anchor_id,
        });

        assert!(!core.skill_has_useful_automatic_cast_target(
            0,
            &enemy_filtered,
            caster_id,
            target,
        ));
        assert!(core.skill_has_useful_automatic_cast_target(0, &ally_filtered, caster_id, target,));

        core.units.get_mut(&ally_id).unwrap().lifecycle = RuntimeUnitLifecycle::Withdrawn;
        assert!(!core.skill_has_useful_automatic_cast_target(0, &ally_filtered, caster_id, target,));
        let mut caster_inclusive = ally_filtered.clone();
        if let DeliveryDef::TileArea { area, .. } = &mut caster_inclusive.steps[0].delivery {
            area.include_caster = true;
        }
        assert!(core.skill_has_useful_automatic_cast_target(
            0,
            &caster_inclusive,
            caster_id,
            target,
        ));

        let affected_enemy_id: UnitInstanceId = Uuid::from_u128(0x512F).into();
        core.units.insert(
            affected_enemy_id,
            runtime_unit(affected_enemy_id, Side::Opponent),
        );
        place_unit(&mut core, affected_enemy_id, Position::new(2, 1));
        assert!(core.skill_has_useful_automatic_cast_target(0, &enemy_filtered, caster_id, target,));
        assert!(core.skill_has_useful_automatic_cast_target(0, &any_filtered, caster_id, target,));

        let clipped = enemy_anchor_tile_area_skill(
            "area_clipped_empty",
            vec![damage_effect()],
            crate::game::ability::SkillHitTargetFilter::Enemies,
            footprint,
        );
        place_unit(&mut core, caster_id, Position::new(1, 0));
        place_unit(&mut core, anchor_id, Position::new(0, 0));
        core.units.get_mut(&affected_enemy_id).unwrap().lifecycle = RuntimeUnitLifecycle::Withdrawn;
        let (_, _, _, _, anchor_tile) = core
            .resolve_tile_area_geometry(
                0,
                0,
                0,
                caster_id,
                target,
                match &clipped.steps[0].delivery {
                    DeliveryDef::TileArea { area, .. } => area,
                    _ => unreachable!(),
                },
            )
            .unwrap();
        let facing = core
            .tile_area_projection_facing(
                caster_id,
                match &clipped.steps[0].delivery {
                    DeliveryDef::TileArea { footprint, .. } => footprint,
                    _ => unreachable!(),
                },
            )
            .unwrap();
        let affected = core
            .valid_tile_area_tiles(
                match &clipped.steps[0].delivery {
                    DeliveryDef::TileArea { footprint, .. } => footprint,
                    _ => unreachable!(),
                },
                anchor_tile,
                facing,
            )
            .unwrap();
        assert_eq!(
            affected.len(),
            3,
            "off-board footprint cells must be clipped"
        );
        assert!(!core.skill_has_useful_automatic_cast_target(0, &clipped, caster_id, target,));
    }

    #[test]
    fn useless_projectile_step_is_a_noop_without_cancelling_later_delivery() {
        let mut skill = single_step_skill(
            "projectile_noop_then_useful",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            1,
            0,
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
            vec![SkillEffectDef::Damage {
                amount: 1,
                damage_type: DamageType::Physical,
            }],
        );
        let mut later_step = skill.steps[0].clone();
        later_step.id = "step_02".to_string();
        later_step.delivery = DeliveryDef::Instant;
        later_step.effects = vec![SkillEffectDef::ApplyBuff {
            buff_id: "stun".to_string(),
            duration_ms: 100,
        }
        .authored("stun")];
        skill.steps.push(later_step);

        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x512F).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0x5130).into();
        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut armored_target = runtime_unit(target_id, Side::Opponent);
        armored_target.stats.defense = 1_000_000;
        core.units.insert(target_id, armored_target);
        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 0));
        sync_unit_to_battlefield_tile_center(&mut core, caster_id);
        sync_unit_to_battlefield_tile_center(&mut core, target_id);
        let target = Some(SkillCastTarget::Unit {
            unit_instance_id: target_id,
        });

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0)),
            target
        );
        core.invoke_ability(
            0,
            caster_id,
            skill.id.as_str(),
            target,
            BattleEventCause::default(),
            false,
        );
        for _ in 0..8 {
            let Some(event) = core.event_queue.pop() else {
                break;
            };
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).unwrap();
            if core
                .unit_control_state(target_id, time_ms)
                .is_some_and(|state| state.hard_cc_active())
            {
                break;
            }
        }

        assert!(!core
            .event_log
            .entries
            .iter()
            .any(|entry| matches!(entry.event, BattleLogEvent::SkillProjectileLaunched { .. })));
        assert!(core
            .unit_control_state(target_id, 0)
            .is_some_and(|state| state.hard_cc_active()));
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
            .any(|event| matches!(event.as_ref(), BattleEvent::AutoCastEnd { .. })));
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
        process_manual_start_gate_fixture(&mut core, 0);
        let scheduled_end = core.event_queue.pop().expect("manual end scheduled");
        let BattleEvent::ManualCastEnd { cause, .. } = scheduled_end.event.view().into_owned()
        else {
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

        let interrupted_cast_references_current_start = |event_log: &BattleEventLog| {
            event_log.entries.iter().all(|entry| {
                let BattleLogEvent::SkillCastInterrupted {
                    caster_instance_id,
                    interrupted_skill_id,
                    interrupted_cast_seq,
                    ..
                } = &entry.event
                else {
                    return true;
                };
                event_log.entries.iter().any(|candidate| {
                    candidate.seq == *interrupted_cast_seq
                        && matches!(
                            &candidate.event,
                            BattleLogEvent::ManualCastStart {
                                caster_instance_id: start_caster_id,
                                skill_id,
                                ..
                            } if start_caster_id == caster_instance_id
                                && skill_id == interrupted_skill_id
                        )
                })
            })
        };
        assert!(interrupted_cast_references_current_start(&core.event_log));

        let mut malformed_interrupt = core.event_log.clone();
        let interrupted = malformed_interrupt
            .entries
            .iter_mut()
            .find_map(|entry| match &mut entry.event {
                BattleLogEvent::SkillCastInterrupted {
                    interrupted_cast_seq,
                    ..
                } => Some(interrupted_cast_seq),
                _ => None,
            })
            .expect("produced interrupted-cast entry");
        *interrupted = u64::MAX;
        assert!(!interrupted_cast_references_current_start(
            &malformed_interrupt
        ));
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
        let BattleEvent::AutoCastEnd { cause, .. } = scheduled_end.event.view().into_owned() else {
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
        process_manual_start_gate_fixture(&mut core, 0);
        let old_end = core.event_queue.pop().expect("old manual end scheduled");
        let BattleEvent::ManualCastEnd {
            cause: old_cause, ..
        } = old_end.event.view().into_owned()
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
        process_manual_start_gate_fixture(&mut core, 0);
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
    fn defense_blocks_new_skill_starts_but_does_not_interrupt_pending_casts() {
        let skill = single_step_skill(
            "defense_pending_cast",
            SkillTarget::SelfUnit,
            0,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Heal { amount: 1 }],
        );
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        let caster_id: UnitInstanceId = Uuid::from_u128(0x5182).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.skill_id = Some(skill.id.clone());
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        place_unit(&mut core, caster_id, Position::new(1, 1));
        core.initialize_deployed_tactical_state(
            caster_id,
            TacticalTileKind::Ground,
            crate::game::data::equipment_data::WeaponCombatProfile::default(),
            0,
        )
        .unwrap();

        core.start_manual_skill_cast(1_500, caster_id, skill.id.clone(), None)
            .expect("manual cast should start in Attack stance");
        process_manual_start_gate_fixture(&mut core, 1500);
        core.set_engagement_stance(caster_id, EngagementStance::Defense, 1_500)
            .expect("stance may change during committed focus");
        let scheduled_end = core
            .event_queue
            .iter()
            .find(|event| matches!(event.as_ref(), BattleEvent::ManualCastEnd { .. }))
            .map(std::borrow::Cow::into_owned)
            .expect("manual end scheduled");
        core.process_event(scheduled_end, 1_505)
            .expect("Defense must not cancel an already committed cast");
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

        core.units.get_mut(&caster_id).unwrap().resonance_current = 100;
        assert!(matches!(
            core.validate_manual_skill_activation(1_506, caster_id, &skill.id, None),
            Err(crate::game::behavior::GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::DefenseStance,
                ..
            })
        ));
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
            start_gate_pending: false,
            reset_resonance: true,
            action: None,
            skill_id: SkillId::from("target_cast"),
            cast_target: None,
            accepted_target_tile: None,
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
            start_gate_pending: false,
            reset_resonance: true,
            action: None,
            skill_id: SkillId::from("target_pending_skill"),
            cast_target: None,
            accepted_target_tile: None,
            start_seq: 77,
        });
        core.units.insert(target_id, target);

        let (commands, _result) = core
            .build_skill_step_commands(caster_id, &skill.steps[0], &[target_id], 0, None)
            .unwrap();
        assert!(matches!(
            commands.as_slice(),
            [(BattleCommand::InterruptCast {
                source_id,
                target_id: command_target_id,
            }, _)] if *source_id == caster_id && *command_target_id == target_id
        ));

        core.process_skill_commands(commands, 10, caster_id, None);

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
                object_placement: None,
                width: 4,
                height: 3,
                valid_tiles: vec![Position::new(1, 1), Position::new(2, 1)],
                obstacles: Vec::new(),
            },
            groups: Vec::new(),
            events: Vec::new(),
            terminal_outcomes: BattleScenario::fixture_survival_outcomes(1_000),
            boss_runtime: None,
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
                control_cancelled: false,
                flow: None,
                fact_source: Default::default(),
                execution_source: None,
                caster_instance_id: caster_id,
                ability_event_seq: cast_seq,
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
    fn reused_target_execute_does_not_reapply_cast_acquisition_range() {
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

        let SkillCastTargetingDef::Explicit { air_capable, .. } = &mut skill.cast_targeting;
        *air_capable = true;

        assert_eq!(
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos),
            Some(SkillCastTarget::Unit {
                unit_instance_id: airborne_target_id
            })
        );
    }

    #[test]
    fn reused_enemy_single_skill_step_uses_cast_air_eligibility() {
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
            .resolve_skill_step_context(
                0,
                0,
                0,
                caster_id,
                &skill,
                skill.first_step().unwrap(),
                cast_target,
            )
            .is_none());

        let SkillCastTargetingDef::Explicit { air_capable, .. } = &mut skill.cast_targeting;
        *air_capable = true;

        assert!(matches!(
            core.resolve_skill_step_context(
                0,
                0,
                0,
                caster_id,
                &skill,
                skill.first_step().unwrap(),
                cast_target,
            ),
            Some(SkillCastTarget::Unit { unit_instance_id }) if unit_instance_id == airborne_target_id
        ));
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
            .any(|event| matches!(event.as_ref(), BattleEvent::AutoCastStart { .. })));

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
            event.as_ref(),
            BattleEvent::AutoCastStart {
                time_ms: 11,
                caster_instance_id,
                ..
            } if *caster_instance_id == caster_id
        )));
    }

    #[test]
    fn hard_cc_cancels_unreleased_autocast_and_retries_after_release() {
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
        let end_cause = match scheduled_end.event.view().into_owned() {
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

        assert!(core
            .units
            .get(&caster_id)
            .unwrap()
            .pending_skill_cast
            .is_none());
        assert!(core.units.get(&caster_id).unwrap().pending_cast);
        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 1
                && matches!(
                    entry.event,
                    BattleLogEvent::SkillCastInterrupted {
                        caster_instance_id,
                        ..
                    } if caster_instance_id == caster_id
                )
        }));

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

        assert_eq!(core.units.get(&caster_id).unwrap().resonance_current, 100);

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 11,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("stun"),
                cause: BattleEventCause::default(),
            },
            11,
        )
        .unwrap();

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 12,
                caster_instance_id: caster_id,
                cause: BattleEventCause::default(),
            },
            12,
        )
        .unwrap();

        let retry_end = core
            .event_queue
            .iter()
            .find(|event| matches!(event.as_ref(), BattleEvent::AutoCastEnd { time_ms: 17, .. }))
            .map(std::borrow::Cow::into_owned)
            .expect("retried cast should schedule a fresh completion");
        core.process_event(retry_end, 17).unwrap();

        assert!(core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 17
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
        let end_cause = match scheduled_end.event.view().into_owned() {
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
            event.as_ref(),
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
        let cast_seq = match &event.event.view().into_owned() {
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
    fn diagnostic_sequence_exhaustion_never_retains_duplicate_sequences() {
        let mut core = new_core();
        core.event_log_seq = u64::MAX;

        let first = core.record_event_log(
            1,
            BattleLogEvent::BattleEnd {
                winner: crate::game::battle::types::BattleWinner::Draw,
            },
        );
        let retained_after_first = core.event_log.entries.len();
        let retry = core.record_event_log(
            2,
            BattleLogEvent::BattleEnd {
                winner: crate::game::battle::types::BattleWinner::Draw,
            },
        );

        assert_eq!(first, u64::MAX);
        assert_eq!(retry, u64::MAX);
        assert!(core.event_log.sequence_exhausted());
        assert_eq!(retained_after_first, 1);
        assert_eq!(core.event_log.entries.len(), 1);
        assert_eq!(core.event_log.entries[0].seq, u64::MAX);
    }

    #[test]
    fn application_unit_map_history_survives_native_clone_and_runtime_reset() {
        let mut core = new_core();
        for index in 1..65 {
            let id = UnitInstanceId(Uuid::from_u128(index));
            core.insert_runtime_unit(id, runtime_unit(id, Side::Player));
        }
        let slots = core.units_clone_history.bucket_bound().unwrap();
        assert!(slots >= core.units.capacity());
        let cloned = core.clone();
        assert_eq!(cloned.units_clone_history.bucket_bound(), Some(slots));
        assert_eq!(
            cloned.units.keys().collect::<Vec<_>>(),
            core.units.keys().collect::<Vec<_>>()
        );
        core.reset_runtime_state_for_battle();
        assert!(core.units.is_empty());
        assert_eq!(core.units_clone_history.bucket_bound(), Some(slots));
        assert_eq!(core.units.capacity(), cloned.units.capacity());
        // The old concrete fixture borrow contract still compiles. It is not
        // used to import arbitrary map history into the private Session path.
        let _: &mut HashMap<UnitInstanceId, RuntimeUnit> = &mut core.units;
    }

    pub(super) fn native_action_fixture(
        skill: SkillDef,
        limits: crate::game::battle::runtime_control::RuntimeFactLimits,
    ) -> (
        BattleCore,
        sim::BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
        Arc::make_mut(&mut core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(10_000);
        core = core.with_runtime_fact_limits(limits).unwrap();
        let execution = core.start_battle_execution().unwrap();
        let caster = Uuid::from_u128(0xFA01).into();
        let target = Uuid::from_u128(0xFA02).into();
        let ally = Uuid::from_u128(0xFA03).into();
        for (id, owner, tile) in [
            (caster, Side::Player, Position::new(1, 1)),
            (target, Side::Opponent, Position::new(2, 1)),
            (ally, Side::Player, Position::new(4, 4)),
        ] {
            let mut unit = runtime_unit(id, owner);
            unit.stats.current_health = 100;
            unit.stats.max_health = 100;
            unit.stats.attack = 0;
            unit.stats.attack_interval_ms = 100_000;
            unit.next_basic_attack_ms = 100_000;
            unit.skill_activation_mode = SkillActivationMode::Manual;
            if owner == Side::Player {
                unit.skill_id = Some(skill.id.clone());
                unit.resonance_current = 100;
            }
            core.insert_runtime_unit(id, unit);
            place_unit(&mut core, id, tile);
            sync_unit_to_battlefield_tile_center(&mut core, id);
        }
        (core, execution, caster, target, ally)
    }

    fn native_action_probe(
        core: &BattleCore,
        state: &sim::BattleExecutionState,
    ) -> (String, Vec<String>, serde_json::Value, String) {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
            format!("{state:?}"),
        )
    }

    #[test]
    fn r06_points_only_change_basic_schedule_not_spell_area_or_projectile() {
        use crate::game::ability::{ProjectileHitPolicy, SkillProjectileCollisionDef};
        use crate::game::battle::runtime_facts::ActionTerminalKind;
        use crate::game::data::equipment_data::WeaponCombatProfile;

        for delivery_kind in ["instant", "area", "projectile"] {
            let mut paired_setup = None;
            let mut paired_schedule = None;
            for (points, interval, next_start) in [(100, 1500, 1900), (200, 750, 1150)] {
                let mut skill = if delivery_kind == "area" {
                    let mut skill = enemy_tile_area_skill(
                        "r06_points_area",
                        vec![SkillEffectDef::Damage {
                            amount: 2,
                            damage_type: DamageType::True,
                        }],
                    );
                    let DeliveryDef::TileArea { area, .. } = &mut skill.steps[0].delivery else {
                        unreachable!()
                    };
                    area.duration_ms = 200;
                    area.tick_interval_ms = Some(100);
                    skill
                } else {
                    let delivery = if delivery_kind == "projectile" {
                        DeliveryDef::Projectile {
                            speed_units_per_ms: 2_500,
                            hit_policy: ProjectileHitPolicy::TargetLocked,
                            allow_targetless_cast: false,
                            max_range_tiles: None,
                            max_lifetime_ms: None,
                            max_kills: None,
                            max_pierces: None,
                            collision: SkillProjectileCollisionDef::default(),
                        }
                    } else {
                        DeliveryDef::Instant
                    };
                    single_step_skill(
                        "r06_points_delivery",
                        SkillTarget::EnemySingle {
                            rule: UnitTargetRule::Nearest,
                        },
                        3,
                        100,
                        delivery,
                        vec![SkillEffectDef::Damage {
                            amount: 7,
                            damage_type: DamageType::True,
                        }],
                    )
                };
                skill.focus_time_ms = 100;
                let (mut core, mut state, caster, target, _) =
                    native_action_fixture(skill.clone(), Default::default());
                core.units.get_mut(&target).unwrap().body.radius = 0.375;
                assert_eq!(core.units[&target].stats.defense, 0);
                let mut profile = UnitCombatProfile::for_employee(100, 5, 0, 0, points);
                profile
                    .apply_weapon_profile(&WeaponCombatProfile::default())
                    .unwrap();
                assert_eq!(profile.stats.attack_interval_ms, interval);
                assert_eq!(profile.basic_attack.interval_ms, interval);
                let actor = core.units.get_mut(&caster).unwrap();
                actor.stats.attack = 5;
                actor.stats.attack_interval_ms = profile.stats.attack_interval_ms;
                actor.basic_attack.interval_ms = profile.basic_attack.interval_ms;
                actor.basic_attack_speed_percent = profile.basic_attack_speed_percent;
                assert_eq!(actor.basic_attack.effective_windup_ms(), 200);
                assert!(matches!(actor.basic_attack.delivery, DeliveryDef::Instant));
                let mut normalized = actor.clone();
                normalized.basic_attack_speed_percent = 100;
                normalized.stats.attack_interval_ms = 1500;
                normalized.basic_attack.interval_ms = 1500;
                let setup = (
                    format!("{normalized:?}"),
                    format!("{:?}", core.units[&target]),
                    serde_json::to_value(&skill).unwrap(),
                );
                if let Some(expected) = &paired_setup {
                    assert_eq!(&setup, expected);
                } else {
                    paired_setup = Some(setup);
                }

                let handle = core
                    .request_skill_action(0, caster, skill.id.clone(), None)
                    .unwrap();
                assert_eq!(core.query_action(&handle).unwrap().accepted_at_ms, 0);
                core.step_battle_execution_until(&mut state, 99).unwrap();
                let before_release = core.query_action(&handle).unwrap();
                assert!(before_release.released_at_ms.is_none());
                assert!(before_release.terminal.is_none());
                assert_eq!(core.units[&target].stats.current_health, 100);
                core.step_battle_execution_until(&mut state, 100).unwrap();
                let released = core.query_action(&handle).unwrap();
                assert_eq!(released.released_at_ms, Some(100));
                assert_eq!(
                    released.terminal.unwrap().kind,
                    ActionTerminalKind::Finished
                );
                assert_eq!(released.terminal.unwrap().time_ms, 100);
                if delivery_kind == "instant" {
                    assert_eq!(released.effects_settled_at_ms, Some(100));
                    assert_eq!(core.units[&target].stats.current_health, 93);
                } else {
                    assert!(released.effects_settled_at_ms.is_none());
                }
                core.step_battle_execution_until(&mut state, 299).unwrap();
                if delivery_kind == "projectile" {
                    assert_eq!(core.units[&target].stats.current_health, 100);
                    assert!(core
                        .query_action(&handle)
                        .unwrap()
                        .facts
                        .latest_damage
                        .is_none());
                }
                core.step_battle_execution_until(&mut state, 300).unwrap();
                let action = core.query_action(&handle).unwrap();
                let (settled, strikes, damage, hp) = if delivery_kind == "instant" {
                    (100, 1, 7, 93)
                } else if delivery_kind == "area" {
                    (300, 2, 4, 96)
                } else {
                    (300, 1, 7, 93)
                };
                assert_eq!(action.effects_settled_at_ms, Some(settled));
                assert_eq!(action.facts.damage_strikes, strikes);
                assert_eq!(action.facts.damage_hp_lost, damage);
                assert_eq!(core.units[&target].stats.current_health, hp);
                if delivery_kind == "projectile" {
                    // (1 tile - .375 target radius - .125 projectile radius) / .0025 =200ms.
                    assert_eq!(action.facts.latest_damage.as_ref().unwrap().time_ms, 300);
                }
                let schedule = (
                    action.accepted_at_ms,
                    action.released_at_ms,
                    action.terminal.unwrap().time_ms,
                    action.effects_settled_at_ms,
                    action.facts.damage_strikes,
                    action.facts.damage_hp_lost,
                    hp,
                );
                if let Some(expected) = &paired_schedule {
                    assert_eq!(&schedule, expected);
                } else {
                    paired_schedule = Some(schedule);
                }

                core.units.get_mut(&caster).unwrap().next_basic_attack_ms = 400;
                core.process_event(
                    BattleEvent::AttackStart {
                        time_ms: 400,
                        attacker_instance_id: caster,
                        target_instance_id: Some(target),
                        schedule_next: true,
                        cause: BattleEventCause::Root {
                            kind: BattleEventRootCause::Period,
                        },
                    },
                    400,
                )
                .unwrap();
                // An actual start and a resolve witness distinguish this from a readiness deferral.
                assert!(core.event_log.entries.iter().any(|entry| entry.time_ms == 400 && matches!(entry.event, BattleLogEvent::AttackStart { attacker_instance_id, target_instance_id, .. } if attacker_instance_id == caster && target_instance_id == target)));
                assert!(core.event_queue.iter().any(|event| matches!(event.as_ref(), BattleEvent::AttackResolve { time_ms: 600, attacker_instance_id, target_instance_id, .. } if *attacker_instance_id == caster && *target_instance_id == target)));
                assert!(core.event_queue.iter().any(|event| matches!(event.as_ref(), BattleEvent::AttackStart { time_ms, attacker_instance_id, schedule_next: true, .. } if *time_ms == next_start && *attacker_instance_id == caster)));
                assert_eq!(core.units[&caster].next_basic_attack_ms, next_start);
                core.step_battle_execution_until(&mut state, 599).unwrap();
                assert_eq!(core.units[&target].stats.current_health, hp);
                core.step_battle_execution_until(&mut state, 600).unwrap();
                assert_eq!(core.units[&target].stats.current_health, hp - 5);
                eprintln!("R06_POINTS_TIMERS delivery={delivery_kind} points={points} interval={interval} release=100 settled={settled} skill_hp={hp} basic_start=400 resolve=600 next={next_start} basic_hp={}", hp - 5);
            }
        }
    }

    #[test]
    fn native_action_area_observes_accept_release_control_finish_and_actual_settlement() {
        use crate::game::battle::runtime_facts::{ActionTerminalKind, CostReason};
        let mut skill = enemy_tile_area_skill(
            "native_area_action",
            vec![SkillEffectDef::Damage {
                amount: 2,
                damage_type: DamageType::True,
            }],
        );
        skill.focus_time_ms = 100;
        let DeliveryDef::TileArea { area, .. } = &mut skill.steps[0].delivery else {
            unreachable!()
        };
        area.duration_ms = 200;
        area.tick_interval_ms = Some(100);
        let (mut core, mut state, caster, target, _) =
            native_action_fixture(skill.clone(), Default::default());
        let handle = core
            .request_skill_action(0, caster, skill.id, None)
            .unwrap();
        let accepted = core.query_action(&handle).unwrap();
        assert_eq!(accepted.accepted_at_ms, 0);
        assert!(accepted.released_at_ms.is_none());
        assert!(accepted.terminal.is_none());
        core.step_battle_execution_until(&mut state, 100).unwrap();
        let released = core.query_action(&handle).unwrap();
        assert_eq!(released.released_at_ms, Some(100));
        assert_eq!(
            released.terminal.unwrap().kind,
            ActionTerminalKind::Finished
        );
        assert_eq!(released.terminal.unwrap().time_ms, 100);
        assert!(released.effects_settled_at_ms.is_none());
        assert!(released.outstanding_deliveries > 0);
        assert_eq!(released.facts.cost_mutations, 1);
        assert_eq!(
            released.facts.latest_cost.as_ref().unwrap().reason,
            CostReason::CastReset
        );
        assert_eq!(released.facts.resource_spent, 100);
        core.step_battle_execution_until(&mut state, 300).unwrap();
        let receipt = core.take_settled_action_result(&handle).unwrap();
        assert_eq!(receipt.action.effects_settled_at_ms, Some(300));
        assert_eq!(receipt.action.facts.damage_strikes, 2);
        assert_eq!(receipt.action.facts.damage_hp_lost, 4);
        assert_eq!(core.units[&target].stats.current_health, 96);
        assert_eq!(
            receipt
                .action
                .facts
                .latest_damage
                .as_ref()
                .unwrap()
                .source
                .action_ordinal(),
            Some(handle.ordinal())
        );
        let live = serde_json::to_value(core.runtime_fact_view()).unwrap();
        let snapshot = core.runtime_fact_snapshot().unwrap();
        assert_eq!(serde_json::to_value(snapshot.as_ref()).unwrap(), live);
        assert_eq!(
            snapshot.latest_settled_action.as_ref().unwrap().ordinal,
            handle.ordinal()
        );
    }

    #[test]
    fn native_action_projectile_first_hit_is_at_300_after_accept_0_release_100() {
        use crate::game::ability::{ProjectileHitPolicy, SkillProjectileCollisionDef};
        use crate::game::battle::runtime_facts::ActionTerminalKind;
        let skill = single_step_skill(
            "native_delayed_hit_action",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            100,
            DeliveryDef::Projectile {
                speed_units_per_ms: 2_500,
                hit_policy: ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: SkillProjectileCollisionDef::default(),
            },
            vec![SkillEffectDef::Damage {
                amount: 7,
                damage_type: DamageType::True,
            }],
        );
        let (mut core, mut state, caster, target, _) =
            native_action_fixture(skill.clone(), Default::default());
        // Homing hits the bodies' contact boundary, not the target center.
        // One tile minus (target .375 + projectile .125) leaves .5 world
        // units, taking 200 ms at .0025 world units/ms. Fixture geometry only.
        core.units.get_mut(&target).unwrap().body.radius = 0.375;
        let handle = core
            .request_skill_action(0, caster, skill.id, None)
            .unwrap();
        for time in [0, 99, 100, 299] {
            core.step_battle_execution_until(&mut state, time).unwrap();
            let action = core.query_action(&handle).unwrap();
            assert_eq!(action.accepted_at_ms, 0);
            assert_eq!(action.facts.damage_mutations, 0, "time={time}");
            assert!(action.facts.latest_damage.is_none());
            assert_eq!(core.units[&target].stats.current_health, 100, "time={time}");
            assert!(action.effects_settled_at_ms.is_none());
            if time < 100 {
                assert!(action.released_at_ms.is_none());
                assert!(action.terminal.is_none());
            } else {
                assert_eq!(action.released_at_ms, Some(100));
                let terminal = action.terminal.unwrap();
                assert_eq!(terminal.kind, ActionTerminalKind::Finished);
                assert_eq!(terminal.time_ms, 100);
                assert!(action.outstanding_deliveries > 0);
            }
        }
        core.step_battle_execution_until(&mut state, 300).unwrap();
        let action = core.query_action(&handle).unwrap();
        assert_eq!(action.effects_settled_at_ms, Some(300));
        assert_eq!(action.terminal.unwrap().time_ms, 100);
        assert_eq!(action.outstanding_deliveries, 0);
        assert_eq!(action.facts.damage_mutations, 1);
        assert_eq!(action.facts.damage_strikes, 1);
        assert_eq!(action.facts.damage_hp_lost, 7);
        let damage = action.facts.latest_damage.as_ref().unwrap();
        assert_eq!(damage.time_ms, 300);
        assert_eq!(damage.target, target);
        assert_eq!((damage.hp_before, damage.hp_after), (100, 93));
        assert_eq!(damage.source.actor, Some(caster));
        assert_eq!(damage.source.action_ordinal(), Some(handle.ordinal()));
        assert_eq!(core.units[&target].stats.current_health, 93);
        let receipt = core.take_settled_action_result(&handle).unwrap();
        assert_eq!(
            receipt.action.facts.latest_damage.as_ref().unwrap().time_ms,
            300
        );
    }

    #[test]
    fn native_action_withdraw_before_release_and_after_release_have_distinct_progress() {
        use crate::game::battle::runtime_facts::{ActionCancelReason, ActionTerminalKind};
        for cancel_time in [50, 150] {
            let mut skill = enemy_tile_area_skill("native_cancel_action", vec![damage_effect()]);
            skill.focus_time_ms = 100;
            let DeliveryDef::TileArea { area, .. } = &mut skill.steps[0].delivery else {
                unreachable!()
            };
            area.duration_ms = 200;
            area.tick_interval_ms = Some(100);
            // A real later control step keeps control unfinished at cancellation150.
            let mut later = skill.steps[0].clone();
            later.id = "later".into();
            later.delay_ms = 400;
            later.delivery = DeliveryDef::Instant;
            skill.steps.push(later);
            let (mut core, mut state, caster, _, _) =
                native_action_fixture(skill.clone(), Default::default());
            let handle = core
                .request_skill_action(0, caster, skill.id, None)
                .unwrap();
            core.step_battle_execution_until(&mut state, cancel_time)
                .unwrap();
            core.apply_live_command(sim::BattleLiveCommand::WithdrawUnit {
                unit_id: caster,
                time_ms: cancel_time,
            })
            .unwrap();
            let cancelled = core.query_action(&handle).unwrap();
            assert_eq!(
                cancelled.terminal.unwrap().kind,
                ActionTerminalKind::Cancelled(ActionCancelReason::Withdrawn)
            );
            assert_eq!(cancelled.terminal.unwrap().time_ms, cancel_time);
            if cancel_time == 50 {
                assert!(cancelled.released_at_ms.is_none());
                assert_eq!(cancelled.effects_settled_at_ms, Some(50));
                assert_eq!(cancelled.facts.cost_mutations, 0);
            } else {
                assert_eq!(cancelled.released_at_ms, Some(100));
                assert!(cancelled.effects_settled_at_ms.is_none());
                core.step_battle_execution_until(&mut state, 300).unwrap();
                assert_eq!(
                    core.query_action(&handle).unwrap().effects_settled_at_ms,
                    Some(300)
                );
            }
        }
    }

    #[test]
    fn native_action_failed_timestamp_keeps_prior_successful_prefix_and_retries_recover() {
        use crate::game::battle::runtime_control::{
            BattleRuntimeFault, RuntimeFactLimits, RuntimeFaultCode,
        };
        let skill = single_step_skill(
            "native_fault_action",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            100,
            DeliveryDef::Instant,
            vec![damage_effect(), damage_effect()],
        );
        let (mut core, mut state, caster, target, _) = native_action_fixture(
            skill.clone(),
            RuntimeFactLimits {
                max_strike_segments: 1,
                ..Default::default()
            },
        );
        let handle = core
            .request_skill_action(0, caster, skill.id, None)
            .unwrap();
        core.event_queue.push(BattleEvent::ApplyBuff {
            time_ms: 50,
            caster_instance_id: target,
            target_instance_id: target,
            buff_id: BuffId::from_name("silence"),
            duration_ms: 1000,
            cause: BattleEventCause::default(),
        });
        let mut prefix = core.clone();
        let mut prefix_state = state.clone();
        prefix
            .step_battle_execution_until(&mut prefix_state, 50)
            .unwrap();
        let error = core
            .step_battle_execution_until(&mut state, 200)
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::StrikeSegmentLimit,
                ..
            })
        ));
        assert_eq!(
            native_action_probe(&core, &state),
            native_action_probe(&prefix, &prefix_state)
        );
        assert_eq!(state.last_event_time_ms(), 50);
        assert_eq!(core.units[&target].stats.current_health, 100);
        assert_eq!(core.units[&caster].resonance_current, 100);
        assert!(core.query_action(&handle).unwrap().released_at_ms.is_none());
        core.apply_live_command(sim::BattleLiveCommand::WithdrawUnit {
            unit_id: caster,
            time_ms: 50,
        })
        .unwrap();
        core.step_battle_execution_until(&mut state, 200).unwrap();
        assert_eq!(
            core.take_settled_action_result(&handle)
                .unwrap()
                .action
                .facts
                .damage_mutations,
            0
        );
    }

    #[test]
    fn native_action_piercing_empty_final_expiry_closes_carrier_without_releasing_gate() {
        use crate::game::ability::{
            ProjectileHitPolicy, SkillProjectileCollisionDef, SkillStepCondition,
        };
        let mut skill = single_step_skill(
            "native_empty_expiry",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            100,
            DeliveryDef::Projectile {
                speed_units_per_ms: 10_000,
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: false,
                max_range_tiles: Some(3),
                max_lifetime_ms: Some(300),
                max_kills: None,
                max_pierces: None,
                collision: SkillProjectileCollisionDef {
                    radius_units: 100_000,
                    piercing: true,
                    ..Default::default()
                },
            },
            vec![SkillEffectDef::Damage {
                amount: 0,
                damage_type: DamageType::Magic,
            }],
        );
        let mut deferred = skill.steps[0].clone();
        deferred.id = "still_deferred".into();
        deferred.delay_ms = 200;
        deferred.when = SkillStepCondition::IfPreviousStepDealtDamage;
        deferred.delivery = DeliveryDef::Instant;
        deferred.effects = vec![damage_effect().authored("deferred_hit")];
        skill.steps.push(deferred);
        let (mut core, mut state, caster, target, _) =
            native_action_fixture(skill.clone(), Default::default());
        let handle = core
            .request_skill_action(0, caster, skill.id, None)
            .unwrap();
        core.step_battle_execution_until(&mut state, 300).unwrap();
        assert_eq!(core.active_projectiles.len(), 1);
        let projectile = core.active_projectiles.values().next().unwrap();
        assert_eq!(projectile.hit_unit_ids, vec![target]);
        let cast_seq = projectile.cast_seq;
        assert_eq!(core.active_skill_casts[&cast_seq].deferred_steps.len(), 1);
        // Authored zero Damage is skipped by the existing command builder.
        // The geometric hit therefore leaves the damage-dependent control pending.
        assert_eq!(
            core.query_action(&handle).unwrap().facts.damage_mutations,
            0
        );
        core.step_battle_execution_until(&mut state, 450).unwrap();
        assert!(core.active_projectiles.is_empty());
        let action = core.query_action(&handle).unwrap();
        assert_eq!(action.released_at_ms, Some(100));
        assert_eq!(action.outstanding_deliveries, 0);
        assert!(action.terminal.is_some());
        assert_eq!(action.effects_settled_at_ms, Some(400));
        assert_eq!(action.facts.damage_mutations, 0);
        assert!(!core.active_skill_casts.contains_key(&cast_seq));
        assert_eq!(core.units[&target].stats.current_health, 100);
    }

    #[test]
    fn native_action_pending_death_preserves_cast_end_cost_without_release() {
        use crate::game::battle::runtime_facts::{
            ActionFailureReason, ActionTerminalKind, CostReason,
        };
        let skill = single_step_skill(
            "native_pending_death",
            SkillTarget::SelfUnit,
            0,
            100,
            DeliveryDef::Instant,
            vec![],
        );
        let (mut core, mut state, caster, target, _) =
            native_action_fixture(skill.clone(), Default::default());
        let handle = core
            .request_skill_action(0, caster, skill.id, None)
            .unwrap();
        core.step_battle_execution_until(&mut state, 50).unwrap();
        core.finalize_unit_death(Some(target), caster, Side::Player, 50);
        assert!(core.units[&caster].pending_skill_cast.is_some());
        assert_eq!(core.units[&caster].resonance_current, 100);
        core.step_battle_execution_until(&mut state, 100).unwrap();
        let action = core.query_action(&handle).unwrap();
        assert_eq!(action.accepted_at_ms, 0);
        assert_eq!(action.released_at_ms, None);
        assert_eq!(
            action.terminal.unwrap().kind,
            ActionTerminalKind::Failed(ActionFailureReason::CasterUnavailable)
        );
        assert_eq!(action.terminal.unwrap().time_ms, 100);
        assert_eq!(action.effects_settled_at_ms, Some(100));
        assert_eq!(action.facts.cost_mutations, 1);
        assert_eq!(
            action.facts.latest_cost.as_ref().unwrap().reason,
            CostReason::CastReset
        );
        assert_eq!(core.units[&caster].resonance_current, 0);
        assert!(core.units[&caster].pending_skill_cast.is_none());
        assert_eq!(core.runtime_fact_view().summary().deaths, 1);
    }

    #[test]
    fn native_action_discarded_request_and_result_take_restore_authority() {
        use crate::game::battle::runtime_facts::ActionQueryError;
        let skill = single_step_skill(
            "native_escaped",
            SkillTarget::SelfUnit,
            0,
            100,
            DeliveryDef::Instant,
            vec![],
        );
        let (mut core, mut state, caster, _, _) =
            native_action_fixture(skill.clone(), Default::default());
        let before = native_action_probe(&core, &state);
        let mut escaped = None;
        let failed: Result<(), GameError> = core.runtime_transaction(|candidate| {
            escaped = Some(candidate.request_skill_action(0, caster, skill.id.clone(), None)?);
            Err(GameError::InvalidAction)
        });
        assert!(failed.is_err());
        assert_eq!(native_action_probe(&core, &state), before);
        let escaped = escaped.unwrap();
        let handle = core
            .request_skill_action(0, caster, skill.id, None)
            .unwrap();
        assert_eq!(escaped.ordinal(), handle.ordinal());
        assert_eq!(
            core.query_action(&escaped).unwrap_err(),
            ActionQueryError::StaleOccurrence
        );
        core.step_battle_execution_until(&mut state, 100).unwrap();
        let before_take = native_action_probe(&core, &state);
        let mut escaped_receipt = None;
        let failed: Result<(), GameError> = core.runtime_transaction(|candidate| {
            escaped_receipt = Some(candidate.take_settled_action_result(&handle).unwrap());
            Err(GameError::InvalidAction)
        });
        assert!(failed.is_err());
        assert_eq!(native_action_probe(&core, &state), before_take);
        let receipt = core.take_settled_action_result(&handle).unwrap();
        assert_eq!(
            serde_json::to_value(&receipt).unwrap(),
            serde_json::to_value(escaped_receipt.unwrap()).unwrap()
        );
        assert_eq!(
            core.query_action(&handle).unwrap_err(),
            ActionQueryError::Unavailable
        );
        core.start_battle_execution().unwrap();
        assert_eq!(
            core.query_action(&handle).unwrap_err(),
            ActionQueryError::DifferentAttempt
        );
        assert_eq!(receipt.action.effects_settled_at_ms, Some(100));
    }

    #[test]
    fn native_action_cloned_battle_has_independent_cancel_and_completion() {
        use crate::game::battle::runtime_facts::{ActionCancelReason, ActionTerminalKind};
        let skill = single_step_skill(
            "native_branch",
            SkillTarget::SelfUnit,
            0,
            100,
            DeliveryDef::Instant,
            vec![],
        );
        let (mut original, mut original_state, caster, _, _) =
            native_action_fixture(skill.clone(), Default::default());
        let handle = original
            .request_skill_action(0, caster, skill.id, None)
            .unwrap();
        let mut branch = original.clone();
        let mut branch_state = original_state.clone();
        branch
            .step_battle_execution_until(&mut branch_state, 100)
            .unwrap();
        let receipt = branch.take_settled_action_result(&handle).unwrap();
        assert_eq!(
            receipt.action.terminal.unwrap().kind,
            ActionTerminalKind::Finished
        );
        assert!(original
            .query_action(&handle)
            .unwrap()
            .released_at_ms
            .is_none());
        original
            .step_battle_execution_until(&mut original_state, 50)
            .unwrap();
        original
            .apply_live_command(sim::BattleLiveCommand::WithdrawUnit {
                unit_id: caster,
                time_ms: 50,
            })
            .unwrap();
        let cancelled = original.take_settled_action_result(&handle).unwrap();
        assert_eq!(
            cancelled.action.terminal.unwrap().kind,
            ActionTerminalKind::Cancelled(ActionCancelReason::Withdrawn)
        );
        assert_eq!(cancelled.action.effects_settled_at_ms, Some(50));
        assert_eq!(receipt.action.effects_settled_at_ms, Some(100));
    }

    #[test]
    fn native_predeath_limit_rejects_lethal_mutation_and_recovers_after_actual_expiry() {
        use crate::game::battle::event_log::HpChangeReason;
        use crate::game::battle::runtime_control::{
            BattleRuntimeFault, RuntimeFactLimits, RuntimeFaultCode,
        };
        let skill = single_step_skill(
            "native_predeath_cap",
            SkillTarget::SelfUnit,
            0,
            100,
            DeliveryDef::Instant,
            vec![],
        );
        let (mut core, mut state, caster, target, ally) = native_action_fixture(
            skill,
            RuntimeFactLimits {
                max_predeath_buffs: 1,
                ..Default::default()
            },
        );
        for (provider, duration_ms) in [(caster, 1000), (ally, 1)] {
            core.event_queue.push(BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: provider,
                target_instance_id: target,
                buff_id: BuffId::from_name("silence"),
                duration_ms,
                cause: BattleEventCause::default(),
            });
        }
        core.step_battle_execution_until(&mut state, 0).unwrap();
        assert_eq!(core.buffs.len(), 2);
        let before = native_action_probe(&core, &state);
        let error = core
            .runtime_transaction(|candidate| {
                candidate.apply_hp_delta_and_record(
                    Some(caster),
                    target,
                    -100,
                    0,
                    HpChangeReason::Command,
                );
                Ok(())
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::PredeathBuffLimit,
                requested: Some(2),
                ..
            })
        ));
        assert_eq!(native_action_probe(&core, &state), before);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.buffs.len(), 1);
        core.runtime_transaction(|candidate| {
            candidate.apply_hp_delta_and_record(
                Some(caster),
                target,
                -100,
                1,
                HpChangeReason::Command,
            );
            Ok(())
        })
        .unwrap();
        let death = core.runtime_fact_summary().latest_death.as_ref().unwrap();
        assert_eq!(death.predeath.hp_before_lethal, 100);
        assert_eq!(death.predeath.buffs.len(), 1);
        assert_eq!(core.runtime_fact_summary().deaths, 1);
        assert_eq!(core.units[&target].stats.current_health, 0);
    }

    #[test]
    fn native_action_result_admission_rejects_before_focus_and_queue_mutation() {
        use crate::game::battle::runtime_control::{
            BattleRuntimeFault, RuntimeFactLimits, RuntimeFaultCode,
        };
        let skill = single_step_skill(
            "native_result_slot",
            SkillTarget::SelfUnit,
            0,
            100,
            DeliveryDef::Instant,
            vec![],
        );
        let (mut core, state, caster, _, ally) = native_action_fixture(
            skill.clone(),
            RuntimeFactLimits {
                max_retained_results: 1,
                ..Default::default()
            },
        );
        core.request_skill_action(0, caster, skill.id.clone(), None)
            .unwrap();
        let before = native_action_probe(&core, &state);
        let error = core
            .request_skill_action(0, ally, skill.id, None)
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::RetainedResultLimit,
                ..
            })
        ));
        assert_eq!(native_action_probe(&core, &state), before);
        assert!(core.units[&ally].pending_skill_cast.is_none());
    }

    #[test]
    fn native_clock_only_step_stages_only_execution_and_preserves_failure_prefix() {
        use crate::game::battle::runtime_control::{RuntimeFactLimits, RuntimeFaultCode};
        let skill = single_step_skill(
            "native_clock_only",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            0,
            DeliveryDef::Instant,
            vec![damage_effect()],
        );
        let (mut core, mut execution, _, _, _) = native_action_fixture(skill, Default::default());
        core.event_queue.clear();
        core.runtime_facts.limits.max_clone_slots = 1;
        let before = native_action_probe(&core, &execution);
        core.step_battle_execution_by(&mut execution, 100).unwrap();
        let after = native_action_probe(&core, &execution);
        assert_eq!(
            (&before.0, &before.1, &before.2),
            (&after.0, &after.1, &after.2)
        );
        assert_eq!(execution.last_event_time_ms(), 100);
        core.runtime_facts.limits.max_work = 1;
        let prefix = native_action_probe(&core, &execution);
        assert!(matches!(core.step_battle_execution_by(&mut execution, 100),
            Err(GameError::BattleRuntimeFault(f)) if f.code == RuntimeFaultCode::WorkLimit));
        assert_eq!(native_action_probe(&core, &execution), prefix);
        core.runtime_facts.limits.max_work = RuntimeFactLimits::default().max_work;
        core.step_battle_execution_by(&mut execution, 100).unwrap();
        assert_eq!(execution.last_event_time_ms(), 200);
        let outcome = core
            .step_battle_execution_until(&mut execution, 10_000)
            .unwrap();
        assert!(matches!(outcome, sim::BattleStepOutcome::Finished(_)));
        let terminal = native_action_probe(&core, &execution);
        assert_eq!(
            (&before.0, &before.1, &before.2),
            (&terminal.0, &terminal.1, &terminal.2)
        );
    }

    #[test]
    fn native_timestamp_buffer_reuses_range_rows_and_preserves_failure_prefix_and_spawn_order() {
        let skill = single_step_skill(
            "native_buffer",
            SkillTarget::SelfUnit,
            0,
            0,
            DeliveryDef::Instant,
            vec![],
        );
        let (mut core, mut state, caster, _, _) = native_action_fixture(skill, Default::default());
        core.units
            .get_mut(&caster)
            .unwrap()
            .basic_attack
            .defense_tile_range = Some(crate::game::battle::tile_range::TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["@###".into(), "####".into()],
        });
        let mut buffer = None;
        core.runtime_timestamp_reusing(&mut state, &mut buffer, |_, _| Ok(()))
            .unwrap();
        let row_pointer = buffer.as_ref().unwrap().0[&caster]
            .basic_attack
            .defense_tile_range
            .as_ref()
            .unwrap()
            .rows[0]
            .as_ptr();
        let before = native_action_probe(&core, &state);
        core.runtime_timestamp_reusing(&mut state, &mut buffer, |_, _| Ok(()))
            .unwrap();
        assert_eq!(
            core.units[&caster]
                .basic_attack
                .defense_tile_range
                .as_ref()
                .unwrap()
                .rows[0]
                .as_ptr(),
            row_pointer
        );
        assert_eq!(native_action_probe(&core, &state), before);
        let error = core
            .runtime_timestamp_reusing(&mut state, &mut buffer, |candidate, _| {
                let unit = candidate.units.get_mut(&caster).unwrap();
                unit.stats.current_health = 1;
                unit.basic_attack.defense_tile_range = None;
                Err::<(), _>(GameError::InvalidAction)
            })
            .unwrap_err();
        assert!(matches!(error, GameError::InvalidAction));
        assert_eq!(native_action_probe(&core, &state), before);
        assert!(buffer.is_none());
        core.runtime_timestamp_reusing(&mut state, &mut buffer, |_, _| Ok(()))
            .unwrap();
        // A real tracked insertion invalidates the earlier bucket lineage.
        let newcomer = Uuid::from_u128(0xFAC0).into();
        core.runtime_timestamp_reusing(&mut state, &mut buffer, |candidate, _| {
            candidate.insert_runtime_unit(newcomer, runtime_unit(newcomer, Side::Player));
            Ok(())
        })
        .unwrap();
        let native_keys: Vec<_> = core.units.clone().keys().copied().collect();
        core.runtime_timestamp_reusing(&mut state, &mut buffer, |_, _| Ok(()))
            .unwrap();
        assert_eq!(core.units.keys().copied().collect::<Vec<_>>(), native_keys);
        assert!(core.units.contains_key(&newcomer));
    }
}

#[cfg(test)]
mod u08_relic_initial_owner_tests {
    use super::*;
    use crate::game::{
        battle::runtime_control as native, behavior::GameError, data::GameDataBuilder,
        events::combat::RelicBattleRuntimeOwner,
    };

    #[test]
    fn relic_initial_owner_is_preconstructor_exact_and_startup_seed_cannot_restart_it() {
        let scenario = BattleScenario::empty((4, 4));
        let expected = BattleCore::initial_runtime_fact_extent(
            BattleCore::scenario_fact_binding_extent(&scenario),
        )
        .unwrap();
        let data = GameDataBuilder::empty().build_arc();
        let ordinary = BattleCore::new_from_scenario(scenario.clone(), data.clone(), 7);
        assert_eq!(ordinary.retained_runtime_fact_extent().unwrap(), expected);
        for (cap, succeeds) in [(expected + 37, true), (expected + 36, false)] {
            let (result, admissions) = native::observe_retained_admissions(|| {
                native::run(
                    native::RuntimeFactLimits {
                        max_retained_bytes: cap,
                        ..Default::default()
                    },
                    || {
                        native::seed_retained_extent(37)?;
                        let core = BattleCore::new_for_relic_owner(
                            scenario.clone(),
                            data.clone(),
                            None,
                            7,
                            RelicBattleRuntimeOwner,
                        )?;
                        assert_eq!(core.retained_runtime_fact_extent()?, expected);
                        native::seed_retained_extent(expected)?;
                        Ok(core)
                    },
                )
            });
            if succeeds {
                result.unwrap();
                assert_eq!(
                    admissions,
                    [("relic_battle_initial_owner", 37, expected + 37)]
                );
            } else {
                assert!(matches!(
                    result.err().expect("one-short initial owner must refuse"),
                    GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                        code: native::RuntimeFaultCode::RetainedByteLimit,
                        site: "relic_battle_initial_owner",
                        ..
                    })
                ));
                assert!(admissions.is_empty());
            }
        }
        println!(
            "relic preconstructor initial R04 extent={expected}; empty source={} limits={}",
            std::mem::size_of::<crate::game::battle::runtime_facts::RuntimeFactSource>(),
            std::mem::size_of::<native::RuntimeFactLimits>()
        );
    }
}
