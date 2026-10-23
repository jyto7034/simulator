use super::*;

fn step_events(core: &GameCore) -> Vec<(u64, String)> {
    core.state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .event_log
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::AbilityStepTriggered {
                skill_id, step_id, ..
            } if skill_id.as_str() == "tc_bite" => Some((entry.time_ms, step_id.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn tc001_released_long_anchor_preserves_parent_timing_and_child_prefix() {
    let skills = r#"SkillDatabase(skills: [SkillDef(
        id: "tc_bite", name: "released original and isolated child", kind: Targeted,
        focus_time_ms: 0, use_lifetime: Action,
        cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
        flow: Some(Sequence(id: "parent", children: [
            Action(id: "prior_node", step_id: "prior", wait: EffectsSettled),
            Action(id: "bite_node", step_id: "bite", wait: Released),
            Action(id: "later_node", step_id: "later", wait: EffectsSettled),
            Action(id: "parent_heal_node", step_id: "parent_heal", wait: EffectsSettled)
        ])),
        steps: [
            (id: "prior", delay_ms: 0, target: CastTarget, delivery: Instant, effects: [(id: "prior_damage", op: Damage(amount: 5, damage_type: True))]),
            (id: "bite", delay_ms: 0, target: CastTarget, delivery: TileArea(
                footprint: (include_anchor_tile: true, rows: ["@"]),
                area: (anchor: CastTarget, tile_origin: Anchor, tracking: GroundFixed,
                    hit_targets: Enemies, include_caster: false, tick_policy: OncePerArea,
                    duration_ms: 40, tick_interval_ms: Some(10))),
                effects: [(id: "bite_damage", op: Damage(amount: 7, damage_type: True))]),
            (id: "later", delay_ms: 0, target: CastTarget, delivery: Instant, effects: [(id: "later_damage", op: Damage(amount: 17, damage_type: True))]),
            (id: "parent_heal", delay_ms: 0, target: SelfUnit, delivery: Instant, effects: [(id: "parent_heal", op: HealExpr(amount: (moment: Live,
                root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))]),
        ]
    )])"#;
    // World tick 1 accepts/releases the queued cast; all authored delays are
    // relative to that actual release. Prior original 5 plus this anchor 7 are the child's permitted prefix.
    // The parent's later 17 settles before the 40 ms anchor closes, but is not
    // an allowed predecessor of that anchor. Legacy parent prefix remains 29.
    let relics = relic_text(None).replace("Heal(amount: 37)",
        "HealExpr(amount: (moment: Live, root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost))))");
    let (mut core, player, employee, actor, targets) = prepare(skills, &relics);
    let before = hp(&core, &targets);
    cast(&mut core, player, employee);
    assert_eq!(before - hp(&core, &targets), 29);
    assert_eq!(
        hp(&core, &[actor]),
        529,
        "parent continuation executes while anchor remains live"
    );
    assert_eq!(
        step_events(&core),
        vec![
            (1, "prior".into()),
            (1, "bite".into()),
            (1, "later".into()),
            (1, "parent_heal".into())
        ]
    );
    let battle = &core.state.active_battle.as_ref().unwrap().battle;
    assert_eq!(native_probe::active_cast_count(battle), 1);
    let ends: Vec<_> = battle.event_log.entries.iter().filter_map(|entry|
        matches!(&entry.event, BattleLogEvent::ManualCastEnd { caster_instance_id } if *caster_instance_id == actor).then_some(entry.time_ms)).collect();
    assert_eq!(
        ends,
        vec![1],
        "cold child does not extend original use control"
    );
    core.advance_active_battle_for_server_tick(38).unwrap();
    assert_eq!(hp(&core, &[actor]), 529);
    assert_eq!(step_count(&core), 4);
    core.advance_active_battle_for_server_tick(2).unwrap();
    assert_eq!(
        hp(&core, &[actor]),
        541,
        "child reads only prior5+anchor7 once"
    );
    assert_eq!(step_count(&core), 5);
    assert_eq!(step_events(&core).last().unwrap().0, 41);
    assert_eq!(
        native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
        0
    );
    assert_eq!(before - hp(&core, &targets), 29);
    println!("TC001 released parent_end=1 parent_prefix_heal=29 child_at=41 child_prefix_heal=12 actual_enemy_loss=29 cast_released=true");
}

#[test]
fn tc001_original_repeat_and_child_repeat_keep_effect_occurrences_independent() {
    let original = r#"Action(id: "original_bite", step_id: "bite", wait: EffectsSettled)"#;
    let skills = skill_text(100).replace(
        original,
        &format!(r#"Repeat(id: "original_twice", count: 2, interval_ms: 10, body: {original})"#),
    );
    let child = r#"Action(id: "heal_action", step_id: "heal_once", wait: EffectsSettled)"#;
    let relics = relic_text(Some(EFFECT_GATE)).replace(
        child,
        &format!(r#"Repeat(id: "child_twice", count: 2, interval_ms: 2, body: {child})"#),
    );
    let (mut core, player, employee, actor, targets) = prepare(&skills, &relics);
    // First bite is absorbed and only sibling damage reaches HP; second bite
    // loses 100 HP. Exactly the second occurrence owns two finite child heals.
    let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
    for target in &targets {
        native_probe::legacy_shield(battle, *target, 100);
    }
    let before = hp(&core, &targets);
    cast(&mut core, player, employee);
    assert_eq!(before - hp(&core, &targets), 13);
    assert_eq!(hp(&core, &[actor]), 500);
    assert_eq!(step_count(&core), 1);
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(before - hp(&core, &targets), 126);
    assert_eq!(hp(&core, &[actor]), 537);
    core.advance_active_battle_for_server_tick(2).unwrap();
    assert_eq!(hp(&core, &[actor]), 574);
    assert_eq!(step_count(&core), 4);
    assert_eq!(
        step_events(&core)
            .iter()
            .map(|row| row.0)
            .collect::<Vec<_>>(),
        vec![1, 11, 11, 13]
    );
    assert_eq!(
        native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
        0
    );
    println!("TC001 original_repeat=2 first_bite_hp=0 second_bite_hp=100 child_occurrences=2 heal=74 timestamps=1,11,11,13 cleanup=true");
}
