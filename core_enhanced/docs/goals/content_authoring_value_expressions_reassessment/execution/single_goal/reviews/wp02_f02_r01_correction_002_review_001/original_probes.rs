// Included only in the archived source's private test module in /tmp.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
thread_local! { static REVIEW_ALLOCATION: Cell<(bool, usize)> = const { Cell::new((false, 0)) }; }
struct ReviewAllocator;
unsafe impl GlobalAlloc for ReviewAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = REVIEW_ALLOCATION.try_with(|s| { let (on,n)=s.get(); if on {s.set((on,n.saturating_add(layout.size())))} });
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { System.dealloc(ptr,layout) }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let _ = REVIEW_ALLOCATION.try_with(|s| {let (on,n)=s.get(); if on {s.set((on,n.saturating_add(size)))} });
        System.realloc(ptr,layout,size)
    }
}
#[global_allocator]
static REVIEW_ALLOCATOR: ReviewAllocator = ReviewAllocator;

fn review_catalog(root: &str, resources: Vec<ResourceDefinition>) -> (BattleCore, UnitInstanceId, UnitInstanceId, CompiledAmountEntry) {
    let input = format!(r#"SkillDatabase(skills: [SkillDef(
        id: "wp02_runtime_frame", name: "review", kind: Targeted, focus_time_ms: 0,
        cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
        steps: [SkillStepDef(id: "step_0", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
            effects: [(id: "review", op: DamageExpr(amount: (moment: Live, root: {root}), damage_type: True))])]
    )])"#);
    let skills = SkillDatabase::from_ron("fixture/review.ron", &input).unwrap();
    let (core, source, target) = core_with_units_and_skills(resources, Some(skills));
    let entry = core.game_data.value_expr_catalog().entries().next().unwrap().1.clone();
    (core, source, target, entry)
}

#[test]
fn review_valid_catalog_capture_control() {
    let (mut core, actor, _, entry) = review_catalog("Read(Stat(subject: Source, stat: Attack))", vec![]);
    let source = occurrence(&mut core, actor, entry.plan());
    let capture = core.capture_value_expr_entry(&entry, &source, None, 10, None).unwrap();
    assert_eq!(core.evaluate_value_expr_capture(&capture, None, 11).unwrap().amount, RuntimeAmount::Damage(10));
    assert_eq!(core.game_data.value_expr_catalog().ensure_runtime_publishable().unwrap_err().code, "content_capability_unsupported");
}

#[test]
fn review_action_actor_mismatch_must_be_rejected() {
    let (mut core, actor, other, entry) = review_catalog("Read(Stat(subject: Source, stat: Attack))", vec![]);
    let mut source = occurrence(&mut core, actor, entry.plan());
    source.actor = Some(other); // Valid action token from actor, different valid source unit.
    let result = core.capture_value_expr_entry(&entry, &source, None, 10, None)
        .and_then(|capture| core.evaluate_value_expr_capture(&capture, None, 11));
    println!("action/source mismatch: {result:?}");
    assert!(result.is_err(), "another actor's action must not authorize this source frame");
}

#[test]
fn review_action_equipment_mismatch_must_be_rejected() {
    use crate::game::battle::runtime_facts::EquipmentSourceSnapshot;
    let (mut core, actor, _, entry) = review_catalog("Read(Stat(subject: Source, stat: Attack))", vec![]);
    let mut basis = RuntimeFactSource {actor:Some(actor), authored:Some(Arc::new(entry.plan().source().clone())),
        equipment:Some(EquipmentSourceSnapshot {definition_uuid:Uuid::from_u128(91), runtime_item_instance_id:Uuid::from_u128(92), persistent_copy_uuid:Some(Uuid::from_u128(93))}), ..Default::default()};
    let action = core.runtime_facts.accept(actor, None, basis.clone(), 1, false).unwrap();
    basis.action = Some(action);
    basis.equipment.as_mut().unwrap().persistent_copy_uuid = Some(Uuid::from_u128(94));
    let result = core.capture_value_expr_entry(&entry, &basis, None, 10, None);
    println!("action/equipment mismatch accepted: {}", result.is_ok());
    assert!(result.is_err(), "capture must preserve actual source-copy binding");
}

#[test]
fn review_live_resource_of_inactive_target_must_be_rejected() {
    let resource = ResourceId::new("meter".to_string()).unwrap();
    // Existing fixture helper installs resources after build(), so resource
    // expressions must be compiled after that setup, as in the original test.
    let (mut core, actor, target) = core_with_units(vec![ResourceDefinition {
        id:resource.clone(), owner:ResourceOwnerKind::Employee,
        lifetime:ResourceLifetime::Attempt, initial:7, cap:30}]);
    let amount = plan(EvalMoment::Live, "dead_resource", ValueExpr::Mul {
        value:Box::new(ValueExpr::Const(TypedInteger::new(1,ValueUnit::Magnitude))),
        scalar:Box::new(ValueExpr::Ratio {
            numerator:Box::new(ValueExpr::Read(ValueRef::Resource {resource:resource.clone(),owner:ResourceOwnerSelector::TargetEmployee,field:ResourceValueField::Balance})),
            denominator:Box::new(ValueExpr::Const(TypedInteger::new(1,ValueUnit::Resource(resource)))),
            rounding:crate::game::value_expr::Rounding::TowardZero,
        }),
    });
    let source = occurrence(&mut core, actor, &amount);
    let capture = core.capture_value_expr_plan(AmountPlanKind::Damage, amount, &source, Some(target), 10, None).unwrap();
    assert_eq!(core.evaluate_value_expr_capture(&capture,Some(target),10).unwrap().amount, RuntimeAmount::Damage(7));
    core.finalize_unit_death(Some(actor), target, Side::Player, 11);
    assert!(core.units.get(&target).unwrap().lifecycle.is_dead());
    let result = core.evaluate_value_expr_capture(&capture, Some(target), 11);
    println!("inactive target resource: {result:?}");
    assert!(result.is_err(), "Live target resource must not read inactive unit state");
}

#[test]
fn review_native_remaining_retained_limit_must_reject_evaluation() {
    use crate::game::battle::runtime_control as native;
    let (mut core, actor, _, entry) = review_catalog("Read(Stat(subject: Source, stat: Attack))", vec![]);
    let source = occurrence(&mut core, actor, entry.plan());
    let capture = core.capture_value_expr_entry(&entry, &source, None, 10, None).unwrap();
    let retained = core.retained_runtime_fact_extent().unwrap() + capture.retained_extent();
    let result = native::run(native::RuntimeFactLimits {max_retained_bytes:retained + 1, ..Default::default()}, || {
        native::seed_retained_extent(retained)?;
        let evaluated = core.evaluate_value_expr_capture(&capture, None, 11);
        println!("remaining retained=1, evaluation={evaluated:?}");
        Ok(evaluated)
    });
    assert!(matches!(result, Err(_) | Ok(Err(_))), "evaluation allocates frame/stack/trace beyond remaining native limit");
}

#[test]
fn review_exhausted_work_must_stop_before_full_evaluator_allocation() {
    use crate::game::battle::runtime_control as native;
    let root = format!("Add([{}])", vec!["Read(Stat(subject: Source, stat: Attack))"; 128].join(","));
    let (mut core, actor, _, entry) = review_catalog(&root, vec![]);
    let source = occurrence(&mut core, actor, entry.plan());
    let capture = core.capture_value_expr_entry(&entry, &source, None, 10, None).unwrap();
    let measure = |limits| {
        let mut bytes = 0;
        let result = native::run(limits, || {
            REVIEW_ALLOCATION.with(|s|s.set((true,0)));
            let evaluated = core.evaluate_value_expr_capture(&capture, None, 11);
            REVIEW_ALLOCATION.with(|s| {bytes=s.get().1; s.set((false,0));});
            Ok(evaluated)
        });
        (result, bytes)
    };
    let (normal, normal_bytes) = measure(native::RuntimeFactLimits::default());
    assert!(normal.unwrap().is_ok());
    let (tight, tight_bytes) = measure(native::RuntimeFactLimits {max_work:1, ..Default::default()});
    println!("normal requested allocation={normal_bytes}; work=1 requested allocation={tight_bytes}; result={tight:?}");
    assert!(tight.is_err());
    assert!(tight_bytes < normal_bytes / 2, "tight host budget still executes/allocates the full expression before refusal");
}
