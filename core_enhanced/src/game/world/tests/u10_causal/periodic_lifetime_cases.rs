use super::*;

#[test]
fn u10_c_periodic_occurrences_are_independent_roots_without_active_fragment() {
    let periodic = r#"(id:"clock",apply:Periodic(interval_ms:50,payload:Program((steps:[
        (id:"pulse",target:Owner,effects:[(id:"pulse_damage",op:Damage(amount:1,damage_type:True))])
    ]))))"#;
    let rules = format!("{periodic},{}", damage_subscriber("echo", 7, 0));
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&rules),
        &["u10_relic"],
        false,
    );
    assert_eq!(
        fixture
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .causal_runtime_snapshot()
            .unwrap()
            .independent_periodic_count,
        1
    );
    let before = fixture.hp(fixture.actor);
    fixture.advance(49);
    assert_eq!(fixture.hp(fixture.actor), before);
    fixture.advance(1);
    assert_eq!(before - fixture.hp(fixture.actor), 8);
    fixture.advance(50);
    assert_eq!(
        before - fixture.hp(fixture.actor),
        16,
        "second independently produced pulse permits echo again"
    );
    execute_current_battle_mutation(
        &mut fixture.core,
        fixture.player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: fixture.employee,
        },
    )
    .unwrap();
    let after_withdrawal = fixture.target_hp();
    fixture.advance(100);
    assert_eq!(fixture.target_hp(), after_withdrawal);
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}

#[test]
fn u10_c_derived_dot_ticks_keep_ancestry_and_do_not_refresh_their_own_source() {
    let rule = r#"(id:"dot",apply:Subscribe((event:DamageHpLost(perspective:Dealt),
        payload:Program((steps:[(id:"apply_dot",target:EventTarget,effects:[
            (id:"dot",op:ApplyBuff(buff_id:"u10_dot",duration_ms:35))
        ])])))))"#;
    let buffs = r#"BuffDatabase(buffs:[(name:"u10_dot",
        kind:PeriodicDamage(damage_per_tick:3,damage_type:True),
        tick_interval_ms:10,max_stacks:1,reapply_policy:RefreshDuration)])"#;
    let mut fixture = CausalWorld::prepare_with_buffs(
        &skill_input(&[(0, &[1])]),
        &relic_input(rule),
        &["u10_relic"],
        true,
        Some(buffs),
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 1);
    fixture.advance(30);
    assert_eq!(
        before - fixture.target_hp(),
        10,
        "original 1 plus three actual DOT ticks of 3"
    );
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        10,
        "each DOT tick has already visited dot; it cannot reapply and extend its own lifetime"
    );
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}

#[test]
fn u10_c_detached_periodic_keeps_accepted_child_program_and_numeric_owner_only() {
    let periodic = r#"(id:"clock",apply:Periodic(interval_ms:50,payload:Program((steps:[
        (id:"accepted_pulse",delay_ms:30,target:Owner,effects:[
            (id:"pulse_damage",op:Damage(amount:10,damage_type:True))
        ])
    ]))))"#;
    let relics = relic_input(periodic).replacen(
        "effects:[]",
        r#"effects:[(id:"accepted_flat",flat:2,percent:0)]"#,
        1,
    );
    let mut fixture =
        CausalWorld::prepare(&skill_input(&[(0, &[1])]), &relics, &["u10_relic"], false);
    let before = fixture.hp(fixture.actor);
    fixture.advance(51);
    assert_eq!(
        fixture.hp(fixture.actor),
        before,
        "the first pulse is accepted but its delayed damage is pending"
    );
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    let staged = battle.causal_runtime_snapshot().unwrap();
    assert_eq!(staged.independent_periodic_count, 1);
    assert!(staged
        .contexts
        .iter()
        .any(|context| context.visited.iter().any(|(_, rule)| rule == "clock")));
    // Native lifecycle preparation removes current publication authority using
    // the normal owner-installation APIs. It does not alter the accepted cast,
    // queued delivery, source, or old numeric owner captured by that source.
    battle.set_effective_owner(None).unwrap();
    battle.install_relic_numeric_owner(None).unwrap();
    let detached = battle.causal_runtime_snapshot().unwrap();
    assert_eq!(detached.independent_periodic_count, 0);
    assert!(detached
        .contexts
        .iter()
        .any(|context| context.visited.iter().any(|(_, rule)| rule == "clock")));
    fixture.advance(30);
    assert_eq!(
        before - fixture.hp(fixture.actor),
        12,
        "accepted old program10 retains its captured flat2 after both current owners are removed"
    );
    fixture.advance(200);
    assert_eq!(
        before - fixture.hp(fixture.actor),
        12,
        "detached periodic authority cannot mint another pulse"
    );
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}

#[test]
fn u10_d_trace_modes_do_not_change_delayed_causal_damage_or_clone_progress() {
    for trace_mode in 0..3 {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&[(0, &[1])]),
            &relic_input(&damage_subscriber("delayed_echo", 7, 30)),
            &["u10_relic"],
            true,
        );
        existing_native_probe::observe_amounts_mode(
            &mut fixture.core.state.active_battle.as_mut().unwrap().battle,
            trace_mode,
        );
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(before - fixture.target_hp(), 1);
        let staged = fixture
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .causal_runtime_snapshot()
            .unwrap();
        assert!(staged.contexts.iter().any(|context| context
            .visited
            .iter()
            .any(|(_, rule)| rule == "delayed_echo")));
        let mut sibling = fixture.core.clone();
        crate::game::battle::core::relic_causal_tests::evict_all_existing_log_rows(
            &mut fixture.core.state.active_battle.as_mut().unwrap().battle,
            1,
        );
        fixture.advance(100);
        assert_eq!(before - fixture.target_hp(), 8, "trace mode {trace_mode}");
        let sibling_hp = |core: &GameCore| {
            fixture
                .targets
                .iter()
                .map(|target| {
                    core.state.active_battle.as_ref().unwrap().battle.units[target]
                        .stats
                        .current_health
                })
                .sum::<u32>()
        };
        assert_eq!(before - sibling_hp(&sibling), 1);
        sibling.advance_active_battle_for_server_tick(100).unwrap();
        assert_eq!(
            before - sibling_hp(&sibling),
            8,
            "trace mode {trace_mode} sibling"
        );
        assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
        assert_causal_reclaimed(&sibling.state.active_battle.as_ref().unwrap().battle);
    }
}

#[test]
fn u10_d_released_subscriber_projectile_keeps_ancestry_after_action_end() {
    let subscriber = damage_subscriber("projectile_echo", 7, 0).replace(
        "target:EventTarget,",
        "target:EventTarget,delivery:Projectile(speed_units_per_ms:100000),",
    );
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&subscriber),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        1,
        "released child has not impacted yet"
    );
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        8,
        "released child damage retains projectile_echo visit after its source action ends"
    );
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        8,
        "impact cannot produce another projectile chain"
    );
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}
