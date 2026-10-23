// F01 normal scenario spawns + common object creation + actual swept contact.
// No block map is constructed or restored by these fixtures.
struct U07BlockFixture {
    core: BattleCore,
    creator: UnitInstanceId,
    guard: UnitInstanceId,
    incumbent: UnitInstanceId,
    newcomer: UnitInstanceId,
    contact_wall: UnitInstanceId,
    runner: UnitInstanceId,
    changed: UnitInstanceId,
    request: IndependentSpawnIdentity,
}

fn u07_block_spawn(
    core: &mut BattleCore,
    name: &str,
    salt: u32,
    side: Side,
    position: Position,
    profile: UnitCombatProfile,
    employee: bool,
) -> UnitInstanceId {
    let mut group = core.scenario.groups[0].clone();
    group.id = ScenarioGroupId::new(name);
    group.side = side;
    group.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
        cells: vec![Position::new(0, position.y), Position::new(5, position.y)],
    });
    let spawn = &mut group.spawns[0];
    spawn.unit_ref = ScenarioUnitRef::new(name);
    spawn.side = side;
    spawn.position = position;
    spawn.instance_salt = salt;
    spawn.draft.owned_uuid = Uuid::from_u128(10_000 + u128::from(salt));
    spawn.draft.equipped_items = if employee {
        vec![crate::game::battle::types::BattleEquipmentBinding {
            weapon_stage: Default::default(),
            base_uuid: Uuid::from_u128(300), persistent_copy_uuid: None, slot: None,
        }]
    } else { vec![] };
    spawn.draft.source = if employee {
        BattleUnitSource::Employee(profile)
    } else {
        BattleUnitSource::TestFixture { base_uuid: spawn.draft.owned_uuid, profile }
    };
    core.spawn_scenario_group_member(&group, 0).unwrap()
}

fn u07_block_fixture(definition: &str, policy: SummonerRemovalPolicyData) -> U07BlockFixture {
    use crate::game::battle::tile_range::FacingDirection;
    let (mut core, creator) = fixture_with_motion_and_blocking(
        Side::Player, crate::game::data::skill_data::SkillDatabase::new(vec![]), policy,
        IndependentActorMotion::Stationary, true, false,
    );
    let mut guard_profile = UnitCombatProfile::for_employee(1000, 7, 0, 0, 100);
    guard_profile.block_capacity = 1;
    guard_profile.basic_attack.windup_ms = 0;
    let guard = u07_block_spawn(&mut core, "guard", 11, Side::Player, Position::new(2, 0), guard_profile, true);
    core.units.get_mut(&guard).unwrap().facing_direction = Some(FacingDirection::Right);
    let mut request = input(creator, definition, vec![Position::new(4, 3)]);
    request.allow_same_source_evolution = true;
    let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(
        SpawnRequestId::from("f01_changed"), Side::Player, request.clone(),
    ).unwrap() else { panic!("normal HP object creation"); };
    let changed = unit_ids[0];
    let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(
        SpawnRequestId::from("f01_contact_wall"), Side::Player,
        input(guard, "wall_large", vec![Position::new(3, 2)]),
    ).unwrap() else { panic!("normal distinct wall creation"); };
    let contact_wall = unit_ids[0];
    let mut moving = UnitCombatProfile::for_employee(1000, 1, 0, 0, 100);
    moving.basic_attack.windup_ms = 0;
    moving.movement.speed_units_per_ms = 10_000_000;
    moving.stats.move_speed_units_per_ms = 10_000_000;
    let incumbent = u07_block_spawn(&mut core, "incumbent", 12, Side::Opponent, Position::new(0, 0), moving.clone(), false);
    moving.blockable = false;
    let runner = u07_block_spawn(&mut core, "wall_runner", 13, Side::Opponent, Position::new(0, 2), moving, false);
    core.run_continuous_attack_movement_tick(20, 100);
    assert_eq!(core.blocked_by(incumbent), Some(guard), "actual incumbent contact");
    assert_eq!(core.blocked_by(runner), Some(contact_wall), "actual swept wall contact");
    let newcomer = u07_block_spawn(&mut core, "newcomer", 14, Side::Opponent, Position::new(2, 0), UnitCombatProfile::for_employee(1000, 0, 0, 0, 100), false);
    assert!(core.enemy_route_progress(newcomer) > core.enemy_route_progress(incumbent));
    core.refresh_block_state();
    assert_eq!(core.blocked_by(incumbent), Some(guard));
    assert_eq!(core.blocked_by(newcomer), None, "capacity already belongs to incumbent");
    assert_eq!(core.blocked_by(runner), Some(contact_wall));
    U07BlockFixture { core, creator, guard, incumbent, newcomer, contact_wall, runner, changed, request }
}

fn u07_block_transition(definition: &str, transition: &str) {
    let policy = if transition == "despawn" { SummonerRemovalPolicyData::Despawn } else { SummonerRemovalPolicyData::Persist };
    let mut f = u07_block_fixture(definition, policy);
    let incumbent_position = f.core.unit_world_position(f.incumbent).unwrap();
    let runner_position = f.core.unit_world_position(f.runner).unwrap();
    let transition_time = if transition == "expire" { 1010 } else { 30 };
    match transition {
        "evolve" => {
            f.request.time_ms = transition_time;
            assert_eq!(f.core.create_independent_objects(SpawnRequestId::from("f01_evolve"), Side::Player, f.request).unwrap(), SpawnRequestSettlement::Evolved { unit_ids: vec![f.changed] });
        }
        "expire" => {
            f.core.expire_independent_object(f.changed, 1, transition_time).unwrap();
            assert!(!f.core.units.contains_key(&f.changed));
        }
        "despawn" => {
            f.core.withdraw_unit(f.creator, transition_time).unwrap();
            f.core.reconcile_independent_object_removals(transition_time).unwrap();
            assert!(!f.core.units.contains_key(&f.changed));
        }
        _ => unreachable!(),
    }
    let expected = (Some(f.guard), None, Some(f.contact_wall));
    let pairs = |core: &BattleCore| (core.blocked_by(f.incumbent), core.blocked_by(f.newcomer), core.blocked_by(f.runner));
    assert_eq!(pairs(&f.core), expected, "transition boundary must already preserve authority");
    f.core.refresh_block_state();
    assert_eq!(pairs(&f.core), expected, "same-time refresh is stable");
    f.core.refresh_block_state();
    eprintln!("F01 {definition}/{transition}: incumbent={:?},newcomer={:?},wallcontact={:?}", f.core.blocked_by(f.incumbent), f.core.blocked_by(f.newcomer), f.core.blocked_by(f.runner));
    assert_eq!((f.core.blocked_by(f.incumbent), f.core.blocked_by(f.newcomer), f.core.blocked_by(f.runner)), (Some(f.guard), None, Some(f.contact_wall)), "unrelated incumbent and distinct swept contact survive {definition}/{transition}");
    assert_eq!(f.core.select_basic_attack_target(f.guard, None, None), Some(f.incumbent));
    assert_eq!(f.core.select_basic_attack_target(f.runner, None, None), Some(f.contact_wall));
    assert!(f.core.is_basic_attack_target_in_range(f.guard, f.incumbent));
    assert!(f.core.is_basic_attack_target_in_range(f.runner, f.contact_wall));
    let query = f.core.basic_action_snapshot(f.guard, true).unwrap();
    assert_eq!(query.blocking, vec![f.incumbent]);
    assert_eq!(query.selected_target_id, Some(f.incumbent));
    f.core.run_continuous_attack_movement_tick(transition_time + 10, 100);
    assert_eq!(f.core.unit_world_position(f.incumbent), Some(incumbent_position));
    assert_eq!(f.core.unit_world_position(f.runner), Some(runner_position));
    for (actor, target) in [(f.guard, f.incumbent), (f.runner, f.contact_wall)] {
        let hp = f.core.units[&target].stats.current_health;
        let time = transition_time + 10;
        f.core.handle_basic_attack_start_event(time, time, actor, Some(target), false,
            BattleEventCause::Root { kind: BattleEventRootCause::Period });
        let resolve_time = time + u64::from(f.core.units[&actor].basic_attack.effective_windup_ms());
        assert!(resolve_time < 1510, "distinct contact wall remains alive for this attack");
        while f.core.event_queue.peek_time_ms().is_some_and(|t| t <= resolve_time) {
            let event = f.core.event_queue.pop().unwrap();
            f.core.process_event(event, resolve_time).unwrap();
        }
        assert!(f.core.units[&target].stats.current_health < hp, "actual basic attack reaches retained target");
        assert!(f.core.event_log.entries.iter().any(|entry| matches!(entry.event,
            crate::game::battle::event_log::BattleLogEvent::HpChanged { source_instance_id: Some(source), target_instance_id, delta, .. }
                if source == actor && target_instance_id == target && delta < 0)));
    }
}

#[test] fn u07_f01_actor_evolution_preserves_unrelated_contacts() { u07_block_transition("actor", "evolve"); }
#[test] fn u07_f01_wall_evolution_preserves_unrelated_contacts() { u07_block_transition("wall", "evolve"); }
#[test] fn u07_f01_actor_expiry_preserves_unrelated_contacts() { u07_block_transition("actor", "expire"); }
#[test] fn u07_f01_wall_expiry_preserves_unrelated_contacts() { u07_block_transition("wall", "expire"); }
#[test] fn u07_f01_actor_source_despawn_preserves_unrelated_contacts() { u07_block_transition("actor", "despawn"); }
#[test] fn u07_f01_wall_source_despawn_preserves_unrelated_contacts() { u07_block_transition("wall", "despawn"); }

#[test]
fn u07_f01_refresh_admission_exact_and_one_short_leave_candidate_unchanged() {
    use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode};
    let f = u07_block_fixture("actor", SummonerRemovalPolicyData::Persist);
    let (work, bytes) = f.core.independent_form_block_refresh_bounds().unwrap();
    let discovery = f.core.units_clone_history.bucket_bound().unwrap() as u64;
    let total = discovery + work + (bytes as u64).div_ceil(8);
    let mut exact = f.core.clone();
    let before = (exact.transaction_debug_snapshot(), exact.event_queue_debug_snapshot());
    let (result, admissions) = native::observe_retained_admissions(|| native::run(
        RuntimeFactLimits { max_work: total, max_retained_bytes: bytes, ..Default::default() },
        || exact.refresh_independent_form_block_state()));
    result.unwrap();
    assert_eq!(admissions, [("object_block_refresh_workspace", 0, bytes)]);
    assert_eq!((exact.transaction_debug_snapshot(), exact.event_queue_debug_snapshot()), before);
    for (work_cap, byte_cap, code, site) in [
        (discovery - 1, bytes, RuntimeFaultCode::WorkLimit, "object_block_refresh_discovery"),
        (discovery + work - 1, bytes, RuntimeFaultCode::WorkLimit, "object_block_refresh_work"),
        (total - 1, bytes, RuntimeFaultCode::WorkLimit, "object_block_refresh_workspace"),
        (total, bytes - 1, RuntimeFaultCode::RetainedByteLimit, "object_block_refresh_workspace"),
    ] {
        let mut short = f.core.clone();
        let before = (short.transaction_debug_snapshot(), short.event_queue_debug_snapshot());
        let (result, admissions) = native::observe_retained_admissions(|| native::run(
            RuntimeFactLimits { max_work: work_cap, max_retained_bytes: byte_cap, ..Default::default() },
            || short.refresh_independent_form_block_state()));
        assert!(matches!(result, Err(GameError::BattleRuntimeFault(fault)) if fault.code == code && fault.site == site), "{result:?}");
        assert!(admissions.is_empty(), "refuse before collecting refresh workspace");
        assert_eq!((short.transaction_debug_snapshot(), short.event_queue_debug_snapshot()), before);
        assert_eq!(short.select_basic_attack_target(f.guard, None, None), Some(f.incumbent));
        assert_eq!(short.select_basic_attack_target(f.runner, None, None), Some(f.contact_wall));
    }
    eprintln!("F01 refresh admission discovery={discovery} work={work} cumulative_bytes={bytes} total_work={total}");
}

#[test]
fn u07_f01_expired_own_wall_releases_only_its_contact_and_route_resumes() {
    let mut f = u07_block_fixture("wall", SummonerRemovalPolicyData::Persist);
    let mut moving = UnitCombatProfile::for_employee(1000, 1, 0, 0, 100);
    moving.movement.speed_units_per_ms = 10_000_000;
    moving.stats.move_speed_units_per_ms = 10_000_000;
    moving.blockable = false;
    let own_runner = u07_block_spawn(&mut f.core, "own_wall_runner", 15, Side::Opponent, Position::new(0, 3), moving, false);
    f.core.run_continuous_attack_movement_tick(21, 100);
    assert_eq!(f.core.blocked_by(own_runner), Some(f.changed));
    assert_eq!(f.core.units[&own_runner].current_target, Some(f.changed));
    let position = f.core.unit_world_position(own_runner).unwrap();
    f.core.expire_independent_object(f.changed, 1, 1010).unwrap();
    assert_eq!(f.core.blocked_by(own_runner), None);
    assert_eq!(f.core.units[&own_runner].current_target, None);
    assert_eq!(f.core.blocked_by(f.incumbent), Some(f.guard));
    assert_eq!(f.core.blocked_by(f.runner), Some(f.contact_wall));
    f.core.run_continuous_attack_movement_tick(1011, 100);
    assert!(f.core.unit_world_position(own_runner).unwrap().x > position.x);
}

#[test]
fn u07_f01_actor_own_ground_contact_survives_and_airborne_form_releases() {
    for airborne in [false, true] {
        let (mut core, creator) = fixture_with_motion_and_blocking(Side::Opponent,
            crate::game::data::skill_data::SkillDatabase::new(vec![]), SummonerRemovalPolicyData::Persist,
            IndependentActorMotion::Path { cells: vec![Position::new(0, 0), Position::new(5, 0)] }, true, airborne);
        let mut guard_profile = UnitCombatProfile::for_employee(1000, 1, 0, 0, 100);
        guard_profile.block_capacity = 1;
        let guard = u07_block_spawn(&mut core, "own_guard", 21, Side::Player, Position::new(2, 0), guard_profile, true);
        let mut request = input(creator, "actor", vec![Position::new(0, 0)]);
        request.allow_same_source_evolution = true;
        let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId::from("own_actor"), Side::Opponent, request.clone()).unwrap() else { panic!() };
        let actor = unit_ids[0];
        core.run_continuous_attack_movement_tick(20, 1_000_000);
        assert_eq!(core.blocked_by(actor), Some(guard), "own actor normal swept contact");
        let position = core.unit_world_position(actor).unwrap();
        request.positions = vec![core.unit_projected_tile(actor).unwrap()];
        request.time_ms = 30;
        if airborne {
            use crate::game::battle::runtime_control as native;
            let original = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
            let evolve = |candidate: &mut BattleCore| candidate.create_independent_objects(SpawnRequestId::from("own_evolve"), Side::Opponent, request.clone());
            let mut probe = core.clone();
            let (result, admissions) = native::observe_retained_admissions(|| probe.runtime_transaction(evolve));
            result.unwrap();
            assert_eq!(probe.blocked_by(actor), None, "successful branch really changes own relationship");
            assert_eq!(probe.units[&guard].current_target, None);
            assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), original);
            let (_, _, requested) = admissions.iter().rfind(|(site, _, _)| *site == "queued_event_payload").copied().unwrap();
            core.runtime_facts.limits.max_retained_bytes = requested - 1;
            let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
            let error = core.runtime_transaction(evolve).unwrap_err();
            assert!(matches!(error, GameError::BattleRuntimeFault(fault) if fault.code == native::RuntimeFaultCode::RetainedByteLimit && fault.site == "queued_event_payload"), "{error:?}");
            assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
            assert_eq!(core.blocked_by(actor), Some(guard));
            assert_eq!(core.units[&actor].current_target, Some(guard));
            assert_eq!(core.units[&guard].current_target, Some(actor));
            core.runtime_facts.limits = Default::default();
        }
        assert_eq!(core.create_independent_objects(SpawnRequestId::from("own_evolve"), Side::Opponent, request).unwrap(), SpawnRequestSettlement::Evolved { unit_ids: vec![actor] });
        assert_eq!(core.blocked_by(actor), (!airborne).then_some(guard), "final form validity checked before observing evolution");
        if airborne {
            assert_eq!(core.units[&actor].current_target, None);
            assert_eq!(core.units[&guard].current_target, None);
        } else {
            assert_eq!(core.select_basic_attack_target(actor, None, None), Some(guard));
            assert!(core.is_basic_attack_target_in_range(actor, guard));
        }
        core.run_continuous_attack_movement_tick(40, 1_000_000);
        if !airborne {
            assert_eq!(core.unit_world_position(actor), Some(position));
        } else {
            // An airborne actor can still choose an in-range attack. Remove
            // that actual opponent through the normal owner to observe route
            // continuation independently of ordinary attack stopping.
            core.withdraw_unit(guard, 41).unwrap();
            assert_eq!(core.blocked_by(actor), None);
            core.run_continuous_attack_movement_tick(42, 1_000_000);
            assert!(core.unit_world_position(actor).unwrap().x > position.x);
        }
    }
}

#[test]
fn u07_f01_evolution_clone_late_queue_refusal_rolls_back_contacts_and_retry() {
    use crate::game::battle::runtime_control as native;
    for definition in ["actor", "wall"] {
        let f = u07_block_fixture(definition, SummonerRemovalPolicyData::Persist);
        let mut request = f.request.clone();
        request.time_ms = 30;
        let original = (f.core.transaction_debug_snapshot(), f.core.event_queue_debug_snapshot());
        let evolve = |core: &mut BattleCore| core.create_independent_objects(SpawnRequestId::from("late_evolve"), Side::Player, request.clone());
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| probe.runtime_transaction(evolve));
        assert_eq!(result.unwrap(), SpawnRequestSettlement::Evolved { unit_ids: vec![f.changed] });
        assert_eq!((f.core.transaction_debug_snapshot(), f.core.event_queue_debug_snapshot()), original, "cloned branch never mutates original authority");
        let (site, _, requested) = admissions.iter().rfind(|(site, _, _)| *site == "queued_event_payload").copied().unwrap();
        let mut short = f.core.clone();
        short.runtime_facts.limits.max_retained_bytes = requested - 1;
        let before = (short.transaction_debug_snapshot(), short.event_queue_debug_snapshot());
        let error = short.runtime_transaction(evolve).unwrap_err();
        assert!(matches!(error, GameError::BattleRuntimeFault(fault) if fault.code == native::RuntimeFaultCode::RetainedByteLimit && fault.site == site), "{error:?}");
        assert_eq!((short.transaction_debug_snapshot(), short.event_queue_debug_snapshot()), before);
        assert_eq!(short.blocked_by(f.incumbent), Some(f.guard));
        assert_eq!(short.blocked_by(f.runner), Some(f.contact_wall));
        short.runtime_facts.limits = Default::default();
        assert_eq!(short.runtime_transaction(evolve).unwrap(), SpawnRequestSettlement::Evolved { unit_ids: vec![f.changed] });
        assert_eq!(short.blocked_by(f.incumbent), Some(f.guard));
        assert_eq!(short.blocked_by(f.runner), Some(f.contact_wall));
    }
}

#[test]
fn u07_f01_refresh_history_overflow_refuses_before_discovery_and_mutation() {
    use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode};
    let mut f = u07_block_fixture("wall", SummonerRemovalPolicyData::Persist);
    f.core.units_clone_history = crate::owned_collections::MapCloneHistory::from_tombstone_free_capacity(usize::MAX);
    let before = (f.core.transaction_debug_snapshot(), f.core.event_queue_debug_snapshot());
    let (result, admissions) = native::observe_retained_admissions(|| native::run(RuntimeFactLimits::default(), || f.core.refresh_independent_form_block_state()));
    assert!(matches!(result, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::CounterOverflow && fault.site == "object_block_refresh_discovery"), "{result:?}");
    assert!(admissions.is_empty());
    assert_eq!((f.core.transaction_debug_snapshot(), f.core.event_queue_debug_snapshot()), before);
}

#[test]
fn u07_f01_refresh_returns_authoring_segmented_capacity_sticky_fault() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    let mut f = u07_block_fixture("actor", SummonerRemovalPolicyData::Persist);
    segmented_fixture_pool(&mut f.core, f.guard, 150);
    let (work, bytes) = f.core.independent_form_block_refresh_bounds().unwrap();
    let discovery = f.core.units_clone_history.bucket_bound().unwrap() as u64;
    let cap = discovery + work + (bytes as u64).div_ceil(8);
    let returned_error = std::cell::Cell::new(false);
    let result = control::run(&ExecutionLimits { max_work: cap, ..Default::default() },
        &CancellationToken::default(), ExecutionPhase::TrialExecution, || {
            returned_error.set(f.core.refresh_independent_form_block_state().is_err());
        });
    assert!(matches!(result, Err(ref failure) if failure.code == "authoring_work_limit" && failure.site == "segmented_block_capacity"), "{result:?}");
    assert!(returned_error.get(), "wrapper itself returns the authoring-only sticky fault");
}
