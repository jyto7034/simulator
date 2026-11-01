use super::*;

#[test]
fn rv001_effect_gate_zero_positive_zero_never_reuses_prior_occurrence() {
    let action = r#"Action(id: "original_bite", step_id: "bite", wait: EffectsSettled)"#;
    let skills = skill_text(100).replace(
        action,
        &format!(r#"Repeat(id: "three", count: 3, interval_ms: 10, body: {action})"#),
    );
    let (mut core, player, employee, actor, targets) =
        prepare(&skills, &relic_text(Some(EFFECT_GATE)));
    let before = hp(&core, &targets);
    native_probe::legacy_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        targets[0],
        100,
    );
    cast(&mut core, player, employee);
    assert_eq!(hp(&core, &[actor]), 500);
    assert_eq!(before - hp(&core, &targets), 13);
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(hp(&core, &[actor]), 537);
    native_probe::legacy_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        targets[0],
        100,
    );
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(hp(&core, &[actor]), 537);
    assert_eq!(before - hp(&core, &targets), 139);
    assert_eq!(step_count(&core), 4);
    assert_eq!(
        native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
        0
    );
    println!("RV001 effect_gate bite_hp=0,100,0 sibling_hp=13,13,13 child_heals=0,37,0 final_enemy_loss139 caster537");
}

#[test]
fn rv001_simultaneous_sibling_children_keep_local_prefix_and_parent_frontier() {
    let skills = skill_text(100)
        .replace(
            r#"Action(id: "original_bite", step_id: "bite", wait: EffectsSettled)"#,
            r#"Sequence(id: "parent", children: [
            Action(id: "original_bite", step_id: "bite", wait: EffectsSettled),
            Action(id: "parent_heal", step_id: "parent_heal", wait: EffectsSettled)])"#,
        )
        .replace(
            "steps: [SkillStepDef",
            r#"steps: [
        (id: "parent_heal", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "parent_heal", op: HealExpr(amount: (moment: Live,
                root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))]),
        SkillStepDef"#,
        );
    let child = |rule: &str, damage: u32| {
        format!(
            r#"
        (id: "{rule}", selector: (provider: (fragment_id: Some("tc_fragment"))),
            apply: Grant(FiniteFlow(anchor_action_id: "bite", steps: [
                (id: "own", target: SelfUnit, delivery: Instant,
                    effects: [(id: "own", op: Damage(amount: {damage}, damage_type: True))]),
                (id: "heal", target: SelfUnit, delivery: Instant,
                    effects: [(id: "heal", op: HealExpr(amount: (moment: Live,
                        root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))]),
            ], flow: Sequence(id: "local", children: [
                Action(id: "own", step_id: "own", wait: EffectsSettled),
                Action(id: "heal", step_id: "heal", wait: EffectsSettled)]))))"#
        )
    };
    let base = relic_text(None);
    let rules = base.find("rules: [").unwrap();
    let relics = format!(
        "{}rules: [{}, {}], )])",
        &base[..rules],
        child("first", 5),
        child("second", 11)
    );
    let (mut core, player, employee, actor, targets) =
        prepare_with_generated_count(&skills, &relics, None, None, 4);
    let before = hp(&core, &targets);
    cast(&mut core, player, employee);
    assert_eq!(before - hp(&core, &targets), 113);
    assert_eq!(
        hp(&core, &[actor]),
        839,
        "parent heals113; child1 loses5/heals118; child2 loses11/heals124"
    );
    assert_eq!(step_count(&core), 6);
    assert_eq!(
        native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
        0
    );
    println!("RV001 simultaneous parent_heal=113 child1=minus5_plus118 child2=minus11_plus124 caster500_to839 enemy_loss113 scopes_isolated=true");
}
