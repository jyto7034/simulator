// Reuse the prior frozen review tests unchanged, including its isolated deep test.
include!("../correction_001_review_001/review_probes.rs");

fn next_seed(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed
}

fn generated_static(seed: &mut u64, depth: usize) -> (ValueExpr, i128) {
    if depth == 0 {
        let n = (next_seed(seed) % 7 + 1) as i128;
        return (constant(n), n);
    }
    match next_seed(seed) % 4 {
        0 => {
            let (a,av) = generated_static(seed,depth-1);
            let (b,bv) = generated_static(seed,depth-1);
            (ValueExpr::Add(vec![a,b]), av+bv)
        }
        1 => {
            let (a,av) = generated_static(seed,depth-1);
            let factor = (next_seed(seed)%3+1) as i128;
            (ValueExpr::Mul {value:Box::new(a),scalar:Box::new(ValueExpr::Const(TypedInteger::new(factor,ValueUnit::Scalar)))},av*factor)
        }
        _ => {
            let count = (next_seed(seed)%4+1) as usize;
            let grouping = match next_seed(seed)%3 {0=>Grouping::WholeSet,1=>Grouping::ByOwner,_=>Grouping::ByProjectedTile};
            let reduction = if next_seed(seed)%2==0 {Reduction::Sum} else {Reduction::Max};
            let outer = if next_seed(seed)%2==0 {Reduction::Sum} else {Reduction::Max};
            let (child, value) = generated_static(seed,depth-1);
            let members = (0..count).rev().map(|i|AuthoredMember {
                identity:format!("{}-{i:04}","id".repeat((next_seed(seed)%64) as usize)),
                value:TypedInteger::new(i as i128,ValueUnit::Magnitude),
                owner:Some(GroupOwner::new(GroupOwnerDomain::Employee,format!("owner{}",i%2)).unwrap()),
                projected_tile:Some(ProjectedTile{x:(i%2) as i32,y:0}),
            }).collect();
            let expected = if grouping==Grouping::WholeSet {
                if reduction==Reduction::Sum {value*count as i128} else {value}
            } else {
                let sizes = [(count+1)/2,count/2];
                let reduced:Vec<i128> = sizes.into_iter().filter(|n|*n>0).map(|n|if reduction==Reduction::Sum {value*n as i128} else {value}).collect();
                if outer==Reduction::Sum {reduced.iter().sum()} else {*reduced.iter().max().unwrap()}
            };
            (ValueExpr::Aggregate {source:FiniteSource::AuthoredMembers(members),grouping,reduction,outer_reduction:if grouping==Grouping::WholeSet {None} else {Some(outer)},member:Box::new(child)},expected)
        }
    }
}

#[test]
fn generated_static_work_and_result_matrix() {
    for case in 0..200 {
        let mut seed = case + 20260913;
        let (root,expected) = generated_static(&mut seed,4);
        let expr = AmountExpr::authored(EvalMoment::Live,root);
        let probe = compile(&expr,&source(),caps(),&ExprLimits::default()).unwrap();
        assert!(!probe.cost().runtime_dynamic_work);
        assert_eq!(probe.cost().static_work,probe.cost().worst_case_work);
        let limits = ExprLimits {max_work:probe.cost().static_work,..ExprLimits::default()};
        let plan = compile(&expr,&source(),caps(),&limits).unwrap();
        let mut meter = ExprMeter::new(limits).unwrap();
        let result = evaluate(&plan,&EvalFrame::default(),&mut meter).unwrap();
        assert_eq!(result.value.value,expected,"case {case}");
        assert!(meter.usage().work<=plan.cost().static_work,"case {case}");
        assert!(meter.usage().peak_stack_values<=plan.cost().max_stack,"case {case}");
    }
    println!("200 seeded static expression cases: result oracle, static-work bound, exact budget admission and nested stack all passed");
}
