#[test]
fn u07_object_trigger_without_damage_target_waits_and_shield_zero_hit_consumes() {
    use crate::game::ability::sustained::SustainedEffectDef;
    let (mut core, creator) = fixture(Side::Player);
    let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId::from("wait"), Side::Player, input(creator, "friend", vec![Position::new(2, 1)])).unwrap() else { panic!() };
    let object = unit_ids[0];
    let target = object_test_enemy(&mut core, 1, Position::new(3, 1), false);
    let definition = core.game_data.boss_runtime_data.independent_objects().get(&IndependentObjectId::new("friend")).unwrap().clone();
    let IndependentObjectKind::Trap(mut trap) = definition.definition.kind.clone() else { panic!() };
    // Deliberately distinct trigger and damage patterns at the compiled owner seam.
    trap.damage_range = TileRangePattern { include_anchor_tile: true, rows: vec!["@".into()] };
    let before = core.transaction_debug_snapshot();
    core.trigger_independent_trap(object, &trap, 11).unwrap();
    assert_eq!(core.transaction_debug_snapshot(), before);
    core.units.get_mut(&target).unwrap().set_world_position(crate::game::battle::core::movement::types::WorldVec2::from_tile_center(Position::new(2, 1)));
    core.begin_sustained(target, target, SustainedEffectDef::Shield { duration_ms: 1000, amount: 100, priority: 0, shared_radius_units: None }, 11).unwrap();
    core.trigger_independent_trap(object, &trap, 12).unwrap();
    assert!(!core.independent_objects.objects.contains_key(&object));
    assert_eq!(core.units[&target].stats.current_health, 1000);
    let facts = core.runtime_fact_snapshot().unwrap(); let damage = facts.summary.latest_damage.as_ref().unwrap();
    assert_eq!(damage.actual_hp_loss(), 0);
    assert_eq!(damage.protection.absorbed, 40);
    assert_eq!(core.independent_objects.last_removal.as_ref().unwrap().reason, super::super::independent_objects::ObjectRemovalReason::AttackConsumed);
}

#[test]
fn u07_object_same_time_traps_are_stable_and_ttl_wins_before_contact() {
    for expires_now in [false, true] {
        let (mut core, creator) = fixture(Side::Player);
        let SpawnRequestSettlement::Spawned { mut unit_ids } = core.create_independent_objects(SpawnRequestId::from("pair"), Side::Player, input(creator, "friend", vec![Position::new(2, 1), Position::new(2, 2)])).unwrap() else { panic!() };
        unit_ids.sort_unstable();
        let target = object_test_enemy(&mut core, 1, Position::new(3, 1), false);
        core.reconcile_independent_objects(if expires_now { 1010 } else { 11 }).unwrap();
        assert!(core.independent_objects.objects.is_empty());
        assert_eq!(core.units[&target].stats.current_health, if expires_now { 1000 } else { 920 });
        let facts = core.runtime_fact_snapshot().unwrap();
        if expires_now { assert!(facts.summary.latest_damage.is_none()); }
        else { assert_eq!(facts.summary.latest_damage.as_ref().unwrap().source.actor, Some(unit_ids[1])); }
        let before = core.transaction_debug_snapshot(); core.reconcile_independent_objects(1011).unwrap(); assert_eq!(core.transaction_debug_snapshot(), before);
    }
}

#[test]
fn u07_object_cleanup_only_cascades_actor_sources_without_attacking() {
    let (mut core, creator) = fixture_with_policy(Side::Player, crate::game::data::skill_data::SkillDatabase::new(vec![]), SummonerRemovalPolicyData::Despawn);
    let mut source = creator;
    for (index, name) in ["actor", "actor", "friend"].into_iter().enumerate() {
        let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId(format!("child{index}")), Side::Player, input(source, name, vec![Position::new(index as i32 + 1, 1)])).unwrap() else { panic!() };
        source = unit_ids[0];
    }
    let target = object_test_enemy(&mut core, 1, Position::new(3, 2), false);
    core.withdraw_unit(creator, 11).unwrap();
    core.reconcile_independent_object_removals(11).unwrap();
    assert!(core.independent_objects.objects.is_empty());
    assert_eq!(core.units[&target].stats.current_health, 1000);
    assert!(core.independent_objects.counters.values().all(|c| c.alive == 0));
    assert!(core.runtime_fact_snapshot().unwrap().summary.latest_damage.is_none());
}

#[test]
fn u07_object_max_hp_zero_and_direct_kill_remove_once_without_death_fact() {
    use crate::game::{stats::{StatModifier, StatModifierKind, StatId}, battle::{damage::BattleCommand, event_log::BattleLogEvent}};
    for definition in ["wall", "actor"] { for direct in [false, true] {
        let (mut core, creator) = fixture(Side::Player);
        let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId::from("destroy"), Side::Player, input(creator, definition, vec![Position::new(2, 1)])).unwrap() else { panic!() };
        let object = unit_ids[0];
        if direct { core.finalize_unit_death(Some(creator), object, Side::Player, 11); }
        else { core.process_commands_with_modifier_source(vec![BattleCommand::ApplyModifier { target_id: object, modifier: StatModifier { stat: StatId::MaxHealth, kind: StatModifierKind::Flat, value: -1000 } }], 11, Some(creator)); }
        assert!(!core.units.contains_key(&object));
        assert!(!core.independent_objects.objects.contains_key(&object));
        assert!(core.runtime_fact_snapshot().unwrap().summary.latest_death.is_none());
        assert!(!core.event_log.entries.iter().any(|e| matches!(e.event, BattleLogEvent::UnitDied { .. } | BattleLogEvent::TriggeredAbilityProc { .. })));
        let before = core.transaction_debug_snapshot(); core.finalize_unit_death(Some(creator), object, Side::Player, 12); assert_eq!(core.transaction_debug_snapshot(), before);
        if !direct {
            let events: Vec<_> = core.event_log.entries.iter().filter(|e| e.time_ms == 11).map(|e| &e.event).collect();
            let stat = events.iter().position(|e| matches!(e, BattleLogEvent::StatChanged { .. })).unwrap();
            let hp = events.iter().position(|e| matches!(e, BattleLogEvent::HpChanged { .. })).unwrap();
            let removed = events.iter().position(|e| matches!(e, BattleLogEvent::IndependentObjectRemoved { .. })).unwrap();
            assert!(stat < hp && hp < removed);
        }
    } }
}

#[test]
fn u07_object_path_actor_moves_on_both_sides_without_employee_stance() {
    for side in [Side::Player, Side::Opponent] {
        let (mut core, creator) = fixture_with_motion(side,
            crate::game::data::skill_data::SkillDatabase::new(vec![]),
            SummonerRemovalPolicyData::Persist,
            IndependentActorMotion::Path { cells: vec![Position::new(1, 1), Position::new(4, 1)] });
        let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(
            SpawnRequestId::from("walker"), side, input(creator, "actor", vec![Position::new(1, 1)])).unwrap() else { panic!() };
        let actor = unit_ids[0];
        core.withdraw_unit(creator, 11).unwrap();
        let before = core.units[&actor].body.position;
        assert!(core.build_continuous_attack_goals().contains_key(&actor));
        core.run_continuous_attack_movement_tick(20, 500);
        let after = core.units[&actor].body.position;
        assert!(after.x > before.x, "{side:?}: {before:?} -> {after:?}");
        assert!((after.y - before.y).abs() < 0.01);
        assert_eq!(core.units[&actor].owner, side);
        let Some(EnemyMovementPlan::PathAlongCells { cells }) = &core.units[&actor].enemy_movement_plan else { panic!() };
        assert_eq!(cells, &vec![Position::new(1, 1), Position::new(4, 1)]);
    }
}

#[test]
fn u07_object_zero_trap_target_limit_is_rejected_by_strict_catalog() {
    let (core, _) = fixture(Side::Player);
    let mut definition = core.game_data.boss_runtime_data.independent_objects().get(&IndependentObjectId::new("friend")).unwrap().definition.clone();
    definition.evolves_to = None;
    let IndependentObjectKind::Trap(trap) = &mut definition.kind else { panic!() };
    trap.max_targets = Some(0);
    assert!(crate::game::data::independent_object_data::IndependentObjectDatabase::compile(&[definition]).is_err());
}

#[test]
fn u07_object_terminal_finalize_reclaims_live_owners_and_next_battle_resets_ledger() {
    let (mut core, creator) = fixture(Side::Player);
    Arc::make_mut(&mut core.scenario).terminal_outcomes = BattleScenario::fixture_survival_outcomes(100);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    segmented_fixture_pool(&mut core, creator, 150);
    for (index, definition) in ["actor", "wall", "friend"].into_iter().enumerate() {
        core.create_independent_objects(SpawnRequestId(format!("terminal:{definition}")), Side::Player,
            input(creator, definition, vec![Position::new(index as i32 + 1, 2)])).unwrap();
    }
    assert_eq!(core.independent_objects.objects.len(), 3);
    core.step_battle_execution_until(&mut execution, 100).unwrap();
    assert!(execution.is_finished());
    let result = core.finalize_battle_execution(&mut execution).unwrap();
    assert!(core.independent_objects.objects.is_empty());
    assert!(core.independent_objects.counters.values().all(|c| c.alive == 0 && c.total == 1));
    assert_eq!(core.spawn_kernel.request_count(), 3, "bounded replay receipts live until battle reset");
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert!(result.runtime_facts.active_actions.is_empty());
    assert!(!core.resonance_use_has_obligations(creator));
    assert_eq!(core.independent_objects.last_removal.as_ref().unwrap().reason, super::super::independent_objects::ObjectRemovalReason::BattleEnded);
    let before = core.transaction_debug_snapshot();
    assert!(core.finalize_battle_execution(&mut execution).is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
    let _next = core.start_battle_execution().unwrap();
    assert!(core.independent_objects.counters.is_empty());
    assert!(core.independent_objects.last_removal.is_none());
    assert_eq!(core.independent_objects.next_id, 0);
    assert_eq!(core.spawn_kernel.request_count(), 0);
}

#[test]
fn native_u07_object_offsets_and_reach_refuse_before_materialization_or_mutation() {
    use crate::game::{ability::objects::{CreateObjectDef, ObjectPlacementAnchor}, battle::{runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode}, event_log::SkillCastTarget}, data::skill_data::SkillDatabase};
    let (core, creator) = fixture(Side::Player);
    let definition = CreateObjectDef { definition_id: IndependentObjectId::new("friend"), anchor: ObjectPlacementAnchor::CastTile,
        offsets: vec![Position::new(0, 0), Position::new(1, 0)], allow_same_source_evolution: false, wall_cost: None };
    let target = Some(SkillCastTarget::Tile { position: Position::new(1, 1) });
    let before = core.transaction_debug_snapshot();
    let (short, admissions) = native::observe_retained_admissions(|| native::run(RuntimeFactLimits { max_work: 1, ..Default::default() }, || core.object_creation_positions(creator, &definition, target)));
    assert!(matches!(short, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::WorkLimit && fault.site == "object_placement_offsets" && fault.requested == Some(2)));
    assert!(admissions.is_empty(), "work must reject before the placement vector is admitted");
    // Two offset visits plus two work units for the 16-byte vector admission.
    let positions = native::run(RuntimeFactLimits { max_work: 4, ..Default::default() }, || core.object_creation_positions(creator, &definition, target)).unwrap();
    assert_eq!(positions, vec![Position::new(1, 1), Position::new(2, 1)]);
    let skills = SkillDatabase::from_ron("u07/reach", TILE_CREATION).unwrap();
    let skill = skills.get_by_id("create_friend").unwrap();
    let short = native::run(RuntimeFactLimits { max_work: 1, ..Default::default() }, || core.validate_object_creation_reach(creator, skill, &positions));
    assert!(matches!(short, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::WorkLimit && fault.site == "object_reach_work" && fault.requested == Some(2)));
    native::run(RuntimeFactLimits { max_work: 2, ..Default::default() }, || core.validate_object_creation_reach(creator, skill, &positions)).unwrap();
    assert_eq!(core.transaction_debug_snapshot(), before);
}

#[test]
fn native_u07_object_and_pool_scans_debit_once_at_exact_work_boundary() {
    use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode};
    let (mut core, creator) = fixture(Side::Player);
    core.create_independent_objects(SpawnRequestId::from("metered"), Side::Player,
        input(creator, "friend", vec![Position::new(2, 2), Position::new(3, 2)])).unwrap();
    assert_eq!(core.independent_objects.objects.len(), 2);
    segmented_fixture_pool(&mut core, creator, 150);
    let before = core.transaction_debug_snapshot();
    let short = native::run(RuntimeFactLimits { max_work: 1, ..Default::default() },
        || core.reconcile_independent_object_removals(10));
    assert!(matches!(short, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::WorkLimit && fault.site == "independent_lifecycle_scan" && fault.requested == Some(2)));
    native::run(RuntimeFactLimits { max_work: 2, ..Default::default() },
        || core.reconcile_independent_object_removals(10)).unwrap();
    let short = native::run(RuntimeFactLimits { max_work: 4, ..Default::default() },
        || core.plan_object_wall_cost(creator, &wall_cost(), None, None, 10));
    assert!(matches!(short, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::WorkLimit && fault.site == "segmented_pool_lookup" && fault.requested == Some(5)));
    let plan = native::run(RuntimeFactLimits { max_work: 5, ..Default::default() },
        || core.plan_object_wall_cost(creator, &wall_cost(), None, None, 10)).unwrap();
    assert_eq!(plan.wall_health(), 50);
    // Two lifecycle and one trigger visits consume three units. Price three pattern
    // rows before scanning their bytes or admitting a candidate-target vector.
    let (short, admissions) = native::observe_retained_admissions(|| native::run(
        RuntimeFactLimits { max_work: 5, ..Default::default() },
        || core.reconcile_independent_objects(10)));
    assert!(matches!(short, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::WorkLimit && fault.site == "trap_pattern_rows" && fault.requested == Some(6)));
    assert!(admissions.is_empty());
    assert_eq!(core.transaction_debug_snapshot(), before);
}

#[test]
fn native_u07_retained_object_request_and_adapter_scans_debit_once_before_walk() {
    use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode};
    let (mut core, creator) = fixture(Side::Player);
    core.create_independent_objects(SpawnRequestId::from("extent"), Side::Player,
        input(creator, "friend", vec![Position::new(2, 2), Position::new(3, 2)])).unwrap();
    // No resource-table traversal participates in these three object scans.
    core.runtime_facts.resources = None;
    let objects = core.independent_objects.entity_count() as u64;
    let requests = core.spawn_kernel.request_count() as u64;
    let references = core.scenario_runtime.spawned.unit_refs.len() as u64;
    let carriers = (core.event_queue.len() + core.buffs.len() + core.sustained.len()
        + core.projectiles.len() + core.active_projectiles.len() + core.active_areas.len()
        + core.active_skill_casts.len()) as u64;
    assert!(objects > 0 && requests > 0 && references > 0 && carriers > 0);
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
    for (limit, site, requested) in [
        (objects - 1, "independent_object_extent_scan", objects),
        (objects + requests - 1, "independent_request_extent_scan", objects + requests),
        (objects + requests + references - 1, "object_adapter_reference_extent", objects + requests + references),
        (objects + requests + references + carriers - 1, "source_carrier_scan", objects + requests + references + carriers),
    ] {
        let failure = native::run(RuntimeFactLimits { max_work: limit, ..Default::default() },
            || core.retained_runtime_fact_extent()).unwrap_err();
        assert!(matches!(failure, GameError::BattleRuntimeFault(fault) if fault.code == RuntimeFaultCode::WorkLimit && fault.site == site && fault.requested == Some(requested)), "{failure:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
    }
    let exact = objects + requests + references + carriers;
    native::run(RuntimeFactLimits { max_work: exact, ..Default::default() },
        || core.retained_runtime_fact_extent()).unwrap();
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
}

#[test]
fn native_u07_form_and_expiry_queue_scans_debit_once_before_pruning() {
    use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode};
    let (mut original, creator) = fixture(Side::Player);
    let SpawnRequestSettlement::Spawned { unit_ids } = original.create_independent_objects(
        SpawnRequestId::from("queue-scan"), Side::Player,
        input(creator, "actor", vec![Position::new(1, 1)])).unwrap() else { panic!() };
    let actor = unit_ids[0];
    for form in [true, false] {
        let mut core = original.clone();
        let queued = core.event_queue.len() as u64;
        let is_target = |event: &BattleEvent| if form {
            matches!(event, BattleEvent::AttackStart { attacker_instance_id: id, .. }
                | BattleEvent::AutoCastStart { caster_instance_id: id, .. }
                | BattleEvent::SkillStartGate { caster_instance_id: id, .. } if *id == actor)
        } else {
            matches!(event, BattleEvent::IndependentObjectExpiry { object_id: id, .. } if *id == actor)
        };
        let targets = core.event_queue.iter_queued()
            .filter(|q| is_target(q.event.view().as_ref())).count();
        assert!(queued > 1 && targets > 0 && (targets as u64) < queued);
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
        let short = native::run(RuntimeFactLimits { max_work: queued - 1, ..Default::default() }, || {
            if form { core.event_queue.discard_independent_form_starts(actor) }
            else { core.event_queue.discard_independent_object_expiry(actor) }
        }).unwrap_err();
        let site = if form { "object_form_queue_scan" } else { "object_expiry_queue_scan" };
        assert!(matches!(short, GameError::BattleRuntimeFault(fault)
            if fault.code == RuntimeFaultCode::WorkLimit && fault.site == site && fault.requested == Some(queued)), "{short:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
        native::run(RuntimeFactLimits { max_work: queued, ..Default::default() }, || {
            if form { core.event_queue.discard_independent_form_starts(actor) }
            else { core.event_queue.discard_independent_object_expiry(actor) }
        }).unwrap();
        assert_eq!(core.event_queue.len(), queued as usize - targets);
        assert!(core.event_queue.iter_queued().all(|q| !is_target(q.event.view().as_ref())));
        let mut expected = original.event_queue.clone().into_vec();
        expected.retain(|q| !is_target(q.event.view().as_ref()));
        expected.sort_by(|left, right| right.cmp(left));
        let expected: Vec<_> = expected.into_iter().map(|q| format!("{q:?}")).collect();
        assert_eq!(core.event_queue_debug_snapshot(), expected);
    }
}

#[test]
fn native_u07_form_overlay_scan_debits_once_before_validation() {
    use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode};
    let (mut core, creator) = fixture(Side::Player);
    let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(
        SpawnRequestId::from("overlay-scan"), Side::Player,
        input(creator, "actor", vec![Position::new(1, 1)])).unwrap() else { panic!() };
    segmented_fixture_pool(&mut core, creator, 150);
    segmented_fixture_pool(&mut core, unit_ids[0], 150);
    let records = core.sustained.len() as u64;
    assert_eq!(records, 2);
    let basic = core.units[&creator].basic_attack.clone();
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
    let short = native::run(RuntimeFactLimits { max_work: records - 1, ..Default::default() },
        || core.validate_independent_form_overlays(creator, &basic)).unwrap_err();
    assert!(matches!(short, GameError::BattleRuntimeFault(fault)
        if fault.code == RuntimeFaultCode::WorkLimit && fault.site == "object_form_overlay_scan" && fault.requested == Some(records)), "{short:?}");
    native::run(RuntimeFactLimits { max_work: records, ..Default::default() },
        || core.validate_independent_form_overlays(creator, &basic)).unwrap();
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
}
