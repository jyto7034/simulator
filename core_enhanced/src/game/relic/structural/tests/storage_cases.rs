//! Metamorphic lower bounds for private completed storage; no visitor replica.
use super::*;
use crate::game::ability::flow::SkillFlowDef;

fn measure(plan: &EffectivePlanComponents, data: &ValidatedGameData) -> (usize, u64) {
    let mut meter = Work::new(data.runtime_safety().expression().max_compile_work);
    let bytes = super::super::storage::measure(plan, data.identity(), &mut meter).unwrap();
    (bytes, meter.work)
}

#[test]
fn tb002_prepared_storage_matches_full_walk_after_owner_detachment_and_hidden_growth() {
    let (data, compiled) = crate::game::battle::core::relic_structural_tests::prepared_plan();
    let original = &compiled.components;
    let cache = super::super::storage::PreparedStorage::new(&original.providers,&original.effects,&original.execution,&mut Work::new(u64::MAX)).unwrap();
    let cached = |plan:&EffectivePlanComponents| {
        let mut work=Work::new(u64::MAX);
        let bytes=cache.measure(plan,data.identity(),&mut work,&original.providers,&original.effects,&original.execution).unwrap();
        (bytes,work.work)
    };
    let (base,full_work)=measure(original,&data);
    assert_eq!(cached(original).0,base);
    assert!(cached(original).1 < full_work);
    let mut changed=original.clone();
    Arc::make_mut(&mut changed.providers).reserve_exact(8);
    Arc::make_mut(&mut changed.effects[0]).tags.push("detached-storage-fixture".into());
    Arc::make_mut(&mut changed.execution[0].effects).reserve_exact(13);
    let flow=Arc::make_mut(&mut changed.execution[0].flow);
    let wire=serde_json::to_vec(flow).unwrap();
    flow.step_ids[0].0.reserve_exact(4096);
    assert_eq!(serde_json::to_vec(flow).unwrap(),wire);
    assert_eq!(cached(&changed).0,measure(&changed,&data).0);
    // A repeated shared edge is still fully priced, even if it points to the
    // same original owner. New positions cannot borrow another entry's extent.
    changed.effects.push(original.effects[0].clone());
    changed.execution.push(original.execution[0].clone());
    assert_eq!(cached(&changed).0,measure(&changed,&data).0);
    assert_eq!(cached(original).0,base,"candidate mutation never invalidates its immutable baseline");
}

#[test]
fn tb002_completed_storage_tracks_spare_capacity_and_serde_skipped_flow() {
    let (data, compiled) = crate::game::battle::core::relic_structural_tests::prepared_plan();
    assert_ne!(
        compiled.usage.reserved_bytes, compiled.retained_extent(),
        "compile reservations and retained shared ownership are distinct accounting bases"
    );
    let mut plan = compiled.components.clone();
    let (base, _) = measure(&plan, &data);
    let fingerprint_capacity = plan.fingerprint.capacity();
    let effects_capacity = plan.effects.capacity();
    let semantic_before = serde_json::to_vec(&compiled.explain().unwrap()).unwrap();
    plan.fingerprint.reserve_exact(4096);
    plan.effects.reserve_exact(17);
    let minimum_growth = plan.fingerprint.capacity() - fingerprint_capacity
        + (plan.effects.capacity() - effects_capacity) * std::mem::size_of::<super::super::SharedEffect>();
    let (spare, _) = measure(&plan, &data);
    assert!(spare >= base + minimum_growth);
    assert_eq!(plan.effects.len(), compiled.effects.len());
    assert_eq!(plan.fingerprint, compiled.fingerprint);
    assert_eq!(
        serde_json::to_vec(&compiled.explain().unwrap()).unwrap(),
        semantic_before
    );

    let mut flow_plan = compiled.components.clone();
    // Detach both shared owners before establishing the capacity baseline.
    // Native COW clone may shrink spare Vec/String capacities independently of
    // the deliberate reserve growth this test measures.
    let detached = Arc::make_mut(&mut flow_plan.execution[0].flow);
    if let Some(raw) = detached.source.as_mut() { Arc::make_mut(raw); }
    let (flow_before, _) = measure(&flow_plan, &data);
    let flow = Arc::make_mut(&mut flow_plan.execution[0].flow);
    let wire = serde_json::to_vec(flow).unwrap();
    let old_capacity = flow.step_ids[0].0.capacity();
    flow.step_ids[0].0.reserve_exact(3072);
    let step_growth = flow.step_ids[0].0.capacity() - old_capacity;
    let Some(raw) = flow.source.as_mut() else {
        panic!("authored Flow source must survive lowering")
    };
    let SkillFlowDef::Action { id, .. } = Arc::make_mut(raw) else {
        panic!("single original Action")
    };
    let old_capacity = id.capacity();
    id.reserve_exact(2048);
    let source_growth = id.capacity() - old_capacity;
    assert_eq!(
        serde_json::to_vec(flow).unwrap(),
        wire,
        "both owners are serde-skipped"
    );
    let (flow_after, charged_work) = measure(&flow_plan, &data);
    assert!(flow_after >= flow_before + step_growth + source_growth);
    assert!(charged_work > 0);
    println!("TB002 storage compile_reserved={} cached={} spare_min={minimum_growth} spare_delta={} skipped_min={} skipped_delta={}",
        compiled.usage.reserved_bytes, compiled.retained_extent(), spare-base, step_growth+source_growth, flow_after-flow_before);
}

#[test]
fn tb002_completed_storage_includes_catalog_source_payload_and_shared_edges() {
    use crate::game::{data::skill_data::SkillDatabase, value_expr::ValueExprCatalog};
    let (data, compiled) = crate::game::battle::core::relic_structural_tests::prepared_plan();
    let raw = r#"SkillDatabase(skills: [SkillDef(
        id: "amount_probe", name: "amount probe", kind: Untargeted, focus_time_ms: 0,
        use_lifetime: Action,
        cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: false),
        steps: [SkillStepDef(id: "restore", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "amount", op: Heal(amount: 11))])]
    )])"#;
    let short_name = "short.ron";
    let long_name = format!("{}{}.ron", short_name, "source".repeat(600));
    let catalog = |name: &str| {
        let skills = SkillDatabase::from_ron(name, raw).unwrap();
        Arc::new(
            ValueExprCatalog::compile_skills(
                &skills,
                &data.resource_data,
                &data.runtime_safety().expression(),
            )
            .unwrap(),
        )
    };
    let short = catalog(short_name);
    let long = catalog(&long_name);
    assert_eq!(short.len(), 1);
    assert_eq!(long.len(), 1);
    assert_eq!(
        long.entries().next().unwrap().1.plan().source().source_file,
        long_name
    );
    let mut plan = compiled.components.clone();
    plan.execution[0].amounts = short;
    let (short_bytes, _) = measure(&plan, &data);
    plan.execution[0].amounts = long;
    let (long_bytes, _) = measure(&plan, &data);
    let source_minimum = long_name.len() - short_name.len();
    assert!(
        long_bytes >= short_bytes + source_minimum,
        "only the private amount catalog changed; its retained source must be counted"
    );
    // A second owning edge references exactly the same immutable graph. It is
    // still conservatively priced; a global pointer set must not erase it.
    let shared = plan.execution[0].clone();
    assert!(Arc::ptr_eq(&shared.amounts, &plan.execution[0].amounts));
    assert!(Arc::ptr_eq(&shared.skill, &plan.execution[0].skill));
    plan.execution.push(shared);
    let (shared_bytes, _) = measure(&plan, &data);
    assert!(shared_bytes >= long_bytes + long_name.len());
    let mut too_little_work = Work::new(0);
    let error =
        super::super::storage::measure(&plan, data.identity(), &mut too_little_work).unwrap_err();
    assert_eq!(error.code, FaultCode::BudgetExceeded);
    println!("TB002 catalog source_min={source_minimum} observed_delta={} shared_edge_delta={} zero_work_refused=true",
        long_bytes-short_bytes, shared_bytes-long_bytes);
}

#[test]
fn owned_copy_admission_counts_skipped_sources_and_rejects_exact_limits() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let (_, compiled) = crate::game::battle::core::relic_structural_tests::prepared_plan();
    let mut flow = (*compiled.execution[0].flow).clone();
    let copy = |flow: &crate::game::ability::flow::CompiledSkillFlow| {
        let mut work = Work::new(u64::MAX);
        super::super::compiler::admit_owned_copy(flow, &mut work)?;
        Ok::<_, RelicPlanError>(work.reserved)
    };
    let base = copy(&flow).unwrap();
    let wire = serde_json::to_vec(&flow).unwrap();
    let capacity = flow.step_ids[0].0.capacity();
    flow.step_ids[0].0.reserve_exact(4096);
    let growth = flow.step_ids[0].0.capacity() - capacity;
    assert_eq!(serde_json::to_vec(&flow).unwrap(), wire);
    assert!(copy(&flow).unwrap() >= base + growth);
    let limits = ExecutionLimits::default();
    let run = |limits: &ExecutionLimits| control::run(limits, &CancellationToken::default(),
        ExecutionPhase::FinalValidation, || copy(&flow));
    let (accepted, usage) = run(&limits).unwrap();
    accepted.unwrap();
    for (cap, good) in [(usage.work - 1, false), (usage.work, true)] {
        let result = run(&ExecutionLimits { max_work: cap, ..limits.clone() });
        if good { result.unwrap().0.unwrap(); }
        else { assert_eq!(result.err().unwrap().code, "authoring_work_limit"); }
    }
    let cancelled = control::with_test_cancellation("effective_retained_scan", 1, || run(&limits));
    assert_eq!(cancelled.err().unwrap().code, "authoring_cancelled");
    run(&limits).unwrap().0.unwrap();
}

#[test]
fn shared_clone_admission_prices_handles_but_retained_and_cow_price_pointees() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let (data, compiled) = crate::game::battle::core::relic_structural_tests::prepared_plan();
    let mut plan = compiled.components.clone();
    let mut skill = (*plan.execution[0].skill).clone();
    skill.name = "shared payload".repeat(4096);
    plan.execution[0].skill = Arc::new(skill);
    let original = Arc::clone(&plan.execution[0].skill);
    let admit = || {
        let mut work = Work::new(u64::MAX);
        super::super::storage::admit_shared_clone(&original, &mut work)?;
        Ok::<_, RelicPlanError>(work.reserved)
    };
    let bytes = admit().unwrap();
    assert!(bytes < original.name.len());
    let copy = Arc::clone(&original);
    assert!(Arc::ptr_eq(&copy, &original));
    let mut cow = Work::new(u64::MAX);
    super::super::compiler::admit_owned_copy(original.as_ref(), &mut cow).unwrap();
    assert!(cow.reserved >= original.name.len());
    let (retained, _) = measure(&plan, &data);
    assert!(retained >= original.name.len());
    let mut detached = copy;
    Arc::make_mut(&mut detached).name.push_str(" changed");
    assert!(!Arc::ptr_eq(&detached, &original));
    assert!(!original.name.ends_with(" changed"));
    let limits = ExecutionLimits::default();
    let run = |limits: &ExecutionLimits| {
        control::run(
            limits,
            &CancellationToken::default(),
            ExecutionPhase::FinalValidation,
            admit,
        )
    };
    let (_, usage) = run(&limits).unwrap();
    for (cap, good) in [(usage.work - 1, false), (usage.work, true)] {
        let result = run(&ExecutionLimits {
            max_work: cap,
            ..limits.clone()
        });
        if good {
            result.unwrap().0.unwrap();
        } else {
            assert_eq!(result.err().unwrap().code, "authoring_work_limit");
        }
    }
    let cancelled = control::with_test_cancellation("effective_retained_scan", 1, || run(&limits));
    assert_eq!(cancelled.err().unwrap().code, "authoring_cancelled");
    run(&limits).unwrap().0.unwrap();
}


#[test]
fn shared_effect_detaches_only_after_admission_and_preserves_cached_definition() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let (_, compiled) = crate::game::battle::core::relic_structural_tests::prepared_plan();
    let mut base = Arc::new(compiled.effects[0].as_ref().clone());
    Arc::get_mut(&mut base).unwrap().tags.push("owned text".repeat(1024));
    let before = format!("{base:?}");
    let attempt = || {
        let mut candidate = Arc::clone(&base);
        let mut work = Work::new(u64::MAX);
        match super::super::compiler::edit_effect(&mut candidate, &mut work) {
            Ok(effect) => effect.present = false,
            Err(error) => {
                assert!(Arc::ptr_eq(&candidate, &base), "failed admission must not detach");
                return Err(error);
            }
        }
        assert!(!Arc::ptr_eq(&candidate, &base));
        assert!(base.present);
        assert_eq!(candidate.tags,base.tags);
        let reserved=work.reserved;
        super::super::compiler::edit_effect(&mut candidate, &mut work)?.present=true;
        assert_eq!(work.reserved,reserved,"unique effect needs no second allocation");
        Ok::<_,RelicPlanError>(candidate)
    };
    let limits=ExecutionLimits::default();
    let run=|limits:&ExecutionLimits|control::run(limits,&CancellationToken::default(),ExecutionPhase::FinalValidation,attempt);
    let (accepted,usage)=run(&limits).unwrap(); accepted.unwrap();
    assert_eq!(format!("{base:?}"),before);
    for (cap,valid) in [(usage.work-1,false),(usage.work,true)] {
        let result=run(&ExecutionLimits{max_work:cap,..limits.clone()});
        if valid {result.unwrap().0.unwrap();} else {assert_eq!(result.err().unwrap().code,"authoring_work_limit");}
        assert_eq!(format!("{base:?}"),before);
    }
    let cancelled=control::with_test_cancellation("relic_structural_shared_clone",1,||run(&limits));
    assert_eq!(cancelled.err().unwrap().code,"authoring_cancelled");
    run(&limits).unwrap().0.unwrap();
    assert_eq!(format!("{base:?}"),before);
}
