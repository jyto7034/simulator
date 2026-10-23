use super::*;

pub(super) fn personal_skill(moment: &str) -> String {
    format!(
        r#"SkillDatabase(skills: [SkillDef(
        id: "tc_bite", name: "Independent personal grants", kind: Untargeted,
        focus_time_ms: 0, use_lifetime: Action,
        cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: true),
        flow: Some(Action(id: "grant", step_id: "shield", wait: EffectsSettled)),
        steps: [(id: "shield", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "pool", op: PersonalShieldGrant(definition: (
                amount: (moment: {moment}, root: Const((value: 200, unit: Magnitude))),
                duration_ms: 1000, priority: 5,
                recipients: PlayerEmployeesAtRelease(radius_units: 2000000),
                source_creation: Some((operations: [Scale(increment_percent: 50)]))
            )))])]
    )])"#
    )
}

pub(super) fn personal_relic(single: Option<u32>) -> String {
    let base = relic_text(None);
    let end = base.find("rules: [").unwrap();
    let add = if single.is_some() { 20 } else { 40 };
    let selector = r#"(provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(PersonalShieldGrant)))"#;
    let single = single.map_or_else(String::new, |value| {
        format!(
            r#",
        (id: "single", selector: {selector}, apply: Recipe(PersonalShieldSingle(
            initial_amount: (moment: Snapshot, root: Const((value: {value}, unit: Magnitude))))))"#
        )
    });
    format!(
        r#"{}rules: [
        (id: "add", selector: {selector}, apply: Edit(property: PersonalShieldInitialAmount, operation: Add(Hp({add})))),
        (id: "scale", selector: {selector}, apply: Edit(property: PersonalShieldInitialAmount, operation: Scale(increment_percent: 25)))
        {single}], )])"#,
        &base[..end]
    )
}

fn shield_fixture(
    moment: &str,
    single: Option<u32>,
) -> (
    GameCore,
    Uuid,
    Uuid,
    UnitInstanceId,
    Vec<UnitInstanceId>,
    UnitInstanceId,
) {
    team_fixture(&personal_skill(moment), &personal_relic(single))
}

pub(super) fn team_fixture(
    skills: &str,
    relics: &str,
) -> (
    GameCore,
    Uuid,
    Uuid,
    UnitInstanceId,
    Vec<UnitInstanceId>,
    UnitInstanceId,
) {
    team_fixture_with_setup(skills, relics, |_, _, _| {})
}

pub(super) fn team_fixture_with_setup(
    skills: &str,
    relics: &str,
    extra_prepare: impl FnOnce(&mut GameCore, Uuid, Uuid),
) -> (
    GameCore,
    Uuid,
    Uuid,
    UnitInstanceId,
    Vec<UnitInstanceId>,
    UnitInstanceId,
) {
    team_fixture_with_generated_count(skills, relics, 0, extra_prepare)
}

pub(super) fn team_fixture_with_generated_count(
    skills: &str,
    relics: &str,
    generated_count: usize,
    extra_prepare: impl FnOnce(&mut GameCore, Uuid, Uuid),
) -> (
    GameCore,
    Uuid,
    Uuid,
    UnitInstanceId,
    Vec<UnitInstanceId>,
    UnitInstanceId,
) {
    let (mut core, player, employee, actor, enemies) =
        prepare_with_setup(skills, relics, None, None, generated_count, extra_prepare);

    let employees = core.roster().unwrap().combat_eligible_employee_ids();
    assert_eq!(employees.len(), 3, "three normal owned starter employees");
    let ground = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .unwrap()
        .cells
        .clone();
    let mut next = 1;
    // Fund normal World deployment of all three independent recipients.
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .set_current_for_test(100);
    for other in employees.into_iter().filter(|id| *id != employee) {
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: other,
                position: ground[next],
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        next += 1;
    }
    let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
    let allies: Vec<_> = battle
        .units
        .values()
        .filter(|unit| {
            unit.owner == Side::Player
                && unit.instance_id != actor
                && matches!(
                    unit.source_identity,
                    crate::game::battle::types::BattleUnitSourceIdentity::Employee { .. }
                )
        })
        .map(|unit| unit.instance_id)
        .collect();
    assert_eq!(allies.len(), 2);
    native_probe::place_stationary(battle, actor, 2, 2);
    for (index, id) in allies.iter().enumerate() {
        native_probe::place_stationary(battle, *id, 3 + index as i32, 2);
        let unit = battle.units.get_mut(id).unwrap();
        unit.stats.max_health = 1000;
        unit.stats.current_health = 1000;
        unit.stats.attack = 0;
        unit.incoming_damage_modifiers.damage_reduction_percent = 50;
    }
    (core, player, employee, actor, allies, enemies[0])
}

pub(super) fn pools(core: &GameCore) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
    core.state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .into_iter()
        .filter(|row| row.personal_grant.is_some())
        .collect()
}

fn remaining(core: &GameCore, actor: UnitInstanceId) -> Option<u32> {
    pools(core)
        .iter()
        .find(|row| row.target == actor)
        .map(|row| row.shield.unwrap().remaining)
}

#[test]
fn rv001_personal_selected_amount_uses_authored_strength_and_allows_qualified_caster() {
    use crate::game::battle::event_log::SkillCastTarget;
    for (base, expected) in [(420, 825), (620, 1200)] {
        for select_caster in [false, true] {
            let (mut core, player, employee, actor, allies, enemy) =
                shield_fixture("Snapshot", Some(base));
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::BeginSkillTargeting {
                    employee_uuid: employee,
                    skill_id: "tc_bite".into(),
                },
            )
            .unwrap();
            let roots =
                native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
            for target in [
                None,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: enemy,
                }),
            ] {
                let rejected = execute_current_battle_mutation(
                    &mut core,
                    player,
                    PlayerBehavior::ActivateSkill {
                        employee_uuid: employee,
                        skill_id: "tc_bite".into(),
                        target,
                    },
                );
                assert!(rejected.is_err(), "explicit eligible selection is required");
                assert_eq!(
                    native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
                    roots
                );
                assert!(pools(&core).is_empty());
            }
            let recipient = if select_caster { actor } else { allies[0] };
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::ActivateSkill {
                    employee_uuid: employee,
                    skill_id: "tc_bite".into(),
                    target: Some(SkillCastTarget::Unit {
                        unit_instance_id: recipient,
                    }),
                },
            )
            .unwrap();
            core.advance_active_battle_for_server_tick(1).unwrap();
            let rows = pools(&core);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].target, recipient);
            assert_eq!(rows[0].shield.unwrap().capacity, expected);
            assert_eq!(rows[0].shield.unwrap().remaining, expected);
            assert_eq!(rows[0].members, vec![recipient]);
            if !select_caster {
                native_probe::damage_probe(
                    &mut core.state.active_battle.as_mut().unwrap().battle,
                    enemy,
                    recipient,
                    1600,
                    1,
                );
                assert_eq!(remaining(&core, recipient), Some(expected - 800));
                native_probe::damage_probe(
                    &mut core.state.active_battle.as_mut().unwrap().battle,
                    enemy,
                    recipient,
                    100,
                    1,
                );
                if expected == 825 {
                    assert_eq!(remaining(&core, recipient), None);
                    assert_eq!(hp(&core, &[recipient]), 975);
                } else {
                    assert_eq!(remaining(&core, recipient), Some(350));
                    assert_eq!(hp(&core, &[recipient]), 1000);
                }
            }
            println!("RV001 selected base={base} amount={expected} select_caster={select_caster} pool_count=1 no_selection_enemy_rejected=true source_stage_once=true");
        }
    }
}

#[test]
fn rv001_personal_range_persistence_whole_oldset_recast_and_provider_end() {
    let (mut core, player, employee, actor, allies, enemy) = shield_fixture("Snapshot", None);
    cast(&mut core, player, employee);
    native_probe::damage_probe(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        enemy,
        allies[0],
        800,
        1,
    );
    native_probe::damage_probe(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        enemy,
        allies[1],
        600,
        1,
    );
    let old = pools(&core);
    // Fractional displacement remains in the same valid tile while crossing
    // the authored 2-world-unit radius: 4.75 - 2.5 = 2.25.
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        4.75,
    );
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(
        pools(&core),
        old,
        "range exit neither removes nor refills a finite personal pool"
    );
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        4.5,
    );
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(pools(&core), old);
    native_probe::hard_cc_cancel(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        actor,
        21,
    );
    assert_eq!(
        pools(&core),
        old,
        "post-release cancellation does not terminate a nonchannel grant"
    );
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        4.75,
    );
    native_probe::legacy_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        77,
    );
    native_probe::refill_skill(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        actor,
        21,
    );
    cast(&mut core, player, employee);
    let new = pools(&core);
    assert_eq!(new.len(), 2);
    assert_eq!(remaining(&core, actor), Some(450));
    assert_eq!(remaining(&core, allies[0]), Some(450));
    assert_eq!(
        remaining(&core, allies[1]),
        None,
        "whole old set includes the now-outside recipient"
    );
    assert!(new.iter().all(|row| row.expires_at_ms == 1022
        && old.iter().all(|prior| prior.activation != row.activation)));
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        4.5,
    );
    core.advance_active_battle_for_server_tick(980).unwrap();
    assert_eq!(
        pools(&core),
        new,
        "old expiry and late entry cannot alter the new fixed set"
    );
    let legacy: Vec<_> = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .into_iter()
        .filter(|row| row.personal_grant.is_none() && row.target == allies[1])
        .collect();
    // Legacy helper's expiry is 1000, so it has ended naturally by t1002.
    assert!(legacy.is_empty());
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    assert!(pools(&core).is_empty());
    println!("RV001 personal range_out_return=unchanged residuals50,150 recast_set=C,A450 B_removed late_entry_no_grant old_expiry_ignored=true provider_withdraw_ends_remaining=true");
}

#[test]
fn rv001_personal_area_source_order_and_independent_finite_pools() {
    for moment in ["Snapshot", "Live"] {
        let (mut core, player, employee, actor, allies, enemy) = shield_fixture(moment, None);
        cast(&mut core, player, employee);
        let rows = pools(&core);
        assert_eq!(rows.len(), 3);
        let mut ids: Vec<_> = rows.iter().map(|row| row.activation).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3);
        for row in &rows {
            assert!(!row.is_channel);
            assert_eq!(row.shield.unwrap().capacity, 450);
            assert_eq!(row.shield.unwrap().remaining, 450);
            assert_eq!(row.expires_at_ms, 1001);
            assert_eq!(row.caster, actor);
            assert_eq!(row.members, vec![row.target]);
            assert!(
                !row.personal_grant
                    .as_ref()
                    .unwrap()
                    .legacy_recharge_supported
            );
            assert!(row
                .personal_grant
                .as_ref()
                .unwrap()
                .fragment_copy_uuid
                .is_some());
        }
        native_probe::damage_probe(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            enemy,
            allies[0],
            800,
            1,
        );
        assert_eq!(remaining(&core, allies[0]), Some(50));
        assert_eq!(remaining(&core, allies[1]), Some(450));
        assert_eq!(remaining(&core, actor), Some(450));
        native_probe::damage_probe(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            enemy,
            allies[1],
            600,
            1,
        );
        assert_eq!(remaining(&core, allies[1]), Some(150));
        native_probe::damage_probe(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            enemy,
            allies[0],
            200,
            1,
        );
        assert_eq!(remaining(&core, allies[0]), None);
        assert_eq!(hp(&core, &[allies[0]]), 950);
        assert_eq!(hp(&core, &[allies[1]]), 1000);
        assert_eq!(remaining(&core, actor), Some(450));
        println!("RV001 personal moment={moment} source=(200+40)*1.25*1.5=450 recipients=3 separate_activations=true after_hits=A_none_hp950_B150_hp1000_C450");
    }
}

#[test]
fn rv002_personal_source_once_snapshot_live_and_optional_trace_invariance() {
    for (moment, expected) in [("Snapshot", 450), ("Live", 825)] {
        let skill = personal_skill(moment)
            .replace("root: Const((value: 200, unit: Magnitude))", "root: Read(Stat(subject: Source, stat: Attack))")
            .replace("effects: [", r#"effects: [(id: "grow", op: ModifyStats(modifier: (stat: Attack, kind: Flat, value: 200))),"#);
        let (base, player, employee, actor, allies, _) =
            team_fixture(&skill, &personal_relic(None));
        let mut baseline = None;
        for mode in 0..3 {
            let mut core = base.clone();
            let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
            battle.units.get_mut(&actor).unwrap().stats.attack = 200;
            battle.units.get_mut(&allies[0]).unwrap().stats.attack = 700;
            battle.units.get_mut(&allies[1]).unwrap().stats.attack = 900;
            native_probe::observe_amounts_mode(battle, mode);
            cast(&mut core, player, employee);
            let rows = pools(&core);
            assert_eq!(rows.len(), 3);
            assert!(rows
                .iter()
                .all(|row| row.shield.unwrap().capacity == expected));
            let receipt =
                native_probe::amount_receipt(&core.state.active_battle.as_ref().unwrap().battle);
            assert_eq!(
                receipt["evaluations"], 1,
                "one source evaluation for three recipients"
            );
            let outcome = (rows, receipt);
            if let Some(prior) = &baseline {
                assert_eq!(&outcome, prior);
            } else {
                baseline = Some(outcome);
            }
            let before = pools(&core);
            let result = native_probe::recharge_personal(
                &mut core.state.active_battle.as_mut().unwrap().battle,
                actor,
                1,
            );
            assert!(
                matches!(result, Err(GameError::BattleRuntimeFault(fault)) if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::ShieldUnavailable)
            );
            assert_eq!(pools(&core), before);
        }
        println!("RV002 personal moment={moment} amount={expected} source200_before_grow400_after=true recipient_stats700_900_ignored=true evaluations1 trace_off_on_saturated_identical=true recharge_rejected_unchanged=true");
    }
}

#[test]
fn rv002_personal_release_set_uses_release_position_and_recipient_end_is_local() {
    let skill = personal_skill("Snapshot").replace("focus_time_ms: 0", "focus_time_ms: 20");
    let (mut core, player, employee, actor, allies, _) =
        team_fixture(&skill, &personal_relic(None));
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        4.75,
    );
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "tc_bite".into(),
            target: None,
        },
    )
    .unwrap();
    assert!(pools(&core).is_empty());
    // Accepted while A was eligible and B outside; release sees the opposite.
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[0],
        4.75,
    );
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        3.5,
    );
    core.advance_active_battle_for_server_tick(21).unwrap();
    let rows = pools(&core);
    assert_eq!(rows.len(), 2);
    assert_eq!(remaining(&core, allies[0]), None);
    assert_eq!(remaining(&core, allies[1]), Some(450));
    assert_eq!(remaining(&core, actor), Some(450));
    let other_employee =
        core.state.active_battle.as_ref().unwrap().battle.units[&allies[1]].source_owned_uuid;
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: other_employee,
        },
    )
    .unwrap();
    let after = pools(&core);
    assert_eq!(after.len(), 1);
    assert_eq!(
        after[0],
        *rows.iter().find(|row| row.target == actor).unwrap()
    );
    println!("RV002 personal acceptance_A_release_B=true recipient_withdraw_only_B_removed_caster_unchanged=true");
}

#[test]
fn rv002_personal_priority_ties_then_generic_shared_stage() {
    let (mut core, player, employee, actor, allies, enemy) = shield_fixture("Snapshot", None);
    let target = allies[0];
    let high = native_probe::priority_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        target,
        target,
        70,
        9,
        false,
    );
    let equal = native_probe::priority_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        target,
        target,
        30,
        5,
        false,
    );
    cast(&mut core, player, employee);
    let personal = pools(&core)
        .iter()
        .find(|row| row.target == target)
        .unwrap()
        .activation;
    let others: Vec<_> = pools(&core)
        .into_iter()
        .filter(|row| row.target != target)
        .collect();
    let low = native_probe::priority_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        target,
        target,
        20,
        -1,
        false,
    );
    let shared = native_probe::priority_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        actor,
        actor,
        100,
        99,
        true,
    );
    native_probe::damage_probe(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        enemy,
        target,
        1400,
        1,
    );
    let fact = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_summary()
        .latest_damage
        .clone()
        .unwrap();
    let absorbed: Vec<_> = fact
        .protection
        .details
        .as_ref()
        .unwrap()
        .shields
        .iter()
        .map(|row| (row.activation, row.absorbed))
        .collect();
    assert_eq!(
        absorbed,
        vec![
            (high, 70),
            (equal, 30),
            (personal, 450),
            (low, 20),
            (shared, 100)
        ]
    );
    assert_eq!(fact.actual_hp_loss(), 30);
    assert_eq!(hp(&core, &[target]), 970);
    assert_eq!(pools(&core), others);
    println!("RV002 personal_order=H70_E30_P450_L20_sharedQ100 mitigated700_hp30 other_personal_pools_unchanged=true");
}

#[test]
fn rv002_personal_initial_and_recast_late_failure_are_atomic_and_retry_exact() {
    use crate::authoring::control::{
        self, CancellationToken, ExecutionLimits, ExecutionPhase, NativeStorageProvenance,
    };
    let roots = |core: &GameCore| {
        let active = core.state.active_battle.as_ref().unwrap();
        serde_json::json!({
            "run": core.get_run_snapshot_json().unwrap(), "grants": serialized_grant_roots(core),
            "revision": active.battle_state_revision, "battle": native_probe::flow_roots(&active.battle),
            "resources": format!("{:?}", core.battle_typed_resources()),
            "sustained": active.battle.sustained_effect_snapshot().unwrap(),
        })
    };
    for recast in [false, true] {
        let ((base, actor), _, provenance) = control::run_with_provenance(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            NativeStorageProvenance::fresh(),
            || {
                let (mut core, player, employee, actor, allies, enemy) =
                    shield_fixture("Snapshot", None);
                if recast {
                    cast(&mut core, player, employee);
                    native_probe::damage_probe(
                        &mut core.state.active_battle.as_mut().unwrap().battle,
                        enemy,
                        allies[0],
                        800,
                        1,
                    );
                    native_probe::damage_probe(
                        &mut core.state.active_battle.as_mut().unwrap().battle,
                        enemy,
                        allies[1],
                        600,
                        1,
                    );
                    native_probe::legacy_shield(
                        &mut core.state.active_battle.as_mut().unwrap().battle,
                        allies[1],
                        77,
                    );
                    native_probe::refill_skill(
                        &mut core.state.active_battle.as_mut().unwrap().battle,
                        actor,
                        1,
                    );
                }
                execute_current_battle_mutation(
                    &mut core,
                    player,
                    PlayerBehavior::ActivateSkill {
                        employee_uuid: employee,
                        skill_id: "tc_bite".into(),
                        target: None,
                    },
                )
                .unwrap();
                (core, actor)
            },
        )
        .unwrap();
        let before = roots(&base);
        let mut clean = base.clone();
        clean.advance_active_battle_for_server_tick(1).unwrap();
        assert_eq!(pools(&clean).len(), 3);
        assert_eq!(remaining(&clean, actor), Some(450));
        let expected = roots(&clean);
        let sites = if recast {
            vec![("sustained_started", 2), ("sustained_ended", 2)]
        } else {
            vec![("sustained_started", 2)]
        };
        for (site, occurrence) in sites {
            let mut core = base.clone();
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, occurrence, || {
                control::run_with_provenance(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    provenance,
                    || core.advance_active_battle_for_server_tick(1),
                )
            });
            let failure = result
                .err()
                .expect("late recipient preparation/end seam reached");
            assert_eq!(failure.code, "authoring_cancelled");
            assert_eq!(failure.site, site);
            assert!(token.is_cancelled());
            assert_eq!(roots(&core), before);
            core.advance_active_battle_for_server_tick(1).unwrap();
            assert_eq!(roots(&core), expected);
            println!("RV002 personal recast={recast} late_site={site} occurrence={occurrence} prestate_exact=true retry_exact=true");
        }
    }
}

#[test]
fn rv002_personal_unrelated_provider_group_survives_recast_and_source_withdrawal() {
    let aux_database = personal_skill("Snapshot").replace("tc_bite", "rv_aux");
    let aux =
        &aux_database[aux_database.find("SkillDef(").unwrap()..aux_database.rfind("])").unwrap()];
    let skills = support_cases::with_aux(&personal_skill("Snapshot"), aux);
    let (mut core, player, employee, actor, allies, _) =
        team_fixture_with_setup(&skills, &personal_relic(None), support_cases::prepare_aux);
    let auxiliary = allies
        .iter()
        .copied()
        .find(|id| {
            core.state.active_battle.as_ref().unwrap().battle.units[id]
                .skill_id
                .as_ref()
                .is_some_and(|id| id.as_str() == "rv_aux")
        })
        .unwrap();
    let auxiliary_employee =
        core.state.active_battle.as_ref().unwrap().battle.units[&auxiliary].source_owned_uuid;
    cast(&mut core, player, employee);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: auxiliary_employee,
            skill_id: "rv_aux".into(),
            target: None,
        },
    )
    .unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    let others: Vec<_> = pools(&core)
        .into_iter()
        .filter(|row| row.caster == auxiliary)
        .collect();
    assert_eq!(others.len(), 3);
    assert!(others.iter().all(|row| row.shield.unwrap().capacity == 300));
    assert_eq!(pools(&core).len(), 6);
    native_probe::refill_skill(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        actor,
        2,
    );
    cast(&mut core, player, employee);
    assert_eq!(pools(&core).len(), 6);
    assert_eq!(
        pools(&core)
            .into_iter()
            .filter(|row| row.caster == auxiliary)
            .collect::<Vec<_>>(),
        others
    );
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    let after = pools(&core);
    assert_eq!(
        after.len(),
        2,
        "own three end; auxiliary's grant on withdrawn recipient ends locally"
    );
    assert_eq!(
        after,
        others
            .into_iter()
            .filter(|row| row.target != actor)
            .collect::<Vec<_>>()
    );
    println!("RV002 personal two_normal_provider_groups main3x450 auxiliary3x300 recast_only_main=true source_withdraw_aux_other2_unchanged=true");
}

#[test]
fn rv002_personal_hp_cost_heal_and_elapsed_time_do_not_recharge_pool() {
    let skill = personal_skill("Snapshot")
        .replace(
            "flow: Some(Action(id: \"grant\", step_id: \"shield\", wait: EffectsSettled))",
            r#"flow: Some(Sequence(id: "grant_cost_heal", children: [
            Action(id: "grant", step_id: "shield", wait: Released),
            Wait(id: "before_cost", duration_ms: 10),
            Action(id: "cost", step_id: "cost", wait: EffectsSettled),
            Wait(id: "before_heal", duration_ms: 10),
            Action(id: "heal", step_id: "heal", wait: EffectsSettled)]))"#,
        )
        .replace(
            ")))])]",
            r#")))]),
        (id: "cost", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "hp_cost", op: HealthCost(amount: 10, allow_lethal: false))]),
        (id: "heal", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "restore", op: Heal(amount: 10))]) ]"#,
        );
    let (mut core, player, employee, actor, _, enemy) = team_fixture(&skill, &personal_relic(None));
    cast(&mut core, player, employee);
    native_probe::damage_probe(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        enemy,
        actor,
        400,
        1,
    );
    let before = pools(&core);
    assert_eq!(remaining(&core, actor), Some(50));
    assert_eq!(hp(&core, &[actor]), 500);
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(hp(&core, &[actor]), 490);
    assert_eq!(pools(&core), before);
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(hp(&core, &[actor]), 500);
    assert_eq!(pools(&core), before);
    core.advance_active_battle_for_server_tick(30).unwrap();
    assert_eq!(pools(&core), before);
    println!(
        "RV002 personal residual50 hp_cost10=500to490 heal10=490to500 elapsed50_no_recharge=true"
    );
}

#[test]
fn rv002_personal_native_retained_initial_and_recast_admission_preserve_prestate() {
    use crate::game::battle::runtime_control as native;
    let roots = |core: &GameCore| {
        serde_json::json!({
            "run": core.get_run_snapshot_json().unwrap(),
            "grants": serialized_grant_roots(core),
            "battle": native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
            "sustained": core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap(),
        })
    };
    for recast in [false, true] {
        let (mut base, player, employee, actor, allies, enemy) = shield_fixture("Snapshot", None);
        if recast {
            cast(&mut base, player, employee);
            native_probe::damage_probe(
                &mut base.state.active_battle.as_mut().unwrap().battle,
                enemy,
                allies[0],
                800,
                1,
            );
            native_probe::refill_skill(
                &mut base.state.active_battle.as_mut().unwrap().battle,
                actor,
                1,
            );
        }
        execute_current_battle_mutation(
            &mut base,
            player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: "tc_bite".into(),
                target: None,
            },
        )
        .unwrap();
        let before = roots(&base);
        let mut clean = base.clone();
        let (result, admissions) =
            native::observe_retained_admissions(|| clean.advance_active_battle_for_server_tick(1));
        result.unwrap();
        assert!(admissions
            .iter()
            .any(|row| row.0 == "personal_grant_batch_storage"));
        let (site, _, boundary) = admissions.iter().max_by_key(|row| row.2).unwrap();
        let expected = roots(&clean);
        let mut trial = base.clone();
        let error = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: boundary - 1,
                ..Default::default()
            },
            || trial.advance_active_battle_for_server_tick(1),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                ..
            })
        ));
        assert_eq!(roots(&trial), before);
        native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: *boundary,
                ..Default::default()
            },
            || trial.advance_active_battle_for_server_tick(1),
        )
        .unwrap();
        assert_eq!(roots(&trial), expected);
        println!("RV002 personal recast={recast} retained_boundary={boundary} site={site} one_short_unchanged=true exact_retry_clean=true");
    }
}

#[test]
fn rv002_personal_lethal_recipient_and_provider_lifetimes_and_battle_retreat() {
    for source_end in [false, true] {
        let (mut core, player, employee, actor, allies, enemy) = shield_fixture("Snapshot", None);
        cast(&mut core, player, employee);
        let before = pools(&core);
        let victim = if source_end { actor } else { allies[0] };
        native_probe::damage_probe(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            enemy,
            victim,
            10000,
            1,
        );
        assert!(core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .units
            .get(&victim)
            .is_none_or(|unit| !unit.is_active()));
        if source_end {
            assert!(pools(&core).is_empty());
        } else {
            assert_eq!(
                pools(&core),
                before
                    .into_iter()
                    .filter(|row| row.target != victim)
                    .collect::<Vec<_>>()
            );
        }
        println!(
            "RV002 personal lethal_source={source_end} source_all_or_recipient_only_ends=true"
        );
    }
    let (mut core, player, employee, _, _, _) = shield_fixture("Snapshot", None);
    cast(&mut core, player, employee);
    assert_eq!(pools(&core).len(), 3);
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    assert!(core.state.active_battle.is_none());
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(pools(&core).is_empty());
    println!("RV002 personal retreat_reentry_has_no_old_battle_pools=true");
}

#[test]
fn rv002_personal_post_release_silence_and_stun_consumers_preserve_nonchannel_grants() {
    for kind in ["silence", "stun"] {
        let skill =
            personal_skill("Snapshot").replace("Independent personal grants", "rv_control_fixture");
        let (mut core, player, employee, actor, _, enemy) =
            team_fixture(&skill, &personal_relic(None));
        cast(&mut core, player, employee);
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .battle
            .process_event_for_test(
                crate::game::battle::enums::BattleEvent::ApplyBuff {
                    time_ms: 1,
                    caster_instance_id: enemy,
                    target_instance_id: actor,
                    buff_id: crate::game::battle::buffs::BuffId::from_name(&format!(
                        "rv_control_{kind}"
                    )),
                    duration_ms: 100,
                    cause: Default::default(),
                },
                1,
            )
            .unwrap();
        let rows = pools(&core);
        assert_eq!(rows.len(), 3);
        assert!(rows
            .iter()
            .all(|row| !row.is_channel && row.shield.unwrap().remaining == 450));
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        let state = battle.unit_control_state(actor, 1).unwrap();
        if kind == "silence" {
            assert!(state.silenced);
        } else {
            assert!(state.hard_cc_active());
        }
        assert!(!native_probe::channel_active(battle, actor));
        core.advance_active_battle_for_server_tick(101).unwrap();
        assert_eq!(pools(&core), rows);
        println!("RV002 personal actual_{kind}_consumer_after_grant=true three450_nonchannel_unchanged_after_control_expiry=true");
    }
}

#[test]
fn rv002_zero_personal_grant_refuses_without_partial_publication() {
    let relic = personal_relic(None).replace("Add(Hp(40))", "Add(Hp(0))");
    let (seed, _, _, _, _, _) = shield_fixture("Snapshot", None);
    let constant_zero = SkillDatabase::from_ron(
        "rv002/zero.ron",
        &personal_skill("Snapshot").replace("value: 200", "value: 0"),
    )
    .unwrap();
    let rejected = GameDataBuilder::from_validated(&seed.game_data)
        .with_skills(constant_zero)
        .with_relics(ron::de::from_str(&relic).unwrap())
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
        .err()
        .unwrap();
    assert_eq!(rejected.code, "relic_structural_out_of_range");
    let skill = personal_skill("Snapshot").replace(
        "root: Const((value: 200, unit: Magnitude))",
        "root: Read(Stat(subject: Source, stat: Attack))",
    );
    let (mut core, player, employee, _, _, _) = team_fixture(&skill, &relic);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "tc_bite".into(),
            target: None,
        },
    )
    .unwrap();
    let before = native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
    let saved = core.get_run_snapshot_json().unwrap();
    assert!(core.advance_active_battle_for_server_tick(1).is_err());
    assert!(pools(&core).is_empty());
    assert_eq!(
        native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
        before
    );
    assert_eq!(core.get_run_snapshot_json().unwrap(), saved);
    println!(
        "RV002 zero_personal release_refused=true pools0 whole_world_tick_prestate_unchanged=true"
    );
}

#[test]
fn rv002_personal_recipient_redeploy_does_not_inherit_original_combat_identity() {
    let skill = personal_skill("Snapshot").replace("duration_ms: 1000", "duration_ms: 60000");
    let (mut core, player, employee, _, allies, _) = team_fixture(&skill, &personal_relic(None));
    cast(&mut core, player, employee);
    let recipient = allies[0];
    let recipient_employee =
        core.state.active_battle.as_ref().unwrap().battle.units[&recipient].source_owned_uuid;
    let expected: Vec<_> = pools(&core)
        .into_iter()
        .filter(|row| row.target != recipient)
        .collect();
    let position = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .unwrap()
        .cells[1];
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: recipient_employee,
        },
    )
    .unwrap();
    assert_eq!(pools(&core), expected);
    let mut redeployed = false;
    let mut cooldown_refusals = 0;
    for _ in 0..6000 {
        match execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: recipient_employee,
                position,
                facing: FacingDirection::Right,
            },
        ) {
            Ok(_) => {
                redeployed = true;
                break;
            }
            Err(GameError::ActionUnavailable {
                reason: crate::game::behavior::ActionDisabledReasonCode::RedeployCooldown,
                ..
            }) => {
                cooldown_refusals += 1;
                core.advance_active_battle_to_next_event_bucket().unwrap();
            }
            other => panic!("unexpected redeploy outcome: {other:?}"),
        }
    }
    assert!(redeployed && cooldown_refusals > 0);
    let new_actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units
        .values()
        .find(|unit| unit.source_owned_uuid == recipient_employee && unit.is_active())
        .unwrap()
        .instance_id;
    assert_ne!(new_actor, recipient);
    assert_eq!(remaining(&core, new_actor), None);
    assert_eq!(pools(&core), expected);
    println!("RV002 personal recipient_redeploy_fresh_id=true old_grant_not_inherited=true other_long_pools_unchanged=true");
}

#[test]
fn rv002_personal_same_provider_distinct_original_effects_remain_distinct_groups() {
    let skill = personal_skill("Snapshot").replace("effects: [", r#"effects: [
        (id: "other_original", op: PersonalShieldGrant(definition: (
            amount: (moment: Snapshot, root: Const((value: 40, unit: Magnitude))),
            duration_ms: 1000, priority: 0, recipients: PlayerEmployeesAtRelease(radius_units: 2000000)))),"#);
    let (mut core, player, employee, actor, allies, _) =
        team_fixture(&skill, &personal_relic(None));
    cast(&mut core, player, employee);
    let first = pools(&core);
    assert_eq!(
        first.len(),
        6,
        "original effect identity participates in the group key"
    );
    for target in [actor, allies[0], allies[1]] {
        let mut capacities: Vec<_> = first
            .iter()
            .filter(|row| row.target == target)
            .map(|row| row.shield.unwrap().capacity)
            .collect();
        capacities.sort();
        assert_eq!(capacities, vec![100, 450]);
    }
    native_probe::legacy_shield(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        77,
    );
    let legacy = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .into_iter()
        .find(|row| row.personal_grant.is_none())
        .unwrap();
    native_probe::refill_skill(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        actor,
        1,
    );
    cast(&mut core, player, employee);
    let after = pools(&core);
    assert_eq!(after.len(), 6);
    assert!(after
        .iter()
        .all(|row| first.iter().all(|old| old.activation != row.activation)));
    assert!(core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .contains(&legacy));
    println!("RV002 personal same_caster_same_fragment_two_original_effects=3x100_plus3x450 recast_groups_separate=true unrelated_legacy77_unchanged=true");
}
