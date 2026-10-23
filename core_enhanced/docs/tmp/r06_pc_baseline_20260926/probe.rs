//! Isolated baseline probe; included only in a temporary copy's private tests.
use super::*;
use std::{alloc::{GlobalAlloc, Layout, System}, cell::Cell, time::Instant};
use serde_json::{json, Value};
use crate::game::battle::{runtime_control as native, runtime_control::RuntimeFactLimits};
use crate::game::value_expr::{EvalMember, ProjectedTile};

struct Observer;
thread_local! {
    static ALLOC: Cell<(bool, u64, u64)> = const { Cell::new((false, 0, 0)) };
}
fn requested(bytes: usize) {
    let _ = ALLOC.try_with(|state| {
        let (on, calls, sum) = state.get();
        if on { state.set((on, calls + 1, sum + bytes as u64)); }
    });
}
unsafe impl GlobalAlloc for Observer {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l); if !p.is_null() { requested(l.size()); } p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = System.alloc_zeroed(l); if !p.is_null() { requested(l.size()); } p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { System.dealloc(p, l); }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = System.realloc(p, l, n); if !q.is_null() { requested(n); } q
    }
}
#[global_allocator]
static OBSERVER: Observer = Observer;

fn measure<T>(allocation: bool, f: impl FnOnce() -> T) -> (T, Value) {
    ALLOC.with(|s| s.set((allocation, 0, 0)));
    let start = Instant::now();
    let value = std::hint::black_box(f());
    let ns = start.elapsed().as_nanos();
    let (_, calls, bytes) = ALLOC.with(|s| { let v = s.get(); s.set((false,0,0)); v });
    (value, json!({"elapsed_ns":ns,"allocation_observer":allocation,
        "allocation_requests":if allocation {Some(calls)} else {None},
        "requested_bytes_sum":if allocation {Some(bytes)} else {None}}))
}
fn digest(core: &BattleCore) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(serde_json::to_vec(&core.event_log).unwrap()))
}
fn battle(n: usize) -> BattleCore {
    use crate::game::battle::scenario::*;
    use crate::game::battle::tile_range::TileRangePattern;
    use crate::game::battle::types::{BattleOutcomeId, BattleVerdict};
    let mut scenario = BattleScenario::empty((25, 25));
    for (side, count, offset) in [(Side::Player,12,0usize),(Side::Opponent,n,1000)] {
        let id = ScenarioGroupId::new(format!("side_{offset}"));
        let spawns = (0..count).map(|i| {
            let mut unit = draft((offset+i+1) as u128, (offset+i+10000) as u128, 1_000_000, 1);
            if let BattleUnitSource::TestFixture { profile, .. } = &mut unit.source {
                profile.weapon_profile = None;
                profile.movement.speed_units_per_ms = 0;
                profile.stats.move_speed_units_per_ms = 0;
                profile.basic_attack.interval_ms = 500;
                profile.basic_attack.windup_ms = 0;
                profile.basic_attack.defense_tile_range = Some(TileRangePattern {
                    include_anchor_tile: true,
                    rows:(0..9).map(|y| if y==4 {"XXXX@XXXX".into()} else {"XXXXXXXXX".into()}).collect(),
                });
            }
            ScenarioUnitSpawn {
                unit_ref:ScenarioUnitRef::new(format!("unit_{offset}_{i}")), side, draft:unit,
                position:if side==Side::Player {Position::new(6+i as i32,1)}
                    else {Position::new((i%25) as i32,3+(i/25) as i32)},
                instance_salt:(offset+i) as u32,
            }
        }).collect();
        scenario.groups.push(ScenarioSpawnGroup {id:id.clone(),side,required_for_victory:false,
            enemy_movement_plan:None,spawns});
        scenario.events.push(ScenarioEvent {id:ScenarioEventId::new(format!("spawn_{offset}")),
            trigger:ScenarioTrigger::AtBattleStart,action:ScenarioAction::SpawnGroup {group_id:id}});
    }
    scenario.terminal_outcomes=vec![TerminalOutcome {id:BattleOutcomeId::new("timeout"),
        verdict:BattleVerdict::Victory,condition:TerminalCondition::TimeReached {time_ms:60_000},priority:0}];
    BattleCore::new_from_scenario(scenario, ValidatedGameData::from(GameDataBuilder::empty().build_arc()),17)
}

fn battle_sample(n: usize, chunk: u64, burst: usize, allocation: bool) -> Value {
    let mut core=battle(n);
    let (setup, setup_measure)=measure(allocation, || {
        let mut state=core.start_battle_execution()?;
        core.step_battle_execution_until(&mut state,0)?;
        Ok::<_,crate::game::behavior::GameError>(state)
    });
    let mut state=match setup {Ok(s)=>s,Err(e)=>return json!({"family":"battle","enemies":n,
        "chunk_ms":chunk,"burst_sources":burst,"setup":setup_measure,"setup_error":format!("{e:?}")})};
    assert_eq!(core.units.len(),n+12,"all fixture units must actually spawn");
    let initial=core.runtime_fact_summary().clone();
    let mut usage=Vec::new();
    let before=digest(&core);
    let actors:Vec<_>=core.units.values().filter(|u|u.owner==Side::Player).map(|u|u.instance_id).collect();
    let targets:Vec<_>=core.units.values().filter(|u|u.owner==Side::Opponent).map(|u|u.instance_id).collect();
    let mut actors=actors;actors.sort();let mut targets=targets;targets.sort();
    let (outcome, timing)=measure(allocation, || {
        if burst>0 {
            use crate::game::battle::{damage::{DamageSource,DamageType,DamageModifiers},
                core::damage_runtime::{CommittedDamageInput,CommittedDamagePolicy}};
            let mut snapshot=None;
            let result=native::run(RuntimeFactLimits::default(),||{
                let result=core.runtime_transaction(|candidate| {
                    for actor in actors.iter().take(burst) {
                        for target in &targets {
                            let source_snapshot=candidate.damage_source_snapshot_for_unit(*actor,*target,
                                DamageSource::Ability,DamageType::True,1,DamageModifiers::default(),0,1,false)
                                .expect("active source and target");
                            let applied=candidate.execute_committed_damage(CommittedDamageInput {
                                target_id:*target,source_snapshot,committed_at_ms:1,
                                policy:CommittedDamagePolicy::GeneralCommand});
                            crate::authoring::control::poll("baseline_burst")?;
                            assert!(applied.is_some());
                        }
                    }
                    Ok(())
                });
                snapshot=native::baseline_usage(); result
            });
            usage.push(snapshot);
            result
        } else {
            let mut result=Ok(());
            for target_time in (chunk..=1000).step_by(chunk as usize) {
                let mut snapshot=None;
                result=native::run(RuntimeFactLimits::default(),||{
                    let r=core.step_battle_execution_until(&mut state,target_time).map(|_|());
                    snapshot=native::baseline_usage(); r
                });
                usage.push(snapshot);
                if result.is_err() {break;}
            }
            result
        }
    });
    let after=digest(&core);
    let hp_loss:u64=core.units.values().map(|u|u64::from(u.stats.max_health-u.stats.current_health)).sum();
    json!({"family":if burst>0 {"damage_burst_private_transaction"} else {"literal_battle_public_step"},
        "enemies":n,"allies":12,"chunk_ms":chunk,"burst_sources":burst,"setup":setup_measure,
        "measurement":timing,"error":outcome.err().map(|e|format!("{e:?}")),"native_usage":usage,
        "last_time_ms":state.last_event_time_ms(),"units":core.units.len(),"hp_loss":hp_loss,
        "initial_facts":initial,"final_facts":core.runtime_fact_summary(),
        "log_entries":core.event_log.entries.len(),"queue_at_end":core.event_queue.len(),
        "retained_extent_at_end":core.retained_runtime_fact_extent().ok(),
        "before_log_sha256":before,"after_log_sha256":after})
}

fn kernel_sample(n: usize, repetitions: usize, grouped: bool, allocation: bool) -> Value {
    let root=ValueExpr::Aggregate {source:FiniteSource::ResolvedTargets,
        grouping:if grouped {Grouping::ByProjectedTile} else {Grouping::WholeSet},
        reduction:Reduction::Sum,outer_reduction:if grouped {Some(Reduction::Sum)} else {None},
        member:Box::new(ValueExpr::Read(ValueRef::AggregateMember {unit:ValueUnit::Magnitude}))};
    let amount=plan(EvalMoment::Live,"aggregate",root);
    let mut frame=EvalFrame::default();
    frame.set_resolved_targets((0..n).map(|i|EvalMember {identity:format!("enemy_{i:04}"),
        value:TypedInteger::new(10,ValueUnit::Magnitude),owner:None,
        projected_tile:Some(ProjectedTile {x:(i%25) as i32,y:(i/25) as i32}),values:Default::default()}).collect());
    let mut sum=0i128;let mut work=0u64;let mut last=None;let mut error=None;let mut completed=0;
    let (_,timing)=measure(allocation,|| {
        for _ in 0..repetitions {
            let mut meter=ExprMeter::new(ExprLimits::default()).unwrap();
            let r=evaluate(&amount,&frame,&mut meter);
            work+=meter.usage().work;last=Some(meter.usage());
            match r {Ok(v)=>{assert_eq!(v.value.value,10*n as i128);sum+=v.value.value;completed+=1;},
                Err(e)=>{error=Some(format!("{e:?}"));break;}}
        }
    });
    json!({"family":"kernel_resolved_aggregate_synthetic","members":n,"repetitions":repetitions,
        "grouped_per_tile":grouped,"measurement":timing,"completed":completed,"sum":sum,
        "total_expr_work":work,"last_expr_usage":last,"error":error})
}

fn adapter_sample(n: usize, sources: usize, reads: usize, allocation: bool) -> Value {
    let (mut core,actor,_)=core_with_units(Vec::new());
    let read=ValueExpr::Read(ValueRef::Stat {subject:ValueSubject::Source,stat:StatSelector::Attack});
    let root=if reads==1 {read} else {ValueExpr::Add(vec![read;reads])};
    let amount=plan(EvalMoment::Live,"adapter",root);
    let source=occurrence(&mut core,actor,&amount);
    let capture=core.capture_value_expr_plan(AmountPlanKind::Damage,amount,&source,None,10,None).unwrap();
    let mut completed=0;let mut total_work=0;let mut last=None;let mut sum=0i128;let mut usage=None;
    let (result,timing)=measure(allocation,|| native::run(RuntimeFactLimits::default(),|| {
        native::seed_retained_extent(core.retained_runtime_fact_extent()?)?;
        let mut result=Ok(());
        for _ in 0..n*sources {
            match core.evaluate_value_expr_capture(&capture,None,11) {
                Ok(v)=>{assert_eq!(v.evaluated.value.value,10*reads as i128);sum+=v.evaluated.value.value;
                    completed+=1;total_work+=v.usage.work;last=Some(v.usage);},
                Err(e)=>{result=Err(crate::game::behavior::GameError::InvalidStaticData(format!("{e:?}")));break;}
            }
        }
        usage=native::baseline_usage(); result
    }));
    json!({"family":"live_adapter_repetition_synthetic","target_equivalent":n,"source_equivalent":sources,
        "reads_per_expr":reads,"measurement":timing,"completed":completed,"sum":sum,
        "total_expr_work":total_work,"last_expr_usage":last,"native_usage":usage,
        "error":result.err().map(|e|format!("{e:?}"))})
}

#[test]
#[ignore]
fn pc_load_baseline() {
    let output=std::env::var("R06_BASELINE_OUTPUT").expect("explicit new output file");
    let sample_count=std::env::var("R06_BASELINE_SAMPLES").ok().map(|s|s.parse().unwrap()).unwrap_or(11usize);
    let mut rows=Vec::new();
    for allocation in [false,true] {
        let count=if allocation {3} else {sample_count};
        for sample in 0..=count {
            for n in [100,300,500] {
                let mut cases=Vec::new();
                for chunk in [100,1000] {cases.push(battle_sample(n,chunk,0,allocation));}
                for sources in [1,12] {
                    cases.push(battle_sample(n,0,sources,allocation));
                    for reads in [1,128] {cases.push(adapter_sample(n,sources,reads,allocation));}
                    for grouped in [false,true] {cases.push(kernel_sample(n,sources,grouped,allocation));}
                }
                for mut row in cases {row["sample"]=json!(sample);row["warmup"]=json!(sample==0);rows.push(row);}
            }
            eprintln!("BASELINE_PROGRESS allocation={allocation} sample={sample}/{count}");
        }
    }
    let file=std::fs::OpenOptions::new().write(true).create_new(true).open(output).unwrap();
    serde_json::to_writer_pretty(file,&json!({"schema":1,"seed":17,"rows":rows})).unwrap();
}
