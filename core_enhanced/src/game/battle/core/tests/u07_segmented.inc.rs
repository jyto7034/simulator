fn segmented_fixture_pool(core: &mut BattleCore, owner: UnitInstanceId, health: u32) {
    core.begin_sustained(owner, owner, crate::game::ability::sustained::SustainedEffectDef::SegmentedShield {
        duration_ms: 5_000,
        pool: crate::game::ability::sustained::SegmentedShieldDef {
            pool_id: "meat".into(), segment_health: 100, max_segments: 3,
            initial_health: health, block_per_segment: 1, priority: 0,
        },
    }, 0).unwrap();
}
fn wall_cost() -> crate::game::ability::objects::ObjectWallCostDef {
    crate::game::ability::objects::ObjectWallCostDef { pool_id: "meat".into(), pool_source: Default::default(), health_cost_percent: 20, wall_health_percent: 200 }
}

#[test]
fn u07_segmented_top_detachment_uses_one_balance_and_actual_wall_health() {
    for (initial, top, remainder) in [(150, 50, 100), (100, 100, 0), (1, 1, 0)] {
        let (mut core, creator) = fixture(Side::Player);
        segmented_fixture_pool(&mut core, creator, initial);
        let before = core.sustained_effect_snapshot().unwrap();
        let pool = before.active[0].segmented.as_ref().unwrap();
        assert_eq!(pool.remaining_health, initial);
        assert_eq!(pool.stacks, initial.div_ceil(100));
        assert_eq!(pool.top_health, top);
        assert_eq!(core.effective_block_capacity(creator), pool.stacks);
        let mut request = input(creator, "wall", vec![Position::new(2, 1)]);
        request.wall_cost = Some(wall_cost());
        let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId::from("wall"), Side::Player, request).unwrap() else { panic!() };
        let wall = unit_ids[0];
        assert_eq!((core.units[&wall].stats.current_health, core.units[&wall].stats.max_health), (top, top));
        assert_eq!(core.units[&creator].stats.current_health, 100);
        let after = core.sustained_effect_snapshot().unwrap();
        assert_eq!(after.latest_detachment.as_ref().unwrap().detached, top);
        assert_eq!(after.latest_detachment.as_ref().unwrap().after, remainder);
        if remainder == 0 { assert!(after.active.is_empty()); }
        else { assert_eq!(after.active[0].segmented.as_ref().unwrap().remaining_health, remainder); }
        assert_eq!(core.effective_block_capacity(creator), remainder.div_ceil(100));
        let encoded = serde_json::to_string(&after).unwrap();
        assert_eq!(serde_json::from_str::<crate::game::ability::sustained::SustainedEffectsSnapshot>(&encoded).unwrap(), after);
    }
}

#[test]
fn u07_segmented_damage_absorbs_once_then_spills_only_remainder_to_hp() {
    use crate::game::battle::{damage::{DamageSource, DamageType, BattleCommand}};
    let (mut core, creator) = fixture(Side::Player);
    segmented_fixture_pool(&mut core, creator, 150);
    let hit = |core: &mut BattleCore, amount, now| {
        let source = core.damage_source_snapshot_for_unit(creator, creator, DamageSource::Ability, DamageType::True, amount, Default::default(), 0, now, false).unwrap();
        core.process_commands_with_modifier_source(vec![BattleCommand::ApplyDamage { target_id: creator, source_snapshot: source }], now, Some(creator));
    };
    hit(&mut core, 120, 10);
    assert_eq!(core.units[&creator].stats.current_health, 100);
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.active[0].segmented.as_ref().unwrap().remaining_health, 30);
    assert_eq!(view.active[0].segmented.as_ref().unwrap().stacks, 1);
    assert_eq!(core.effective_block_capacity(creator), 1);
    let facts = core.runtime_fact_snapshot().unwrap();
    assert_eq!(facts.summary.latest_damage.as_ref().unwrap().actual_hp_loss(), 0);
    hit(&mut core, 70, 20);
    assert_eq!(core.units[&creator].stats.current_health, 60);
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert_eq!(core.effective_block_capacity(creator), 0);
    let facts = core.runtime_fact_snapshot().unwrap();
    let damage = facts.summary.latest_damage.as_ref().unwrap();
    assert_eq!(damage.actual_hp_loss(), 40);
    assert_eq!(damage.protection.absorbed, 30);
}

#[test]
fn u07_segmented_foreign_copy_uses_health_fallback_without_consuming_other_pool() {
    use crate::game::battle::runtime_facts::EquipmentSourceSnapshot;
    let (mut core, creator) = fixture(Side::Player);
    segmented_fixture_pool(&mut core, creator, 150);
    let mut request = input(creator, "wall", vec![Position::new(2, 1)]);
    request.wall_cost = Some(wall_cost());
    request.source.equipment = Some(EquipmentSourceSnapshot { definition_uuid: Uuid::from_u128(990), runtime_item_instance_id: Uuid::from_u128(991), persistent_copy_uuid: None });
    let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId::from("foreign"), Side::Player, request).unwrap() else { panic!() };
    assert_eq!(core.units[&creator].stats.current_health, 80);
    assert_eq!(core.units[&unit_ids[0]].stats.max_health, 40);
    assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].segmented.as_ref().unwrap().remaining_health, 150);
}

#[test]
fn u07_segmented_public_fallback_rejects_lethal_before_payment_and_late_failure_is_normal() {
    use crate::game::{ability::{SkillId, SkillActivationMode}, battle::event_log::SkillCastTarget, data::skill_data::SkillDatabase};
    let ron = TILE_CREATION.replace("definition_id:\"friend\"", "definition_id:\"wall\",wall_cost:Some((pool_id:\"meat\",health_cost_percent:20,wall_health_percent:200))");
    let skills = SkillDatabase::from_ron("u07/cost.ron", &ron).unwrap();
    let (mut core, creator) = fixture_with_skills(Side::Player, skills);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let caster = core.units.get_mut(&creator).unwrap();
    caster.skill_id = Some(SkillId::from("create_friend"));
    caster.skill_activation_mode = SkillActivationMode::Manual;
    caster.resonance_current = 100;
    caster.stats.current_health = 20;
    let target = Some(SkillCastTarget::Tile { position: Position::new(1, 1) });
    let before = core.transaction_debug_snapshot();
    assert!(core.request_skill_action(0, creator, SkillId::from("create_friend"), target).is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
    assert_eq!(core.units[&creator].resonance_current, 100);
    core.units.get_mut(&creator).unwrap().stats.current_health = 21;
    let before_preview = core.transaction_debug_snapshot();
    let previews = core.object_creation_preview(0, creator, &SkillId::from("create_friend"), target).unwrap();
    assert_eq!(previews[0].health_cost, 20);
    assert_eq!(previews[0].expected_health, Some(40));
    assert!(previews[0].segment_cost.is_none());
    assert_eq!(core.transaction_debug_snapshot(), before_preview);
    let accepted = core.request_skill_action(0, creator, SkillId::from("create_friend"), target).unwrap();
    assert_eq!(core.units[&creator].resonance_current, 0);
    core.units.get_mut(&creator).unwrap().stats.current_health = 20;
    core.step_battle_execution_until(&mut execution, 100).unwrap();
    assert_eq!(core.units[&creator].stats.current_health, 20);
    assert_eq!(core.units[&creator].resonance_current, 0);
    assert!(core.independent_objects.objects.is_empty());
    assert!(core.query_action(&accepted).unwrap().terminal.is_some());
    assert!(matches!(&core.independent_objects_snapshot(100).unwrap().creations[0].settlement, Some(SpawnRequestSettlement::Rejected { reason: SpawnRejection::MutationFailed { game_error_code } }) if game_error_code == "object_wall_cost_unavailable"));
    core.units.get_mut(&creator).unwrap().stats.current_health = 21;
    core.units.get_mut(&creator).unwrap().resonance_current = 100;
    core.request_skill_action(101, creator, SkillId::from("create_friend"), target).unwrap();
    core.step_battle_execution_until(&mut execution, 201).unwrap();
    assert_eq!(core.units[&creator].stats.current_health, 1);
    assert_eq!(core.independent_objects_snapshot(201).unwrap().active[0].max_health, Some(40));
}

const SEGMENT_PRODUCER: &str = r#"(skills:[(
    id:"seed_pool",name:"Seed pool",kind:Targeted,use_lifetime:Action,focus_time_ms:1,
    cast_targeting:Explicit(selection:self,target:SelfUnit,reach:pattern,air_capable:false),
    steps:[(id:"pool",delay_ms:0,target:SelfUnit,delivery:Instant,effects:[
        (id:"segments",op:Sustained(definition:SegmentedShield(duration_ms:5000,pool:(
            pool_id:"meat",segment_health:100,max_segments:3,initial_health:150,block_per_segment:1,priority:0
        ))))
    ])]
)])"#;

#[test]
fn u07_segmented_public_producer_duplicate_rejection_and_preview_are_read_only() {
    use crate::game::{ability::{SkillId, SkillActivationMode}, battle::event_log::SkillCastTarget, data::skill_data::SkillDatabase};
    let mut skills = SkillDatabase::from_ron("u07/producer.ron", SEGMENT_PRODUCER).unwrap();
    let wall = TILE_CREATION.replace("definition_id:\"friend\"", "definition_id:\"wall\",wall_cost:Some((pool_id:\"meat\",health_cost_percent:20,wall_health_percent:200))");
    skills.skills.extend(SkillDatabase::from_ron("u07/wall.ron", &wall).unwrap().skills);
    let (mut core, creator) = fixture_with_skills(Side::Player, skills);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let unit = core.units.get_mut(&creator).unwrap();
    unit.skill_id = Some(SkillId::from("seed_pool"));
    unit.skill_activation_mode = SkillActivationMode::Manual;
    unit.resonance_current = 100;
    let producer = core.request_skill_action(0, creator, SkillId::from("seed_pool"), None).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let pools = core.sustained_effect_snapshot().unwrap();
    assert_eq!(pools.active[0].segmented.as_ref().unwrap().remaining_health, 150);
    assert_eq!(pools.active[0].skill_id.as_ref().unwrap().as_str(), "seed_pool");
    assert!(!core.resonance_use_has_obligations(creator));
    assert!(core.query_action(&producer).unwrap().terminal.is_some());
    let before_recharge = core.transaction_debug_snapshot();
    assert!(core.runtime_transaction(|core| core.recharge_sustained_shield(creator, &SkillId::from("seed_pool"), 100, 1)).is_err());
    assert_eq!(core.transaction_debug_snapshot(), before_recharge);
    core.units.get_mut(&creator).unwrap().resonance_current = 100;
    let before = core.transaction_debug_snapshot();
    assert!(core.request_skill_action(2, creator, SkillId::from("seed_pool"), None).is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
    // The fixture exposes the independent consumer after proving the public producer.
    core.units.get_mut(&creator).unwrap().skill_id = Some(SkillId::from("create_friend"));
    let target = Some(SkillCastTarget::Tile { position: Position::new(2, 1) });
    let before = core.transaction_debug_snapshot();
    let view = core.object_creation_preview(2, creator, &SkillId::from("create_friend"), target).unwrap();
    assert_eq!(view[0].health_cost, 0);
    assert_eq!(view[0].expected_max_health, Some(50));
    let segment = view[0].segment_cost.as_ref().unwrap();
    assert_eq!((segment.before, segment.detached, segment.after), (150, 50, 100));
    assert_eq!(core.transaction_debug_snapshot(), before);
    assert_eq!(core.object_creation_preview(2, creator, &SkillId::from("create_friend"), target).unwrap(), view);
    core.request_skill_action(2, creator, SkillId::from("create_friend"), target).unwrap();
    core.step_battle_execution_until(&mut execution, 102).unwrap();
    assert_eq!(core.units[&creator].stats.current_health, 100);
    assert_eq!(core.independent_objects_snapshot(102).unwrap().active[0].max_health, Some(50));
    assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].segmented.as_ref().unwrap().remaining_health, 100);
}

#[test]
fn u07_segmented_strict_schema_rejects_zero_overflow_and_wrong_consumer_kind() {
    use crate::game::data::skill_data::SkillDatabase;
    SkillDatabase::from_ron("u07/valid_pool.ron", SEGMENT_PRODUCER).unwrap();
    assert!(SkillDatabase::from_ron("u07/repeated_pool.ron", &SEGMENT_PRODUCER.replace("id:\"pool\",delay_ms:0", "id:\"pool\",delay_ms:0,repeat:Times(count:2)")).is_err());
    assert!(SkillDatabase::from_ron("u07/invalid_lease.ron", &SEGMENT_PRODUCER.replace("use_lifetime:Action", "use_lifetime:ActionAndEffects(effects:[(step:\"pool\",effect:\"segments\")])")).is_err());
    for (from, to) in [("segment_health:100", "segment_health:0"), ("max_segments:3", "max_segments:0"),
        ("initial_health:150", "initial_health:301"), ("segment_health:100", "segment_health:4294967295"),
        ("block_per_segment:1", "block_per_segment:4294967295"), ("pool_id:\"meat\"", "pool_id:\"\"")] {
        assert!(SkillDatabase::from_ron("u07/invalid_pool.ron", &SEGMENT_PRODUCER.replace(from, to)).is_err(), "{to}");
    }
    let bad = TILE_CREATION.replace("anchor:CastTile", "anchor:CastTile,wall_cost:Some((pool_id:\"meat\",health_cost_percent:20,wall_health_percent:200))");
    let skills = SkillDatabase::from_ron("u07/bad_kind.ron", &bad).unwrap();
    let (core, _) = fixture(Side::Player);
    assert!(core.game_data.boss_runtime_data.get_summoned_unit_skill_validation(&skills).is_err());
}

#[test]
fn u07_segmented_damage_and_detachment_release_only_excess_blocked_enemies() {
    use crate::game::battle::{scenario::PlayerMovementPlan, damage::{DamageSource, DamageType, BattleCommand}};
    let (mut core, creator) = fixture(Side::Player);
    Arc::make_mut(&mut core.scenario).tactical_plan.player_plan = PlayerMovementPlan::FixedDefense;
    core.units.get_mut(&creator).unwrap().block_capacity = 1;
    core.units.get_mut(&creator).unwrap().block_radius_units = 2.0;
    for index in 0..3 {
        let mut group = core.scenario.groups[0].clone();
        group.side = Side::Opponent;
        group.spawns[0].side = Side::Opponent;
        group.spawns[0].instance_salt = 10 + index;
        group.spawns[0].position = Position::new(1, index as i32);
        group.spawns[0].unit_ref = ScenarioUnitRef::new(format!("enemy{index}"));
        core.spawn_scenario_group_member(&group, 0).unwrap();
    }
    segmented_fixture_pool(&mut core, creator, 150);
    let original = core.block_state.blocker_to_enemies[&creator].clone();
    assert_eq!(original.len(), 3);
    let source = core.damage_source_snapshot_for_unit(creator, creator, DamageSource::Ability, DamageType::True, 60, Default::default(), 0, 10, false).unwrap();
    core.process_commands_with_modifier_source(vec![BattleCommand::ApplyDamage { target_id: creator, source_snapshot: source }], 10, Some(creator));
    assert_eq!(core.block_state.blocker_to_enemies[&creator], original[..2]);
    let mut request = input(creator, "wall", vec![Position::new(3, 2)]);
    request.wall_cost = Some(wall_cost());
    request.time_ms = 11;
    core.create_independent_objects(SpawnRequestId::from("detach"), Side::Player, request).unwrap();
    assert_eq!(core.block_state.blocker_to_enemies[&creator], original[..1]);
    assert_eq!(core.units[&creator].stats.current_health, 100);
}

#[test]
fn native_u07_segmented_late_spawn_fault_restores_pool_health_facts_and_retry() {
    use crate::game::battle::runtime_control as native;
    for with_pool in [false, true] {
        let (mut base, creator) = fixture(Side::Player);
        if with_pool { segmented_fixture_pool(&mut base, creator, 150); }
        let mut request = input(creator, "wall", vec![Position::new(2, 1)]);
        request.wall_cost = Some(wall_cost());
        let mut probe = base.clone();
        let (result, admissions) = native::observe_retained_admissions(|| probe.runtime_transaction(|core| core.create_independent_objects(SpawnRequestId::from("paid"), Side::Player, request.clone())));
        assert!(matches!(result.unwrap(), SpawnRequestSettlement::Spawned { .. }));
        let limit = admissions.iter().rfind(|(site, _, _)| *site == "queued_event_payload").unwrap().2 - 1;
        base.runtime_facts.limits.max_retained_bytes = limit;
        let before = base.transaction_debug_snapshot();
        let queue = base.event_queue_debug_snapshot();
        let pools = base.sustained_effect_snapshot().unwrap();
        let failure = base.runtime_transaction(|core| core.create_independent_objects(SpawnRequestId::from("paid"), Side::Player, request.clone())).unwrap_err();
        assert!(matches!(failure, GameError::BattleRuntimeFault(native::BattleRuntimeFault { code: native::RuntimeFaultCode::RetainedByteLimit, site: "queued_event_payload", .. })), "{failure:?}");
        assert_eq!(base.transaction_debug_snapshot(), before);
        assert_eq!(base.event_queue_debug_snapshot(), queue);
        assert_eq!(base.sustained_effect_snapshot().unwrap(), pools);
        assert_eq!(base.units[&creator].stats.current_health, 100);
        assert!(base.independent_objects.objects.is_empty());
        base.runtime_facts.limits = Default::default();
        base.runtime_transaction(|core| core.create_independent_objects(SpawnRequestId::from("paid"), Side::Player, request.clone())).unwrap();
        assert_eq!(base.independent_objects.objects.len(), 1);
        assert_eq!(base.units[&creator].stats.current_health, if with_pool { 100 } else { 80 });
    }
}

#[test]
fn u07_segmented_ordinary_shield_priority_healing_expiry_and_clone_are_independent() {
    use crate::game::{ability::sustained::SustainedEffectDef, battle::{damage::{DamageSource, DamageType, BattleCommand}, event_log::HpChangeReason}};
    let (mut core, creator) = fixture(Side::Player);
    segmented_fixture_pool(&mut core, creator, 150);
    core.begin_sustained(creator, creator, SustainedEffectDef::Shield { duration_ms: 6000, amount: 20, priority: 10, shared_radius_units: None }, 0).unwrap();
    let original = core.clone();
    let source = core.damage_source_snapshot_for_unit(creator, creator, DamageSource::Ability, DamageType::True, 50, Default::default(), 0, 10, false).unwrap();
    core.process_commands_with_modifier_source(vec![BattleCommand::ApplyDamage { target_id: creator, source_snapshot: source }], 10, Some(creator));
    let pools = core.sustained_effect_snapshot().unwrap();
    assert_eq!(pools.active.iter().find_map(|p| p.segmented.as_ref()).unwrap().remaining_health, 120);
    assert_eq!(original.sustained_effect_snapshot().unwrap().active.iter().find_map(|p| p.segmented.as_ref()).unwrap().remaining_health, 150);
    core.units.get_mut(&creator).unwrap().stats.current_health = 60;
    core.apply_hp_delta_and_record(Some(creator), creator, 25, 11, HpChangeReason::Command);
    assert_eq!(core.units[&creator].stats.current_health, 85);
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.iter().find_map(|p| p.segmented.as_ref()).unwrap().remaining_health, 120);
    core.reconcile_sustained(4999).unwrap();
    assert_eq!(core.effective_block_capacity(creator), 2);
    core.reconcile_sustained(5000).unwrap();
    assert_eq!(core.effective_block_capacity(creator), 0);
    assert!(core.sustained_effect_snapshot().unwrap().active.iter().all(|p| p.segmented.is_none()));
}

#[test]
fn u07_segmented_fallback_health_cost_bypasses_shield_parry_and_survival() {
    use crate::game::ability::{SkillId, sustained::{SustainedEffectDef, SurvivalPolicy}};
    let (mut core, creator) = fixture(Side::Player);
    for definition in [
        SustainedEffectDef::Shield { duration_ms: 5000, amount: 100, priority: 0, shared_radius_units: None },
        SustainedEffectDef::Survival { duration_ms: 5000, policy: SurvivalPolicy::FloorOne },
        SustainedEffectDef::Parry { duration_ms: 5000, window_ms: 1000, invulnerable_ms: 1000, miss_recovery_ms: 1000, reduction_percent: 100, counter_skill: SkillId::from("counter") },
    ] { core.begin_sustained(creator, creator, definition, 0).unwrap(); }
    let before = core.sustained_effect_snapshot().unwrap();
    let mut request = input(creator, "wall", vec![Position::new(2, 1)]);
    request.wall_cost = Some(wall_cost());
    let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId::from("protected_cost"), Side::Player, request).unwrap() else { panic!() };
    assert_eq!(core.units[&creator].stats.current_health, 80);
    assert_eq!(core.units[&unit_ids[0]].stats.max_health, 40);
    assert_eq!(core.sustained_effect_snapshot().unwrap(), before);
    assert!(core.runtime_fact_snapshot().unwrap().summary.latest_damage.is_none());
}
