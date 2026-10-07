use super::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
thread_local! { static TRACK: Cell<(bool,usize,usize)> = const { Cell::new((false,0,0)) }; }
struct Alloc;
unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self,l:Layout)->*mut u8 { let _=TRACK.try_with(|s|{let(a,n,m)=s.get();if a{s.set((a,n+l.size(),m.max(l.size())))}});System.alloc(l) }
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){System.dealloc(p,l)}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {let _=TRACK.try_with(|s|{let(a,v,m)=s.get();if a{s.set((a,v+n,m.max(n)))}});System.realloc(p,l,n)}
}
#[global_allocator] static ALLOC:Alloc=Alloc;

#[test]
fn review_prefix_literal_resolved_target_count_is_not_effect_count() {
    let skills=ordered_skill(r#"(id: "d", op: Damage(amount: 1, damage_type: True)),(id: "h", op: Heal(amount: 1))"#);
    let(mut core,actor,target)=core_with_units_and_skills(vec![],Some(skills));
    let skill=core.game_data.skill_data.get_by_id("wp02_runtime_frame").unwrap().clone();
    let old=core.build_skill_step_commands(actor,&skill.steps[0],&[target],10,None).unwrap().1;
    let result=core.execute_ordered_skill_effects(actor,&skill.id,&skill.steps[0],&[target],10,10,None,None).unwrap();
    println!("old resolved={} new resolved={}",old.resolved_target_count,result.resolved_target_count);
    assert_eq!(result.resolved_target_count,old.resolved_target_count);
}

#[test]
fn review_prefix_capture_scratch_must_fit_admitted_bytes() {
    use crate::game::battle::runtime_control as native;
    let effects=r#"(id: "heal", op: HealExpr(amount: (moment: Live, root: Const((value: 1, unit: Magnitude)))))"#;
    let(mut core,_,template_id,skill,source)=ordered_fixture(effects);
    let targets:Vec<UnitInstanceId>=(1000..1512).map(|i|Uuid::from_u128(i).into()).collect();
    let template=core.units[&template_id].clone();
    for id in &targets { let mut unit=template.clone();unit.instance_id=*id;core.units.insert(*id,unit); }
    let cap=6000;
    let basis=core.retained_runtime_fact_extent().unwrap();
    TRACK.with(|s|s.set((true,0,0)));
    let result=native::run(native::RuntimeFactLimits{max_retained_bytes:basis+cap,..Default::default()},||{
        native::seed_retained_extent(basis)?;
        core.build_value_expr_delivery_capture(&skill.steps[0],&source,&targets,10,None)
    });
    let(_,sum,max)=TRACK.with(|s|{let v=s.get();s.set((false,0,0));v});
    println!("capture allocation: cap={cap} sum={sum} max_request={max} error={:?}",result.as_ref().err());
    assert!(result.is_err());
    assert!(max<=cap,"one request exceeds the entire allowance before rejection");
}

#[test]
fn review_prefix_runtime_rejects_missing_current_frontier() {
    let p=plan(EvalMoment::Live,"current",ValueExpr::Read(ValueRef::Result{
        selector:ResultSelector::CurrentActionPrefix,field:ResultField::DamageHpLost}));
    let(mut core,actor,_)=core_with_units(vec![]);
    let source=occurrence(&mut core,actor,&p);
    let prefix=EffectResultPrefix{
        action:source.action.clone().unwrap(),delivery:None,action_site:p.source().site.action.clone(),
        effect_ids:Arc::new(vec!["first".into(),"current".into()]),resource_ids:Arc::new(vec![]),
        settled:vec![None,None],cumulative:EffectResultProjection::zero(0)};
    let result=core.build_value_expr_frame_with_prefix(&p,&source,None,10,None,Some(&prefix),1);
    println!("missing frontier accepted={}",result.is_ok());
    assert_eq!(result.unwrap_err().code,ValueExprFaultCode::NotSettled);
}

#[test]
fn review_prefix_actual_damage_control() {
    ordered_live_result_uses_actual_damage_after_mitigation();
}

#[test]
fn review_prefix_cross_step_is_complete_or_explicitly_unsupported() {
    let skills=SkillDatabase::from_ron("fixture/cross_step_review.ron",r#"SkillDatabase(skills: [SkillDef(
        id: "wp02_runtime_frame",name: "cross",kind: Targeted,focus_time_ms: 0,
        cast_targeting: Explicit(selection: automatic_unit,target: EnemySingle(rule: Nearest),reach: global,air_capable: false),
        steps: [
            SkillStepDef(id: "step_0",delay_ms: 0,target: EnemySingle(rule: Nearest),delivery: Instant,
                effects: [(id: "first",op: Damage(amount: 7,damage_type: True))]),
            SkillStepDef(id: "step_1",delay_ms: 0,target: EnemySingle(rule: Nearest),delivery: Instant,
                effects: [(id: "second",op: Damage(amount: 3,damage_type: True)),
                (id: "heal",op: HealExpr(amount: (moment: Live,root: Read(Result(selector: CurrentActionPrefix,field: DamageHpLost)))))])
        ])])"#).unwrap();
    let(mut core,actor,target)=core_with_units_and_skills(vec![],Some(skills));
    let skill=core.game_data.skill_data.get_by_id("wp02_runtime_frame").unwrap().clone();
    let entry=core.game_data.value_expr_catalog().entries().find(|(_,e)|e.is_authored_expression()).unwrap().1.clone();
    let source=occurrence(&mut core,actor,entry.plan());
    let action=source.action.clone().unwrap();
    core.runtime_fact_source=source;
    core.execute_ordered_skill_effects(actor,&skill.id,&skill.steps[0],&[target],10,10,None,None).unwrap();
    let result=core.execute_ordered_skill_effects(actor,&skill.id,&skill.steps[1],&[target],11,11,None,None);
    if let Err(error)=result { println!("explicitly refused: {error:?}");return; }
    let facts=&core.runtime_facts.query(&action).unwrap().facts;
    println!("action loss={} heal={} remaining_hp={}",facts.damage_hp_lost,facts.healing_delta,core.units[&target].stats.current_health);
    assert_eq!(facts.healing_delta,10,"do not silently return the current step's partial prefix as CurrentActionPrefix");
}
