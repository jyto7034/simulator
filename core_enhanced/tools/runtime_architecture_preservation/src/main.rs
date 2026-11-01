use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs,
    hint::black_box,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
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
        GameDataBase, GameDataBuilder, ValidatedGameData,
    },
    enums::{RiskLevel, Side, Tier},
    growth::GrowthStack,
    resources::Position,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const WORKLOAD_VERSION: &str = "crar-preservation-baseline/2";
const SAMPLE_COUNT: usize = 5;
const EVENT_HEAVY_DURATION_MS: u64 = 5_000;
const SESSION_COUNT: usize = 64;
const LANE_COUNT: usize = 8;
const SESSION_STEP_COUNT: usize = 100;
const SESSION_STEP_MS: u64 = 10;

struct ObservedSystemAllocator;

static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static DEALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static REALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);

fn record_allocation(size: usize) {
    let current = LIVE_BYTES.fetch_add(size as u64, Ordering::SeqCst) + size as u64;
    PEAK_LIVE_BYTES.fetch_max(current, Ordering::SeqCst);
    ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
}

fn record_deallocation(size: usize) {
    LIVE_BYTES.fetch_sub(size as u64, Ordering::SeqCst);
    DEALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
}

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
        if !new_ptr.is_null() {
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

#[global_allocator]
static GLOBAL_ALLOCATOR: ObservedSystemAllocator = ObservedSystemAllocator;

#[derive(Debug, Clone, Copy)]
struct AllocationSnapshot {
    live_bytes: u64,
    allocation_count: u64,
    deallocation_count: u64,
    reallocation_count: u64,
}

impl AllocationSnapshot {
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
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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
    environment: Environment,
    event_heavy: WorkloadReceipt<EventHeavySample>,
    concurrent_sessions: WorkloadReceipt<ConcurrentSample>,
}

#[derive(Debug, Clone, Serialize)]
struct Environment {
    os: &'static str,
    arch: &'static str,
    rust_profile: &'static str,
    logical_cpus: usize,
}

#[derive(Debug, Clone, Serialize)]
struct WorkloadReceipt<T> {
    repeatable: bool,
    raw_samples: Vec<T>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

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

fn preservation_data() -> Arc<GameDataBase> {
    let base = ValidatedGameData::load_live_embedded().expect("load live embedded data");
    let mut abnormalities = base.abnormality_data.items.clone();
    let little_red = abnormalities
        .iter_mut()
        .find(|metadata| metadata.id == "f-01-57_little_red")
        .expect("Little Red live metadata");
    little_red.basic_attack.defense_tile_range = Some(broad_range());
    little_red.resonance.start = little_red.resonance.max.saturating_sub(10);
    abnormalities.push(dummy_metadata());

    GameDataBuilder::from_validated(&base)
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
        .build_arc()
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

fn event_heavy_core(data: Arc<GameDataBase>, seed: u64) -> BattleCore {
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

fn active_session_core(data: Arc<GameDataBase>, session: usize) -> BattleCore {
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

fn event_heavy_sample(data: Arc<GameDataBase>, sample: usize) -> EventHeavySample {
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
        heap_allocation.allocation_count > 0 && heap_allocation.peak_delta_bytes > 0,
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

fn concurrent_sample(data: Arc<GameDataBase>, sample: usize) -> ConcurrentSample {
    let allocation_snapshot = AllocationSnapshot::begin();
    let started = Instant::now();
    let lane_results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for lane in 0..LANE_COUNT {
            let lane_data = Arc::clone(&data);
            handles.push(scope.spawn(move || {
                let mut sessions = (0..SESSION_COUNT / LANE_COUNT)
                    .map(|slot| {
                        let session = lane * (SESSION_COUNT / LANE_COUNT) + slot;
                        let mut core = active_session_core(Arc::clone(&lane_data), session);
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
        heap_allocation.allocation_count > 0 && heap_allocation.peak_delta_bytes > 0,
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

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: preservation_baseline <output-receipt.json>");
    let data = preservation_data();

    let _warmup_event = event_heavy_sample(Arc::clone(&data), 0);
    let _warmup_concurrent = concurrent_sample(Arc::clone(&data), 0);
    let event_samples = (1..=SAMPLE_COUNT)
        .map(|sample| event_heavy_sample(Arc::clone(&data), sample))
        .collect::<Vec<_>>();
    let concurrent_samples = (1..=SAMPLE_COUNT)
        .map(|sample| concurrent_sample(Arc::clone(&data), sample))
        .collect::<Vec<_>>();
    let event_repeatable = all_event_samples_repeat(&event_samples);
    let concurrent_repeatable = all_concurrent_samples_repeat(&concurrent_samples);
    assert!(
        event_repeatable,
        "event-heavy deterministic outputs diverged"
    );
    assert!(
        concurrent_repeatable,
        "64-session/8-way deterministic outputs diverged"
    );

    let input = serde_json::json!({
        "workload_version": WORKLOAD_VERSION,
        "sample_count": SAMPLE_COUNT,
        "event_heavy_duration_ms": EVENT_HEAVY_DURATION_MS,
        "session_count": SESSION_COUNT,
        "lane_count": LANE_COUNT,
        "session_step_count": SESSION_STEP_COUNT,
        "session_step_ms": SESSION_STEP_MS,
        "little_red_id": "f-01-57_little_red",
        "seed": "fixed per workload",
    });
    let receipt = Receipt {
        schema_version: 2,
        workload_version: WORKLOAD_VERSION,
        generated_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_millis(),
        tool_source_sha256: sha256(include_bytes!("main.rs")),
        workload_input_sha256: sha256(&serde_json::to_vec(&input).unwrap()),
        environment: Environment {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            rust_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            logical_cpus: std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
        },
        event_heavy: WorkloadReceipt {
            repeatable: event_repeatable,
            raw_samples: event_samples,
        },
        concurrent_sessions: WorkloadReceipt {
            repeatable: concurrent_repeatable,
            raw_samples: concurrent_samples,
        },
    };
    let parent = output.parent().expect("output must have a parent");
    fs::create_dir_all(parent).expect("create receipt directory");
    fs::write(&output, serde_json::to_vec_pretty(&receipt).unwrap())
        .expect("write baseline receipt");
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
}
