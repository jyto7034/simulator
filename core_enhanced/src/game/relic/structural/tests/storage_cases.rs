//! Metamorphic lower bounds for private completed storage; no visitor replica.
use super::*;
use crate::game::ability::flow::SkillFlowDef;

fn measure(plan: &EffectivePlanComponents, data: &ValidatedGameData) -> (usize, u64) {
    let mut meter = Work::new(data.runtime_safety().expression().max_compile_work);
    let bytes = super::super::storage::measure(plan, data.identity(), &mut meter).unwrap();
    (bytes, meter.work)
}

#[test]
fn tb002_completed_storage_tracks_spare_capacity_and_serde_skipped_flow() {
    let (data, compiled) = crate::game::battle::core::relic_structural_tests::prepared_plan();
    assert!(
        compiled.usage.reserved_bytes > compiled.retained_extent(),
        "this fixture must distinguish compile scratch from retained data"
    );
    let mut plan = compiled.components.clone();
    let (base, _) = measure(&plan, &data);
    let fingerprint_capacity = plan.fingerprint.capacity();
    let effects_capacity = plan.effects.capacity();
    let semantic_before = serde_json::to_vec(&compiled.explain().unwrap()).unwrap();
    plan.fingerprint.reserve_exact(4096);
    plan.effects.reserve_exact(17);
    let minimum_growth = plan.fingerprint.capacity() - fingerprint_capacity
        + (plan.effects.capacity() - effects_capacity) * std::mem::size_of::<EffectiveEffect>();
    let (spare, _) = measure(&plan, &data);
    assert!(spare >= base + minimum_growth);
    assert_eq!(plan.effects.len(), compiled.effects.len());
    assert_eq!(plan.fingerprint, compiled.fingerprint);
    assert_eq!(
        serde_json::to_vec(&compiled.explain().unwrap()).unwrap(),
        semantic_before
    );

    let mut flow_plan = compiled.components.clone();
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
