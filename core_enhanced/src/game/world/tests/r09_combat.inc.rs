
fn r09_live_corrosion_fixture() -> (GameCore, Uuid, Uuid) { r09_live_corrosion_with_later_wave(false) }
fn r09_live_corrosion_with_later_wave(later: bool) -> (GameCore, Uuid, Uuid) { r09_live_corrosion_configured(later, false) }
fn r09_live_corrosion_configured(later: bool, structured: bool) -> (GameCore, Uuid, Uuid) { r10_live_flow_fixture(later, structured, 80) }
fn r10_live_flow_fixture(later: bool, structured: bool, trauma: u32) -> (GameCore, Uuid, Uuid) { r11_live_sustained_fixture(later, structured, trauma, None) }
fn r11_live_sustained_fixture(later: bool, structured: bool, trauma: u32, authored: Option<&str>) -> (GameCore, Uuid, Uuid) {
    r12_live_guard_fixture(later, structured, trauma, authored, false)
}
fn r12_live_guard_fixture(later: bool, structured: bool, trauma: u32, authored: Option<&str>, guard: bool) -> (GameCore, Uuid, Uuid) {
    r12_live_full_fixture(later, structured, trauma, authored, guard, false)
}
fn r12_live_full_fixture(later: bool, structured: bool, trauma: u32, authored: Option<&str>, guard: bool, hostile: bool) -> (GameCore, Uuid, Uuid) {
    r12_live_cross_fixture(later, structured, trauma, authored, guard, hostile, false)
}
fn r12_live_cross_fixture(later: bool, structured: bool, trauma: u32, authored: Option<&str>, guard: bool, hostile: bool, boss: bool) -> (GameCore, Uuid, Uuid) {
    r12_live_cross_deployment_fixture(later,structured,trauma,authored,guard,hostile,boss,true)
}
fn r12_live_cross_deployment_fixture(later: bool, structured: bool, trauma: u32, authored: Option<&str>, guard: bool, hostile: bool, boss: bool, deploy:bool) -> (GameCore, Uuid, Uuid) {
    let skills = SkillDatabase::from_ron("r09/world_skills.ron", &include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r08/skills.ron")).replace(" steps:", if structured { " flow: Some(Repeat(id: \"r09_flow\", count: 3, interval_ms: 10, body: Action(id: \"r09_action\", step_id: \"hit\", wait: EffectsSettled))), steps:" } else { " steps:" }).replace("selection: automatic_unit, target: EnemySingle(rule: Nearest)", "selection: self, target: SelfUnit").replace("reach: global", "reach: pattern").replace("Damage(amount: 100, damage_type: True)", "Heal(amount: 100)")).unwrap();
    let skills = if structured && trauma == 0 { SkillDatabase::from_ron("r10/world_result.ron", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r10/world_result.ron"))).unwrap() } else { skills };
    let skills = authored.map(|ron| SkillDatabase::from_ron("r11/world.ron", ron).unwrap()).unwrap_or(skills);
    let fragments: SkillFragmentDatabase = ron::de::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r09/fragments.ron"))).unwrap();
    let fragments = if guard { ron::de::from_str(&include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r09/fragments.ron")).replace("ActiveSkill(forms:", "ActiveSkill(passive_guard: Some((reduction_percent: 30, reflection_chance_percent: 100, reflection_percent: 50)), forms:")).unwrap() } else { fragments };
    let fragment = fragments.fragments[0].clone();
    let mut core = GameCore::new(game_data_with_pve_equipment_and_active_skill_fragments(vec![], skills.skills.clone(), vec![fragment.clone()]), 909);
    // The terminal/future-wave cases exercise ordinary corrosion deaths. Boss
    // cases below deliberately retain their abnormality and percentage damage.
    if trauma >= 80 && !hostile && !boss {
        let mut encounters = core.game_data.pve_data.encounters.clone();
        let encounter = encounters.iter_mut().find(|e| e.id == "defense_encounter").unwrap();
        encounter.encounter_class = crate::game::data::pve_data::PveEncounterClass::Normal;
        encounter.primary_abnormality_id = None;
        encounter.waves[0].source = test_corroded_pve_wave("corroded_guard").source;
        let sources = core.game_data.pve_data.reward_sources.iter().filter(|source| source.encounter_id != "defense_encounter").cloned().collect();
        let data = GameDataBuilder::from_validated(&core.game_data).with_pve(PveEncounterDatabase::new_with_reward_sources(encounters, sources)).build_arc();
        core = GameCore::new(data, 909);
    }
    if hostile {
        let mut abnormalities = core.game_data.abnormality_data.items.clone();
        let enemy = abnormalities.iter_mut().find(|a| a.id == "defense_risk_abno").unwrap();
        enemy.max_health = 10000; enemy.attack = 100;
        enemy.basic_attack.range_policy = crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        enemy.basic_attack.defense_tile_range = None;
        enemy.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        enemy.basic_attack.ranged_reposition_ms = 0;
        enemy.basic_attack.windup_ms = 1000; enemy.basic_attack.interval_ms = 2000;
        enemy.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
        let data = GameDataBuilder::from_validated(&core.game_data).with_abnormalities(abnormalities).build_arc();
        core = GameCore::new(data, 909);
    }
    if later {
        let mut encounters = core.game_data.pve_data.encounters.clone();
        let encounter = encounters.iter_mut().find(|e| e.id == "defense_encounter").unwrap();
        let mut wave = encounter.waves[0].clone();
        wave.id = "wave_later".into();
        wave.activation = crate::game::data::pve_data::PveWaveActivationData::at_time("wave_later", 500);
        encounter.waves.push(wave);
        let data = GameDataBuilder::from_validated(&core.game_data).with_pve(PveEncounterDatabase::new_with_reward_sources(encounters, core.game_data.pve_data.reward_sources.clone())).build_arc();
        core = GameCore::new(data, 909);
    }
    if boss {
        use crate::game::data::{boss_runtime_data::*, pve_data::*};
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r08/boss.ron")).replace("entry_state_id: \"acquire\"", "entry_state_id: \"cast\"").replace("skill_id: \"r08_strike\", target: LockedUnit", "skill_id: \"r12_enemy_guard\", target: SelfUnit").replace("timeout_ms: 1000", "timeout_ms: 10000");
        let mut catalog: BossRuntimeCatalogData = ron::de::from_str(&source).unwrap();
        catalog.patterns[0].states.retain(|state| state.id.as_str() == "cast");
        let mut encounters = core.game_data.pve_data.encounters.clone();
        let encounter = encounters.iter_mut().find(|e| e.id == "defense_encounter").unwrap();
        encounter.boss_runtime = Some(PveBossRuntimeBindingData { encounter_id: BossEncounterRuntimeId::new("r08_encounter"), actors: vec![PveBossActorBindingData { actor_slot_id: BossActorSlotId::new("primary"), wave_id: encounter.waves[0].id.clone(), spawn_index: 0 }] });
        let data = GameDataBuilder::from_validated(&core.game_data).with_boss_runtime_data(std::sync::Arc::new(catalog.compile().unwrap())).with_pve(PveEncounterDatabase::new_with_reward_sources(encounters, core.game_data.pve_data.reward_sources.clone())).build_arc();
        core = GameCore::new(data, 909);
    }
    let player = Uuid::from_u128(909);
    start_new_game_with_default_starters(&mut core, player);
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let serum = consumable_meta(0x909, "r09_initial_charge", ConsumableTier::Critical, ConsumableEffect::InitialSkillCharge { percent: 100 });
    let unit = core.roster_mut().unwrap().get_mut(&employee).unwrap();
    unit.condition.trauma = trauma;
    unit.apply_consumable_modifier(Uuid::from_u128(0x9090), &serum);
    let copy = core.apply_grant_effects(&[RewardEffect::GrantSkillFragment { fragment_id: fragment.id.clone() }]).unwrap().skill_fragment_diffs[0].copy_uuid;
    core.execute(player, PlayerBehavior::EquipSkillFragment { employee_uuid: employee, copy_uuid: copy }).unwrap();
    if boss {
        let other = core.roster().unwrap().combat_eligible_employee_ids().into_iter().find(|id| *id != employee).unwrap();
        let unit = core.roster_mut().unwrap().get_mut(&other).unwrap();
        unit.condition.trauma = 80;
        unit.apply_consumable_modifier(Uuid::from_u128(0x9091), &serum);
        let copy = core.apply_grant_effects(&[RewardEffect::GrantSkillFragment { fragment_id: fragment.id.clone() }]).unwrap().skill_fragment_diffs[0].copy_uuid;
        core.execute(player, PlayerBehavior::EquipSkillFragment { employee_uuid: other, copy_uuid: copy }).unwrap();
    }
    let node = force_map_combat_node(&mut core, MapNodeCategory::Combat, "combat_defense", "defense_encounter");
    let preview = core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
    let position = first_ground_deployment_cell(&preview);
    core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    if deploy {execute_current_battle_mutation(&mut core, player, PlayerBehavior::DeployUnit { employee_uuid: employee, position, facing: FacingDirection::Right }).unwrap();}
    (core, player, employee)
}

#[test]
fn r09_world_pause_start_host_speed_terminal_employee_cost_and_save() { r09_world_corrosion_roundtrip(false); }
#[test]
fn r10_structured_flow_preserves_corrosion_start_freeze_results_and_save() { r09_world_corrosion_roundtrip(true); }
fn r09_world_corrosion_roundtrip(structured: bool) {
    use crate::game::behavior::BattlePlaybackSpeed;
    for speed in [BattlePlaybackSpeed::X0_5, BattlePlaybackSpeed::X1, BattlePlaybackSpeed::X2] {
        let (mut core, player, employee) = r09_live_corrosion_configured(false, structured);
        if speed != BattlePlaybackSpeed::X1 { execute_current_battle_mutation(&mut core, player, PlayerBehavior::SetBattleSpeed { speed }).unwrap(); }
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::PauseBattle).unwrap();
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
        let before = battle_advance_atomic_snapshot(&core);
        for _ in 0..2 {
            core.execute(player, PlayerBehavior::RequestBattleState { since_seq: None }).unwrap();
            assert!(core.advance_active_battle_for_server_tick(2000).unwrap().is_none());
            assert_eq!(battle_advance_atomic_snapshot(&core), before);
        }
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::ResumeBattle).unwrap();
        core.advance_active_battle_for_server_tick(2).unwrap();
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(active.battle.forced_pause_remaining_ms(), Some(1000));
        assert_eq!(active.employee_attempt.roster.get(&employee).unwrap().condition, EmployeeCondition::new(0, 100));
        assert!(matches!(core.get_state(), GameState::InBattle { .. }));
        let frozen = battle_advance_atomic_snapshot(&core);
        assert!(execute_current_battle_mutation(&mut core, player, PlayerBehavior::ResumeBattle).is_err());
        assert!(core.advance_active_battle_to_next_event_bucket().is_err());
        assert_eq!(battle_advance_atomic_snapshot(&core), frozen);
        core.auto_pause_active_battle_for_disconnect().unwrap();
        core.advance_active_battle_for_server_tick(999).unwrap();
        assert_eq!(core.state.active_battle.as_ref().unwrap().battle.forced_pause_remaining_ms(), Some(1));
        core.advance_active_battle_for_server_tick(9000).unwrap();
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(active.battle.forced_pause_remaining_ms(), None);
        assert!(active.playback.paused);
        assert_eq!(active.execution.last_event_time_ms(), 0);
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::ResumeBattle).unwrap();
        core.advance_active_battle_for_server_tick(2).unwrap();
        assert!(!matches!(core.get_state(), GameState::InBattle { .. }));
        core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
        let employee_state = core.state.roster.get(&employee).unwrap();
        assert_eq!(employee_state.condition, EmployeeCondition::new(0, 100));
        assert_eq!(employee_state.breakdown.remaining_cost(), Some(4));
        let expected_counter = employee_state.breakdown;
        assert!(core.execute(player, PlayerBehavior::CompleteCombatResult).is_err());
        core.save_run_checkpoint_for_test().unwrap();
        let saved = core.export_active_run_checkpoint().unwrap().unwrap();
        let mut restored = core_with_profile_for_continue(&core, 1);
        restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json, 0, 1).unwrap();
        assert_eq!(restored.state.roster.get(&employee).unwrap().breakdown, expected_counter);
    }
}

#[test]
fn r09_forced_pause_all_application_routes_reject_before_mutation() {
    use crate::game::world::application::ApplicationRoute;
    let (mut core, player, employee) = r09_live_corrosion_fixture();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    let before = battle_advance_atomic_snapshot(&core);
    let mut routes = ActionKind::ALL.iter().copied().filter(|kind| !matches!(kind, ActionKind::RequestBattleState | ActionKind::PreviewTacticalCommand | ActionKind::PreviewSkillTarget | ActionKind::RequestDeploymentRangePreview)).map(ApplicationRoute::Behavior).collect::<Vec<_>>();
    routes.extend([ApplicationRoute::BattleElapsedAdvance, ApplicationRoute::BattleEventBucket, ApplicationRoute::AdminCommand, ApplicationRoute::ActiveRunContinue, ApplicationRoute::MergeEncounterCompendiumProfile, ApplicationRoute::MergePlayerProfile, ApplicationRoute::ResetPlayerProfile, ApplicationRoute::ActiveRunRollbackRestore, ApplicationRoute::SetEnkephalinFixture]);
    for route in routes {
        let result: Result<(), GameError> = core.execute_application(route, |_| panic!("forced-pause operation must never run: {route:?}"));
        assert!(matches!(result, Err(GameError::ActionUnavailable { reason: ActionDisabledReasonCode::CorrosionForcedPause, .. })), "{route:?}: {result:?}");
        assert_eq!(battle_advance_atomic_snapshot(&core), before);
    }
    for _ in 0..3 { core.execute(player, PlayerBehavior::RequestBattleState { since_seq: None }).unwrap(); }
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
}

#[test]
fn r09_corrosion_captures_current_enemies_and_preserves_required_future_wave() {
    let (mut core, player, employee) = r09_live_corrosion_with_later_wave(true);
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    core.advance_active_battle_for_server_tick(1000).unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(!active.battle.units.values().any(|unit| unit.owner == crate::game::enums::Side::Opponent && unit.is_active()));
    core.advance_active_battle_for_server_tick(500).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.battle.units.values().any(|unit| unit.owner == crate::game::enums::Side::Opponent && unit.is_active()));
    assert_eq!(active.employee_attempt.roster.get(&employee).unwrap().condition.trauma, 100);
}

#[test]
fn r10_normal_world_flow_reaches_result_and_current_checkpoint() {
    let (mut core, player, employee) = r10_live_flow_fixture(false, true, 0);
    let prior_costs = core.state.active_battle.as_ref().unwrap().battle.runtime_fact_summary().cost_mutations;
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(800).unwrap();
    let active = core.state.active_battle.as_ref().expect("flow finishes before ordinary battle");
    assert_eq!(active.battle.forced_pause_remaining_ms(), None);
    assert_eq!(active.battle.runtime_fact_summary().cost_mutations, prior_costs + 1);
    assert_eq!(active.employee_attempt.roster.get(&employee).unwrap().condition.trauma, 0);
    assert_eq!(active.battle.event_log.entries.iter().filter(|entry| matches!(&entry.event, crate::game::battle::event_log::BattleLogEvent::AbilityStepTriggered { skill_id, .. } if skill_id.as_str() == "r08_strike")).count(), 3);
    for _ in 0..120 {
        if !matches!(core.get_state(), GameState::InBattle { .. }) { break; }
        core.advance_active_battle_for_server_tick(1000).unwrap();
    }
    assert!(!matches!(core.get_state(), GameState::InBattle { .. }));
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    core.save_run_checkpoint_for_test().unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 1);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json, 0, saved.run_peak_certified_power).unwrap();
    assert_eq!(restored.state.roster.get(&employee).unwrap().condition, core.state.roster.get(&employee).unwrap().condition);
    assert_eq!(restored.state.roster.get(&employee).unwrap().breakdown.remaining_cost(), None);
    assert!(restored.state.active_battle.is_none());
}

#[test]
fn r11_world_channel_pause_public_query_terminal_and_save() {
    let (mut core, player, employee) = r11_live_sustained_fixture(false, false, 0, Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r11/world_result.ron"))));
    let ally = core.roster().unwrap().combat_eligible_employee_ids().into_iter().find(|id| *id != employee).unwrap();
    let position = core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter().find(|z| z.kind == crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[1];
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::DeployUnit { employee_uuid: ally, position, facing: FacingDirection::Right }).unwrap();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(400).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    let snapshot = active.battle.sustained_effect_snapshot().unwrap();
    assert_eq!(snapshot.active.len(), 1);
    assert_eq!(snapshot.active[0].members.len(), 1);
    assert!(snapshot.active[0].fragment_copy_uuid.is_some());
    assert_eq!(snapshot.active[0].authored_site.as_ref().unwrap().effect_id, "heal");
    let query = core.execute(player, PlayerBehavior::RequestBattleState { since_seq: None }).unwrap();
    let json = serde_json::to_value(query).unwrap();
    assert!(json.to_string().contains("sustained_effects"));
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::PauseBattle).unwrap();
    core.advance_active_battle_for_server_tick(5000).unwrap();
    assert_eq!(core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap(), snapshot);
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ResumeBattle).unwrap();
    for _ in 0..40 {
        if !matches!(core.get_state(), GameState::InBattle { .. }) { break; }
        core.advance_active_battle_for_server_tick(1000).unwrap();
    }
    assert!(!matches!(core.get_state(), GameState::InBattle { .. }));
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    core.save_run_checkpoint_for_test().unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    assert!(!std::str::from_utf8(&saved.core_payload_json).unwrap().contains("sustained_effects"));
    let mut restored = core_with_profile_for_continue(&core, 1);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json, 0, saved.run_peak_certified_power).unwrap();
    assert!(restored.state.active_battle.is_none());
}
#[test]
fn r11_world_exact_fragment_hello_reinput_goodbye_result_and_save() {
    let (mut core, player, employee) = r11_live_sustained_fixture(false, false, 0, Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r11/hello_world_result.ron"))));
    let active = core.state.active_battle.as_mut().unwrap();
    active.battle.enable_authoring_observation_with_options(Default::default(), true);
    active.battle.bind_authoring_observation(crate::game::battle::core::authoring_observation::ObservationKey { entry_ordinal: 1, battle_uuid: active.battle_uuid }).unwrap();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(400).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    let snapshot = active.battle.sustained_effect_snapshot().unwrap();
    let hello = &snapshot.active[0];
    let copy = hello.fragment_copy_uuid.unwrap();
    let activation = hello.activation;
    assert_eq!(hello.goodbye_skill.as_ref().unwrap().as_str(), "r11_goodbye");
    let before_cost = active.battle.runtime_fact_summary().cost_mutations;
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ReinputSustainedSkill { employee_uuid: employee, skill_id: "r08_strike".into(), activation, target: None }).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.battle.sustained_effect_snapshot().unwrap().active.is_empty());
    assert_eq!(active.battle.runtime_fact_summary().cost_mutations, before_cost);
    let amount = active.battle.authoring_observation().unwrap().unwrap().latest_amount.as_ref().unwrap();
    assert_eq!(amount.source.fragment.as_ref().unwrap().copy_uuid, copy);
    assert_eq!(amount.source.cast.as_ref().unwrap().skill_id.as_str(), "r11_goodbye");
    assert_eq!(amount.source.reinput().unwrap().remaining_ms, 900);
    assert!(execute_current_battle_mutation(&mut core, player, PlayerBehavior::ReinputSustainedSkill { employee_uuid: employee, skill_id: "r08_strike".into(), activation, target: None }).is_err());
    for _ in 0..20 {
        if !matches!(core.get_state(), GameState::InBattle { .. }) { break; }
        core.advance_active_battle_for_server_tick(1000).unwrap();
    }
    assert!(!matches!(core.get_state(), GameState::InBattle { .. }));
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    core.save_run_checkpoint_for_test().unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 1);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json, 0, saved.run_peak_certified_power).unwrap();
    assert!(restored.state.active_battle.is_none());
}

#[test]
fn r11_world_retreat_discards_channel_and_overlay_and_restores_after_outer_failure() {
    for authored in [include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r11/world_result.ron")), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r11/hello_world_result.ron"))] {
        let (mut core, player, employee) = r11_live_sustained_fixture(false, false, 0, Some(authored));
        let ally = core.roster().unwrap().combat_eligible_employee_ids().into_iter().find(|id| *id != employee).unwrap();
        let position = core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter().find(|z| z.kind == crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[1];
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::DeployUnit { employee_uuid: ally, position, facing: FacingDirection::Right }).unwrap();
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
        core.advance_active_battle_for_server_tick(400).unwrap();
        let snapshot = core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap();
        assert_eq!(snapshot.active.len(), 1);
        let before = battle_advance_atomic_snapshot(&core);
        let result = core.with_state_transaction(|candidate| {
            execute_current_battle_mutation(candidate, player, PlayerBehavior::RetreatBattle)?;
            assert!(candidate.state.active_battle.is_none());
            Err::<(), _>(GameError::InvalidAction)
        });
        assert!(result.is_err());
        assert_eq!(battle_advance_atomic_snapshot(&core), before);
        assert_eq!(core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap(), snapshot);
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
        assert!(core.state.active_battle.is_none());
        core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
        let fresh = core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap();
        assert_eq!((fresh.started, fresh.ended, fresh.active.len()), (0, 0, 0));
    }
}

#[test]
fn r12_world_passive_guard_actual_copy_withdraw_redeploy_and_retreat() {
    let authored = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r12/passive_guard.ron"));
    let (mut core, player, employee) = r12_live_guard_fixture(true, false, 0, Some(authored), true);
    let snapshot = core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap();
    assert_eq!(snapshot.passive_guards.len(), 1);
    let original = snapshot.passive_guards[0].clone();
    assert_eq!(snapshot.started, 0);
    let result = core.execute(player, PlayerBehavior::RequestBattleState { since_seq: None }).unwrap();
    assert!(serde_json::to_string(&result).unwrap().contains(&original.fragment.copy_uuid.to_string()));
    let position = core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter().find(|z| z.kind == crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[0];
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(2).unwrap();
    let snapshot = core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap();
    assert_eq!(snapshot.passive_guards[0].fragment, original.fragment);
    assert_eq!(snapshot.active.len(), 1);
    assert_eq!(snapshot.active[0].fragment_copy_uuid, Some(original.fragment.copy_uuid));
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::WithdrawUnit { employee_uuid: employee }).unwrap();
    let after = core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap();
    assert!(after.passive_guards.is_empty() && after.active.is_empty());
    let mut deployed = false;
    for _ in 0..3000 {
        match execute_current_battle_mutation(&mut core, player, PlayerBehavior::DeployUnit { employee_uuid: employee, position, facing: FacingDirection::Right }) {
            Ok(_) => { deployed = true; break; }
            Err(GameError::ActionUnavailable { reason: ActionDisabledReasonCode::RedeployCooldown, .. }) => { core.advance_active_battle_to_next_event_bucket().unwrap(); }
            Err(error) => panic!("guard redeploy: {error:?}"),
        }
    }
    assert!(deployed);
    let snapshot = core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap();
    assert_eq!(snapshot.passive_guards.len(), 1);
    assert_eq!(snapshot.passive_guards[0].fragment, original.fragment);
    assert!(snapshot.passive_guards[0].reduction_suppressed_intervals_ms.is_empty());
    assert!(snapshot.active.is_empty());
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    assert!(core.state.active_battle.is_none());
    core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    assert!(core.state.active_battle.as_ref().unwrap().battle.sustained_effect_snapshot().unwrap().passive_guards.is_empty());
}

#[test]
fn r12_world_two_employees_shared_personal_parry_cost_result_and_fresh_restore() {
    let authored = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r12/world_protection.ron"));
    let (mut core, player, employee) = r12_live_full_fixture(false, false, 0, Some(authored), false, true);
    let ally = core.roster().unwrap().combat_eligible_employee_ids().into_iter().find(|id| *id != employee).unwrap();
    let position = core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter().find(|z| z.kind == crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[1];
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::DeployUnit { employee_uuid: ally, position, facing: FacingDirection::Right }).unwrap();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(400).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    let effects = active.battle.sustained_effect_snapshot().unwrap();
    let shared = effects.active.iter().find(|e| e.shield.is_some_and(|p| p.shared_radius_units.is_some())).unwrap();
    assert_eq!(shared.members.len(), 2);
    assert_eq!(shared.shield.unwrap().remaining, 600);
    assert!(effects.active.iter().any(|e| e.shield.is_some_and(|p| p.remaining == 100 && p.shared_radius_units.is_none())));
    let copy = shared.fragment_copy_uuid.unwrap();
    assert!(effects.active.iter().all(|e| e.fragment_copy_uuid == Some(copy)));
    let facts = active.battle.runtime_fact_summary();
    assert_eq!(facts.latest_cost.as_ref().unwrap().reason, crate::game::battle::runtime_facts::CostReason::AuthoredHealth);
    assert_eq!(facts.latest_cost.as_ref().unwrap().actual_spent(), 10);
    let mut shield_hit = false;
    let mut deflected = false;
    let mut last_log = Vec::new();
    // Bounded observation; protection/counter and natural result remain required.
    for _ in 0..2000 {
        if !matches!(core.get_state(), GameState::InBattle { .. }) { break; }
        core.advance_active_battle_for_server_tick(50).unwrap();
        if let Some(active) = &core.state.active_battle {
            let view = active.battle.sustained_effect_snapshot().unwrap();
            shield_hit |= view.active.iter().filter_map(|v| v.shield).any(|p| p.remaining < p.capacity);
            last_log = active.battle.event_log.entries.iter().filter(|e| matches!(&e.event, crate::game::battle::event_log::BattleLogEvent::HpChanged { .. } | crate::game::battle::event_log::BattleLogEvent::AbilityStepTriggered { .. })).map(|e| format!("{e:?}")).collect();
            deflected |= active.battle.event_log.entries.iter().any(|e| matches!(&e.event, crate::game::battle::event_log::BattleLogEvent::AbilityStepTriggered { skill_id, .. } if skill_id.as_str() == "r12_counter"));
            if let Some(fact) = &active.battle.runtime_fact_summary().latest_damage {
                shield_hit |= fact.protection.absorbed > 0;
                deflected |= fact.protection.deflection.is_some();
                if fact.protection.absorbed > 0 {
                    assert_eq!(fact.actual_hp_loss(), 0);
                    assert!(fact.protection.shields.iter().all(|p| p.source.fragment.as_ref().unwrap().copy_uuid == copy));
                }
            }
        }
    }
    assert!(shield_hit, "actual enemy attack must consume a public authored shield: {last_log:?}");
    // Counter can be the latest fact of the same timestamp. Its explicit skill
    // log proves successful parry even when it supersedes the blocked hit fact.
    assert!(deflected, "actual enemy attack must trigger the public parry counter: {last_log:?}");
    assert!(!matches!(core.get_state(), GameState::InBattle { .. }));
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    core.save_run_checkpoint_for_test().unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 1);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json, 0, saved.run_peak_certified_power).unwrap();
    assert!(restored.state.active_battle.is_none());
    assert_eq!(serde_json::to_value(&restored.state.roster.get(&employee).unwrap().skill_fragments).unwrap(), serde_json::to_value(&core.state.roster.get(&employee).unwrap().skill_fragments).unwrap());
}


#[test]
fn r12_world_corrosion_during_boss_flow_channel_pool_freezes_then_preserves_boss() {
    let authored = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r12/world_corrosion.ron"));
    let (mut core, player, caster) = r12_live_cross_fixture(false, false, 0, Some(authored), false, true, true);
    let other = core.roster().unwrap().combat_eligible_employee_ids().into_iter().find(|id| *id != caster).unwrap();
    let position = core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter().find(|z| z.kind == crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[1];
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::DeployUnit { employee_uuid: other, position, facing: FacingDirection::Right }).unwrap();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: caster, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(400).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    let view = active.battle.sustained_effect_snapshot().unwrap();
    assert!(view.active.iter().any(|v| v.shield.is_some_and(|p| p.shared_radius_units.is_some())));
    assert!(view.active.iter().any(|v| v.is_channel));
    assert!(view.active.iter().any(|v| v.survival.is_some()));
    assert!(active.battle.event_log.entries.iter().any(|e| matches!(&e.event, crate::game::battle::event_log::BattleLogEvent::AbilityStepTriggered { skill_id, .. } if skill_id.as_str() == "r12_enemy_guard")));
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ActivateSkill { employee_uuid: other, skill_id: "r08_strike".into(), target: None }).unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    let facts = active.battle.runtime_fact_summary();
    assert!(facts.latest_death.is_none());
    assert!(active.battle.units.values().any(|unit| unit.owner == crate::game::enums::Side::Opponent && unit.is_active()));
    assert!(active.battle.event_log.entries.iter().any(|entry| matches!(entry.event, crate::game::battle::event_log::BattleLogEvent::HpChanged { reason: crate::game::battle::event_log::HpChangeReason::Corrosion, .. })));
    assert_eq!(active.employee_attempt.roster.get(&other).unwrap().condition.trauma, 100);
    assert_eq!(active.battle.forced_pause_remaining_ms(), Some(1000));
    let frozen = battle_advance_atomic_snapshot(&core);
    core.execute(player, PlayerBehavior::RequestBattleState { since_seq: None }).unwrap();
    assert_eq!(battle_advance_atomic_snapshot(&core), frozen);
    core.advance_active_battle_for_server_tick(1000).unwrap();
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
    core.advance_active_battle_for_server_tick(1).unwrap();
    assert!(matches!(core.get_state(), GameState::InBattle { .. }), "corrosion cannot skip the surviving boss");
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.battle.units.values().any(|unit| unit.owner == crate::game::enums::Side::Opponent && unit.is_active()));
    assert_eq!(active.employee_attempt.roster.get(&other).unwrap().condition.trauma, 100);
    assert_eq!(active.battle.forced_pause_remaining_ms(), None);
    // Result/checkpoint transfer remains covered by the ordinary-death cases;
    // this boss fixture must keep its native encounter running after the pause.

}
