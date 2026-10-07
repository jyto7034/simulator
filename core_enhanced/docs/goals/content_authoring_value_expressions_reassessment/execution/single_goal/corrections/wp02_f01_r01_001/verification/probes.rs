use super::*;
use std::{alloc::{GlobalAlloc, Layout, System}, cell::Cell};
use crate::game::battle::runtime_control as native;

thread_local! {static COUNT: Cell<(bool,usize,usize)> = const {Cell::new((false,0,0))};}
thread_local! {static FRAME_ALLOC: Cell<(usize,usize,usize)> = const {Cell::new((0,0,0))};}
struct Allocator;
fn count(l:Layout) {let _=COUNT.try_with(|s|{let (on,sum,max)=s.get();if on{s.set((on,sum+l.size(),max.max(l.size())));let _=FRAME_ALLOC.try_with(|f|{let (size,align,hits)=f.get();if size==l.size()&&align==l.align(){f.set((size,align,hits+1));}});}});}
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {let p=System.alloc(l);if !p.is_null(){count(l);}p}
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8 {let p=System.alloc_zeroed(l);if !p.is_null(){count(l);}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){System.dealloc(p,l);}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {let q=System.realloc(p,l,n);if !q.is_null(){count(Layout::from_size_align(n,l.align()).unwrap());}q}
}
#[global_allocator] static GLOBAL:Allocator=Allocator;
fn measured<T>(f:impl FnOnce()->T)->(T,usize,usize) {
    COUNT.with(|s|s.set((true,0,0)));
    let result=f();
    let (_,sum,max)=COUNT.with(|s|{let r=s.get();s.set((false,0,0));r});
    (result,sum,max)
}
#[test]
fn reviewer_live_frame_first_node_must_fit_admitted_bytes() {
    let expression=plan(EvalMoment::Live,"frame_allocation",ValueExpr::Read(ValueRef::Stat{
        subject:ValueSubject::Source,stat:StatSelector::Attack}));
    let (mut core,actor,_)=core_with_units(Vec::new());
    let source=occurrence(&mut core,actor,&expression);
    let admitted=std::mem::size_of::<(ValueRef,TypedInteger)>()+256;
    let pair=Layout::new::<(ValueRef,TypedInteger)>();
    let baseline=core.retained_runtime_fact_extent().unwrap();
    FRAME_ALLOC.with(|s|s.set((pair.size(),pair.align(),0)));
    let rejected=native::run(native::RuntimeFactLimits{max_retained_bytes:baseline+admitted-1,..Default::default()},||{
        native::seed_retained_extent(baseline)?;
        Ok(measured(||core.build_value_expr_frame(&expression,&source,None,11,None)).0)
    });
    assert!(rejected.is_err(),"one byte short must be refused");
    assert_eq!(FRAME_ALLOC.with(|s|s.get().2),0,"frame Vec allocation occurred before admission");
    let mut actual=0;let mut largest=0;
    let result=native::run(native::RuntimeFactLimits{max_retained_bytes:baseline+admitted,..Default::default()},||{
        native::seed_retained_extent(baseline)?;
        let (r,sum,max)=measured(||core.build_value_expr_frame(&expression,&source,None,11,None));
        actual=sum;largest=max;Ok(r)
    });
    eprintln!("FRAME_ADMISSION admitted={admitted} requests={actual} largest={largest} success={}",matches!(result,Ok(Ok(_))));
    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());
    assert!(actual<=admitted,"frame allocation exceeded the entire admitted remainder: actual={actual}, admitted={admitted}");
}
#[test]
fn reviewer_full_live_evaluation_must_fit_admitted_bytes() {
    let expression=plan(EvalMoment::Live,"eval_allocation",ValueExpr::Read(ValueRef::Stat{
        subject:ValueSubject::Source,stat:StatSelector::Attack}));
    let (mut core,actor,_)=core_with_units(Vec::new());
    let source=occurrence(&mut core,actor,&expression);
    let capture=core.capture_value_expr_plan(AmountPlanKind::Damage,expression,&source,None,10,None).unwrap();
    let ordinary=native::run(native::RuntimeFactLimits::default(),||Ok(core.evaluate_value_expr_capture(&capture,None,11))).unwrap().unwrap();
    let admitted=std::mem::size_of::<(ValueRef,TypedInteger)>()+256+ordinary.usage.retained_bytes;
    let baseline=core.retained_runtime_fact_extent().unwrap()+capture.retained_extent();
    let pair=Layout::new::<(ValueRef,TypedInteger)>();
    FRAME_ALLOC.with(|s|s.set((pair.size(),pair.align(),0)));
    let rejected=native::run(native::RuntimeFactLimits{max_retained_bytes:baseline+std::mem::size_of::<(ValueRef,TypedInteger)>()+255,..Default::default()},||{
        native::seed_retained_extent(baseline)?;
        Ok(measured(||core.evaluate_value_expr_capture(&capture,None,11)).0)
    });
    assert!(rejected.is_err(),"frame budget shortfall must reject full evaluation");
    assert_eq!(FRAME_ALLOC.with(|s|s.get().2),0,"full evaluation allocated frame Vec before admission");
    let mut actual=0;let mut largest=0;
    let result=native::run(native::RuntimeFactLimits{max_retained_bytes:baseline+admitted,..Default::default()},||{
        native::seed_retained_extent(baseline)?;
        let (r,sum,max)=measured(||core.evaluate_value_expr_capture(&capture,None,11));
        actual=sum;largest=max;Ok(r)
    });
    eprintln!("EVAL_ADMISSION admitted={admitted} requests={actual} largest={largest} success={}",matches!(result,Ok(Ok(_))));
    let result=result.unwrap().unwrap();
    assert_eq!(result.amount,RuntimeAmount::Damage(10));
    assert!(actual<=admitted,"successful evaluation exceeded its conservative allocation allowance: actual={actual}, admitted={admitted}");
}

#[test]
fn reviewer_bounded_frame_growth_and_repeated_evaluation() {
    let expression=plan(EvalMoment::Live,"two_refs",ValueExpr::Add(vec![
        ValueExpr::Read(ValueRef::Stat{subject:ValueSubject::Source,stat:StatSelector::Attack}),
        ValueExpr::Read(ValueRef::Stat{subject:ValueSubject::Target,stat:StatSelector::Attack}),
    ]));
    let (mut core,actor,target)=core_with_units(Vec::new());
    let source=occurrence(&mut core,actor,&expression);
    let (count,admitted,_)=frame_storage_bound(&expression).unwrap();
    assert_eq!(count,2);
    let baseline=core.retained_runtime_fact_extent().unwrap();
    for _ in 0..8 {
        let mut actual=0;
        let result=native::run(native::RuntimeFactLimits{max_retained_bytes:baseline+admitted,..Default::default()},||{
            native::seed_retained_extent(baseline)?;
            let (r,sum,_)=measured(||core.build_value_expr_frame(&expression,&source,Some(target),11,None));
            actual=sum;Ok(r)
        }).unwrap().unwrap();
        assert!(actual<=admitted,"two-reference frame exceeded charge: {actual}>{admitted}");
        assert!(result.get(&ValueRef::Stat{subject:ValueSubject::Target,stat:StatSelector::Attack}).is_some());
        assert_eq!(raw(&expression,&result).value,13);
    }
}

#[test]
fn reviewer_string_key_frame_allocations_fit_precharge() {
    let id=ResourceId::new(format!("meter_{}","x".repeat(100))).unwrap();
    let definition=ResourceDefinition{id:id.clone(),owner:ResourceOwnerKind::Employee,lifetime:ResourceLifetime::Attempt,initial:7,cap:30};
    let expression=plan(EvalMoment::Live,"string_key",ValueExpr::Read(ValueRef::Resource{
        resource:id,owner:ResourceOwnerSelector::SourceEmployee,field:ResourceValueField::Balance}));
    let (mut core,actor,target)=core_with_units(vec![definition]);
    let source=occurrence(&mut core,actor,&expression);
    let (_,admitted,_)=frame_storage_bound(&expression).unwrap();
    let baseline=core.retained_runtime_fact_extent().unwrap();
    let mut actual=0;
    let frame=native::run(native::RuntimeFactLimits{max_retained_bytes:baseline+admitted,..Default::default()},||{
        native::seed_retained_extent(baseline)?;
        let (r,sum,_)=measured(||core.build_value_expr_frame(&expression,&source,Some(target),11,None));
        actual=sum;Ok(r)
    }).unwrap().unwrap();
    eprintln!("STRING_FRAME admitted={admitted} requests={actual}");
    assert!(actual<=admitted,"string-key frame exceeded charge: {actual}>{admitted}");
    assert_eq!(raw(&expression,&frame).value,7);
}
