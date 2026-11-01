fn canonical_rudolta_fixture() -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut doc = crate::authoring::composition::canonical_deferred_tests::resolved("rudolta");
    doc["skills"][0]["id"] = serde_json::json!("scheduled");
    doc["skills"][0]["admission"]["capacity"] = serde_json::json!(100);
    let artifact = crate::authoring::composition::compile_json(&doc.to_string()).unwrap();
    let native = crate::authoring::composition::Artifact::from_ron(&artifact.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let (mut core, state, actor, enemy, ally) = native_action_fixture(
        native.get_by_id("scheduled").unwrap().clone(),
        Default::default(),
    );
    let unit = core.units.get_mut(&actor).unwrap();
    unit.source_identity = BattleUnitSourceIdentity::Employee {
        employee_uuid: actor.as_uuid(),
        base_uuid: unit.base_uuid,
    };
    unit.stats.attack = 100;
    for id in [enemy, ally] {
        let unit = core.units.get_mut(&id).unwrap();
        unit.stats.max_health = 1000;
        unit.stats.current_health = 1000;
    }
    (core, state, actor, enemy, ally)
}
#[test]
fn canonical_rudolta_three_drops_sample_power_at_release_and_allow_basic_attack() {
    let (mut core, mut state, actor, enemy, _) = canonical_rudolta_fixture();
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert!(!core.resonance_use_has_obligations(actor));
    assert!(core.units[&actor].action_locks.can_basic_attack(1));
    core.units.get_mut(&actor).unwrap().stats.attack = 200;
    core.units
        .get_mut(&actor)
        .unwrap()
        .basic_attack_speed_percent = 900;
    core.step_battle_execution_until(&mut state, 101).unwrap();
    core.units.get_mut(&actor).unwrap().stats.attack = 300;
    core.step_battle_execution_until(&mut state, 202).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    core.step_battle_execution_until(&mut state, 302).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 550);
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 100);
    assert!(core.active_areas.is_empty());
    assert_eq!(
        core.query_action(&action).unwrap().facts.damage_hp_lost,
        900
    );
}
#[test]
fn canonical_rudolta_recast_replaces_future_drops_but_preserves_already_falling_gift() {
    let (mut core, mut state, actor, enemy, _) = canonical_rudolta_fixture();
    let first = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 50).unwrap();
    core.units.get_mut(&actor).unwrap().resonance_current = 100;
    let second = core
        .request_skill_action(50, actor, "scheduled".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 452).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 400);
    assert_eq!(core.query_action(&first).unwrap().facts.damage_hp_lost, 150);
    assert_eq!(
        core.query_action(&second).unwrap().facts.damage_hp_lost,
        450
    );
    assert!(core.active_areas.is_empty());
}
