use super::*;

#[test]
fn u10_a_same_copy_effect_revisit_stops_but_next_independent_cast_runs() {
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&damage_subscriber("echo", 7, 0)),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        8,
        "original 1 plus one echo 7; echo's own positive damage cannot revisit it"
    );
    fixture.advance(20);
    assert_eq!(
        before - fixture.target_hp(),
        8,
        "no deferred self recursion"
    );
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    existing_native_probe::refill_skill(battle, fixture.actor, 21);
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        16,
        "the independent second cast has a fresh causal path"
    );
}

#[test]
fn u10_a_equal_definition_copies_and_distinct_rule_ids_are_independent_visits() {
    for (rules, equipped, expected) in [
        (
            damage_subscriber("echo", 7, 0),
            vec!["u10_relic", "u10_relic"],
            29,
        ),
        (
            format!(
                "{},{}",
                damage_subscriber("first", 7, 0),
                damage_subscriber("second", 11, 0)
            ),
            vec!["u10_relic"],
            37,
        ),
    ] {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&[(0, &[1])]),
            &relic_input(&rules),
            &equipped,
            true,
        );
        assert_eq!(
            fixture
                .copies
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len(),
            equipped.len()
        );
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        // Two sibling entries independently admit the other identity once:
        // copies: 1 + 7 + 7 + 7 + 7; rules: 1 + 7 + 11 + 11 + 7.
        assert_eq!(before - fixture.target_hp(), expected);
        fixture.advance(20);
        assert_eq!(before - fixture.target_hp(), expected);
    }
}

#[test]
fn u10_a_derived_delayed_damage_keeps_the_visited_identity() {
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&damage_subscriber("delayed_echo", 7, 30)),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 1);
    fixture.advance(30);
    assert_eq!(before - fixture.target_hp(), 8);
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        8,
        "a timer or new child action must not reset the subscribed rule visit"
    );
}

#[test]
fn u10_e_full_absorption_and_zero_damage_do_not_emit_hp_loss_subscriptions() {
    for (base, shield, expected) in [(100, 100, 0), (100, 99, 8), (0, 0, 0)] {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&[(0, &[base])]),
            &relic_input(&damage_subscriber("actual_loss", 7, 0)),
            &["u10_relic"],
            true,
        );
        if shield != 0 {
            let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
            for target in &fixture.targets {
                existing_native_probe::legacy_shield(battle, *target, shield);
            }
        }
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(
            before - fixture.target_hp(),
            expected,
            "base={base} shield={shield}"
        );
    }
}

#[test]
fn u10_a_recompiled_owner_and_plan_do_not_reset_same_copy_rule_visits() {
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&damage_subscriber("same_identity", 7, 30)),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    let first = fixture
        .core
        .capture_structural_battle_owner()
        .unwrap()
        .unwrap();
    let rebuilt = fixture
        .core
        .capture_structural_battle_owner()
        .unwrap()
        .unwrap();
    assert!(!Arc::ptr_eq(&first, &rebuilt));
    assert!(!Arc::ptr_eq(
        &first.plans[&fixture.employee],
        &rebuilt.plans[&fixture.employee]
    ));
    assert_eq!(
        first.plans[&fixture.employee].fingerprint,
        rebuilt.plans[&fixture.employee].fingerprint
    );
    // Native replacement consumes another normally compiled immutable plan;
    // it does not manufacture ancestry or grant a new authored rule identity.
    fixture
        .core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .set_effective_owner(Some(rebuilt))
        .unwrap();
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        8,
        "new owner and plan Arc identity must not allow the same copy/rule to revisit itself"
    );
}

#[test]
fn u10_e_healing_and_health_cost_are_not_damage_hp_loss_events() {
    let rule = r#"(id:"damage_only",apply:Subscribe((event:DamageHpLost(perspective:Any),
        payload:Program((steps:[(id:"heal",target:Owner,
            effects:[(id:"heal",op:Heal(amount:7))])])))))"#;
    for (operation, expected_hp) in [
        ("HealthCost(amount:100,allow_lethal:false)", 4900),
        ("Heal(amount:100)", 5100),
    ] {
        let skills = skill_input(&[(0,&[1])])
            .replace("Damage(amount:1,damage_type:True)",operation)
            .replace("kind:Targeted","kind:Untargeted")
            .replace("selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true",
                "selection:self,target:SelfUnit,reach:pattern,air_capable:false")
            .replace("target:CastTarget","target:SelfUnit");
        let mut fixture = CausalWorld::prepare(&skills, &relic_input(rule), &["u10_relic"], true);
        fixture.cast();
        fixture.advance(1);
        assert_eq!(fixture.hp(fixture.actor), expected_hp, "{operation}");
    }
}

#[test]
fn u10_e_overkill_commits_one_actual_death_and_keeps_damage_and_kill_facts_distinct() {
    let rule = r#"
        (id:"damage",apply:Subscribe((event:DamageHpLost(perspective:Dealt),
            payload:Program((steps:[(id:"damage_heal",target:Owner,
                effects:[(id:"heal",op:Heal(amount:7))])]))))),
        (id:"kill",apply:Subscribe((event:Kill,
            payload:Program((steps:[(id:"kill_heal",target:Owner,
                effects:[(id:"heal",op:Heal(amount:11))])])))))
    "#;
    let mut fixture = CausalWorld::prepare_full(
        &skill_input(&[(0, &[100, 100])]),
        &relic_input(rule),
        &["u10_relic"],
        true,
        None,
        Some(2),
    );
    let target = fixture.targets[0];
    fixture
        .core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .units
        .get_mut(&target)
        .unwrap()
        .stats
        .current_health = 10;
    fixture.cast();
    fixture.advance(1);
    assert_eq!(fixture.hp(target), 0);
    assert_eq!(fixture.hp(fixture.actor),5018,
        "one actual positive HP loss grants7, one actual death grants11; repeated dead-target input grants0");
}

#[test]
fn u10_a_fact_selector_and_same_effect_conditions_cannot_mix_segment_witnesses() {
    let skills = skill_input(&[(0, &[1, 1, 1])])
        .replace(
            r#"id:"damage_0_0",tags:["u10_direct","u10_multi_hit"]"#,
            r#"id:"damage_0_0",tags:["red"]"#,
        )
        .replace(
            r#"id:"damage_0_1",tags:["u10_direct","u10_multi_hit"]"#,
            r#"id:"damage_0_1",tags:["blue"]"#,
        )
        .replace(
            r#"id:"damage_0_2",tags:["u10_direct","u10_multi_hit"]"#,
            r#"id:"damage_0_2",tags:["red","blue"]"#,
        );
    for (condition, expected) in [
        (r#"selector:(effect:(all_tags:["blue"])),"#, 10),
        (r#"when:SameEffect((all_tags:["blue"])),"#, 10),
        (r#"when:Exists((effect:(all_tags:["blue"]))),"#, 17),
    ] {
        let rule = format!(
            r#"(id:"filtered",{condition}apply:Subscribe((
            event:DamageHpLost(perspective:Dealt),source:(all_tags:["red"]),
            payload:Program((steps:[(id:"child",target:EventTarget,
                effects:[(id:"damage",op:Damage(amount:7,damage_type:True))])])))))"#
        );
        let mut fixture = CausalWorld::prepare(&skills, &relic_input(&rule), &["u10_relic"], true);
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(before - fixture.target_hp(), expected, "{condition}");
    }
}

#[test]
fn u10_e_each_positive_damage_fact_can_subscribe_even_within_one_strike() {
    for (shield, expected_loss) in [(0, 44), (10, 27), (29, 8), (30, 0)] {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&[(0, &[10, 20])]),
            &relic_input(&damage_subscriber("per_fact", 7, 0)),
            &["u10_relic"],
            true,
        );
        if shield != 0 {
            let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
            for target in &fixture.targets {
                existing_native_probe::legacy_shield(battle, *target, shield);
            }
        }
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(
            before - fixture.target_hp(),
            expected_loss,
            "shield={shield}"
        );
    }
}

#[test]
fn u10_a_basic_attack_subscription_without_fragment_is_active_and_queries_are_pure() {
    let rule = damage_subscriber("basic_echo", 7, 0).replace(
        "perspective:Dealt),",
        "perspective:Dealt),source:(target:BasicAttackDamage),",
    );
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&rule),
        &["u10_relic"],
        false,
    );
    let before = existing_native_probe::flow_roots(
        &fixture.core.state.active_battle.as_ref().unwrap().battle,
    );
    for _ in 0..3 {
        let result = fixture
            .core
            .execute(
                fixture.player,
                PlayerBehavior::RequestRelicEffectivePlan {
                    employee_uuid: fixture.employee,
                },
            )
            .unwrap();
        let BehaviorResult::RelicEffectivePlan { result: Some(plan) } = result else {
            panic!("active no-fragment subscriber plan must remain queryable");
        };
        assert!(plan.providers.is_empty());
        assert_eq!(plan.causal_rules.len(), 1);
        assert_eq!(plan.causal_rules[0].contribution.rule_id, "basic_echo");
    }
    assert_eq!(
        existing_native_probe::flow_roots(
            &fixture.core.state.active_battle.as_ref().unwrap().battle
        ),
        before
    );
    let before_hp = fixture.target_hp();
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    battle.units.get_mut(&fixture.actor).unwrap().stats.attack = 10;
    crate::game::battle::core::relic_causal_tests::actual_basic_attack_transaction(
        battle,
        fixture.actor,
        fixture.targets[0],
        0,
    )
    .unwrap();
    fixture.advance(1);
    assert_eq!(
        before_hp - fixture.target_hp(),
        17,
        "normal committed basic damage10 can trigger its equipped copy's echo7 without a provider"
    );
}

#[test]
fn u10_a_same_copy_rule_survives_form_and_fingerprint_replacement() {
    let normal = damage_subscriber("same_rule", 7, 30);
    let corrupt = damage_subscriber("same_rule", 11, 0);
    let relics = relic_input(&normal).replace(
        "allow_duplicates:true,effects:[]",
        &format!(
            r#"allow_duplicates:true,corruption:Some((effects:[],rules:[{corrupt}],
            penalties:[DefenseFlat(amount:-1)])),effects:[]"#
        ),
    );
    let mut fixture =
        CausalWorld::prepare(&skill_input(&[(0, &[1])]), &relics, &["u10_relic"], true);
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    let old = fixture
        .core
        .capture_structural_battle_owner()
        .unwrap()
        .unwrap();
    // Native state preparation selects another already validated authored form.
    // The pending source and its accepted ancestry remain untouched.
    fixture
        .core
        .state
        .relics
        .corrupt(fixture.copies[0], &fixture.core.game_data.relic_data)
        .unwrap();
    let changed = fixture
        .core
        .capture_structural_battle_owner()
        .unwrap()
        .unwrap();
    assert_ne!(
        old.plans[&fixture.employee].fingerprint,
        changed.plans[&fixture.employee].fingerprint
    );
    assert_eq!(
        changed.plans[&fixture.employee].causal_rules[0]
            .contribution
            .copy_uuid,
        fixture.copies[0]
    );
    assert_eq!(
        changed.plans[&fixture.employee].causal_rules[0]
            .contribution
            .rule_id,
        "same_rule"
    );
    fixture
        .core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .set_effective_owner(Some(changed))
        .unwrap();
    fixture.advance(100);
    assert_eq!(before - fixture.target_hp(),8,
        "accepted normal-form child7 retains its visit; current corrupt-form child11 must not reenter");
    existing_native_probe::refill_skill(
        &mut fixture.core.state.active_battle.as_mut().unwrap().battle,
        fixture.actor,
        101,
    );
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        20,
        "a new independent cast uses the current corrupt form: original1 plus child11"
    );
}

#[test]
fn u10_a_same_copy_rule_visit_survives_rebinding_to_another_employee() {
    let rule =
        damage_subscriber("transferred", 7, 30).replace("perspective:Dealt", "perspective:Any");
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&rule),
        &["u10_relic"],
        true,
    );
    let (other, other_actor) = fixture.deploy_other_employee();
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 1);
    // Select a valid differently owned plan as native lifecycle preparation.
    // This does not claim that World permits equipment changes during battle.
    fixture
        .core
        .state
        .relics
        .equip(
            fixture.copies[0],
            other,
            0,
            &fixture.core.game_data.relic_data,
        )
        .unwrap();
    let replacement = fixture
        .core
        .capture_structural_battle_owner()
        .unwrap()
        .unwrap();
    assert_eq!(
        replacement.plans[&other].causal_rules[0]
            .contribution
            .copy_uuid,
        fixture.copies[0]
    );
    fixture
        .core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .set_effective_owner(Some(replacement))
        .unwrap();
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        8,
        "the same copy/rule on another employee cannot reenter its old accepted ancestry"
    );
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    battle.units.get_mut(&other_actor).unwrap().stats.attack = 1;
    crate::game::battle::core::relic_causal_tests::actual_basic_attack_transaction(
        battle,
        other_actor,
        fixture.targets[0],
        101,
    )
    .unwrap();
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        16,
        "the transferred copy can subscribe to another employee's new independent basic hit"
    );
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}

#[test]
fn u10_e_no_fragment_dead_owner_death_subscriber_uses_actual_killer_once() {
    let rule = r#"(id:"death_echo",apply:Subscribe((event:Death,
        payload:Program((steps:[(id:"dead_owner",target:Killer,
            effects:[(id:"damage",op:Damage(amount:7,damage_type:True))])])))))"#;
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(rule),
        &["u10_relic"],
        false,
    );
    fixture.deploy_other_employee();
    let killer = fixture.targets[0];
    let before = fixture.hp(killer);
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    battle
        .units
        .get_mut(&fixture.actor)
        .unwrap()
        .stats
        .current_health = 1;
    battle.units.get_mut(&fixture.actor).unwrap().stats.defense = 0;
    battle.units.get_mut(&killer).unwrap().stats.attack = 1;
    crate::game::battle::core::relic_causal_tests::actual_basic_attack_transaction(
        battle,
        killer,
        fixture.actor,
        0,
    )
    .unwrap();
    fixture.advance(1);
    assert_eq!(fixture.hp(fixture.actor), 0);
    assert_eq!(before - fixture.hp(killer),7,
        "a real Death occurrence permits its dead owner and resolves Killer from the committed death fact");
    fixture.advance(100);
    assert_eq!(before - fixture.hp(killer), 7);
    assert_eq!(
        fixture
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .runtime_fact_summary()
            .deaths,
        1
    );
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}

#[test]
fn u10_a_static_dormancy_and_event_conditions_preserve_no_fragment_active_consumers() {
    let basic = damage_subscriber("always_basic", 7, 0);
    let absent_provider = damage_subscriber("absent_provider", 11, 0).replacen(
        "apply:Subscribe",
        r#"selector:(provider:(fragment_id:Some("u10_fragment"))),apply:Subscribe"#,
        1,
    );
    let absent_effect = damage_subscriber("absent_effect", 13, 0).replacen(
        "apply:Subscribe",
        r#"when:Exists((effect:(all_tags:["unprovided_tag"]))),apply:Subscribe"#,
        1,
    );
    let event_condition = damage_subscriber("event_condition", 17, 0).replacen(
        "apply:Subscribe",
        r#"when:SameEffect((all_tags:["actual_event_tag"])),apply:Subscribe"#,
        1,
    );
    let periodic = |id: &str, condition: &str, amount: u32| {
        format!(
            r#"(id:"{id}",{condition}
        apply:Periodic(interval_ms:50,payload:Program((steps:[(id:"cost",target:Owner,
            effects:[(id:"cost",op:HealthCost(amount:{amount},allow_lethal:false))])]))))"#
        )
    };
    let rules = format!(
        "{basic},{absent_provider},{absent_effect},{event_condition},{},{},{}",
        periodic("always_clock", "", 1),
        periodic(
            "absent_clock_effect",
            r#"when:Exists((effect:(all_tags:["unprovided_tag"]))),"#,
            19
        ),
        periodic(
            "absent_clock_provider",
            r#"selector:(provider:(fragment_id:Some("u10_fragment"))),"#,
            23
        ),
    );
    let mut fixture = CausalWorld::prepare_with_unowned_fragment(
        &skill_input(&[(0, &[1])]),
        &relic_input(&rules),
        &["u10_relic"],
        false,
        None,
        None,
        true,
    );
    let before_query = causal_roots(&fixture.core.state.active_battle.as_ref().unwrap().battle);
    for _ in 0..3 {
        let result = fixture
            .core
            .execute(
                fixture.player,
                PlayerBehavior::RequestRelicEffectivePlan {
                    employee_uuid: fixture.employee,
                },
            )
            .unwrap();
        let BehaviorResult::RelicEffectivePlan { result: Some(plan) } = result else {
            panic!("normal owner query must remain available with no equipped provider");
        };
        assert!(plan.providers.is_empty());
        let states: std::collections::BTreeMap<_, _> = plan
            .rules
            .iter()
            .map(|rule| {
                (
                    rule.contribution.rule_id.as_str(),
                    serde_json::to_value(&rule.state).unwrap(),
                )
            })
            .collect();
        for active in ["always_basic", "always_clock"] {
            assert_eq!(states[active], serde_json::json!("Active"));
        }
        for dormant in ["absent_provider", "absent_clock_provider"] {
            assert_eq!(
                states[dormant],
                serde_json::json!({"Dormant":{"reason":"ProviderUnavailable"}})
            );
        }
        for dormant in ["absent_effect", "absent_clock_effect"] {
            assert_eq!(
                states[dormant],
                serde_json::json!({"Dormant":{"reason":"ConditionUnmet"}})
            );
        }
        assert_eq!(
            states["event_condition"],
            serde_json::json!({"Pending":{"reason":"ActualEventPredicate"}}),
            "an actual-event condition cannot be reported as statically witnessed Active"
        );
    }
    assert_eq!(
        causal_roots(&fixture.core.state.active_battle.as_ref().unwrap().battle),
        before_query
    );
    let before_hp = fixture.target_hp();
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    battle.units.get_mut(&fixture.actor).unwrap().stats.attack = 1;
    crate::game::battle::core::relic_causal_tests::actual_basic_attack_transaction(
        battle,
        fixture.actor,
        fixture.targets[0],
        0,
    )
    .unwrap();
    fixture.advance(1);
    assert_eq!(before_hp - fixture.target_hp(),8,
        "only Always basic activates; missing provider/Exists and false actual event predicate produce no child");
    assert_eq!(fixture.hp(fixture.actor), 5000);
    fixture.advance(49);
    assert_eq!(
        fixture.hp(fixture.actor),
        4999,
        "only the no-fragment Always periodic producer pays1; dormant periodic rules pay0"
    );
    assert_eq!(before_hp - fixture.target_hp(), 8);
}

#[test]
fn u10_e_taken_delayed_child_preserves_public_original_source_and_actual_killer_after_log_eviction()
{
    let rules = r#"
        (id:"taken_echo",apply:Subscribe((event:DamageHpLost(perspective:Taken),
            payload:Program((steps:[(id:"retaliate",delay_ms:30,target:EventSource,
                effects:[(id:"damage",op:Damage(amount:7,damage_type:True))])]))))),
        (id:"kill_heal",apply:Subscribe((event:Kill,
            payload:Program((steps:[(id:"kill_reward",delay_ms:10,target:Owner,
                effects:[(id:"heal",op:Heal(amount:11))])])))))
    "#;
    let mut fixture = CausalWorld::prepare_full(
        &skill_input(&[(0, &[1])]),
        &relic_input(rules),
        &["u10_relic"],
        true,
        None,
        Some(2),
    );
    let attacker = fixture.targets[0];
    assert_ne!(attacker, fixture.actor);
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    battle.units.get_mut(&attacker).unwrap().stats.attack = 1;
    battle
        .units
        .get_mut(&attacker)
        .unwrap()
        .stats
        .current_health = 7;
    battle.units.get_mut(&fixture.actor).unwrap().stats.defense = 0;
    crate::game::battle::core::relic_causal_tests::actual_basic_attack_transaction(
        battle,
        attacker,
        fixture.actor,
        0,
    )
    .unwrap();
    let original = serde_json::to_value(
        &battle
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap()
            .source,
    )
    .unwrap();
    assert_eq!(original["actor"], serde_json::json!(attacker));
    fixture.advance(1);
    assert_eq!(fixture.hp(fixture.actor), 4999);
    crate::game::battle::core::relic_causal_tests::evict_all_existing_log_rows(
        &mut fixture.core.state.active_battle.as_mut().unwrap().battle,
        1,
    );
    fixture.advance(30);
    assert_eq!(fixture.hp(attacker), 0);
    assert_eq!(
        fixture.hp(fixture.actor),
        4999,
        "Kill child is still delayed"
    );
    let battle = &fixture.core.state.active_battle.as_ref().unwrap().battle;
    let child = serde_json::to_value(
        &battle
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap()
            .source,
    )
    .unwrap();
    assert_eq!(child["actor"], serde_json::json!(fixture.actor));
    let cause = &child["causal_cause"];
    assert_eq!(cause["owner"], serde_json::json!(fixture.actor));
    assert_eq!(cause["copy_uuid"], serde_json::json!(fixture.copies[0]));
    assert_eq!(cause["definition_id"], "u10_relic");
    assert_eq!(cause["rule_id"], "taken_echo");
    assert_eq!(
        cause["event"],
        serde_json::json!({"DamageHpLost":{"perspective":"Taken"}})
    );
    assert_eq!(cause["event_target"], serde_json::json!(fixture.actor));
    assert!(cause["actual_killer"].is_null());
    for field in ["actor", "equipment", "fragment", "cast", "authored"] {
        assert_eq!(cause["root_source"][field], original[field], "root {field}");
        assert_eq!(
            cause["triggering_source"][field], original[field],
            "trigger {field}"
        );
    }
    assert_eq!(child["authored"]["site"]["effect_id"], "damage");
    assert_eq!(
        battle
            .runtime_fact_summary()
            .latest_death
            .as_ref()
            .unwrap()
            .predeath
            .victim,
        attacker
    );
    assert_eq!(
        battle
            .runtime_fact_summary()
            .latest_death
            .as_ref()
            .unwrap()
            .source
            .actor,
        Some(fixture.actor)
    );
    fixture.advance(20);
    assert_eq!(fixture.hp(fixture.actor), 5010);
    let battle = &fixture.core.state.active_battle.as_ref().unwrap().battle;
    let heal = serde_json::to_value(
        &battle
            .runtime_fact_summary()
            .latest_heal
            .as_ref()
            .unwrap()
            .source,
    )
    .unwrap();
    let kill = &heal["causal_cause"];
    assert_eq!(kill["root"], cause["root"]);
    assert_eq!(kill["owner"], serde_json::json!(fixture.actor));
    assert_eq!(kill["actual_killer"], serde_json::json!(fixture.actor));
    assert_eq!(kill["event_target"], serde_json::json!(attacker));
    assert_eq!(kill["copy_uuid"], serde_json::json!(fixture.copies[0]));
    assert_eq!(kill["rule_id"], "kill_heal");
    assert_eq!(kill["event"], "Kill");
    for field in ["actor", "equipment", "fragment", "cast", "authored"] {
        assert_eq!(
            kill["root_source"][field], original[field],
            "kill root {field}"
        );
        assert_eq!(
            kill["triggering_source"][field], child[field],
            "kill trigger {field}"
        );
    }
    // The public cause keeps exactly two flat fact descriptions. It does not
    // expose a recursive source chain or an executable action/resource token.
    for observed in [cause, kill] {
        for fact in ["root_source", "triggering_source"] {
            let fields = observed[fact].as_object().unwrap();
            assert_eq!(fields.len(), 5);
            for forbidden in [
                "action_ordinal",
                "delivery",
                "program",
                "causal_cause",
                "resource_capabilities",
            ] {
                assert!(!fields.contains_key(forbidden));
            }
        }
    }
    assert_causal_reclaimed(battle);
}
