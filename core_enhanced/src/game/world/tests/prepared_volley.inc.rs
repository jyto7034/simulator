fn prepared_volley_world_fixture(deploy: bool, defeat: bool) -> (GameCore, Uuid, Uuid, Uuid) {
    use crate::game::data::pve_data::*;
    let mut doc = crate::authoring::node_graph::tests::prepared_volley_fixture();
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-portal-policy" {
            n["values"]["maximum"] = json!(20);
        }
        if n["kind"] == "skill-seconds" {
            let title = n["title"].as_str().unwrap();
            if title.contains("첫 포탈") {
                n["values"]["value"] = json!(0.01);
            } else if title.contains("마지막 포탈") {
                n["values"]["value"] = json!(0.005);
            }
        }
    }
    let native = crate::authoring::node_graph::compile(&doc.to_string(), "r08_strike".into())
        .unwrap()
        .load_native()
        .unwrap();
    let fragments: SkillFragmentDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/r09/fragments.ron"
    )))
    .unwrap();
    let fragment = fragments.fragments[0].clone();
    let mut data = game_data_with_pve_equipment_and_active_skill_fragments(
        vec![],
        native.skills,
        vec![fragment.clone()],
    );
    let db = Arc::get_mut(&mut data).unwrap();
    let pve = Arc::make_mut(&mut db.pve_data);
    let encounter = pve
        .encounters
        .iter_mut()
        .find(|e| e.id == "defense_encounter")
        .unwrap();
    encounter.terminal_outcomes = vec![PveTerminalOutcomeData {
        id: "volley_timed".into(),
        verdict: if defeat {
            BattleVerdict::Defeat
        } else {
            BattleVerdict::Victory
        },
        condition: PveTerminalConditionData::TimeReached { time_ms: 500 },
        priority: 0,
    }];
    encounter.outcome_policies = vec![PveOutcomePolicyData {
        outcome_id: "volley_timed".into(),
        reward_mode: RewardMode::ClaimAll,
        reward_uuids: vec![],
        grant_node_bonus: false,
        advance_encounter_progression: false,
    }];
    if defeat {
        encounter.terminal_outcomes.push(PveTerminalOutcomeData {
            id: "volley_later_victory".into(),
            verdict: BattleVerdict::Victory,
            condition: PveTerminalConditionData::TimeReached { time_ms: 1000 },
            priority: 0,
        });
        encounter.outcome_policies.push(PveOutcomePolicyData {
            outcome_id: "volley_later_victory".into(),
            reward_mode: RewardMode::ClaimAll,
            reward_uuids: vec![],
            grant_node_bonus: false,
            advance_encounter_progression: false,
        });
    }
    for w in &mut encounter.waves {
        w.required_for_victory = false;
    }
    let mut core = GameCore::new(data, 0xabc88);
    let player = Uuid::from_u128(0xabc88);
    start_new_game_with_default_starters(&mut core, player);
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let serum = consumable_meta(
        0xabc89,
        "volley_initial_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xabc90), &serum);
    let copy = core
        .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
            fragment_id: fragment.id,
        }])
        .unwrap()
        .skill_fragment_diffs[0]
        .copy_uuid;
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee,
            copy_uuid: copy,
        },
    )
    .unwrap();
    prepared_volley_enter_world(&mut core, player, employee, deploy);
    (core, player, employee, copy)
}
fn prepared_volley_enter_world(core: &mut GameCore, player: Uuid, employee: Uuid, deploy: bool) {
    let node = force_map_combat_node(
        core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    let preview = core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let position = first_ground_deployment_cell(&preview);
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    if deploy {
        execute_current_battle_mutation(
            core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: employee,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    }
}
fn prepared_volley_borrow_world(core: &mut GameCore, player: Uuid, employee: Uuid) {
    execute_current_battle_mutation(
        core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "r08_strike".into(),
            target: None,
        },
    )
    .unwrap();
    core.advance_active_battle_for_server_tick(250).unwrap();
    let v = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .prepared_volleys_snapshot()
        .unwrap();
    assert_eq!(v.owners[0].balance, -10);
    assert!(v.owners[0].borrowed);
}
#[test]
fn prepared_volley_world_result_settles_once_and_undeployed_next_battle_repays_debt() {
    let (mut c, p, e, k) = prepared_volley_world_fixture(true, false);
    prepared_volley_borrow_world(&mut c, p, e);
    assert!(c
        .state
        .run
        .as_ref()
        .unwrap()
        .credit_ledgers
        .entries
        .is_empty());
    c.advance_active_battle_for_server_tick(250).unwrap();
    assert!(c.state.active_battle.is_none());
    let entry = &c.state.run.as_ref().unwrap().credit_ledgers.entries[&k];
    assert_eq!(entry.unpaid, 10);
    assert!(entry.borrowed);
    c.execute(p, PlayerBehavior::CompleteCombatResult).unwrap();
    let prior = c.state.run.as_ref().unwrap().credit_ledgers.clone();
    assert!(c.execute(p, PlayerBehavior::CompleteCombatResult).is_err());
    assert_eq!(prior, c.state.run.as_ref().unwrap().credit_ledgers);
    prepared_volley_enter_world(&mut c, p, e, false);
    let v = c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .prepared_volleys_snapshot()
        .unwrap();
    assert_eq!(v.owners.len(), 1);
    assert_eq!(v.owners[0].caster, None);
    assert_eq!(v.owners[0].balance, 0);
    assert!(!v.owners[0].can_borrow);
    c.advance_active_battle_for_server_tick(500).unwrap();
    let entry = &c.state.run.as_ref().unwrap().credit_ledgers.entries[&k];
    assert_eq!(entry.unpaid, 0);
    assert!(!entry.borrowed);
}
#[test]
fn prepared_volley_world_retreat_restores_entry_ledger_and_reentry_gets_fresh_grant() {
    let (mut c, p, e, _) = prepared_volley_world_fixture(true, false);
    let before = c.state.run.as_ref().unwrap().credit_ledgers.clone();
    prepared_volley_borrow_world(&mut c, p, e);
    let battle_before = battle_advance_atomic_snapshot(&c);
    let result = c.with_state_transaction(|c| {
        execute_current_battle_mutation(c, p, PlayerBehavior::RetreatBattle)?;
        Err::<(), _>(GameError::InvalidAction)
    });
    assert!(result.is_err());
    assert_eq!(battle_before, battle_advance_atomic_snapshot(&c));
    execute_current_battle_mutation(&mut c, p, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(before, c.state.run.as_ref().unwrap().credit_ledgers);
    c.execute(p, PlayerBehavior::ConfirmEnterNode).unwrap();
    let v = c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .prepared_volleys_snapshot()
        .unwrap();
    assert_eq!(v.owners[0].balance, 10);
    assert!(v.owners[0].can_borrow);
    assert!(!v.owners[0].borrowed);
}
#[test]
fn prepared_volley_world_continue_and_transfer_preserve_exact_copy_debt() {
    let (mut c, p, e, k) = prepared_volley_world_fixture(true, false);
    prepared_volley_borrow_world(&mut c, p, e);
    c.advance_active_battle_for_server_tick(250).unwrap();
    c.execute(p, PlayerBehavior::CompleteCombatResult).unwrap();
    c.save_run_checkpoint_for_test().unwrap();
    let saved = c.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(&c, 1);
    restored
        .continue_from_active_run_checkpoint(
            p,
            &saved.core_payload_json,
            0,
            saved.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(
        c.state.run.as_ref().unwrap().credit_ledgers,
        restored.state.run.as_ref().unwrap().credit_ledgers
    );
    let other = restored
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .into_iter()
        .find(|id| *id != e)
        .unwrap();
    restored
        .execute(
            p,
            PlayerBehavior::TransferSkillFragment {
                target_employee_uuid: other,
                copy_uuid: k,
            },
        )
        .unwrap();
    assert_eq!(
        restored.state.run.as_ref().unwrap().credit_ledgers.entries[&k].unpaid,
        10
    );
    prepared_volley_enter_world(&mut restored, p, other, false);
    let v = restored
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .prepared_volleys_snapshot()
        .unwrap();
    assert_eq!(v.owners[0].copy, k);
    assert_eq!(v.owners[0].employee, other);
    assert_eq!(v.owners[0].balance, 0);
    assert!(!v.owners[0].can_borrow);
}

#[test]
fn prepared_volley_world_unequipped_battle_skips_debt_and_run_end_resets_it() {
    let (mut c, p, e, k) = prepared_volley_world_fixture(true, false);
    prepared_volley_borrow_world(&mut c, p, e);
    c.advance_active_battle_for_server_tick(250).unwrap();
    c.execute(p, PlayerBehavior::CompleteCombatResult).unwrap();
    c.execute(
        p,
        PlayerBehavior::UnequipSkillFragment {
            employee_uuid: e,
            copy_uuid: k,
        },
    )
    .unwrap();
    let prior = c.state.run.as_ref().unwrap().credit_ledgers.clone();
    prepared_volley_enter_world(&mut c, p, e, false);
    assert!(c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .prepared_volleys_snapshot()
        .unwrap()
        .owners
        .is_empty());
    c.advance_active_battle_for_server_tick(500).unwrap();
    c.execute(p, PlayerBehavior::CompleteCombatResult).unwrap();
    assert_eq!(prior, c.state.run.as_ref().unwrap().credit_ledgers);
    c.execute(p, PlayerBehavior::AbandonRun).unwrap();
    assert!(c
        .state
        .run
        .as_ref()
        .is_none_or(|r| r.credit_ledgers.entries.is_empty()));
    start_new_game_with_default_starters(&mut c, p);
    assert!(c
        .state
        .run
        .as_ref()
        .unwrap()
        .credit_ledgers
        .entries
        .is_empty());
}
#[test]
fn prepared_volley_world_defeat_settles_before_acknowledgement_once() {
    let (mut c, p, e, k) = prepared_volley_world_fixture(true, true);
    prepared_volley_borrow_world(&mut c, p, e);
    c.advance_active_battle_for_server_tick(250).unwrap();
    assert!(matches!(
        &c.state.confirmed_combat_result.as_ref().unwrap().summary,
        crate::game::behavior::SelectedEventSnapshotDto::CombatBattle {
            verdict: BattleVerdict::Defeat,
            ..
        }
    ));
    assert_eq!(
        c.state.run.as_ref().unwrap().credit_ledgers.entries[&k].unpaid,
        10
    );
    let prior = c.state.run.as_ref().unwrap().credit_ledgers.clone();
    c.execute(p, PlayerBehavior::CompleteCombatResult).unwrap();
    assert_eq!(prior, c.state.run.as_ref().unwrap().credit_ledgers);
    assert!(c.execute(p, PlayerBehavior::CompleteCombatResult).is_err());
    assert_eq!(prior, c.state.run.as_ref().unwrap().credit_ledgers);
}
