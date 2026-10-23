use super::*;

fn tagged_program(step: &str, amount: u32, delay: u32, tags: &str) -> String {
    format!(
        r#"(steps:[(id:"{step}",delay_ms:{delay},target:EventTarget,
        effects:[(id:"damage",tags:[{tags}],op:Damage(amount:{amount},damage_type:True))])])"#
    )
}

fn tagged_subscriber(id: &str, tag: &str, payload: &str) -> String {
    format!(
        r#"(id:"{id}",apply:Subscribe((event:DamageHpLost(perspective:Dealt),
        source:(all_tags:["{tag}"]),payload:{payload})))"#
    )
}

pub(super) fn branch_pair(join: bool) -> String {
    let left = tagged_program("left_hit", 5, 0, r#""branch","u10_direct""#);
    let right = tagged_program("right_hit", 13, 20, r#""branch","u10_direct""#);
    let branches = format!(r#"branches:[(id:"left",program:{left}),(id:"right",program:{right})]"#);
    let root_payload = if join {
        let then = tagged_program(
            "joined_hit",
            11,
            0,
            r#""branch","u10_direct","joined","d_trigger""#,
        );
        format!("Join({branches},then:{then})")
    } else {
        format!("Fork({branches})")
    };
    let root = tagged_subscriber("A", "u10_direct", &root_payload);
    let b = tagged_subscriber(
        "B",
        "branch",
        &format!("Program({})", tagged_program("b_hit", 7, 30, r#""branch""#)),
    );
    format!("{root},{b}")
}

#[test]
fn u10_b_real_fork_siblings_inherit_a_but_each_may_visit_b_once() {
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&branch_pair(false)),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 6);
    fixture.advance(20);
    assert_eq!(before - fixture.target_hp(), 19);
    fixture.advance(40);
    assert_eq!(
        before - fixture.target_hp(),
        33,
        "1 + left 5 + right 13 + independently admitted B 7 on each sibling"
    );
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        33,
        "neither original A nor already visited B may recur within a branch"
    );
}

#[test]
fn u10_b_real_join_waits_for_descendants_then_uses_participant_union() {
    let c = tagged_subscriber(
        "C",
        "joined",
        &format!("Program({})", tagged_program("c_hit", 17, 0, "")),
    );
    let rules = format!("{},{c}", branch_pair(true));
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&rules),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 6);
    fixture.advance(20);
    assert_eq!(
        before - fixture.target_hp(),
        19,
        "both branch root programs finished, but two actual B children remain"
    );
    let waiting = fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .causal_runtime_snapshot()
        .unwrap();
    assert_eq!(waiting.joins.len(), 1);
    assert_eq!(waiting.joins[0].rule_id, "A");
    assert_eq!(waiting.joins[0].participants.len(), 2);
    for participant in &waiting.joins[0].participants {
        assert!(participant.started && participant.live_carriers > 0);
        assert_eq!(
            participant
                .visited
                .iter()
                .map(|(_, rule)| rule.as_str())
                .collect::<Vec<_>>(),
            vec!["A", "B"]
        );
    }
    fixture.advance(10);
    assert_eq!(
        before - fixture.target_hp(),
        26,
        "left receipt cannot stand in for the unfinished right participant"
    );
    fixture.advance(30);
    assert_eq!(
        before - fixture.target_hp(),
        61,
        "1 + 5 + 13 + 7 + 7 + own joined damage 11 + unvisited C 17"
    );
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        61,
        "join's tags match A and B, but their actual participant visits suppress re-entry"
    );
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}

#[test]
fn u10_b_join_excludes_same_clock_nonparticipant_and_clones_waiting_state() {
    let d = tagged_subscriber(
        "D",
        "d_trigger",
        &format!(
            "Program({})",
            tagged_program("d_hit", 17, 0, r#""d_trigger""#)
        ),
    );
    let rules = format!("{},{d}", branch_pair(true));
    let skills = skill_input(&[(0, &[1]), (0, &[1])]);
    let needle = r#"id:"damage_1_0",tags:["u10_direct","u10_multi_hit"]"#;
    assert!(skills.contains(needle));
    let skills = skills.replacen(needle, r#"id:"damage_1_0",tags:["d_trigger"]"#, 1);
    let mut fixture = CausalWorld::prepare(&skills, &relic_input(&rules), &["u10_relic"], true);
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        24,
        "A branch 1+5 and unrelated original 1 plus D17 coexist at one clock"
    );
    let mut sibling = fixture.core.clone();
    fixture.advance(60);
    assert_eq!(
        before - fixture.target_hp(),
        79,
        "joined D17 remains allowed: nonparticipant D visit must not enter the union"
    );
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
    assert_eq!(
        before - sibling_hp(&sibling),
        24,
        "clone still owns an unfinished join"
    );
    sibling.advance_active_battle_for_server_tick(60).unwrap();
    assert_eq!(
        before - sibling_hp(&sibling),
        79,
        "one clone's participant consumption cannot consume the other's receipts"
    );
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        79,
        "joined payload committed once"
    );
}

#[test]
fn u10_b_local_bucket_descendants_hold_join_until_their_actual_visits_complete() {
    let left = tagged_program("left", 5, 20, r#""branch""#);
    let right = tagged_program("right", 13, 20, r#""branch""#);
    let then = tagged_program("joined", 11, 0, r#""u10_direct","branch","late""#);
    let a = tagged_subscriber(
        "A",
        "u10_direct",
        &format!(
            r#"Join(branches:[(id:"left",program:{left}),(id:"right",program:{right})],then:{then})"#
        ),
    );
    let b = tagged_subscriber(
        "B",
        "branch",
        &format!("Program({})", tagged_program("b", 7, 10, r#""late""#)),
    );
    let c = tagged_subscriber(
        "C",
        "late",
        &format!("Program({})", tagged_program("c", 17, 10, "")),
    );
    let skills = skill_input(&[(0, &[1]), (20, &[1])]);
    let needle = r#"(id:"damage_1_0",tags:["u10_direct","u10_multi_hit"],op:Damage(amount:1,damage_type:True))"#;
    assert!(skills.contains(needle));
    let skills = skills.replacen(
        needle,
        r#"(id:"nonparticipant_buff",op:ApplyBuff(buff_id:"u10_bucket_marker",duration_ms:1))"#,
        1,
    );
    // This genuine queued ApplyBuff has no Join membership. Its tick interval
    // exceeds its lifetime, so it contributes no damage or extra subscriber.
    // At t20 its higher-priority event runs before the two pending B invokes,
    // after the branch casts have finished and left the normal live owners.
    let buffs = r#"BuffDatabase(buffs:[(name:"u10_bucket_marker",
        kind:PeriodicDamage(damage_per_tick:3,damage_type:True),
        tick_interval_ms:1000,max_stacks:1,reapply_policy:RefreshDuration)])"#;
    let mut fixture = CausalWorld::prepare_with_buffs(
        &skills,
        &relic_input(&format!("{a},{b},{c}")),
        &["u10_relic"],
        true,
        Some(buffs),
    );
    let b_skill = {
        let owner = fixture
            .core
            .capture_structural_battle_owner()
            .unwrap()
            .unwrap();
        owner.plans[&fixture.employee]
            .causal_rules
            .iter()
            .find(|rule| rule.contribution.rule_id == "B")
            .unwrap()
            .programs[0]
            .execution
            .skill
            .id
            .clone()
    };
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 1);
    fixture.advance(20);
    assert_eq!(
        before - fixture.target_hp(),
        19,
        "original1 + branches5/13; pending local-bucket B invokes must prevent an early joined11"
    );
    let battle = &fixture.core.state.active_battle.as_ref().unwrap().battle;
    let marker = crate::game::battle::buffs::BuffId::from_name("u10_bucket_marker");
    let applications: Vec<_> = battle
        .event_log
        .entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| match &entry.event {
            crate::game::battle::event_log::BattleLogEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
            } if *caster_instance_id == fixture.actor
                && *target_instance_id == fixture.targets[0]
                && *buff_id == marker
                && *duration_ms == 1 =>
            {
                Some((index, entry.time_ms))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        applications.len(),
        1,
        "the normal nonparticipant ApplyBuff boundary must actually execute"
    );
    let invokes: Vec<_> = battle
        .event_log
        .entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| match &entry.event {
            crate::game::battle::event_log::BattleLogEvent::AbilityCast {
                caster_instance_id,
                skill_id,
                ..
            } if *caster_instance_id == fixture.actor && *skill_id == b_skill => {
                Some((index, entry.time_ms))
            }
            _ => None,
        })
        .collect();
    assert_eq!(invokes.len(), 2);
    assert!(
        invokes
            .iter()
            .all(|(index, time)| *index > applications[0].0 && *time == applications[0].1),
        "the genuine ApplyBuff settles before both B invokes in the same bucket"
    );
    let waiting = fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .causal_runtime_snapshot()
        .unwrap();
    assert_eq!(waiting.joins.len(), 1);
    assert_eq!(waiting.joins[0].participants.len(), 2);
    for participant in &waiting.joins[0].participants {
        assert!(participant.started && !participant.cancelled && participant.live_carriers > 0);
        assert_eq!(
            participant
                .visited
                .iter()
                .map(|(_, rule)| rule.as_str())
                .collect::<Vec<_>>(),
            vec!["A", "B"]
        );
    }
    fixture.advance(10);
    assert_eq!(
        before - fixture.target_hp(),
        33,
        "both B7 descendants commit before their C children"
    );
    let waiting = fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .causal_runtime_snapshot()
        .unwrap();
    assert_eq!(waiting.joins.len(), 1);
    for participant in &waiting.joins[0].participants {
        assert!(participant.started && !participant.cancelled && participant.live_carriers > 0);
        assert_eq!(
            participant
                .visited
                .iter()
                .map(|(_, rule)| rule.as_str())
                .collect::<Vec<_>>(),
            vec!["A", "B", "C"]
        );
    }
    fixture.advance(10);
    assert_eq!(
        before - fixture.target_hp(),
        78,
        "two actual C17 children precede joined11; their participant union suppresses joined A/B/C re-entry"
    );
    fixture.advance(100);
    assert_eq!(
        before - fixture.target_hp(),
        78,
        "joined payload commits once"
    );
    assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
}

#[test]
fn u10_b_cancelled_incomplete_participant_never_commits_joined_payload() {
    for (cancel_at, expected_prefix) in [(1, 6), (31, 26)] {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&[(0, &[1])]),
            &relic_input(&branch_pair(true)),
            &["u10_relic"],
            true,
        );
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(cancel_at);
        assert_eq!(before - fixture.target_hp(), expected_prefix);
        let mut control = fixture.core.clone();
        execute_current_battle_mutation(
            &mut fixture.core,
            fixture.player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: fixture.employee,
            },
        )
        .unwrap();
        fixture.advance(100);
        assert_eq!(
            before - fixture.target_hp(),
            expected_prefix,
            "cancel_at={cancel_at}: no synthetic successful receipt for an unreleased child"
        );
        assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
        control.advance_active_battle_for_server_tick(100).unwrap();
        let control_hp: u32 = fixture
            .targets
            .iter()
            .map(|target| {
                control.state.active_battle.as_ref().unwrap().battle.units[target]
                    .stats
                    .current_health
            })
            .sum();
        assert_eq!(
            before - control_hp,
            44,
            "uncancelled sibling still consumes both real participant completions once"
        );
    }
}

#[test]
fn u10_b_authored_then_branch_and_join_continuation_have_distinct_program_and_amount_identity() {
    let branch = tagged_program("same_step", 3, 0, "");
    let continuation = tagged_program("same_step", 7, 20, "");
    let rule = tagged_subscriber(
        "same_local_names",
        "u10_direct",
        &format!(r#"Join(branches:[(id:"then",program:{branch})],then:{continuation})"#,),
    );
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&rule),
        &["u10_relic"],
        true,
    );
    let owner = fixture
        .core
        .capture_structural_battle_owner()
        .unwrap()
        .unwrap();
    let programs = &owner.plans[&fixture.employee].causal_rules[0].programs;
    assert_eq!(programs.len(), 2);
    assert_ne!(
        programs[0].execution.skill.id, programs[1].execution.skill.id,
        "an authored branch named then must not share the synthetic continuation identity"
    );
    let amount_sites: Vec<_> = programs
        .iter()
        .map(|program| {
            assert_eq!(program.execution.amounts.len(), 1);
            let (site, entry) = program.execution.amounts.entries().next().unwrap();
            assert_eq!(site.action.definition_id, program.execution.skill.id);
            assert_eq!(site.action.action_id, "same_step");
            assert_eq!(site.effect_id, "damage");
            assert_eq!(
                entry.kind(),
                crate::game::value_expr::AmountPlanKind::Damage
            );
            site.clone()
        })
        .collect();
    assert_ne!(amount_sites[0], amount_sites[1]);
    existing_native_probe::observe_amounts_mode(
        &mut fixture.core.state.active_battle.as_mut().unwrap().battle,
        1,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        4,
        "original1 plus actual authored then-branch3"
    );
    let battle = &fixture.core.state.active_battle.as_ref().unwrap().battle;
    let branch_source = battle
        .runtime_fact_summary()
        .latest_damage
        .as_ref()
        .unwrap()
        .source
        .authored
        .as_ref()
        .unwrap()
        .site
        .clone();
    assert_eq!(branch_source, amount_sites[0]);
    assert_eq!(
        existing_native_probe::amount_receipt(battle)["amount"],
        "Damage(3)"
    );
    fixture.advance(20);
    assert_eq!(
        before - fixture.target_hp(),
        11,
        "continuation7 executes exactly once after branch3"
    );
    let battle = &fixture.core.state.active_battle.as_ref().unwrap().battle;
    let continuation_source = &battle
        .runtime_fact_summary()
        .latest_damage
        .as_ref()
        .unwrap()
        .source
        .authored
        .as_ref()
        .unwrap()
        .site;
    assert_eq!(continuation_source, &amount_sites[1]);
    assert_ne!(&branch_source, continuation_source);
    assert_eq!(
        existing_native_probe::amount_receipt(battle)["amount"],
        "Damage(7)"
    );
    assert_causal_reclaimed(battle);
}
