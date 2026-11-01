//! Native requested-byte accounting is separate from uninstrumented latency.
use game_core::{application::ApplicationSnapshotKind, authoring::{self, CancellationToken,
    ExplainQuery, TraceLimits, TraceOptions, TrialLimits, TrialScenario, TrialSession},
    game::data::content_pack::{ContentSelection, DefinitionKey, DefinitionKind}};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};

#[cfg(feature = "allocation-observer")]
mod allocator {
    use std::{alloc::{GlobalAlloc, Layout, System}, sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst}};
    pub struct Observer;
    static LIVE: AtomicUsize = AtomicUsize::new(0);
    static PEAK: AtomicUsize = AtomicUsize::new(0);
    static TRACK: AtomicBool = AtomicBool::new(false);
    static ALLOCS: AtomicUsize = AtomicUsize::new(0);
    static REALLOCS: AtomicUsize = AtomicUsize::new(0);
    static FREES: AtomicUsize = AtomicUsize::new(0);
    static ADDED: AtomicUsize = AtomicUsize::new(0);
    static REMOVED: AtomicUsize = AtomicUsize::new(0);
    fn add(n: usize) { let live = LIVE.fetch_add(n, SeqCst) + n; PEAK.fetch_max(live, SeqCst); }
    unsafe impl GlobalAlloc for Observer {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let p = System.alloc(l);
            if !p.is_null() { add(l.size()); if TRACK.load(SeqCst) { ALLOCS.fetch_add(1, SeqCst); ADDED.fetch_add(l.size(), SeqCst); } }
            p
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            let p = System.alloc_zeroed(l);
            if !p.is_null() { add(l.size()); if TRACK.load(SeqCst) { ALLOCS.fetch_add(1, SeqCst); ADDED.fetch_add(l.size(), SeqCst); } }
            p
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            System.dealloc(p,l); LIVE.fetch_sub(l.size(),SeqCst);
            if TRACK.load(SeqCst) { FREES.fetch_add(1,SeqCst); REMOVED.fetch_add(l.size(),SeqCst); }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            let q = System.realloc(p,l,n);
            if !q.is_null() {
                if n >= l.size() { add(n-l.size()); } else { LIVE.fetch_sub(l.size()-n,SeqCst); }
                if TRACK.load(SeqCst) { REALLOCS.fetch_add(1,SeqCst); ADDED.fetch_add(n,SeqCst); REMOVED.fetch_add(l.size(),SeqCst); }
            }
            q
        }
    }
    pub fn begin() -> usize {
        assert!(!TRACK.load(SeqCst));
        for counter in [&ALLOCS,&REALLOCS,&FREES,&ADDED,&REMOVED] { counter.store(0,SeqCst); }
        let baseline=LIVE.load(SeqCst); PEAK.store(baseline,SeqCst); TRACK.store(true,SeqCst); baseline
    }
    pub fn end(baseline:usize) -> super::Memory {
        TRACK.store(false,SeqCst);
        let end=LIVE.load(SeqCst);
        let m=super::Memory{baseline,end,peak:PEAK.load(SeqCst),allocs:ALLOCS.load(SeqCst),reallocs:REALLOCS.load(SeqCst),
            frees:FREES.load(SeqCst),added:ADDED.load(SeqCst),removed:REMOVED.load(SeqCst)};
        assert_eq!(end as i128-baseline as i128,m.added as i128-m.removed as i128);
        m
    }
}
#[cfg(feature = "allocation-observer")]
#[global_allocator]
static ALLOCATOR: allocator::Observer = allocator::Observer;
#[derive(Default)]
struct Memory { baseline:usize,end:usize,peak:usize,allocs:usize,reallocs:usize,frees:usize,added:usize,removed:usize }
struct Sample { name:String,iteration:usize,ns:u128,memory:Option<Memory> }
fn measure<T>(samples:&mut Vec<Sample>,iteration:usize,name:impl Into<String>,f:impl FnOnce()->T)->T {
    let name=name.into();
    #[cfg(feature="allocation-observer")] let baseline=allocator::begin();
    let start=Instant::now();
    let value=f();
    let ns=start.elapsed().as_nanos();
    #[cfg(feature="allocation-observer")] let memory=Some(allocator::end(baseline));
    #[cfg(not(feature="allocation-observer"))] let memory=None;
    samples.push(Sample{name,iteration,ns,memory});
    value
}
fn receipts(session:&TrialSession)->String {session.receipt_documents().map(|s|format!("{s}\n")).collect()}
fn suite(root:&Path,selection:&ContentSelection,raw:&[u8],iteration:usize,samples:&mut Vec<Sample>)->Value {
    let cancel=CancellationToken::default();
    let load=||authoring::validate(root,selection,&Default::default(),&cancel).unwrap();
    let parse=||TrialScenario::from_json_slice(raw,&Default::default()).unwrap();
    let content=measure(samples,iteration,"validate",load);
    let usage=serde_json::to_value(content.validation_usage()).unwrap();
    let identity=serde_json::to_value(content.token().identity()).unwrap();
    let definition=DefinitionKey{kind:DefinitionKind::Skill,id:"paradise_lost_judgement".into()};
    let query=ExplainQuery::Definition{key:definition};
    let validation=measure(samples,iteration,"validation_receipt",||authoring::validation_receipt(&content,&Default::default(),&cancel).unwrap());
    let description=measure(samples,iteration,"describe",||authoring::describe(&content,&Default::default(),&cancel).unwrap());
    let explanation=measure(samples,iteration,"explain",||authoring::explain(&content,&query,&Default::default(),&cancel).unwrap());
    measure(samples,iteration,"drop_queries",||drop((validation,description,explanation)));
    measure(samples,iteration,"drop_content",||drop(content));
    let mut expected=None;
    let mut modes=Vec::new();
    for (mode,trace_limits) in [("off",None),("on",Some(TraceLimits::default())),
        ("gap",Some(TraceLimits{max_bytes:1,max_row_bytes:1,..Default::default()}))] {
        let content=load();
        let scenario=measure(samples,iteration,format!("{mode}/parse"),parse);
        let settings=TrialLimits{trace:TraceOptions{limits:trace_limits},..Default::default()};
        let mut session=measure(samples,iteration,format!("{mode}/prepare"),||TrialSession::prepare(content,scenario,settings,&cancel).unwrap());
        let prepared_bytes=session.receipt_bytes();
        let mut native_high_water = game_core::game::battle::runtime_facts::RuntimeFactUsage::default();
        while !session.is_complete() {
            let index=session.next_step_index();
            measure(samples,iteration,format!("{mode}/step/{index}"),||{session.advance(&cancel).unwrap();});
            let usage = measure(samples,iteration,format!("{mode}/runtime_fact_query/{index}"),|| {
                let view=session.observation().unwrap();
                view.active_runtime_facts().or_else(||view.last_terminal().map(|v|v.runtime_facts())).map(|v|v.usage())
            });
            measure(samples,iteration,format!("{mode}/typed_resource_query/{index}"),|| {
                let view=session.observation().unwrap();
                let run_entries=view.run_resources().map_or(0,|run|run.entries.len());
                let active_entries=view.active_typed_resources().map_or(0,|resources|resources.entries().count());
                std::hint::black_box((run_entries,active_entries))
            });
            if let Some(usage)=usage {
                native_high_water.active_actions=native_high_water.active_actions.max(usage.active_actions);
                native_high_water.public_result_reservations=native_high_water.public_result_reservations.max(usage.public_result_reservations);
                native_high_water.completed_results=native_high_water.completed_results.max(usage.completed_results);
                native_high_water.outstanding_deliveries=native_high_water.outstanding_deliveries.max(usage.outstanding_deliveries);
                native_high_water.actor_summaries=native_high_water.actor_summaries.max(usage.actor_summaries);
                native_high_water.provider_summaries=native_high_water.provider_summaries.max(usage.provider_summaries);
                native_high_water.retained_bytes_bound=native_high_water.retained_bytes_bound.max(usage.retained_bytes_bound);
                native_high_water.accepted_actions=native_high_water.accepted_actions.max(usage.accepted_actions);
                native_high_water.settled_actions=native_high_water.settled_actions.max(usage.settled_actions);
            }
        }
        let snapshot=measure(samples,iteration,format!("{mode}/snapshot"),||session.snapshot(ApplicationSnapshotKind::Run,&Default::default(),&cancel).unwrap());
        let reconnect=measure(samples,iteration,format!("{mode}/reconnect"),||session.reconnect(0,&Default::default(),&cancel));
        if session.observation().unwrap().active().is_some() { assert!(reconnect.is_ok()); }
        else {
            let error=reconnect.as_ref().err().expect("no active battle cannot reconnect");
            let rendered:Value=serde_json::from_str(authoring::render_error(error,&Default::default(),&cancel).unwrap().document()).unwrap();
            assert_eq!(rendered["diagnostic"]["code"],"invalid_action");
        }
        measure(samples,iteration,format!("{mode}/drop_inspection"),||drop((snapshot,reconnect)));
        let receipt=receipts(&session);
        if let Some(previous)=&expected {assert_eq!(&receipt,previous);} else {expected=Some(receipt.clone());}
        let trace=session.trace().unwrap();
        let trace_summary=trace.active().map(|v|json!({"retained_rows":v.retained_rows(),"gap":v.gap(),"bytes":v.ndjson().len()}));
        modes.push(json!({"mode":mode,"accepted_steps":session.next_step_index(),"accepted_work":session.accepted_work(),
            "receipt_bytes":session.receipt_bytes(),"prepared_receipt_bytes":prepared_bytes,"trace":trace_summary,
            "runtime_facts_sampled_high_water":native_high_water,
            "usage_by_receipt":session.receipt_documents().map(|s|serde_json::from_str::<Value>(s).unwrap()).collect::<Vec<_>>()}));
        // Failed replacement owns a fully loaded B; A is kept unchanged.
        for failure in ["cancelled","late_receipt_cap","fuel"] {
            let b=load(); let scenario=parse(); let token=CancellationToken::default();
            let mut limits=TrialLimits::default();
            match failure {"cancelled"=>token.cancel(),"late_receipt_cap"=>limits.max_receipt_bytes=prepared_bytes-1,
                "fuel"=>limits.execution.max_work=1,_=>unreachable!()}
            let error=measure(samples,iteration,format!("{mode}/replace/{failure}"),||session.replace(b,scenario,limits,&token).unwrap_err());
            assert_eq!(receipts(&session),receipt);
            measure(samples,iteration,format!("{mode}/drop_error/{failure}"),||drop(error));
        }
        let b=load(); let scenario=parse();
        measure(samples,iteration,format!("{mode}/replace/success"),||{session.replace(b,scenario,Default::default(),&cancel).unwrap();});
        measure(samples,iteration,format!("{mode}/drop_session"),||drop(session));
    }
    let mut live_sessions=Vec::with_capacity(4);
    for n in 1..=4 {
        let session=measure(samples,iteration,format!("coexist/load_prepare/{n}"),||TrialSession::prepare(load(),parse(),Default::default(),&cancel).unwrap());
        live_sessions.push(session);
    }
    for n in (1..=4).rev() {
        let session=live_sessions.pop().unwrap();
        measure(samples,iteration,format!("coexist/drop/{n}"),||drop(session));
    }
    json!({"iteration":iteration,"identity":identity,"validation_usage":usage,"modes":modes,
        "coexisting_independently_loaded_generations":4,"generation_count_kind":"logical ownership; Weak reclamation is a separate focused test"})
}
fn main() {
    let args:Vec<_>=std::env::args_os().skip(1).collect(); assert_eq!(args.len(),4);
    let root=Path::new(&args[0]);
    let selection:ContentSelection=serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let raw=fs::read(&args[2]).unwrap();
    let mut warmup=Vec::new(); drop(suite(root,&selection,&raw,0,&mut warmup)); drop(warmup);
    let repetitions=if cfg!(feature="allocation-observer"){3}else{5};
    let mut samples=Vec::with_capacity(2048); let mut results=Vec::new();
    for iteration in 1..=repetitions {results.push(suite(root,&selection,&raw,iteration,&mut samples));}
    let rows:Vec<_>=samples.into_iter().map(|s|json!({"phase":s.name,"iteration":s.iteration,"elapsed_ns":s.ns,
        "native_requested_bytes":s.memory.map(|m|json!({"baseline_live":m.baseline,"end_live":m.end,"peak_live":m.peak,
            "peak_above_baseline":m.peak-m.baseline,"retained_delta":m.end as i128-m.baseline as i128,
            "alloc_calls":m.allocs,"realloc_calls":m.reallocs,"free_calls":m.frees,"allocated_bytes":m.added,"freed_bytes":m.removed}))})).collect();
    let output=json!({"passed":true,"allocator_instrumented":cfg!(feature="allocation-observer"),"repetitions":repetitions,
        "warmup_suites":1,"samples":rows,"results":results,"measurement":"System allocator requested bytes, not RSS; instrumented elapsed time is not plain latency"});
    fs::write(Path::new(&args[3]),serde_json::to_vec_pretty(&output).unwrap()).unwrap();
    println!("{}",json!({"passed":true,"allocator_instrumented":cfg!(feature="allocation-observer"),"repetitions":repetitions,"samples":output["samples"].as_array().unwrap().len()}));
}
