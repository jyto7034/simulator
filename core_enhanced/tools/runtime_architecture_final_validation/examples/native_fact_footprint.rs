//! Allocation-size diagnosis on the unchanged concurrent kernel fixture.
//! This is a separate measurement; final_performance and its assertions stay intact.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
static ACTIVE: AtomicBool = AtomicBool::new(false);
static COUNTS: [AtomicI64; 262144] = [const { AtomicI64::new(0) }; 262144];
struct HistogramAllocator;
fn record(size: usize, delta: i64) {
    if ACTIVE.load(Ordering::Relaxed) && size < COUNTS.len() {
        COUNTS[size].fetch_add(delta, Ordering::Relaxed);
    }
}
unsafe impl GlobalAlloc for HistogramAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() { record(layout.size(), 1); }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc_zeroed(layout);
        if !pointer.is_null() { record(layout.size(), 1); }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(layout.size(), -1);
        System.dealloc(pointer, layout);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = System.realloc(pointer, layout, size);
        if !next.is_null() { record(layout.size(), -1); record(size, 1); }
        next
    }
}
#[global_allocator]
static ALLOCATOR: HistogramAllocator = HistogramAllocator;
fn capture(label: &str) {
    ACTIVE.store(false, Ordering::Relaxed);
    let mut values: Vec<_> = COUNTS.iter().enumerate().filter_map(|(size, count)| {
        let count = count.load(Ordering::Relaxed);
        (count != 0).then_some((size as i64 * count, size, count))
    }).collect();
    values.sort_unstable_by(|a,b| b.cmp(a));
    println!("{} {}", label, serde_json::to_string(&values).unwrap());
    drop(values);
    ACTIVE.store(true, Ordering::Relaxed);
}
#[allow(dead_code)]
mod kernel {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/final_performance.rs"));
    pub fn diagnose() {
        use game_core::game::battle::{runtime_facts::*, damage::{DamageSourceSnapshot, DamageResult}, enums::BattleEvent};
        println!("sizes {}", serde_json::json!({
            "source": std::mem::size_of::<RuntimeFactSource>(),
            "action": std::mem::size_of::<ActionSnapshot>(),
            "actor": std::mem::size_of::<ActorFactSummary>(),
            "damage": std::mem::size_of::<DamageFact>(),
            "segment": std::mem::size_of::<DamageSegmentFact>(),
            "launch": std::mem::size_of::<DamageSourceSnapshot>(),
            "result": std::mem::size_of::<DamageResult>(),
            "event": std::mem::size_of::<BattleEvent>(),
        }));
        let (data, _) = preservation_data();
        super::ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut core = active_session_core(data, 0);
        let mut execution = core.start_battle_execution().unwrap();
        super::capture("start");
        for _ in 0..SESSION_STEP_COUNT { core.step_battle_execution_by(&mut execution, SESSION_STEP_MS).unwrap(); }
        super::capture("battle_end");
        let serialized = serde_json::to_vec(&core.event_log).unwrap();
        super::capture("serialized");
        super::ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
        println!("actual {}", serde_json::json!({"events":core.event_log.entries.len(),"bytes":serialized.len(),"sha256":sha256(&serialized),"usage":core.runtime_fact_usage()}));
    }
}
fn main() { kernel::diagnose(); }
