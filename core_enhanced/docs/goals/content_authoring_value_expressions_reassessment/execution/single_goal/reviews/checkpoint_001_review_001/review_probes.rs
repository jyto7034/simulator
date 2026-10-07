use game_core::game::ability::{authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource}, SkillId};
use game_core::game::resource::{ResourceId, ResourceOwnerSelector};
use game_core::game::value_expr::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct Measured;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TRACK: AtomicBool = AtomicBool::new(false);
fn allocated(n: usize) {
    let live = LIVE.fetch_add(n, Ordering::SeqCst) + n;
    if TRACK.load(Ordering::SeqCst) { PEAK.fetch_max(live, Ordering::SeqCst); }
}
unsafe impl GlobalAlloc for Measured {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l); if !p.is_null() { allocated(l.size()); } p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = System.alloc_zeroed(l); if !p.is_null() { allocated(l.size()); } p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { LIVE.fetch_sub(l.size(), Ordering::SeqCst); System.dealloc(p, l); }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = System.realloc(p,l,n);
        if !q.is_null() { LIVE.fetch_sub(l.size(),Ordering::SeqCst); allocated(n); } q
    }
}
#[global_allocator] static ALLOC: Measured = Measured;
fn start_measure() -> usize {
    let base = LIVE.load(Ordering::SeqCst); PEAK.store(base,Ordering::SeqCst); TRACK.store(true,Ordering::SeqCst); base
}
fn end_measure(base: usize) -> usize { TRACK.store(false,Ordering::SeqCst); PEAK.load(Ordering::SeqCst).saturating_sub(base) }
fn source() -> EffectSource {
    EffectSource { source_file:"review.ron".into(), site:EffectSite {
        action:ActionSite { definition_id:SkillId::new("review"), action_id:"strike".into() }, effect_id:"damage".into()
    }, field_path:"amount".into(), origin_definition_id:SkillId::new("review"), origin:EffectOrigin::Original }
}
fn caps() -> DeliveryCapabilities {
    DeliveryCapabilities { snapshot:true, live:true, target_at_release:true, target_at_consume:true,
        resolved_targets_at_release:true, resolved_targets_at_consume:true, resources:true,
        result_prefix:true, deadline:true, owner_grouping:true, projected_tile_grouping:true }
}
fn stat(stat: StatSelector) -> ValueRef { ValueRef::Stat { subject:ValueSubject::Source, stat } }
fn constant(n:i128) -> ValueExpr { ValueExpr::Const(TypedInteger::new(n,ValueUnit::Magnitude)) }
fn aggregate(source:FiniteSource, grouping:Grouping) -> ValueExpr {
    ValueExpr::Aggregate { source, grouping, reduction:Reduction::Sum,
        outer_reduction:if grouping==Grouping::WholeSet {None} else {Some(Reduction::Sum)},
        member:Box::new(ValueExpr::Read(ValueRef::AggregateMember {unit:ValueUnit::Magnitude})) }
}

#[test]
fn control_dynamic_rounding_all_signs() {
    for rounding in [Rounding::TowardZero,Rounding::Floor,Rounding::Ceil,Rounding::NearestTiesEven] {
        let root=ValueExpr::Ratio { numerator:Box::new(ValueExpr::Read(stat(StatSelector::Attack))),
            denominator:Box::new(ValueExpr::Read(stat(StatSelector::SpellPower))), rounding };
        let plan=compile(&AmountExpr::authored(EvalMoment::Live,root),&source(),caps(),&ExprLimits::default()).unwrap();
        for a in -21..=21 { for b in -9..=9 { if b==0 {continue}
            let mut frame=EvalFrame::default();
            frame.insert(stat(StatSelector::Attack),TypedInteger::new(a,ValueUnit::Magnitude)).unwrap();
            frame.insert(stat(StatSelector::SpellPower),TypedInteger::new(b,ValueUnit::Magnitude)).unwrap();
            let value=a as f64 / b as f64;
            let expected=match rounding { Rounding::TowardZero=>value.trunc(),Rounding::Floor=>value.floor(),
                Rounding::Ceil=>value.ceil(),Rounding::NearestTiesEven=>value.round_ties_even() } as i128;
            let actual=evaluate(&plan,&frame,&mut ExprMeter::new(ExprLimits::default()).unwrap()).unwrap().value.value;
            assert_eq!(actual,expected,"{a}/{b} {rounding:?}");
        }}
    }
}

#[test]
fn lower_runtime_stack_limit_is_enforced() {
    let root=ValueExpr::Add(vec![ValueExpr::Read(stat(StatSelector::Attack)),constant(1)]);
    let plan=compile(&AmountExpr::authored(EvalMoment::Live,root),&source(),caps(),&ExprLimits::default()).unwrap();
    let mut frame=EvalFrame::default();frame.insert(stat(StatSelector::Attack),TypedInteger::new(41,ValueUnit::Magnitude)).unwrap();
    let mut limits=ExprLimits::default(); limits.max_stack=1;
    let result=evaluate(&plan,&frame,&mut ExprMeter::new(limits).unwrap());
    println!("plan_stack={} meter_stack=1 result={:?}",plan.cost().max_stack,result.as_ref().map(|r|r.value.value));
    assert!(matches!(result,Err(e) if e.code==ValueExprFaultCode::BudgetExceeded));
}

#[test]
fn target_resource_snapshot_requires_target_at_release() {
    let mut capabilities=caps(); capabilities.target_at_release=false;
    let reference=ValueRef::Resource { resource:ResourceId::new("mana".into()).unwrap(),
        owner:ResourceOwnerSelector::TargetEmployee,field:ResourceValueField::Balance };
    let result=compile(&AmountExpr::authored(EvalMoment::Snapshot,ValueExpr::Read(reference)),&source(),capabilities,&ExprLimits::default());
    println!("unknown target Snapshot resource compilation accepted={}",result.is_ok());
    assert!(matches!(result,Err(e) if e.code==ValueExprFaultCode::UnsupportedMoment || e.code==ValueExprFaultCode::MissingContext));
}

#[test]
fn oversized_owner_does_not_fit_retained_budget() {
    let member=AuthoredMember {identity:"m".into(),value:TypedInteger::new(1,ValueUnit::Magnitude),
        owner:Some(GroupOwner::new(GroupOwnerDomain::Employee,"x".repeat(131072)).unwrap()),projected_tile:None};
    let expr=AmountExpr::authored(EvalMoment::Live,aggregate(FiniteSource::AuthoredMembers(vec![member]),Grouping::ByOwner));
    let mut limits=ExprLimits::default();limits.max_retained_bytes=1024;
    let base=start_measure();let compiled=compile(&expr,&source(),caps(),&limits);let compile_peak=end_measure(base);
    println!("compile accepted={} extra_live_heap_peak={compile_peak} retained_limit=1024",compiled.is_ok());
    let plan=compiled.unwrap(); let mut meter=ExprMeter::new(limits).unwrap();
    let base=start_measure();let result=evaluate(&plan,&EvalFrame::default(),&mut meter);let peak=end_measure(base);
    println!("eval={:?} charged_retained={} extra_live_heap_peak={peak}",result.as_ref().map(|r|r.value.value),meter.usage().retained_bytes);
    assert!(matches!(result,Err(e) if e.code==ValueExprFaultCode::BudgetExceeded));
}

#[test]
fn duplicate_resolved_preprocessing_respects_work_budget() {
    let plan=compile(&AmountExpr::authored(EvalMoment::Live,aggregate(FiniteSource::ResolvedTargets,Grouping::WholeSet)),
        &source(),caps(),&ExprLimits::default()).unwrap();
    let member=EvalMember {identity:"m".into(),value:TypedInteger::new(1,ValueUnit::Magnitude),owner:None,
        projected_tile:None,values:Default::default()};
    let mut frame=EvalFrame::default();frame.set_resolved_targets(vec![member;16384]);
    let mut limits=ExprLimits::default();limits.max_work=32;limits.max_dynamic_members=1;
    let mut meter=ExprMeter::new(limits).unwrap();
    let base=start_measure();let result=evaluate(&plan,&frame,&mut meter);let peak=end_measure(base);
    println!("raw_members=16384 work_limit=32 member_limit=1 eval={:?} charged={:?} extra_live_heap_peak={peak}",result.as_ref().map(|r|r.value.value),meter.usage());
    assert!(matches!(result,Err(e) if e.code==ValueExprFaultCode::BudgetExceeded));
}

#[test]
fn compiler_wide_add_does_not_allocate_past_admission() {
    let expr=AmountExpr::authored(EvalMoment::Live,ValueExpr::Add((0..65536).map(|_|constant(1)).collect()));
    let mut limits=ExprLimits::default();limits.max_nodes=4;limits.max_retained_bytes=1024;
    let base=start_measure();let result=compile(&expr,&source(),caps(),&limits);let peak=end_measure(base);
    println!("children=65536 node_limit=4 retained_limit=1024 compile_error={:?} extra_live_heap_peak={peak}",result.as_ref().err().map(|e|e.code));
    assert!(matches!(result,Err(e) if e.code==ValueExprFaultCode::BudgetExceeded));
    assert!(peak<=limits.max_retained_bytes,"allocation happened before node admission: {peak}");
}

#[test]
fn control_fingerprint_tracks_semantics_not_source_path() {
    let expr=AmountExpr::authored(EvalMoment::Live,constant(42)); let a=source();let mut b=a.clone();b.source_file="other.ron".into();
    let pa=compile(&expr,&a,caps(),&ExprLimits::default()).unwrap();
    let pb=compile(&expr,&b,caps(),&ExprLimits::default()).unwrap();assert_eq!(pa.semantic_fingerprint(),pb.semantic_fingerprint());
    b.site.effect_id="heal".into();let pc=compile(&expr,&b,caps(),&ExprLimits::default()).unwrap();
    assert_ne!(pa.semantic_fingerprint(),pc.semantic_fingerprint());
}
