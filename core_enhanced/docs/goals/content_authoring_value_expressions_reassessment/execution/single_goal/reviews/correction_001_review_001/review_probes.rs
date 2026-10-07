// Compile as an external test against the frozen correction's game_core rlib.
// Reuse the preserved correction probes without editing their source.
include!("../../corrections/checkpoint_001_f01_f04/probes/correction_probes.rs");

fn authored_sum(count: usize, identity_len: usize) -> AmountExpr {
    AmountExpr::authored(EvalMoment::Live, aggregate(
        FiniteSource::AuthoredMembers((0..count).map(|i| AuthoredMember {
            identity: format!("{}{:04}", "x".repeat(identity_len), i),
            value: TypedInteger::new(1, ValueUnit::Magnitude),
            owner: None,
            projected_tile: None,
        }).collect()), Grouping::WholeSet))
}

#[test]
fn authored_work_bound_covers_actual_work_without_external_context() {
    let mut failures = Vec::new();
    for count in [0, 1, 2, 8, 32] {
        for identity_len in [0, 32, 128] {
            let expr = authored_sum(count, identity_len);
            let limits = ExprLimits::default();
            let plan = compile(&expr, &source(), caps(), &limits).unwrap();
            let mut meter = ExprMeter::new(limits).unwrap();
            let result = evaluate(&plan, &EvalFrame::default(), &mut meter).unwrap();
            let actual = meter.usage().work;
            println!("authored count={count} identity_len={identity_len} cost={} actual={actual} value={}", plan.cost().worst_case_work, result.value.value);
            if actual > plan.cost().worst_case_work {
                failures.push((count, identity_len, plan.cost().worst_case_work, actual));
            }
        }
    }
    assert!(failures.is_empty(), "known authored input exceeds declared worst-case work: {failures:?}");
}

#[test]
fn authored_compile_admission_at_declared_work_is_executable() {
    let expr = authored_sum(1, 0);
    let probe = compile(&expr, &source(), caps(), &ExprLimits::default()).unwrap();
    let limits = ExprLimits { max_work: probe.cost().worst_case_work, ..ExprLimits::default() };
    let plan = compile(&expr, &source(), caps(), &limits).unwrap();
    let mut meter = ExprMeter::new(limits).unwrap();
    let result = evaluate(&plan, &EvalFrame::default(), &mut meter);
    println!("exact admitted work={} result={:?} usage={:?}", limits.max_work, result.as_ref().map(|r|r.value.value), meter.usage());
    assert!(result.is_ok(), "statically known one-member expression was admitted but cannot execute at its own work bound");
}

#[test]
fn successful_runtime_allocations_fit_reported_reservation() {
    for count in [0, 1, 8, 64] {
        for grouping in [Grouping::WholeSet, Grouping::ByOwner, Grouping::ByProjectedTile] {
            let root = aggregate(FiniteSource::AuthoredMembers((0..count).map(|i|AuthoredMember {
                identity: format!("m{i:04}"),
                value: TypedInteger::new(1, ValueUnit::Magnitude),
                owner: Some(GroupOwner::new(GroupOwnerDomain::Employee, format!("o{i:04}")).unwrap()),
                projected_tile: Some(ProjectedTile{x:i as i32,y:0}),
            }).collect()), grouping);
            let plan = compile(&AmountExpr::authored(EvalMoment::Live, root), &source(), caps(), &ExprLimits::default()).unwrap();
            let frame = EvalFrame::default();
            let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
            let base = start_measure();
            let result = evaluate(&plan, &frame, &mut meter);
            let peak = end_measure(base);
            println!("allocation count={count} grouping={grouping:?} peak={peak} retained={}", meter.usage().retained_bytes);
            assert!(result.is_ok(), "{result:?}");
            assert!(peak <= meter.usage().retained_bytes, "allocation peak exceeded reservation");
        }
    }
}

// Isolated process only: overflow aborts Rust, not a catchable panic. This probe
// deliberately tests caller-configurable limits, not default-depth workloads.
#[test]
#[ignore]
fn deep_shape_on_normal_worker_stack() {
    std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
        let count = 2048;
        let mut root = constant(1);
        for _ in 0..count {
            root = ValueExpr::Aggregate {
                source: FiniteSource::AuthoredMembers(vec![AuthoredMember {
                    identity: "m".into(), value: TypedInteger::new(1, ValueUnit::Magnitude),
                    owner: None, projected_tile: None,
                }]), grouping: Grouping::WholeSet, reduction: Reduction::Sum,
                outer_reduction: None, member: Box::new(root),
            };
        }
        // Avoid recursive AST destruction confounding compiler-stack evidence.
        let expr = std::mem::ManuallyDrop::new(AmountExpr::authored(EvalMoment::Live, root));
        let limits = ExprLimits {
            max_depth: count+1, max_nodes: count+1,
            max_retained_bytes: 64 * 1024 * 1024,
            max_compile_work: 16 * 1024 * 1024,
            ..ExprLimits::default()
        };
        eprintln!("enter compile: nesting={count}, worker_stack=2097152, AST drop disabled");
        let result = compile(&expr, &source(), caps(), &limits);
        eprintln!("returned from compile: {:?}", result.as_ref().map(|p|p.cost()).map_err(|e|e.code));
        assert!(result.is_ok());
    }).unwrap().join().unwrap();
}
