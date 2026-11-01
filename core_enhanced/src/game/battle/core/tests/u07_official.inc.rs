fn official_object_fixture() -> (BattleCore, UnitInstanceId) {
    let data = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
    let (mut core, creator) = fixture_with_skills(Side::Player, crate::game::data::skill_data::SkillDatabase::from_ron("u07/fixture.ron", TILE_CREATION).unwrap());
    // Geometry/readiness is synthetic here. Definitions and skills are the full
    // strict-loaded official catalog; World ownership is tested separately.
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        serde_json::from_value(serde_json::to_value(data.database().equipment_data.as_ref()).unwrap()).unwrap();
    equipment.items.extend(core.game_data.equipment_data.items.clone());
    core.game_data = GameDataBuilder::from_validated(&data).with_equipment_data(Arc::new(equipment)).build_arc();
    (core, creator)
}

fn object_test_enemy(core: &mut BattleCore, index: u32, position: Position, airborne: bool) -> UnitInstanceId {
    let mut enemy = core.scenario.groups[0].clone();
    enemy.id = ScenarioGroupId::new(format!("enemy{index}"));
    enemy.side = Side::Opponent;
    enemy.spawns[0].side = Side::Opponent;
    enemy.spawns[0].instance_salt = index;
    enemy.spawns[0].unit_ref = ScenarioUnitRef::new(format!("enemy{index}"));
    enemy.spawns[0].position = position;
    let mut profile = UnitCombatProfile::for_employee(1000, 0, 0, 0, 100);
    if airborne { profile.mobility_kind = crate::game::battle::types::MobilityKind::Airborne; }
    enemy.spawns[0].draft.source = BattleUnitSource::TestFixture { base_uuid: Uuid::from_u128(100), profile };
    enemy.spawns[0].draft.equipped_items.clear();
    core.spawn_scenario_group_member(&enemy, 0).unwrap()
}

#[test]
fn u07_official_small_hits_one_stable_target_big_hits_area_and_ignores_air() {
    for (definition, damage, count) in [("laetitia_small_friend", 60, 1), ("laetitia_big_friend", 135, 2)] {
        let (mut core, creator) = official_object_fixture();
        let SpawnRequestSettlement::Spawned { unit_ids } = core.create_independent_objects(SpawnRequestId::from("friend"), Side::Player, input(creator, definition, vec![Position::new(2, 1)])).unwrap() else { panic!() };
        let object = unit_ids[0];
        core.units.get_mut(&creator).unwrap().stats.attack = 999;
        let air = object_test_enemy(&mut core, 3, Position::new(2, 1), true);
        core.reconcile_independent_objects(11).unwrap();
        assert!(core.independent_objects.objects.contains_key(&object), "air-only trigger cannot consume");
        let mut ground = vec![object_test_enemy(&mut core, 2, Position::new(3, 1), false), object_test_enemy(&mut core, 1, Position::new(2, 2), false)];
        ground.sort_unstable();
        core.reconcile_independent_objects(12).unwrap();
        assert!(!core.independent_objects.objects.contains_key(&object));
        assert_eq!(core.units[&air].stats.current_health, 1000);
        for (i, target) in ground.iter().enumerate() { assert_eq!(core.units[target].stats.current_health, 1000 - if i < count { damage } else { 0 }); }
        let after = core.transaction_debug_snapshot();
        core.reconcile_independent_objects(12).unwrap();
        assert_eq!(core.transaction_debug_snapshot(), after, "same-time reconcile cannot hit again");
    }
}

#[test]
fn u07_official_base_second_cast_evolves_and_awakened_cast_creates_big() {
    use crate::game::{ability::{SkillId, SkillActivationMode}, battle::event_log::SkillCastTarget};
    for awakened in [false, true] {
        let (mut core, creator) = official_object_fixture();
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let skill = SkillId::from(if awakened { "laetitia_place_great_friend" } else { "laetitia_place_friend" });
        let caster = core.units.get_mut(&creator).unwrap();
        caster.skill_id = Some(skill.clone()); caster.skill_activation_mode = SkillActivationMode::Manual; caster.resonance_current = 100;
        let target = Some(SkillCastTarget::Tile { position: Position::new(2, 1) });
        let first = core.request_skill_action(0, creator, skill.clone(), target).unwrap();
        core.step_battle_execution_until(&mut execution, 500).unwrap();
        assert!(core.query_action(&first).unwrap().terminal.is_some());
        assert!(!core.resonance_use_has_obligations(creator));
        let initial = core.independent_objects_snapshot(500).unwrap();
        assert_eq!(initial.active.len(), 1);
        let id = initial.active[0].object_id;
        if !awakened {
            assert_eq!(initial.active[0].definition_id.as_str(), "laetitia_small_friend");
            core.units.get_mut(&creator).unwrap().resonance_current = 100;
            let preview = core.object_creation_preview(501, creator, &skill, target).unwrap();
            assert_eq!(preview[0].evolves_object_id, Some(id));
            core.request_skill_action(501, creator, skill.clone(), target).unwrap();
            core.step_battle_execution_until(&mut execution, 1001).unwrap();
        }
        let final_view = core.independent_objects_snapshot(1001).unwrap();
        assert_eq!(final_view.active.len(), 1);
        assert_eq!(final_view.active[0].object_id, id);
        assert_eq!(final_view.active[0].definition_id.as_str(), "laetitia_big_friend");
        assert_eq!(core.units[&creator].resonance_current, 0);
        core.units.get_mut(&creator).unwrap().resonance_current = 100;
        let before = core.transaction_debug_snapshot();
        assert!(core.request_skill_action(1002, creator, skill, target).is_err());
        assert_eq!(core.transaction_debug_snapshot(), before);
    }
}

#[test]
fn u07_official_wall_preview_and_actual_hp_cost_defenses_match() {
    use crate::game::{ability::{SkillId, SkillActivationMode}, battle::event_log::SkillCastTarget};
    for meat in [false, true] {
        let (mut core, creator) = official_object_fixture();
        let mut execution = core.start_battle_execution().unwrap(); core.step_battle_execution_until(&mut execution, 0).unwrap();
        let skill = SkillId::from(if meat { "melting_love_place_meat_wall" } else { "common_place_ice_wall" });
        let unit = core.units.get_mut(&creator).unwrap(); unit.skill_id = Some(skill.clone()); unit.skill_activation_mode = SkillActivationMode::Manual; unit.resonance_current = 100;
        let target = Some(SkillCastTarget::Tile { position: Position::new(2, 1) });
        let before = core.transaction_debug_snapshot();
        let preview = core.object_creation_preview(0, creator, &skill, target).unwrap();
        assert_eq!(preview[0].health_cost, if meat { 20 } else { 0 });
        assert_eq!(preview[0].expected_max_health, Some(if meat { 40 } else { 120 }));
        assert_eq!(core.transaction_debug_snapshot(), before);
        core.request_skill_action(0, creator, skill, target).unwrap(); core.step_battle_execution_until(&mut execution, 500).unwrap();
        let view = core.independent_objects_snapshot(500).unwrap(); let wall = &core.units[&view.active[0].object_id];
        assert_eq!(wall.stats.max_health, if meat { 40 } else { 120 });
        assert_eq!(wall.stats.defense, if meat { 7 } else { 10 });
        assert_eq!(wall.stats.magic_resist, if meat { 8 } else { 10 });
        assert_eq!(core.units[&creator].stats.current_health, if meat { 80 } else { 100 });
        core.withdraw_unit(creator, 501).unwrap(); core.reconcile_independent_object_removals(501).unwrap();
        assert_eq!(core.independent_objects_snapshot(501).unwrap().active.len(), 1);
    }
}

#[test]
fn u07_official_meat_wall_uses_exact_equipped_weapon_pool_across_fragment_source() {
    use crate::game::{ability::{SkillId, SkillActivationMode, sustained::{SustainedEffectDef, SegmentedShieldDef}}, battle::{event_log::SkillCastTarget, runtime_facts::EquipmentSourceSnapshot}, skill_fragment::FragmentSourceSnapshot};
    for with_weapon_pool in [false, true] {
        let (mut core, creator) = official_object_fixture();
        let mut execution = core.start_battle_execution().unwrap(); core.step_battle_execution_until(&mut execution, 0).unwrap();
        let weapon = core.items.values().find(|item| item.owner_unit_instance == creator).unwrap();
        let equipment = EquipmentSourceSnapshot { definition_uuid: weapon.base_uuid, runtime_item_instance_id: weapon.instance_id, persistent_copy_uuid: weapon.persistent_copy_uuid };
        let fragment = Arc::new(FragmentSourceSnapshot { copy_uuid: Uuid::from_u128(8807), definition_id: "u07_test_consumer".into(), form_id: "base".into(), skill_id: SkillId::from("melting_love_place_meat_wall") });
        let policy = SustainedEffectDef::SegmentedShield { duration_ms: 5000, pool: SegmentedShieldDef { pool_id: "melting_love_meat_shield".into(), segment_health: 100, max_segments: 3, initial_health: 150, block_per_segment: 1, priority: 0 } };
        // Same actor and pool name with fragment provenance is NOT the weapon pool.
        core.runtime_fact_source.fragment = Some(fragment.clone());
        core.begin_sustained(creator, creator, policy.clone(), 0).unwrap();
        core.runtime_fact_source = RuntimeFactSource::default();
        if with_weapon_pool {
            core.runtime_fact_source.equipment = Some(equipment);
            core.begin_sustained(creator, creator, policy, 0).unwrap();
            core.runtime_fact_source = RuntimeFactSource::default();
        }
        let unit = core.units.get_mut(&creator).unwrap(); unit.fragment_source = Some(fragment); unit.skill_id = Some(SkillId::from("melting_love_place_meat_wall")); unit.skill_activation_mode = SkillActivationMode::Manual; unit.resonance_current = 100;
        let target = Some(SkillCastTarget::Tile { position: Position::new(2, 1) });
        let before = core.transaction_debug_snapshot();
        let preview = core.object_creation_preview(0, creator, &SkillId::from("melting_love_place_meat_wall"), target).unwrap();
        assert_eq!(preview[0].health_cost, if with_weapon_pool { 0 } else { 20 });
        assert_eq!(preview[0].expected_max_health, Some(if with_weapon_pool { 50 } else { 40 }));
        assert_eq!(core.transaction_debug_snapshot(), before);
        core.request_skill_action(0, creator, SkillId::from("melting_love_place_meat_wall"), target).unwrap(); core.step_battle_execution_until(&mut execution, 500).unwrap();
        assert_eq!(core.units[&creator].stats.current_health, if with_weapon_pool { 100 } else { 80 });
        let pools = core.sustained_effect_snapshot().unwrap();
        assert_eq!(pools.active[0].segmented.as_ref().unwrap().remaining_health, 150);
        if with_weapon_pool { assert_eq!(pools.active[1].segmented.as_ref().unwrap().remaining_health, 100); assert_eq!(pools.latest_detachment.as_ref().unwrap().equipment_runtime_uuid, Some(equipment.runtime_item_instance_id)); }
        assert_eq!(core.independent_objects_snapshot(500).unwrap().active[0].max_health, Some(if with_weapon_pool { 50 } else { 40 }));
    }
}
