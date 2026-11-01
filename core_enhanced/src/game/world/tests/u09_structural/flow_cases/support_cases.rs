use super::*;
use crate::game::ability::sustained::FixedSupportConnectionState;
use crate::game::battle::event_log::SkillCastTarget;

pub(super) fn support_skill() -> &'static str {
    r#"SkillDatabase(skills: [SkillDef(
        id: "tc_bite", name: "Original self-carried channel", kind: Targeted, focus_time_ms: 0,
        use_lifetime: ActionAndEffects(effects: [(step: "support", effect: "channel")]),
        cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: true),
        steps: [(id: "support", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "channel", op: Sustained(definition: ChannelAura(duration_ms: 30000,
                aura: (family: "rv_support", priority: 0, radius_units: 2000000,
                    basic_damage_percent: 20, basic_speed_points: 15))))])]
    )])"#
}

pub(super) fn support_relic() -> String {
    let base = relic_text(None);
    let end = base.find("rules: [").unwrap();
    format!(
        r#"{}rules: [
        (id: "fixed", selector: (provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(ChannelAura))),
            apply: Recipe(FixedRecipientSupport)),
        (id: "regular", selector: (provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(ChannelAura))),
            apply: Recipe(RegularBasicSupport(damage_percent: 20, speed_points: 15))),
        (id: "damage", selector: (provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(ChannelAura))),
            apply: Edit(property: RegularBasicDamagePercent, operation: Add(PercentPoints(20)))),
        (id: "speed", selector: (provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(ChannelAura))),
            apply: Edit(property: RegularBasicSpeedPoints, operation: Add(SpeedPoints(5))))
    ], )])"#,
        &base[..end]
    )
}

#[test]
fn rv002_fixed_support_preserves_self_carrier_selected_identity_and_terminal_connection() {
    fixed_support_geometry(None, false);
}

fn f01_mixed_relic(finite: bool, reach: u32) -> String {
    let effect = format!(
        r#"(id: "generated_pool", op: PersonalShieldGrant(definition: (
        amount: (moment: Snapshot, root: Const((value: 100, unit: Magnitude))),
        duration_ms: 30000, priority: 5,
        recipients: PlayerEmployeesAtRelease(radius_units: {reach}))))"#
    );
    let body = if finite {
        format!(
            r#"FiniteFlow(anchor_action_id: "f01_anchor",
            steps: [(id: "generated_child", target: SelfUnit, delivery: Instant, effects: [{effect}])],
            flow: Action(id: "generated_release", step_id: "generated_child", wait: EffectsSettled))"#
        )
    } else {
        format!(r#"InlineEffects(action_id: "support", effects: [{effect}])"#)
    };
    // Keep authored RON text: the supported parser accepts the typed i128
    // amount, but this RON serializer cannot serialize i128 values back out.
    support_relic().replace("rules: [", &format!(r#"rules: [
        (id: "generated_pool", selector: (provider: (fragment_id: Some("tc_fragment"))), apply: Grant({body})),
        (id: "generated_single",
        selector: (provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(PersonalShieldGrant))),
        apply: Recipe(PersonalShieldSingle(initial_amount: (moment: Snapshot, root: Const((value: 200, unit: Magnitude)))))),
    "#))
}

fn f01_mixed_skill(finite: bool) -> String {
    if finite {
        // A child of the channel itself correctly waits for that channel to
        // settle. Use another ordinary Self/Instant action in the same skill
        // so the personal child can release while the original channel lives.
        // The publication matrix separately retains the exact channel-anchor
        // conflict construction for both generated representations.
        support_skill().replace(
            "steps: [(",
            r#"steps: [(id: "f01_anchor", delay_ms: 0, target: SelfUnit, delivery: Instant,
                effects: [(id: "anchor_heal", op: Heal(amount: 1))]), ("#,
        )
    } else {
        support_skill().into()
    }
}

#[test]
fn f01_equal_generated_reach_keeps_world_selection_and_fixed_connection_geometry() {
    for (finite, remove_current_owner) in [(false, false), (true, false), (true, true)] {
        fixed_support_geometry(Some(finite), remove_current_owner);
        println!(
            "F01 WORLD finite={finite} equal2m selected_personal200 self_channel=true out_back_same_connection=true terminal_no_retarget=true finite_child_uses_separate_instant_anchor={finite} current_owner_removed={remove_current_owner}"
        );
    }
}

fn fixed_support_geometry(generated: Option<bool>, remove_current_owner: bool) {
    let (mut core, player, employee, actor, allies, enemy) = if let Some(finite) = generated {
        personal_cases::team_fixture_with_generated_count(
            &f01_mixed_skill(finite),
            &f01_mixed_relic(finite, 2_000_000),
            1,
            |_, _, _| {},
        )
    } else {
        personal_cases::team_fixture(support_skill(), &support_relic())
    };
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid: employee,
            skill_id: "tc_bite".into(),
        },
    )
    .unwrap();
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        4.75,
    );
    for target in [None, Some(actor), Some(enemy), Some(allies[1])] {
        let before = native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
        let error = execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: "tc_bite".into(),
                target: target.map(|unit_instance_id| SkillCastTarget::Unit { unit_instance_id }),
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, GameError::ActionUnavailable { .. }),
            "{error:?}"
        );
        assert_eq!(
            native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
            before
        );
    }
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "tc_bite".into(),
            target: Some(SkillCastTarget::Unit {
                unit_instance_id: allies[0],
            }),
        },
    )
    .unwrap();
    if remove_current_owner {
        // The accepted action already owns the effective PlayerUnit plan.
        // Removing only the current owner exposes canonical Self selection;
        // the pending and subsequent actions must retain their accepted plan.
        native_probe::remove_current_owner_keep_pending(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            actor,
        );
    }
    core.advance_active_battle_for_server_tick(1).unwrap();
    if generated.is_some() {
        let pools = personal_cases::pools(&core);
        if pools.len() != 1 {
            println!(
                "F01 WORLD diagnostic finite={generated:?} {}",
                native_probe::selected_flow_diagnostic(
                    &core.state.active_battle.as_ref().unwrap().battle
                )
            );
        }
        assert_eq!(pools.len(), 1);
        assert_eq!(pools[0].target, allies[0]);
        assert_eq!(pools[0].shield.unwrap().capacity, 200);
    }
    let view = |core: &GameCore| {
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .find(|row| row.fixed_connection.is_some())
            .unwrap()
    };
    let first = view(&core);
    assert_eq!(
        (first.caster, first.target),
        (actor, actor),
        "original SelfUnit channel carrier is unchanged"
    );
    assert!(first.is_channel);
    assert_eq!(first.members, vec![allies[0]]);
    assert_eq!(first.expires_at_ms, 30001);
    assert_eq!(first.fixed_connection.as_ref().unwrap().recipient, allies[0]);
    assert_eq!(
        first.fixed_connection.as_ref().unwrap().authored_reach_units,
        2000000
    );
    assert_eq!(
        native_probe::support_bonus(
            &core.state.active_battle.as_ref().unwrap().battle,
            allies[0],
            true
        ),
        (40, 20)
    );
    assert_eq!(
        native_probe::support_bonus(
            &core.state.active_battle.as_ref().unwrap().battle,
            allies[0],
            false
        ),
        (0, 0)
    );
    assert_eq!(
        native_probe::support_bonus(
            &core.state.active_battle.as_ref().unwrap().battle,
            actor,
            true
        ),
        (0, 0)
    );
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[0],
        4.75,
    );
    core.advance_active_battle_for_server_tick(10).unwrap();
    // Direct geometry preparation has no movement event; a committed damage
    // consumer exercises the normal sustained reconciliation boundary.
    native_probe::damage_probe(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        enemy,
        allies[0],
        2,
        11,
    );
    let outside = view(&core);
    assert_eq!(outside.activation, first.activation);
    assert_eq!(
        outside.fixed_connection.unwrap().state,
        FixedSupportConnectionState::OutOfRange
    );
    assert!(outside.members.is_empty());
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[0],
        3.5,
    );
    core.advance_active_battle_for_server_tick(10).unwrap();
    native_probe::damage_probe(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        enemy,
        allies[0],
        2,
        21,
    );
    assert_eq!(view(&core), first);
    let recipient_employee =
        core.state.active_battle.as_ref().unwrap().battle.units[&allies[0]].source_owned_uuid;
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: recipient_employee,
        },
    )
    .unwrap();
    let terminal = view(&core);
    assert_eq!(terminal.activation, first.activation);
    assert_eq!(
        terminal.fixed_connection.as_ref().unwrap().state,
        FixedSupportConnectionState::TerminalInvalid
    );
    assert!(terminal.members.is_empty());
    native_probe::place_stationary_x(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        allies[1],
        3.5,
    );
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(
        view(&core),
        terminal,
        "other eligible ally cannot inherit a terminal connection"
    );
    assert!(native_probe::channel_active(
        &core.state.active_battle.as_ref().unwrap().battle,
        actor
    ));
    println!(
        "RV002 fixed_support self_carrier=true selected_only=true original_reach2 caster_excluded=true regular40_speed20 triggered0 out_return_same_activation=true recipient_withdraw_terminal_no_retarget_source_channel_alive=true"
    );
}

#[test]
fn f01_publication_refusal_preserves_live_catalog_loadout_resources_and_owner() {
    // A conflict is rejected while constructing the candidate generation. It
    // cannot become a legal Equip/Corrupt command in a published World; do not
    // manufacture such an impossible catalog. Retain an actually deployed,
    // normally published equal-reach World across every attempted replacement.
    let mut unexpected = Vec::new();
    for finite in [false, true] {
        let (mut core, player, employee, _, allies, _) =
            personal_cases::team_fixture_with_generated_count(
                &f01_mixed_skill(finite),
                &f01_mixed_relic(finite, 2_000_000),
                1,
                |_, _, _| {},
            );
        let original_data = core.game_data.clone();
        let before_state = core.get_run_snapshot_json().unwrap();
        let before_plan = serde_json::to_value(query(&mut core, player, employee)).unwrap();
        let before_roots =
            native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
        let owner = native_probe::effective_owner_identity(
            &core.state.active_battle.as_ref().unwrap().battle,
        );
        assert!(owner.is_some());
        for reach in [1_000_000, 4_000_000] {
            let candidate: RelicDatabase =
                ron::de::from_str(&f01_mixed_relic(finite, reach)).unwrap();
            let result = GameDataBuilder::from_validated(&core.game_data)
                .with_relics(candidate)
                .build_with_runtime_safety(
                    crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
                );
            let actual = match result {
                Ok(_) => "published".to_string(),
                Err(error) => format!("{}: {error}", error.code),
            };
            if !actual.starts_with("relic_structural_invalid_reference:")
                || !actual.contains("selected support reach conflict")
            {
                unexpected.push(format!("finite={finite} reach={reach}: {actual}"));
            }
            assert!(std::ptr::eq(
                core.game_data.database(),
                original_data.database()
            ));
            assert_eq!(core.game_data.identity(), original_data.identity());
            assert_eq!(core.get_run_snapshot_json().unwrap(), before_state);
            assert_eq!(
                native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
                before_roots
            );
            assert_eq!(
                native_probe::effective_owner_identity(
                    &core.state.active_battle.as_ref().unwrap().battle
                ),
                owner
            );
            assert_eq!(
                serde_json::to_value(query(&mut core, player, employee)).unwrap(),
                before_plan
            );
            println!(
                "F01 STAGED finite={finite} reach={reach} actual={actual} catalog_loadout_resources_save_roots_owner_unchanged=true invalid_world_command=not_publishable finite_child_uses_separate_instant_anchor={finite}"
            );
        }
        // The retained legal generation still admits and executes its command.
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::BeginSkillTargeting {
                employee_uuid: employee,
                skill_id: "tc_bite".into(),
            },
        )
        .unwrap();
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: "tc_bite".into(),
                target: Some(SkillCastTarget::Unit {
                    unit_instance_id: allies[0],
                }),
            },
        )
        .unwrap();
        core.advance_active_battle_for_server_tick(1).unwrap();
        let pools = personal_cases::pools(&core);
        if pools.len() != 1 {
            println!(
                "F01 STAGED diagnostic finite={finite} {}",
                native_probe::selected_flow_diagnostic(
                    &core.state.active_battle.as_ref().unwrap().battle
                )
            );
        }
        assert_eq!(pools.len(), 1);
        assert_eq!(pools[0].target, allies[0]);
        assert_eq!(pools[0].shield.unwrap().capacity, 200);
        assert_eq!(
            native_probe::support_bonus(
                &core.state.active_battle.as_ref().unwrap().battle,
                allies[0],
                true
            ),
            (40, 20)
        );
    }
    assert!(
        unexpected.is_empty(),
        "F01 candidate publications unexpectedly accepted: {unexpected:#?}"
    );
}

#[test]
fn rv002_support_regular_triggered_legacy_and_released_credit_resource_isolation() {
    // The shared speed contract rounds intervals up: at time1, a1000ms
    // interval with15/20/30 bonus points next fires at871/835/771ms.
    for (form, regular_damage, triggered_damage, speed, next_attack_ms) in [
        ("area", 120, 100, 15, 871),
        ("fixed", 140, 100, 20, 835),
        ("fixed_corrupt", 180, 100, 30, 771),
        ("legacy", 120, 120, 15, 871),
    ] {
        let relic = if form.starts_with("fixed") {
            let value = support_relic();
            if form == "fixed_corrupt" {
                // Keep the normal 40/20 definition and publish a distinct
                // corrupted 80/30 variant. The World command below selects it.
                let mut normal: RelicDatabase = ron::de::from_str(&value).unwrap();
                let corrupt: RelicDatabase = ron::de::from_str(
                    &value
                        .replace("Add(PercentPoints(20))", "Add(PercentPoints(60))")
                        .replace("Add(SpeedPoints(5))", "Add(SpeedPoints(15))"),
                )
                .unwrap();
                normal.relics[0].corruption =
                    Some(crate::game::data::relic_data::RelicCorruptionVariant {
                        effects: vec![],
                        rules: corrupt.relics[0].rules.clone(),
                        penalties: vec![crate::game::data::relic_data::RelicPenalty::DefenseFlat {
                            amount: -1,
                        }],
                    });
                ron::ser::to_string(&normal).unwrap()
            } else {
                value
            }
        } else {
            let base = relic_text(None);
            let end = base.find("rules: [").unwrap();
            let operation = if form == "legacy" {
                "Edit(property: SupportAreaRadius, operation: Scale(increment_percent: 0))"
            } else {
                "Recipe(RegularBasicSupport(damage_percent: 20, speed_points: 15))"
            };
            format!(
                r#"{}rules: [(id: "area", selector: (provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(ChannelAura))), apply: {operation})], )])"#,
                &base[..end]
            )
        };
        let (base, player, employee, caster, allies, enemy) =
            personal_cases::team_fixture_with_setup(support_skill(), &relic, |core, player, _| {
                if form == "fixed_corrupt" {
                    let copy = core.state.relics.copies().next().unwrap().copy_uuid;
                    assert_eq!(
                        core.state.relics.get(copy).unwrap().variant,
                        crate::game::relic::RelicVariant::Normal
                    );
                    core.execute_admin_command(
                        crate::game::world::AdminCommand::AdminGrantRelicCorruptionCatalyst {
                            amount: 1,
                        },
                    )
                    .unwrap();
                    core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
                        .unwrap();
                    assert_eq!(
                        core.state.relics.get(copy).unwrap().variant,
                        crate::game::relic::RelicVariant::Corrupt
                    );
                    assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 0);
                }
            });
        for regular in [false, true] {
            let mut core = base.clone();
            let spent_before = core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .runtime_fact_summary()
                .resource_spent;
            if form.starts_with("fixed") {
                execute_current_battle_mutation(
                    &mut core,
                    player,
                    PlayerBehavior::BeginSkillTargeting {
                        employee_uuid: employee,
                        skill_id: "tc_bite".into(),
                    },
                )
                .unwrap();
            }
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::ActivateSkill {
                    employee_uuid: employee,
                    skill_id: "tc_bite".into(),
                    target: form.starts_with("fixed").then_some(SkillCastTarget::Unit {
                        unit_instance_id: allies[0],
                    }),
                },
            )
            .unwrap();
            core.advance_active_battle_for_server_tick(1).unwrap();
            let (target, source) = native_probe::start_basic_for_support(
                &mut core.state.active_battle.as_mut().unwrap().battle,
                allies[0],
                enemy,
                regular,
                1,
            );
            let expected = if regular {
                regular_damage
            } else {
                triggered_damage
            };
            assert_eq!(source.base_damage, expected);
            assert_eq!(source.source_id, allies[0]);
            assert_eq!(source.fact_source.actor, Some(allies[0]));
            assert!(source.fact_source.cast.is_none());
            let contributors = native_probe::support_contributors(&source);
            if regular && form != "legacy" {
                assert_eq!(contributors, vec![(expected - 100, speed)]);
            } else {
                assert!(contributors.is_empty());
            }
            let battle = &core.state.active_battle.as_ref().unwrap().battle;
            if regular {
                assert_eq!(
                    battle.units[&allies[0]].next_basic_attack_ms,
                    next_attack_ms
                );
            }
            if regular && form == "fixed_corrupt" {
                assert_eq!(
                    battle.units[&allies[0]].next_basic_attack_ms, 771,
                    "literal 1000ms interval at 130 speed, released at time1"
                );
            }
            assert_eq!(
                battle.runtime_fact_summary().resource_spent - spent_before,
                100
            );
            let hp_before = battle.units[&target].stats.current_health;
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::WithdrawUnit {
                    employee_uuid: employee,
                },
            )
            .unwrap();
            assert!(!native_probe::channel_active(
                &core.state.active_battle.as_ref().unwrap().battle,
                caster
            ));
            core.advance_active_battle_for_server_tick(21).unwrap();
            let battle = &core.state.active_battle.as_ref().unwrap().battle;
            assert_eq!(
                battle.units[&target].stats.current_health,
                hp_before - expected
            );
            let facts = battle.runtime_fact_summary();
            assert_eq!(
                facts.resource_spent - spent_before,
                100,
                "support origin cannot spend caster resources again"
            );
            assert_eq!(
                facts.latest_damage.as_ref().unwrap().source.actor,
                Some(allies[0])
            );
            println!(
                "RV002 support form={form} actual_corrupt={} regular={regular} damage={expected} speed={speed} released_survives_provider_withdraw=true credit=attacker cost100_no_double_spend=true contributors_no_authority=true",
                form == "fixed_corrupt"
            );
        }
    }
}

pub(super) fn with_aux(main: &str, aux: &str) -> String {
    format!("{}, {}])", &main[..main.rfind("])").unwrap()], aux)
}

pub(super) fn prepare_aux(core: &mut GameCore, player: Uuid, employee: Uuid) {
    let other = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .into_iter()
        .find(|id| *id != employee)
        .unwrap();
    let serum = consumable_meta(
        0xc021,
        "rv_aux_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&other)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xc022), &serum);
    let copy = core
        .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
            fragment_id: "rv_aux_fragment".into(),
        }])
        .unwrap()
        .skill_fragment_diffs[0]
        .copy_uuid;
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: other,
            copy_uuid: copy,
        },
    )
    .unwrap();
}

fn regular_area_relic() -> String {
    let base = relic_text(None);
    let end = base.find("rules: [").unwrap();
    format!(
        r#"{}rules: [(id: "regular", selector: (provider: (fragment_id: Some("tc_fragment")), effect: (op: Some(ChannelAura))), apply: Recipe(RegularBasicSupport(damage_percent: 40, speed_points: 20)))], )])"#,
        &base[..end]
    )
}

#[test]
fn rv002_support_eligibility_precedes_family_priority_ties_and_distinct_family_sum() {
    for (family, priority, expected_regular) in [
        ("rv_support", 0, 40),
        ("rv_support", 1, 20),
        ("other", 0, 60),
    ] {
        let aux = support_skill()
            .replace("tc_bite", "rv_aux")
            .replace("rv_support", family)
            .replace("priority: 0", &format!("priority: {priority}"));
        let aux = &aux[aux.find("SkillDef(").unwrap()..aux.rfind("])").unwrap()];
        let skills = with_aux(support_skill(), aux);
        let (mut core, player, employee, actor, allies, enemy) =
            personal_cases::team_fixture_with_setup(&skills, &regular_area_relic(), prepare_aux);
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
        let recipient = allies.iter().copied().find(|id| *id != auxiliary).unwrap();
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
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        assert!(native_probe::channel_active(battle, actor));
        assert!(native_probe::channel_active(battle, auxiliary));
        assert_eq!(
            native_probe::support_bonus(battle, recipient, true).0,
            expected_regular
        );
        assert_eq!(
            native_probe::support_bonus(battle, recipient, false),
            (20, 15)
        );
        for regular in [false, true] {
            let mut clone = core.clone();
            let (_, source) = native_probe::start_basic_for_support(
                &mut clone.state.active_battle.as_mut().unwrap().battle,
                recipient,
                enemy,
                regular,
                2,
            );
            assert_eq!(
                source.base_damage,
                100 + if regular { expected_regular } else { 20 }
            );
            let mut contributors = native_probe::support_contributors(&source);
            contributors.sort();
            let expected = if !regular || expected_regular == 20 {
                vec![(20, 15)]
            } else if expected_regular == 40 {
                vec![(40, 20)]
            } else {
                vec![(20, 15), (40, 20)]
            };
            assert_eq!(contributors, expected);
        }
        println!(
            "RV002 support_family={family} legacy_priority={priority} regular_bonus={expected_regular} triggered_legacy20 eligibility_before_winner=true earlier_activation_tie=true contributor_winners_exact=true"
        );
    }
}

#[test]
fn rv002_authored_extra_attack_remains_ineligible_for_regular_only_coefficients() {
    let aux = r#"SkillDef(id: "rv_aux", name: "Authored extra", kind: Targeted, focus_time_ms: 0, use_lifetime: Action,
        cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
        steps: [(id: "extra", delay_ms: 0, target: CastTarget, delivery: Instant,
            effects: [(id: "one_extra", op: ExtraAttack(count: 1))])])"#;
    let (mut core, player, employee, _, allies, _) = personal_cases::team_fixture_with_setup(
        &with_aux(support_skill(), aux),
        &regular_area_relic(),
        prepare_aux,
    );
    let attacker = allies
        .iter()
        .copied()
        .find(|id| {
            core.state.active_battle.as_ref().unwrap().battle.units[id]
                .skill_id
                .as_ref()
                .is_some_and(|id| id.as_str() == "rv_aux")
        })
        .unwrap();
    let attacker_employee =
        core.state.active_battle.as_ref().unwrap().battle.units[&attacker].source_owned_uuid;
    cast(&mut core, player, employee);
    native_probe::prepare_support_attacker(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        attacker,
    );
    let spent = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_summary()
        .resource_spent;
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: attacker_employee,
            skill_id: "rv_aux".into(),
            target: None,
        },
    )
    .unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    let battle = &core.state.active_battle.as_ref().unwrap().battle;
    let attacks = native_probe::pending_support_attacks(battle, attacker);
    assert_eq!(attacks.len(), 1);
    let (target, source) = &attacks[0];
    assert_eq!(source.base_damage, 100);
    assert!(native_probe::support_contributors(source).is_empty());
    assert_eq!(source.source_id, attacker);
    assert_eq!(
        battle.units[&attacker].next_basic_attack_ms, 1002,
        "ordinary skill recovery is one unmodified 1000ms interval"
    );
    let hp_before = battle.units[target].stats.current_health;
    core.advance_active_battle_for_server_tick(21).unwrap();
    let battle = &core.state.active_battle.as_ref().unwrap().battle;
    assert_eq!(battle.units[target].stats.current_health, hp_before - 100);
    assert_eq!(
        battle.runtime_fact_summary().resource_spent - spent,
        100,
        "one auxiliary skill cost, no support recost"
    );
    println!(
        "RV002 authored_ExtraAttack damage100 regular40_ignored=true ordinary_skill_recovery1000_preserved=true aux_cost100_only=true"
    );
}

#[test]
fn rv005_fixed_channel_preserves_authored_health_cost_pause_stop_cc_and_insufficient_atomicity() {
    use crate::game::battle::runtime_control::RuntimeFaultCode;
    use crate::game::battle::runtime_facts::{CostReason, FactResource};
    let skills = support_skill()
        .replace("Original self-carried channel", "rv_control_fixed_cost")
        .replace(
            "effects: [(id: \"channel\",",
            r#"effects: [(id: "payment", op: HealthCost(amount: 10, allow_lethal: false)), (id: "channel","#,
        );
    for mode in ["reinput", "stun", "insufficient"] {
        let (mut core, player, employee, actor, allies, enemy) =
            personal_cases::team_fixture(&skills, &support_relic());
        if mode == "insufficient" {
            core.state
                .active_battle
                .as_mut()
                .unwrap()
                .battle
                .units
                .get_mut(&actor)
                .unwrap()
                .stats
                .current_health = 10;
        }
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::BeginSkillTargeting {
                employee_uuid: employee,
                skill_id: "tc_bite".into(),
            },
        )
        .unwrap();
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: "tc_bite".into(),
                target: Some(SkillCastTarget::Unit {
                    unit_instance_id: allies[0],
                }),
            },
        )
        .unwrap();
        if mode == "insufficient" {
            // Acceptance has committed. The following failing consumption
            // tick must preserve that exact state, not refund prior acceptance.
            let before =
                native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
            let saved = core.get_run_snapshot_json().unwrap();
            let error = core.advance_active_battle_for_server_tick(1).unwrap_err();
            assert!(
                matches!(error, GameError::BattleRuntimeFault(ref fault) if fault.code == RuntimeFaultCode::HealthCostUnavailable),
                "{error:?}"
            );
            assert_eq!(
                native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
                before
            );
            assert_eq!(core.get_run_snapshot_json().unwrap(), saved);
            assert_eq!(hp(&core, &[actor]), 10);
            assert!(!native_probe::channel_active(
                &core.state.active_battle.as_ref().unwrap().battle,
                actor
            ));
            assert!(
                core.state
                    .active_battle
                    .as_ref()
                    .unwrap()
                    .battle
                    .sustained_effect_snapshot()
                    .unwrap()
                    .active
                    .is_empty()
            );
            // Repair only fixture health and retry the same accepted payload.
            core.state
                .active_battle
                .as_mut()
                .unwrap()
                .battle
                .units
                .get_mut(&actor)
                .unwrap()
                .stats
                .current_health = 500;
        }
        core.advance_active_battle_for_server_tick(1).unwrap();
        assert_eq!(hp(&core, &[actor]), 490);
        assert_eq!(
            hp(&core, &[allies[0]]),
            1000,
            "authored Self cost never moves to selected recipient"
        );
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        let snapshot = battle.sustained_effect_snapshot().unwrap();
        assert_eq!(snapshot.active.len(), 1);
        let channel = &snapshot.active[0];
        assert_eq!((channel.caster, channel.target), (actor, actor));
        assert_eq!(channel.members, vec![allies[0]]);
        assert_eq!(
            native_probe::support_bonus(battle, allies[0], true),
            (40, 20)
        );
        let facts = battle.runtime_fact_summary();
        let cost = facts.latest_cost.as_ref().unwrap();
        assert_eq!(cost.resource, FactResource::Health);
        assert_eq!(cost.reason, CostReason::AuthoredHealth);
        assert_eq!(cost.actual_spent(), 10);
        assert_eq!(facts.damage_hp_lost, 0);
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::PauseBattle).unwrap();
        let paused = native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
        assert!(
            core.advance_active_battle_for_server_tick(100)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
            paused
        );
        assert_eq!(
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .sustained_effect_snapshot()
                .unwrap(),
            snapshot
        );
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::ResumeBattle).unwrap();
        if mode == "stun" {
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
                        buff_id: crate::game::battle::buffs::BuffId::from_name("rv_control_stun"),
                        duration_ms: 100,
                        cause: Default::default(),
                    },
                    1,
                )
                .unwrap();
            assert!(
                core.state
                    .active_battle
                    .as_ref()
                    .unwrap()
                    .battle
                    .unit_control_state(actor, 1)
                    .unwrap()
                    .hard_cc_active()
            );
        } else {
            // The public stop command consumes the observed live activation;
            // a new ActivateSkill request requires a new targeting focus.
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::ReinputSustainedSkill {
                    employee_uuid: employee,
                    skill_id: "tc_bite".into(),
                    activation: channel.activation,
                    target: None,
                },
            )
            .unwrap();
        }
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        assert!(!native_probe::channel_active(battle, actor));
        assert_eq!(native_probe::support_bonus(battle, allies[0], true), (0, 0));
        assert_eq!(
            hp(&core, &[actor]),
            490,
            "stop/CC cannot repeat authored HP cost"
        );
        assert_eq!(battle.sustained_effect_snapshot().unwrap().ended, 1);
        println!(
            "RV005 fixed_channel mode={mode} self_cost10=500to490 selected_hp1000=true pause_unchanged=true stop_clears_support_no_recost=true insufficient_tick_atomic_retry={}",
            mode == "insufficient"
        );
    }
}
