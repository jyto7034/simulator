use super::*;
use crate::game::resource::{
    ResourceCatalog, ResourceDefinition, ResourceId, ResourceLifetime, ResourceOwnerKind,
};

fn delivery_scopes(core: &GameCore) -> Vec<u64> {
    core.battle_typed_resources()
        .unwrap()
        .scopes()
        .filter_map(|(id, kind, _, _)| (kind == ResourceLifetime::Delivery).then_some(id))
        .collect()
}

#[test]
fn tc001_child_actions_have_distinct_delivery_resources_and_cancelled_clone_releases_chain() {
    let resources = ResourceCatalog {
        resources: vec![ResourceDefinition {
            id: ResourceId::new("tc_delivery".into()).unwrap(),
            owner: ResourceOwnerKind::Employee,
            lifetime: ResourceLifetime::Delivery,
            initial: 7,
            cap: 7,
        }],
    };
    let skills = skill_text(100).replace("Damage(amount: 100, damage_type: Physical)",
        "DamageExpr(amount: (moment: Live, root: Const((value: 100, unit: Magnitude))), damage_type: Physical)");
    let action = r#"Action(id: "heal_action", step_id: "heal_once", wait: EffectsSettled)"#;
    let relics = relic_text(None).replace(action, &format!(r#"Sequence(id: "child", children: [
        Wait(id: "cold_wait", duration_ms: 5),
        Repeat(id: "two_deliveries", count: 2, interval_ms: 10, body: {action})])"#))
        .replace("Heal(amount: 37)", r#"HealExpr(amount: (moment: Live, root: Mul(
            value: Const((value: 1, unit: Magnitude)), scalar: Ratio(
                numerator: Read(Resource(resource: "tc_delivery", owner: SourceEmployee, field: Balance)),
                denominator: Const((value: 1, unit: Resource("tc_delivery"))), rounding: TowardZero))))"#);
    let (mut core, player, employee, actor, _) =
        prepare_with_options(&skills, &relics, Some(resources), None);
    native_probe::observe_amounts(&mut core.state.active_battle.as_mut().unwrap().battle);
    cast(&mut core, player, employee);
    let anchor =
        native_probe::latest_amount_source(&core.state.active_battle.as_ref().unwrap().battle);
    let anchor_scope = anchor.resource_delivery().unwrap().scope.ordinal();
    let cold = delivery_scopes(&core);
    assert_eq!(
        cold.len(),
        1,
        "the pending branch owns one reservation, not a released child action"
    );
    assert_ne!(cold[0], anchor_scope);
    assert_eq!(hp(&core, &[actor]), 500);
    // Real World transaction cloning retains the cold child. Withdrawing in
    // just this clone revokes its unreleased branch; the original continues.
    let mut cancelled = core.clone();
    execute_current_battle_mutation(
        &mut cancelled,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    cancelled.advance_active_battle_for_server_tick(20).unwrap();
    assert_eq!(step_count(&cancelled), 1);
    assert!(delivery_scopes(&cancelled).is_empty());
    assert_eq!(
        native_probe::active_cast_count(&cancelled.state.active_battle.as_ref().unwrap().battle),
        0
    );
    assert!(!cancelled
        .battle_typed_resources()
        .unwrap()
        .scopes()
        .any(|(_, kind, _, _)| matches!(
            kind,
            ResourceLifetime::Chain | ResourceLifetime::Activation
        )));

    core.advance_active_battle_for_server_tick(5).unwrap();
    assert_eq!(
        hp(&core, &[actor]),
        507,
        "first actual child reads its Delivery initial7"
    );
    let first =
        native_probe::latest_amount_source(&core.state.active_battle.as_ref().unwrap().battle);
    let first_scope = first.resource_delivery().unwrap().scope.ordinal();
    assert_ne!(first_scope, anchor_scope);
    assert_ne!(first_scope, cold[0]);
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(
        hp(&core, &[actor]),
        514,
        "next child Action has its own Delivery initial7"
    );
    let second =
        native_probe::latest_amount_source(&core.state.active_battle.as_ref().unwrap().battle);
    let second_scope = second.resource_delivery().unwrap().scope.ordinal();
    assert!(![anchor_scope, cold[0], first_scope].contains(&second_scope));
    assert_eq!(step_count(&core), 3);
    assert!(delivery_scopes(&core).is_empty());
    assert!(!core
        .battle_typed_resources()
        .unwrap()
        .scopes()
        .any(|(_, kind, _, _)| matches!(
            kind,
            ResourceLifetime::Chain | ResourceLifetime::Activation
        )));
    let battle = &core.state.active_battle.as_ref().unwrap().battle;
    assert_eq!(native_probe::active_cast_count(battle), 0);
    assert!(first.relic_effective().is_some() && second.relic_effective().is_some());
    assert!(!native_probe::source_is_authorized(battle, &first));
    assert!(!native_probe::source_is_authorized(battle, &second));
    println!("TC001 resources anchor={anchor_scope} reservation={} first={first_scope} second={second_scope} heals=7,7 normal_cleanup=true clone_cancel_cleanup=true retained_plan_not_authority=true", cold[0]);
}
