fn canonical_queen_fixture(edited: bool) -> Fixture {
    let mut doc = crate::authoring::composition::canonical_deferred_tests::resolved("queen-bee");
    // Preserve the canonical behavior; this low-level helper owns test identity
    // and capacity. Published World verifies unchanged IDs and actual copies.
    doc["skills"][0]["id"] = json!("authored-state");
    doc["skills"][0]["admission"]["capacity"] = json!(100);
    if edited {
        crate::authoring::composition::canonical_deferred_tests::edit_power(&mut doc, "queen-bee");
    }
    fixture(compile_json(&doc.to_string()).unwrap())
}
#[test]
fn canonical_queen_two_hostile_actions_heal_once_each_and_source_exit_keeps_lease() {
    for edited in [false, true] {
        let (mut core, mut state, actor, enemy, ally) = canonical_queen_fixture(edited);
        install(&mut core, &mut state, actor, ally);
        assert!(!core.resonance_use_has_obligations(actor));
        core.apply_live_command(
            crate::game::battle::core::sim::BattleLiveCommand::WithdrawUnit {
                unit_id: actor,
                time_ms: 1,
            },
        )
        .unwrap();
        attack(&mut core, &mut state, enemy, ally, 10).unwrap();
        assert_eq!(states(&core)[0].events, 1);
        assert_eq!(states(&core)[0].fields[0].value, StateScalar::Integer(1));
        assert_eq!(
            core.units[&ally].stats.current_health,
            if edited { 500 } else { 600 }
        );
        attack(&mut core, &mut state, enemy, ally, 20).unwrap();
        assert!(states(&core).is_empty());
        assert_eq!(
            core.units[&ally].stats.current_health,
            if edited { 400 } else { 600 }
        );
        attack(&mut core, &mut state, enemy, ally, 30).unwrap();
        assert_eq!(
            core.units[&ally].stats.current_health,
            if edited { 200 } else { 400 }
        );
    }
}
#[test]
fn canonical_queen_regrant_resets_charges_and_deadline_then_expiry_discards_unused_bees() {
    let (mut core, mut state, actor, enemy, ally) = canonical_queen_fixture(false);
    install(&mut core, &mut state, actor, ally);
    attack(&mut core, &mut state, enemy, ally, 10).unwrap();
    core.units.get_mut(&actor).unwrap().resonance_current = 100;
    core.request_skill_action(
        20,
        actor,
        "authored-state".into(),
        Some(SkillCastTarget::Unit {
            unit_instance_id: ally,
        }),
    )
    .unwrap();
    core.step_battle_execution_until(&mut state, 21).unwrap();
    assert_eq!(states(&core).len(), 1);
    assert_eq!(states(&core)[0].fields[0].value, StateScalar::Integer(2));
    core.step_battle_execution_until(&mut state, 6001).unwrap();
    assert_eq!(
        states(&core).len(),
        1,
        "old deadline must not end replacement"
    );
    core.step_battle_execution_until(&mut state, 6021).unwrap();
    assert!(states(&core).is_empty());
    let before = core.units[&ally].stats.current_health;
    attack(&mut core, &mut state, enemy, ally, 6030).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, before - 200);
}
