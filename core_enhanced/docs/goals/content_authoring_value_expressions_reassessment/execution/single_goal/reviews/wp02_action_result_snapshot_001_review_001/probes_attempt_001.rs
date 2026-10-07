use super::*;

#[test]
fn review_resource_actual_delta_through_executor() {
    let effects = r#"
      (id: "damage", op: Damage(amount: 20, damage_type: True)),
      (id: "grant", op: Resources(operations: [(resource: "meter", owner: TargetEmployee, operation: Grant(10))])),
      (id: "heal_first", op: HealExpr(amount: (moment: Live, root: Mul(
         value: Const((value: 1, unit: Magnitude)), scalar: Ratio(
         numerator: Read(Result(selector: CurrentActionPrefix, field: ResourceChangeDelta("meter"))),
         denominator: Const((value: 1, unit: Resource("meter"))), rounding: TowardZero))))),
      (id: "consume", op: Resources(operations: [(resource: "meter", owner: TargetEmployee, operation: Consume(3))])),
      (id: "heal_second", op: HealExpr(amount: (moment: Live, root: Mul(
         value: Const((value: 1, unit: Magnitude)), scalar: Ratio(
         numerator: Read(Result(selector: CurrentActionPrefix, field: ResourceChangeDelta("meter"))),
         denominator: Const((value: 1, unit: Resource("meter"))), rounding: TowardZero)))))
    "#;
    let resource = ResourceId::new("meter".to_string()).unwrap();
    let (mut core, actor, target) = core_with_units_and_skills(vec![ResourceDefinition {
        id: resource, owner: ResourceOwnerKind::Employee, lifetime: ResourceLifetime::Attempt, initial: 0, cap: 5,
    }], Some(ordered_skill(effects)));
    let skill = core.game_data.skill_data.get_by_id("wp02_runtime_frame").unwrap().clone();
    let entry = core.game_data.value_expr_catalog().entries().find(|(_, e)| e.is_authored_expression()).unwrap().1.clone();
    let mut source = occurrence(&mut core, actor, entry.plan());
    source.allows_finite_children = true;
    let delivery = core.reserve_finite_source(source).unwrap();
    core.runtime_fact_source = delivery.clone();
    core.runtime_transaction(|candidate| candidate.execute_ordered_skill_effects(actor, &skill.id, &skill.steps[0], &[target], 10, 10, None, None)).unwrap();
    assert_eq!(core.units[&target].stats.current_health, 67, "actual resource delta is +5 then -3; prefix heals are 5 and 2");
    let action = delivery.action.as_ref().unwrap();
    let witness = core.runtime_facts.action_result_witness(action).unwrap().unwrap();
    assert_eq!(witness.cumulative.resource_deltas, vec![2]);
    assert_eq!(witness.cumulative.healing_delta, 7);
    assert_eq!(witness.completed_effects, 5);
}

#[test]
fn review_witness_lives_until_last_delivery_then_releases_extent() {
    use crate::game::battle::runtime_facts::ActionTerminalKind;
    let (mut core, actor, target, skill, mut source) = ordered_fixture(r#"
        (id: "damage", op: Damage(amount: 7, damage_type: True)),
        (id: "heal", op: HealExpr(amount: (moment: Live, root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))
    "#);
    source.allows_finite_children = true;
    let first = core.reserve_finite_source(source.clone()).unwrap();
    let last = core.reserve_finite_source(source).unwrap();
    let action = first.action.as_ref().unwrap().clone();
    core.runtime_fact_source = first.clone();
    core.execute_ordered_skill_effects(actor, &skill.id, &skill.steps[0], &[target], 10, 10, None, None).unwrap();
    let mut control = core.clone();
    // Remove only the independent review's witness from a clone by comparing final accounting
    // to an otherwise identical action without any installed witness.
    let (mut baseline, _, _, baseline_skill, baseline_source) = ordered_fixture(r#"
        (id: "damage", op: Damage(amount: 7, damage_type: True)),
        (id: "heal", op: HealExpr(amount: (moment: Live, root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))
    "#);
    baseline.ensure_value_expr_action_witness(&baseline_skill.id, &baseline_source).unwrap();
    assert!(baseline.runtime_facts.action_result_witness(baseline_source.action.as_ref().unwrap()).unwrap().is_some());
    core.runtime_facts.terminal(&action, ActionTerminalKind::Finished, 11).unwrap();
    assert!(core.runtime_facts.action_result_witness(&action).unwrap().is_some());
    core.runtime_facts.close_delivery(first.delivery.as_deref().unwrap(), 12).unwrap();
    assert!(core.runtime_facts.action_result_witness(&action).unwrap().is_some());
    core.runtime_facts.close_delivery(last.delivery.as_deref().unwrap(), 13).unwrap();
    assert!(core.runtime_facts.action_result_witness(&action).is_err());
    assert!(control.runtime_facts.action_result_witness(&action).unwrap().is_some(), "clone retains independent active result");
    control.runtime_facts.terminal(&action, ActionTerminalKind::Finished, 11).unwrap();
    control.runtime_facts.close_delivery(first.delivery.as_deref().unwrap(), 12).unwrap();
    control.runtime_facts.close_delivery(last.delivery.as_deref().unwrap(), 13).unwrap();
    assert_eq!(core.retained_runtime_fact_extent().unwrap(), control.retained_runtime_fact_extent().unwrap());
}
