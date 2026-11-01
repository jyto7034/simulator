#[test]
fn priority_heal_world_granted_weapon_platform_ground_and_same_action() {
    priority_heal_world_with_support(0, false);
}
#[test]
fn composition_equipment_support_world_basic_heal_two_full_amplified_amounts() {
    priority_heal_world_with_support(25, false);
}
#[test]
fn composition_heal_barrier_world_basic_heal_two_independent_pools() {
    priority_heal_world_with_support(25, true);
}
fn priority_heal_world_with_support(percent: u32, barrier: bool) {
    use crate::game::{
        battle::event_log::BattleLogEvent,
        data::{equipment_data::WeaponCombatProfile, ValidatedGameData},
    };
    let profile: WeaponCombatProfile = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/queen-healing.weapon.ron"
    )))
    .unwrap();
    let live = ValidatedGameData::load_live_embedded().unwrap();
    let mut equipment = live.equipment_data.as_ref().clone();
    let staff = equipment
        .items
        .iter_mut()
        .find(|item| item.id == "standard_training_healing_staff")
        .unwrap();
    let definition = staff.uuid;
    staff.weapon_profile = Some(profile);
    equipment.equipment_supports.push(
        crate::game::ability::equipment_support::EquipmentSupportBinding {
            equipment: "standard_training_healing_staff".into(),
            program: crate::game::ability::equipment_support::EquipmentSupportProgram { shield_beneficiary_damage_percent: None, received_heal_percent: None, control_duration: None,
                basic_heal_echo: None, resonance_gain: None, actual_heal_reaction: None,
                heal_percent: percent,
                shield_percent: percent,
                direct_heal_barrier: barrier.then_some(
                    crate::game::ability::equipment_support::DirectHealBarrierProgram {
                        hp_threshold_percent: 50,
                        actual_heal_percent: 25,
                        duration_ms: 5000,
                        cooldown_ms: 8000,
                        priority: 0,
                    },
                ),
            },
            source: None,
        },
    );
    let data = GameDataBuilder::from_validated(&live)
        .with_equipment_data(Arc::new(equipment))
        .build_arc();
    let (mut core, player) =
        start_selected_with_data(&["field_medic", "breach_guard", "maintenance_tech"], data);
    let employees: Vec<_> = ["field_medic", "breach_guard", "maintenance_tech"]
        .into_iter()
        .map(|name| {
            core.state
                .roster
                .iter()
                .find(|employee| employee.employee_definition_id == name)
                .unwrap()
                .uuid
        })
        .collect();
    let copy = core
        .state
        .roster
        .iter()
        .find(|e| e.uuid == employees[0])
        .unwrap()
        .loadout
        .item_slot
        .iter()
        .find(|item| item.base_uuid == definition)
        .unwrap()
        .instance_uuid;
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        MapNodePayload::Encounter {
            encounter_id: Some("suppress_burrowing_heaven".into()),
        },
    );
    select_and_confirm_map_node(&mut core, player, node);
    // Test-only TP and injuries; grants, slots, profile loading, deployment and clock are real.
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .apply_delta(99);
    let mut ids = Vec::new();
    let mut kinds = Vec::new();
    for employee in &employees {
        let profile =
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                core.roster().unwrap(),
                core.inventory().unwrap(),
                &core.state.skill_fragments,
                core.game_data.as_ref(),
                *employee,
            )
            .unwrap();
        let active = core.state.active_battle.as_ref().unwrap();
        let (kind, position) = active
            .combat_preview
            .deployment_zones
            .iter()
            .filter(|zone| zone.kind.supports_affinity(profile.deployment_affinity))
            .flat_map(|zone| {
                zone.cells
                    .iter()
                    .map(move |position| (zone.kind, *position))
            })
            .find(|(_, position)| {
                active
                    .battle
                    .validate_player_tactical_destination(*position, None)
                    .is_ok()
            })
            .unwrap();
        kinds.push(format!("{kind:?}"));
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: *employee,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        ids.push(
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .live_deployment
                .as_ref()
                .unwrap()
                .deployed_units[employee]
                .unit_instance_id,
        );
    }
    assert_eq!(kinds[0], "Platform");
    assert_eq!(kinds[1], "Ground");
    assert_eq!(kinds[2], "Platform");
    {
        let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
        for (i, id) in ids.iter().enumerate() {
            let unit = battle.units.get_mut(id).unwrap();
            unit.stats.max_health = 100;
            unit.stats.current_health = if i == 0 {
                100
            } else if i == 1 {
                50
            } else {
                25
            };
        }
        battle.units.get_mut(&ids[0]).unwrap().stats.spell_power = 20;
        battle.enable_authoring_observation();
        let query = battle.basic_action_snapshot(ids[0], true).unwrap();
        assert_eq!(query.eligible_target_ids.len(), 2);
        assert_eq!(query.selected_target_id, Some(ids[2]));
    }
    core.advance_active_battle_by(100).unwrap();
    let battle = &core.state.active_battle.as_ref().unwrap().battle;
    let amount = if percent == 0 { 20 } else { 25 };
    assert_eq!(battle.units[&ids[1]].stats.current_health, 50 + amount);
    assert_eq!(battle.units[&ids[2]].stats.current_health, 25 + amount);
    let heals: Vec<_> = battle
        .event_log
        .entries_after_seq(0)
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source),
                target_instance_id,
                delta,
                ..
            } if source == ids[0] && delta > 0 => Some((entry.time_ms, target_instance_id, delta)),
            _ => None,
        })
        .collect();
    assert_eq!(heals.len(), 2);
    assert_eq!(heals[0].0, heals[1].0);
    assert_eq!((heals[0].1, heals[1].1), (ids[2], ids[1]));
    let view = battle.runtime_fact_view();
    let first = view
        .actors()
        .find(|(id, _)| **id == ids[1])
        .unwrap()
        .1
        .received
        .latest_heal
        .as_ref()
        .unwrap();
    let second = view
        .actors()
        .find(|(id, _)| **id == ids[2])
        .unwrap()
        .1
        .received
        .latest_heal
        .as_ref()
        .unwrap();
    assert!(first.source.action.is_some());
    assert_eq!(first.source.action, second.source.action);
    assert!(first.source.cast.is_none() && second.source.cast.is_none());
    let source = first.source.equipment.as_ref().unwrap();
    assert_eq!(source.definition_uuid, definition);
    assert_eq!(source.persistent_copy_uuid, Some(copy));
    assert_eq!(second.source.equipment.as_ref(), Some(source));
    assert_eq!(
        view.actors()
            .find(|(id, _)| **id == ids[0])
            .unwrap()
            .1
            .caused
            .damage_hp_lost,
        0
    );
    assert_eq!(first.requested_delta, i64::from(amount));
    assert_eq!(second.requested_delta, i64::from(amount));
    if barrier {
        let pools = battle.sustained_effect_snapshot().unwrap();
        for id in [ids[1], ids[2]] {
            let pool = pools
                .active
                .iter()
                .find(|p| p.target == id && p.shield.is_some())
                .unwrap();
            assert_eq!(pool.shield.as_ref().unwrap().remaining, 7);
        }
        let observation = battle.authoring_observation().unwrap().unwrap();
        let witness = observation.latest_heal_barrier.as_ref().unwrap();
        assert_eq!(witness.actual_healed, 25);
        assert_eq!(witness.healing_source.action, first.source.action);
        assert_eq!(witness.equipment.persistent_copy_uuid, Some(copy));
    }
    if percent > 0 && !barrier {
        let support = battle
            .authoring_observation()
            .unwrap()
            .unwrap()
            .latest_equipment_support
            .as_ref()
            .unwrap();
        assert_eq!(
            support.contributors[0].equipment.persistent_copy_uuid,
            Some(copy)
        );
        assert_eq!(support.source.action, first.source.action);
        assert_eq!(support.increased_amount, 25);
    }
    println!("H42 World: granted copy={copy}, layers={kinds:?}, HP=70/45, same action={:?}, release={}ms",first.source.action,heals[0].0);
}
