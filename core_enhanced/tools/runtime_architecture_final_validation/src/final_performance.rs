use std::{
    fs,
    hint::black_box,
    path::PathBuf,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "allocation-observer")]
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use game_core::game::{
    ability::DeliveryDef,
    battle::{
        core::BattleCore,
        event_log::BattleLogEvent,
        scenario::{
            BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioEvent, ScenarioEventId,
            ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef,
            ScenarioUnitSpawn, TacticalPlan, TerminalCondition, TerminalOutcome,
        },
        tile_range::TileRangePattern,
        types::{
            BattleOutcomeId, BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass,
            BattleVerdict,
        },
    },
    data::{
        abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef},
        GameDataBuilder, ValidatedGameData,
    },
    enums::{RiskLevel, Side, Tier},
    growth::GrowthStack,
    resources::Position,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

mod concurrent_inspection;

const WORKLOAD_VERSION: &str = "crar-final-performance/1";
const LATENCY_SAMPLE_COUNT: usize = 21;
const ALLOCATION_SAMPLE_COUNT: usize = 5;
const EVENT_HEAVY_DURATION_MS: u64 = 5_000;
const SESSION_COUNT: usize = 64;
const LANE_COUNT: usize = 8;
const SESSION_STEP_COUNT: usize = 100;
const SESSION_STEP_MS: u64 = 10;

#[cfg(feature = "allocation-observer")]
struct ObservedSystemAllocator;

#[cfg(feature = "allocation-observer")]
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static DEALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static REALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "allocation-observer")]
fn record_allocation(size: usize) {
    if !TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
        return;
    }
    let current = LIVE_BYTES.fetch_add(size as u64, Ordering::SeqCst) + size as u64;
    PEAK_LIVE_BYTES.fetch_max(current, Ordering::SeqCst);
    ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[cfg(feature = "allocation-observer")]
fn record_deallocation(size: usize) {
    if !TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
        return;
    }
    LIVE_BYTES.fetch_sub(size as u64, Ordering::SeqCst);
    DEALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[cfg(feature = "allocation-observer")]
unsafe impl GlobalAlloc for ObservedSystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record_deallocation(layout.size());
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() && TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
            match new_size.cmp(&layout.size()) {
                std::cmp::Ordering::Greater => {
                    let added = new_size - layout.size();
                    let current =
                        LIVE_BYTES.fetch_add(added as u64, Ordering::SeqCst) + added as u64;
                    PEAK_LIVE_BYTES.fetch_max(current, Ordering::SeqCst);
                }
                std::cmp::Ordering::Less => {
                    LIVE_BYTES.fetch_sub((layout.size() - new_size) as u64, Ordering::SeqCst);
                }
                std::cmp::Ordering::Equal => {}
            }
            REALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        new_ptr
    }
}

#[cfg(feature = "allocation-observer")]
#[global_allocator]
static GLOBAL_ALLOCATOR: ObservedSystemAllocator = ObservedSystemAllocator;

#[cfg(feature = "allocation-observer")]
fn set_allocation_tracking(enabled: bool) {
    TRACK_ALLOCATIONS.store(enabled, Ordering::Relaxed);
}

#[cfg(not(feature = "allocation-observer"))]
fn set_allocation_tracking(_enabled: bool) {}

#[cfg(feature = "allocation-observer")]
fn allocation_tracking_enabled() -> bool {
    TRACK_ALLOCATIONS.load(Ordering::Relaxed)
}

#[cfg(not(feature = "allocation-observer"))]
fn allocation_tracking_enabled() -> bool {
    false
}

#[derive(Debug, Clone, Copy)]
struct AllocationSnapshot {
    live_bytes: u64,
    allocation_count: u64,
    deallocation_count: u64,
    reallocation_count: u64,
}

impl AllocationSnapshot {
    #[cfg(feature = "allocation-observer")]
    fn begin() -> Self {
        let live_bytes = LIVE_BYTES.load(Ordering::SeqCst);
        PEAK_LIVE_BYTES.store(live_bytes, Ordering::SeqCst);
        Self {
            live_bytes,
            allocation_count: ALLOCATION_COUNT.load(Ordering::SeqCst),
            deallocation_count: DEALLOCATION_COUNT.load(Ordering::SeqCst),
            reallocation_count: REALLOCATION_COUNT.load(Ordering::SeqCst),
        }
    }

    #[cfg(not(feature = "allocation-observer"))]
    fn begin() -> Self {
        Self {
            live_bytes: 0,
            allocation_count: 0,
            deallocation_count: 0,
            reallocation_count: 0,
        }
    }

    #[cfg(feature = "allocation-observer")]
    fn finish(self) -> HeapAllocationObservation {
        let end_live_bytes = LIVE_BYTES.load(Ordering::SeqCst);
        let peak_live_bytes = PEAK_LIVE_BYTES.load(Ordering::SeqCst);
        HeapAllocationObservation {
            baseline_live_bytes: self.live_bytes,
            peak_live_bytes,
            peak_delta_bytes: peak_live_bytes.saturating_sub(self.live_bytes),
            end_live_bytes,
            retained_delta_bytes: i128::from(end_live_bytes) - i128::from(self.live_bytes),
            allocation_count: ALLOCATION_COUNT.load(Ordering::SeqCst) - self.allocation_count,
            deallocation_count: DEALLOCATION_COUNT.load(Ordering::SeqCst) - self.deallocation_count,
            reallocation_count: REALLOCATION_COUNT.load(Ordering::SeqCst) - self.reallocation_count,
        }
    }

    #[cfg(not(feature = "allocation-observer"))]
    fn finish(self) -> HeapAllocationObservation {
        HeapAllocationObservation {
            baseline_live_bytes: self.live_bytes,
            peak_live_bytes: self.live_bytes,
            peak_delta_bytes: 0,
            end_live_bytes: self.live_bytes,
            retained_delta_bytes: 0,
            allocation_count: self.allocation_count,
            deallocation_count: self.deallocation_count,
            reallocation_count: self.reallocation_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeapAllocationObservation {
    baseline_live_bytes: u64,
    peak_live_bytes: u64,
    peak_delta_bytes: u64,
    end_live_bytes: u64,
    retained_delta_bytes: i128,
    allocation_count: u64,
    deallocation_count: u64,
    reallocation_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventHeavySample {
    sample: usize,
    elapsed_ns: u128,
    event_count: usize,
    serialized_bytes: usize,
    event_log_sha256: String,
    ability_steps: usize,
    projectile_launches: usize,
    projectile_impacts: usize,
    effect_events: usize,
    heap_allocation: HeapAllocationObservation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConcurrentSample {
    sample: usize,
    elapsed_ns: u128,
    session_count: usize,
    lane_count: usize,
    step_count_per_session: usize,
    event_count: usize,
    serialized_bytes: usize,
    combined_event_log_sha256: String,
    heap_allocation: HeapAllocationObservation,
}

#[derive(Debug, Clone, Serialize)]
struct Receipt {
    schema_version: u32,
    workload_version: &'static str,
    generated_at_unix_ms: u128,
    tool_source_sha256: String,
    workload_input_sha256: String,
    allocation_observer_enabled: bool,
    fixture_conversion_elapsed_ns: Vec<u128>,
    kernel_input: &'static str,
    latency_gate_applicable: bool,
    allocation_gate_applicable: bool,
    baseline: BaselineIdentity,
    environment: Environment,
    event_heavy: FinalWorkloadReceipt<EventHeavySample>,
    concurrent_sessions: FinalWorkloadReceipt<ConcurrentSample>,
}

#[derive(Debug, Clone, Serialize)]
struct BaselineIdentity {
    receipt_path: &'static str,
    receipt_sha256: String,
    schema_version: u32,
    workload_version: String,
    tool_source_sha256: String,
    workload_input_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
struct Environment {
    os: &'static str,
    arch: &'static str,
    rust_profile: &'static str,
    logical_cpus: usize,
    affinity_policy: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkloadReceipt<T> {
    repeatable: bool,
    raw_samples: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
struct BaselineReceipt {
    schema_version: u32,
    workload_version: String,
    tool_source_sha256: String,
    workload_input_sha256: String,
    event_heavy: WorkloadReceipt<EventHeavySample>,
    concurrent_sessions: WorkloadReceipt<ConcurrentSample>,
}

#[derive(Debug, Clone, Serialize)]
struct FinalWorkloadReceipt<T> {
    repeatable: bool,
    latency_raw_samples: Vec<T>,
    allocation_raw_samples: Vec<T>,
    nearest_rank_p95_ns: u128,
    absolute_cap_ns: u128,
    absolute_cap_pass: bool,
    one_sided_exact_regression_p: f64,
    statistically_supported_regression: bool,
    deterministic_baseline_parity: bool,
    allocation_retained_peak_guard_pass: bool,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(target_os = "linux")]
fn pin_current_thread(cpu: usize) {
    let mut set = unsafe { std::mem::zeroed::<libc::cpu_set_t>() };
    unsafe {
        libc::CPU_ZERO(&mut set);
        libc::CPU_SET(cpu, &mut set);
        assert_eq!(
            libc::pthread_setaffinity_np(
                libc::pthread_self(),
                std::mem::size_of::<libc::cpu_set_t>(),
                &set,
            ),
            0,
            "pin benchmark thread to logical CPU {cpu}"
        );
    }
}

#[cfg(not(target_os = "linux"))]
fn pin_current_thread(_cpu: usize) {}

fn broad_range() -> TileRangePattern {
    TileRangePattern {
        include_anchor_tile: false,
        rows: vec![
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXX@XXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
        ],
    }
}

fn dummy_metadata() -> AbnormalityMetadata {
    AbnormalityMetadata {
        id: "crar_baseline_dummy".to_string(),
        uuid: Uuid::from_u128(0xD00D_0000_0000_0000_0000_0000_0000_1001),
        name: "CRAR Baseline Dummy".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 20_000,
        attack: 1,
        defense: 0,
        magic_resist: 0,
        threat_class: BattleUnitThreatClass::Elite,
        omen_chain_id: None,
        response_complete_skill_fragment_id: Some(
            game_core::game::data::skill_fragment_data::SkillFragmentId::from(
                "starter_basic_attack_enhancement",
            ),
        ),
        movement: MovementDef {
            speed_units_per_ms: 0,
            radius_units: 350_000,
        },
        basic_attack: BasicAttackDef {
            defense_tile_range: Some(broad_range()),
            interval_ms: 100,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
            ..BasicAttackDef::default()
        },
        resonance: Default::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    }
}

fn preservation_data() -> (ValidatedGameData, Vec<u128>) {
    let base = ValidatedGameData::load_live_embedded().expect("load live embedded data");
    let mut abnormalities = base.abnormality_data.items.clone();
    let little_red = abnormalities
        .iter_mut()
        .find(|metadata| metadata.id == "f-01-57_little_red")
        .expect("Little Red live metadata");
    little_red.basic_attack.defense_tile_range = Some(broad_range());
    little_red.resonance.start = little_red.resonance.max.saturating_sub(10);
    abnormalities.push(dummy_metadata());

    let raw = GameDataBuilder::from_validated(&base)
        .with_abnormalities(abnormalities)
        .with_corroded_employee_data(Arc::clone(&base.corroded_employee_data))
        .with_corroded_wave_data(Arc::clone(&base.corroded_wave_data))
        .with_consumable_data(Arc::clone(&base.consumable_data))
        .with_equipment_data(Arc::clone(&base.equipment_data))
        .with_shop_data(Arc::clone(&base.shop_data))
        .with_reward_data(Arc::clone(&base.reward_data))
        .with_event_data(Arc::clone(&base.event_data))
        .with_pve_data(Arc::clone(&base.pve_data))
        .with_boss_runtime_data(Arc::clone(&base.boss_runtime_data))
        .with_boss_omen_data(Arc::clone(&base.boss_omen_data))
        .with_buff_data(Arc::clone(&base.buff_data))
        .with_skill_data(Arc::clone(&base.skill_data))
        .with_skill_fragment_data(Arc::clone(&base.skill_fragment_data))
        .build_arc();
    // R02 makes a raw fixture conversion a full generation publication. Record
    // that cost separately, then share one real token at the runtime boundary.
    let mut conversion_ns = Vec::with_capacity(5);
    let mut publication = None;
    for _ in 0..5 {
        let started = Instant::now();
        let token = ValidatedGameData::from(Arc::clone(&raw));
        conversion_ns.push(started.elapsed().as_nanos());
        if let Some(previous) = &publication {
            let previous: &ValidatedGameData = previous;
            assert_eq!(token.identity(), previous.identity());
        }
        publication = Some(token);
    }
    (publication.unwrap(), conversion_ns)
}

fn spawn(
    unit_ref: impl Into<String>,
    owned_uuid: Uuid,
    base_uuid: Uuid,
    side: Side,
    position: Position,
    instance_salt: u32,
) -> ScenarioUnitSpawn {
    ScenarioUnitSpawn {
        unit_ref: ScenarioUnitRef::new(unit_ref),
        side,
        draft: BattleUnitDraft {
            owned_uuid,
            source: BattleUnitSource::Abnormality { base_uuid },
            threat_class: BattleUnitThreatClass::Elite,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        },
        position,
        instance_salt,
    }
}

fn scenario(groups: Vec<ScenarioSpawnGroup>) -> BattleScenario {
    let events = groups
        .iter()
        .enumerate()
        .map(|(index, group)| ScenarioEvent {
            id: ScenarioEventId::new(format!("spawn_{index}")),
            trigger: ScenarioTrigger::AtBattleStart,
            action: ScenarioAction::SpawnGroup {
                group_id: group.id.clone(),
            },
        })
        .collect();
    BattleScenario {
        battlefield: BattleFieldSpec {
            object_placement: None,
            width: 9,
            height: 9,
            valid_tiles: Vec::new(),
            obstacles: Vec::new(),
        },
        groups,
        events,
        terminal_outcomes: vec![TerminalOutcome {
            id: BattleOutcomeId::new("baseline_timeout"),
            verdict: BattleVerdict::Victory,
            condition: TerminalCondition::TimeReached { time_ms: 60_000 },
            priority: 0,
        }],
        tactical_plan: TacticalPlan::default(),
        boss_runtime: None,
    }
}

fn event_heavy_core(data: ValidatedGameData, seed: u64) -> BattleCore {
    let red = data
        .abnormality_data
        .items
        .iter()
        .find(|metadata| metadata.id == "f-01-57_little_red")
        .expect("Little Red metadata");
    let dummy = data
        .abnormality_data
        .items
        .iter()
        .find(|metadata| metadata.id == "crar_baseline_dummy")
        .expect("dummy metadata");
    let player = ScenarioSpawnGroup {
        id: ScenarioGroupId::new("event_heavy_player"),
        side: Side::Player,
        required_for_victory: false,
        enemy_movement_plan: None,
        spawns: (0..4)
            .map(|index| {
                spawn(
                    format!("little_red_{index}"),
                    Uuid::from_u128(0xA100 + index as u128),
                    red.uuid,
                    Side::Player,
                    Position::new(1, 1 + index as i32),
                    index,
                )
            })
            .collect(),
    };
    let opponent = ScenarioSpawnGroup {
        id: ScenarioGroupId::new("event_heavy_opponent"),
        side: Side::Opponent,
        required_for_victory: true,
        enemy_movement_plan: None,
        spawns: (0..12)
            .map(|index| {
                spawn(
                    format!("event_dummy_{index}"),
                    Uuid::from_u128(0xB100 + index as u128),
                    dummy.uuid,
                    Side::Opponent,
                    Position::new(2 + (index % 3) as i32, 1 + (index / 3) as i32),
                    100 + index,
                )
            })
            .collect(),
    };
    BattleCore::new_from_scenario(scenario(vec![player, opponent]), data, seed)
}

fn active_session_core(data: ValidatedGameData, session: usize) -> BattleCore {
    let dummy = data
        .abnormality_data
        .items
        .iter()
        .find(|metadata| metadata.id == "crar_baseline_dummy")
        .expect("dummy metadata");
    let group = |id: &str, side: Side, base: u128, x: i32| ScenarioSpawnGroup {
        id: ScenarioGroupId::new(id),
        side,
        required_for_victory: side == Side::Opponent,
        enemy_movement_plan: None,
        spawns: (0..8)
            .map(|index| {
                spawn(
                    format!("{id}_{index}"),
                    Uuid::from_u128(base + (session * 32 + index) as u128),
                    dummy.uuid,
                    side,
                    Position::new(x, index as i32),
                    (session * 32 + index) as u32,
                )
            })
            .collect(),
    };
    BattleCore::new_from_scenario(
        scenario(vec![
            group("session_player", Side::Player, 0xC000_0000, 3),
            group("session_opponent", Side::Opponent, 0xD000_0000, 4),
        ]),
        data,
        0xC0DE_0000 + session as u64,
    )
}

fn event_heavy_sample(data: ValidatedGameData, sample: usize) -> EventHeavySample {
    let allocation_snapshot = AllocationSnapshot::begin();
    let mut core = event_heavy_core(data, 0xA11C_E000);
    let started = Instant::now();
    let mut execution = core
        .start_battle_execution()
        .expect("start event-heavy battle");
    black_box(
        core.step_battle_execution_by(&mut execution, EVENT_HEAVY_DURATION_MS)
            .expect("step event-heavy battle"),
    );
    let elapsed_ns = started.elapsed().as_nanos();
    let serialized = serde_json::to_vec(&core.event_log).expect("serialize event-heavy log");
    let mut ability_steps = 0;
    let mut projectile_launches = 0;
    let mut projectile_impacts = 0;
    let mut effect_events = 0;
    for entry in &core.event_log.entries {
        match &entry.event {
            BattleLogEvent::AbilityStepTriggered { .. } => ability_steps += 1,
            BattleLogEvent::SkillProjectileLaunched { .. } => projectile_launches += 1,
            BattleLogEvent::SkillProjectileImpacted { .. } => projectile_impacts += 1,
            BattleLogEvent::AbilityCast { .. }
            | BattleLogEvent::AttackResolve { .. }
            | BattleLogEvent::BuffApplied { .. }
            | BattleLogEvent::BuffTick { .. } => effect_events += 1,
            _ => {}
        }
    }
    assert!(
        ability_steps > 0,
        "event-heavy workload must trigger ability steps"
    );
    assert!(
        projectile_launches > 0 && projectile_impacts > 0,
        "event-heavy workload must exercise projectile launch/impact"
    );
    assert!(
        effect_events > 0,
        "event-heavy workload must exercise effects"
    );
    let heap_allocation = allocation_snapshot.finish();
    assert!(
        !allocation_tracking_enabled()
            || (heap_allocation.allocation_count > 0 && heap_allocation.peak_delta_bytes > 0),
        "event-heavy workload must produce a direct allocation/heap observation"
    );
    EventHeavySample {
        sample,
        elapsed_ns,
        event_count: core.event_log.entries.len(),
        serialized_bytes: serialized.len(),
        event_log_sha256: sha256(&serialized),
        ability_steps,
        projectile_launches,
        projectile_impacts,
        effect_events,
        heap_allocation,
    }
}

fn concurrent_sample(data: ValidatedGameData, sample: usize) -> ConcurrentSample {
    let allocation_snapshot = AllocationSnapshot::begin();
    let started = Instant::now();
    let lane_results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for lane in 0..LANE_COUNT {
            let lane_data = data.clone();
            handles.push(scope.spawn(move || {
                pin_current_thread(lane * 2);
                let mut sessions = (0..SESSION_COUNT / LANE_COUNT)
                    .map(|slot| {
                        let session = lane * (SESSION_COUNT / LANE_COUNT) + slot;
                        let mut core = active_session_core(lane_data.clone(), session);
                        let execution = core
                            .start_battle_execution()
                            .expect("start concurrent session battle");
                        (session, core, execution)
                    })
                    .collect::<Vec<_>>();
                for _ in 0..SESSION_STEP_COUNT {
                    for (_, core, execution) in &mut sessions {
                        black_box(
                            core.step_battle_execution_by(execution, SESSION_STEP_MS)
                                .expect("step concurrent session"),
                        );
                    }
                }
                sessions
                    .into_iter()
                    .map(|(session, core, _)| {
                        let bytes = serde_json::to_vec(&core.event_log)
                            .expect("serialize concurrent event log");
                        (session, core.event_log.entries.len(), bytes)
                    })
                    .collect::<Vec<_>>()
            }));
        }
        let mut results = handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("concurrent lane"))
            .collect::<Vec<_>>();
        results.sort_by_key(|(session, _, _)| *session);
        results
    });
    let elapsed_ns = started.elapsed().as_nanos();
    let event_count = lane_results.iter().map(|(_, count, _)| count).sum();
    let serialized_bytes = lane_results.iter().map(|(_, _, bytes)| bytes.len()).sum();
    let mut digest = Sha256::new();
    for (session, _, bytes) in &lane_results {
        digest.update(session.to_le_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    let combined_event_log_sha256 = format!("{:x}", digest.finalize());
    let heap_allocation = allocation_snapshot.finish();
    assert!(
        !allocation_tracking_enabled()
            || (heap_allocation.allocation_count > 0 && heap_allocation.peak_delta_bytes > 0),
        "64-session workload must produce a direct allocation/heap observation"
    );
    ConcurrentSample {
        sample,
        elapsed_ns,
        session_count: SESSION_COUNT,
        lane_count: LANE_COUNT,
        step_count_per_session: SESSION_STEP_COUNT,
        event_count,
        serialized_bytes,
        combined_event_log_sha256,
        heap_allocation,
    }
}

fn all_event_samples_repeat(samples: &[EventHeavySample]) -> bool {
    samples.windows(2).all(|pair| {
        pair[0].event_count == pair[1].event_count
            && pair[0].serialized_bytes == pair[1].serialized_bytes
            && pair[0].event_log_sha256 == pair[1].event_log_sha256
            && pair[0].ability_steps == pair[1].ability_steps
            && pair[0].projectile_launches == pair[1].projectile_launches
            && pair[0].projectile_impacts == pair[1].projectile_impacts
            && pair[0].effect_events == pair[1].effect_events
    })
}

fn all_concurrent_samples_repeat(samples: &[ConcurrentSample]) -> bool {
    samples.windows(2).all(|pair| {
        pair[0].session_count == pair[1].session_count
            && pair[0].lane_count == pair[1].lane_count
            && pair[0].step_count_per_session == pair[1].step_count_per_session
            && pair[0].event_count == pair[1].event_count
            && pair[0].serialized_bytes == pair[1].serialized_bytes
            && pair[0].combined_event_log_sha256 == pair[1].combined_event_log_sha256
    })
}

fn nearest_rank_p95(samples: impl Iterator<Item = u128>) -> u128 {
    let mut values = samples.collect::<Vec<_>>();
    values.sort_unstable();
    let rank = (95 * values.len()).div_ceil(100);
    values[rank.saturating_sub(1)]
}

fn pairwise_u_twice(target: &[u128], baseline: &[u128]) -> u64 {
    target
        .iter()
        .flat_map(|target| baseline.iter().map(move |baseline| target.cmp(baseline)))
        .map(|ordering| match ordering {
            std::cmp::Ordering::Greater => 2,
            std::cmp::Ordering::Equal => 1,
            std::cmp::Ordering::Less => 0,
        })
        .sum()
}

fn one_sided_exact_regression_p(baseline: &[u128], target: &[u128]) -> f64 {
    let mut combined = Vec::with_capacity(baseline.len() + target.len());
    combined.extend_from_slice(baseline);
    combined.extend_from_slice(target);
    assert!(combined.len() < usize::BITS as usize);
    let observed = pairwise_u_twice(target, baseline);
    let mut at_least_as_extreme = 0_u64;
    let mut permutations = 0_u64;
    for mask in 0_usize..(1_usize << combined.len()) {
        if mask.count_ones() as usize != target.len() {
            continue;
        }
        let mut candidate_target = Vec::with_capacity(target.len());
        let mut candidate_baseline = Vec::with_capacity(baseline.len());
        for (index, value) in combined.iter().copied().enumerate() {
            if mask & (1 << index) == 0 {
                candidate_baseline.push(value);
            } else {
                candidate_target.push(value);
            }
        }
        if pairwise_u_twice(&candidate_target, &candidate_baseline) >= observed {
            at_least_as_extreme += 1;
        }
        permutations += 1;
    }
    at_least_as_extreme as f64 / permutations as f64
}

fn event_parity(samples: &[EventHeavySample], baseline: &EventHeavySample) -> bool {
    samples.iter().all(|sample| {
        sample.event_count == baseline.event_count
            && sample.serialized_bytes == baseline.serialized_bytes
            && sample.event_log_sha256 == baseline.event_log_sha256
            && sample.ability_steps == baseline.ability_steps
            && sample.projectile_launches == baseline.projectile_launches
            && sample.projectile_impacts == baseline.projectile_impacts
            && sample.effect_events == baseline.effect_events
    })
}

fn concurrent_parity(samples: &[ConcurrentSample], baseline: &ConcurrentSample) -> bool {
    samples.iter().all(|sample| {
        sample.session_count == baseline.session_count
            && sample.lane_count == baseline.lane_count
            && sample.step_count_per_session == baseline.step_count_per_session
            && sample.event_count == baseline.event_count
            && sample.serialized_bytes == baseline.serialized_bytes
            && sample.combined_event_log_sha256 == baseline.combined_event_log_sha256
    })
}

fn allocation_guard<T>(
    samples: &[T],
    baseline: &[T],
    allocation: impl Fn(&T) -> &HeapAllocationObservation,
) -> bool {
    let baseline_allocation_operations = baseline
        .iter()
        .map(|sample| allocation(sample).allocation_count + allocation(sample).reallocation_count)
        .max()
        .unwrap();
    let baseline_peak = baseline
        .iter()
        .map(|sample| allocation(sample).peak_delta_bytes)
        .max()
        .unwrap();
    let baseline_retained = baseline
        .iter()
        .map(|sample| allocation(sample).retained_delta_bytes)
        .max()
        .unwrap();
    samples.iter().all(|sample| {
        let final_allocation = allocation(sample);
        final_allocation.allocation_count + final_allocation.reallocation_count
            <= baseline_allocation_operations
            && final_allocation.peak_delta_bytes <= baseline_peak
            && final_allocation.retained_delta_bytes <= baseline_retained
    })
}

// Diagnostic measurement is deliberately separate from the frozen P01 guard.
// It produces current raw samples and cannot certify baseline parity or pass.
fn measure_current(output: PathBuf) {
    let logical_cpus = std::thread::available_parallelism().map(usize::from).unwrap_or(1);
    assert!(logical_cpus >= 31, "same affinity workload requires 31 CPUs");
    let (data, fixture_conversion_elapsed_ns) = preservation_data();
    pin_current_thread(30);
    set_allocation_tracking(false);
    let _ = event_heavy_sample(data.clone(), 0);
    let _ = concurrent_sample(data.clone(), 0);
    let count = if cfg!(feature = "allocation-observer") {
        ALLOCATION_SAMPLE_COUNT
    } else {
        LATENCY_SAMPLE_COUNT
    };
    set_allocation_tracking(cfg!(feature = "allocation-observer"));
    let event = (1..=count).map(|i| event_heavy_sample(data.clone(), i)).collect::<Vec<_>>();
    let concurrent = (1..=count).map(|i| concurrent_sample(data.clone(), i)).collect::<Vec<_>>();
    set_allocation_tracking(false);
    let event_repeatable = all_event_samples_repeat(&event);
    let concurrent_repeatable = all_concurrent_samples_repeat(&concurrent);
    let receipt = serde_json::json!({
        "schema_version": 1, "mode": "current_measurement_only",
        "workload_version": WORKLOAD_VERSION, "baseline_comparison_applicable": false,
        "performance_pass_claimed": false,
        "allocation_observer_enabled": cfg!(feature = "allocation-observer"),
        "latency_measurement_applicable": !cfg!(feature = "allocation-observer"),
        "tool_source_sha256": sha256(include_bytes!("final_performance.rs")),
        "fixture_conversion_elapsed_ns": fixture_conversion_elapsed_ns,
        "environment": {"profile": if cfg!(debug_assertions) {"debug"} else {"release"},
            "os": std::env::consts::OS, "arch": std::env::consts::ARCH,
            "logical_cpus": logical_cpus, "affinity": "coordinator=30; lanes=0,2,4,6,8,10,12,14"},
        "event_heavy": {"duration_ms": EVENT_HEAVY_DURATION_MS, "repeatable": event_repeatable,
            "p95_ns": nearest_rank_p95(event.iter().map(|s| s.elapsed_ns)), "raw_samples": event},
        "concurrent_sessions": {"repeatable": concurrent_repeatable, "step_ms": SESSION_STEP_MS,
            "p95_ns": nearest_rank_p95(concurrent.iter().map(|s| s.elapsed_ns)), "raw_samples": concurrent}
    });
    fs::write(output, serde_json::to_vec_pretty(&receipt).unwrap()).expect("write current measurement");
    assert!(event_repeatable && concurrent_repeatable, "current outputs must repeat");
}

fn main() {
    let mut args = std::env::args().skip(1);
    let baseline_path = args
        .next()
        .map(PathBuf::from)
        .expect("usage: final_performance <p01-baseline.json> <output-receipt.json>");
    let output = args
        .next()
        .map(PathBuf::from)
        .expect("usage: final_performance <p01-baseline.json> <output-receipt.json>");
    assert!(
        args.next().is_none(),
        "unexpected final_performance argument"
    );
    if baseline_path == std::path::Path::new("--measure-current") {
        measure_current(output);
        return;
    }
    if baseline_path == std::path::Path::new("--inspect-concurrent") {
        concurrent_inspection::run(output);
        return;
    }
    let baseline_bytes = fs::read(&baseline_path).expect("read accepted P01 baseline receipt");
    let baseline: BaselineReceipt =
        serde_json::from_slice(&baseline_bytes).expect("decode accepted P01 baseline receipt");
    assert_eq!(baseline.schema_version, 2);
    assert_eq!(baseline.workload_version, "crar-preservation-baseline/2");
    assert_eq!(
        baseline.tool_source_sha256,
        sha256(include_bytes!(
            "../../runtime_architecture_preservation/src/main.rs"
        ))
    );
    assert_eq!(baseline.event_heavy.raw_samples.len(), 5);
    assert_eq!(baseline.concurrent_sessions.raw_samples.len(), 5);
    let logical_cpus = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    assert!(logical_cpus >= 31, "P08 affinity contract requires 31 CPUs");
    let (data, fixture_conversion_elapsed_ns) = preservation_data();
    pin_current_thread(30);

    set_allocation_tracking(false);
    let _warmup_event = event_heavy_sample(data.clone(), 0);
    let _warmup_concurrent = concurrent_sample(data.clone(), 0);
    let event_latency_samples = (1..=LATENCY_SAMPLE_COUNT)
        .map(|sample| event_heavy_sample(data.clone(), sample))
        .collect::<Vec<_>>();
    let concurrent_latency_samples = (1..=LATENCY_SAMPLE_COUNT)
        .map(|sample| concurrent_sample(data.clone(), sample))
        .collect::<Vec<_>>();

    set_allocation_tracking(true);
    let mut event_allocation_samples = Vec::with_capacity(ALLOCATION_SAMPLE_COUNT);
    let mut concurrent_allocation_samples = Vec::with_capacity(ALLOCATION_SAMPLE_COUNT);
    let allocation_sample_count = if cfg!(feature = "allocation-observer") {
        ALLOCATION_SAMPLE_COUNT
    } else {
        0
    };
    event_allocation_samples.extend(
        (1..=allocation_sample_count).map(|sample| event_heavy_sample(data.clone(), sample)),
    );
    concurrent_allocation_samples.extend(
        (1..=allocation_sample_count).map(|sample| concurrent_sample(data.clone(), sample)),
    );
    set_allocation_tracking(false);

    let event_repeatable = all_event_samples_repeat(&event_latency_samples)
        && all_event_samples_repeat(&event_allocation_samples);
    let concurrent_repeatable = all_concurrent_samples_repeat(&concurrent_latency_samples)
        && all_concurrent_samples_repeat(&concurrent_allocation_samples);
    assert!(
        event_repeatable,
        "event-heavy deterministic outputs diverged"
    );
    assert!(
        concurrent_repeatable,
        "64-session/8-way deterministic outputs diverged"
    );

    let event_parity = event_parity(&event_latency_samples, &baseline.event_heavy.raw_samples[0])
        && event_parity(
            &event_allocation_samples,
            &baseline.event_heavy.raw_samples[0],
        );
    let concurrent_parity = concurrent_parity(
        &concurrent_latency_samples,
        &baseline.concurrent_sessions.raw_samples[0],
    ) && concurrent_parity(
        &concurrent_allocation_samples,
        &baseline.concurrent_sessions.raw_samples[0],
    );
    assert!(
        event_parity && concurrent_parity,
        "P01 deterministic parity failed"
    );

    let event_p95 = nearest_rank_p95(event_latency_samples.iter().map(|sample| sample.elapsed_ns));
    let concurrent_p95 = nearest_rank_p95(
        concurrent_latency_samples
            .iter()
            .map(|sample| sample.elapsed_ns),
    );
    let event_regression_p = one_sided_exact_regression_p(
        &baseline
            .event_heavy
            .raw_samples
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>(),
        &event_latency_samples
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>(),
    );
    let concurrent_regression_p = one_sided_exact_regression_p(
        &baseline
            .concurrent_sessions
            .raw_samples
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>(),
        &concurrent_latency_samples
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>(),
    );
    let event_allocation_guard = allocation_guard(
        &event_allocation_samples,
        &baseline.event_heavy.raw_samples,
        |sample| &sample.heap_allocation,
    );
    let concurrent_allocation_guard = allocation_guard(
        &concurrent_allocation_samples,
        &baseline.concurrent_sessions.raw_samples,
        |sample| &sample.heap_allocation,
    );

    let input = serde_json::json!({
        "workload_version": WORKLOAD_VERSION,
        "baseline_workload_version": baseline.workload_version,
        "latency_sample_count": LATENCY_SAMPLE_COUNT,
        "allocation_sample_count": ALLOCATION_SAMPLE_COUNT,
        "event_heavy_duration_ms": EVENT_HEAVY_DURATION_MS,
        "session_count": SESSION_COUNT,
        "lane_count": LANE_COUNT,
        "session_step_count": SESSION_STEP_COUNT,
        "session_step_ms": SESSION_STEP_MS,
        "little_red_id": "f-01-57_little_red",
        "seed": "fixed per workload",
    });
    let receipt = Receipt {
        schema_version: 1,
        workload_version: WORKLOAD_VERSION,
        generated_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_millis(),
        tool_source_sha256: sha256(include_bytes!("final_performance.rs")),
        workload_input_sha256: sha256(&serde_json::to_vec(&input).unwrap()),
        allocation_observer_enabled: cfg!(feature = "allocation-observer"),
        fixture_conversion_elapsed_ns,
        kernel_input: "one validated generation shared across all cores; raw fixture conversion measured separately",
        latency_gate_applicable: !cfg!(feature = "allocation-observer"),
        allocation_gate_applicable: cfg!(feature = "allocation-observer"),
        baseline: BaselineIdentity {
            receipt_path: "evidence/P01_PERFORMANCE_BASELINE.json",
            receipt_sha256: sha256(&baseline_bytes),
            schema_version: baseline.schema_version,
            workload_version: baseline.workload_version,
            tool_source_sha256: baseline.tool_source_sha256,
            workload_input_sha256: baseline.workload_input_sha256,
        },
        environment: Environment {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            rust_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            logical_cpus,
            affinity_policy: "linux coordinator=cpu30; lanes=cpu0,2,4,6,8,10,12,14",
        },
        event_heavy: FinalWorkloadReceipt {
            repeatable: event_repeatable,
            latency_raw_samples: event_latency_samples,
            allocation_raw_samples: event_allocation_samples,
            nearest_rank_p95_ns: event_p95,
            absolute_cap_ns: 25_000_000,
            absolute_cap_pass: event_p95 < 25_000_000,
            one_sided_exact_regression_p: event_regression_p,
            statistically_supported_regression: event_regression_p < 0.05,
            deterministic_baseline_parity: event_parity,
            allocation_retained_peak_guard_pass: event_allocation_guard,
        },
        concurrent_sessions: FinalWorkloadReceipt {
            repeatable: concurrent_repeatable,
            latency_raw_samples: concurrent_latency_samples,
            allocation_raw_samples: concurrent_allocation_samples,
            nearest_rank_p95_ns: concurrent_p95,
            absolute_cap_ns: 10_000_000,
            absolute_cap_pass: concurrent_p95 < 10_000_000,
            one_sided_exact_regression_p: concurrent_regression_p,
            statistically_supported_regression: concurrent_regression_p < 0.05,
            deterministic_baseline_parity: concurrent_parity,
            allocation_retained_peak_guard_pass: concurrent_allocation_guard,
        },
    };
    let parent = output.parent().expect("output must have a parent");
    fs::create_dir_all(parent).expect("create receipt directory");
    fs::write(&output, serde_json::to_vec_pretty(&receipt).unwrap())
        .expect("write final performance receipt");
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    if receipt.latency_gate_applicable {
        assert!(receipt.event_heavy.absolute_cap_pass);
        assert!(receipt.concurrent_sessions.absolute_cap_pass);
        assert!(!receipt.event_heavy.statistically_supported_regression);
        assert!(
            !receipt
                .concurrent_sessions
                .statistically_supported_regression
        );
    }
    if receipt.allocation_gate_applicable {
        assert!(receipt.event_heavy.allocation_retained_peak_guard_pass);
        assert!(
            receipt
                .concurrent_sessions
                .allocation_retained_peak_guard_pass
        );
    }
}
