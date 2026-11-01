use super::*;
use crate::authoring::composition::canonical_little_red_equipment_tests as gear;
use crate::game::{
    battle::{
        core::equipment_magazine::MagazineKey,
        scenario::BattleScenario,
        types::{BattleEquipmentBinding, BattleUnitSource, BattleVerdict},
    },
    data::skill_fragment_data::SkillFragmentDatabase,
    resource::{ResourceCatalog, ResourceId, ResourceOwner, RunResourceState},
    resources::item_slot::{EquipmentSlotAddress, SlotLayoutKind},
};

fn setup(
    fragment: bool,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let program = gear::artifact();
    let fragment_program = compile_json(&canonical::resolved().to_string()).unwrap();
    let native = crate::authoring::node_graph::NativeGraphContent {
        skills: fragment_program.load_native().unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    };
    let equipment = gear::catalog();
    let weapon = equipment[0].clone();
    let armor = equipment[1].uuid;
    let accessory = equipment[2].uuid;
    let metadata = canonical::catalog();
    let fragment_id = metadata.id.clone();
    let abnormalities: crate::game::data::abnormality_data::AbnormalityDatabase =
        ron::de::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../game_resources/data/abnormalities/base.ron"
        )))
        .unwrap();
    let mut origin = abnormalities
        .get_by_id("f-01-57_little_red")
        .unwrap()
        .clone();
    origin.skill_id = None;
    origin.omen_chain_id = None;
    let (mut core, state, a, enemy, b) = fixture_content_with_data(
        &[],
        &[],
        native,
        |item| *item = weapon,
        ResourceCatalog {
            resources: program.resources().to_vec(),
        },
        |builder| {
            builder
                .with_relics(canonical::relics())
                .with_equipment(equipment)
                .with_skill_fragments(SkillFragmentDatabase::new(vec![metadata]))
                .with_abnormalities(vec![origin])
                .with_equipment_attacks(program.equipment_attacks().to_vec())
                .with_equipment_defenses(program.equipment_defenses().to_vec())
        },
        |scenario| {
            scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(120000);
            for spawn in &mut scenario.groups[1].spawns {
                let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                    panic!()
                };
                if fragment {
                    profile.skill_id = Some("fragment_little_red_volley".into());
                    let source = profile.fragment_source.as_mut().unwrap();
                    source.definition_id = fragment_id.clone();
                    source.skill_id = "fragment_little_red_volley".into();
                } else {
                    profile.skill_id = None;
                    profile.fragment_source = None;
                }
                spawn.draft.equipment_layout = Some(SlotLayoutKind::ByType);
                spawn.draft.equipped_items[0].slot = Some(EquipmentSlotAddress::Weapon);
                spawn.draft.equipped_items[0].persistent_copy_uuid =
                    Some(Uuid::from_u128(0xf11700 + spawn.instance_salt as u128 * 3));
                for (offset, uuid, slot) in [
                    (1, armor, EquipmentSlotAddress::Armor),
                    (2, accessory, EquipmentSlotAddress::Accessory),
                ] {
                    spawn.draft.equipped_items.push(BattleEquipmentBinding {
                        weapon_stage: Default::default(),
                        base_uuid: uuid,
                        persistent_copy_uuid: Some(Uuid::from_u128(
                            0xf11700 + spawn.instance_salt as u128 * 3 + offset,
                        )),
                        slot: Some(slot),
                    });
                }
            }
        },
    );
    for u in core.units.values_mut() {
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
        u.stats.defense = 0;
        u.stats.move_speed_units_per_ms = 0;
        u.stats.attack_interval_ms = 100000;
        u.next_basic_attack_ms = 100000;
        u.basic_attack_epoch += 1;
        u.action_locks = Default::default();
        u.basic_attack_windup_until_ms = None;
        u.pending_basic_attack = false;
    }
    place(&mut core, a, 2, 1);
    place(&mut core, enemy, 4, 1);
    place(&mut core, b, 7, 3);
    let owners = core
        .items
        .values()
        .filter_map(|i| i.persistent_copy_uuid)
        .map(ResourceOwner::EquipmentCopy)
        .collect();
    core.mount_run_resources(
        &RunResourceState {
            revision: 0,
            closed: false,
            entries: vec![],
        },
        &owners,
    )
    .unwrap();
    core.enable_authoring_observation();
    (core, state, a, enemy, b)
}
fn accessory_copy(core: &BattleCore, actor: UnitInstanceId) -> Uuid {
    let definition = gear::catalog()[2].uuid;
    core.items
        .values()
        .find(|item| item.owner_unit_instance == actor && item.base_uuid == definition)
        .unwrap()
        .persistent_copy_uuid
        .unwrap()
}
fn bounty(core: &BattleCore, actor: UnitInstanceId, confirmed: bool) -> u32 {
    let runtime = core.runtime_facts.resources.as_ref().unwrap();
    let (id, scope) = if confirmed {
        (
            "little_red_bounty_confirmed",
            &runtime.run.as_ref().unwrap().scope,
        )
    } else {
        ("little_red_bounty_pending", &runtime.attempt)
    };
    let lease = runtime
        .store
        .bind(
            &ResourceId::new(id.into()).unwrap(),
            ResourceOwner::EquipmentCopy(accessory_copy(core, actor)),
            scope,
        )
        .unwrap();
    runtime.store.value(&lease).unwrap().balance
}
fn magazine(
    core: &BattleCore,
    actor: UnitInstanceId,
) -> &super::super::super::equipment_magazine::Magazine {
    let definition = gear::catalog()[0].uuid;
    let copy = core
        .items
        .values()
        .find(|i| i.owner_unit_instance == actor && i.base_uuid == definition)
        .unwrap()
        .persistent_copy_uuid
        .unwrap();
    &core.equipment_attack_state.magazines[&MagazineKey::Persistent(copy)]
}

#[test]
fn canonical_little_red_actual_gear_shares_cohort_and_keeps_three_kill_rewards_separate() {
    for fragment in [false, true] {
        let (mut core, mut state, a, enemy, b) = setup(fragment);
        let owners = core.shared_cohorts_snapshot().unwrap().owners;
        assert_eq!(owners.len(), 2);
        assert!(owners
            .iter()
            .all(|o| o.providers.len() == if fragment { 2 } else { 1 }));
        core.process_event(
            crate::game::battle::enums::BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: a,
                target_instance_id: Some(enemy),
                schedule_next: false,
                cause: Default::default(),
            },
            0,
        )
        .unwrap();
        core.step_battle_execution_until(&mut state, 800).unwrap();
        assert_eq!(magazine(&core, a).remaining, 5);
        assert!(!magazine(&core, a).enhanced);
        core.step_battle_execution_until(&mut state, 5000).unwrap();
        let spawned = core.shared_cohorts_snapshot().unwrap();
        assert_eq!(spawned.units.len(), 2);
        assert!(spawned
            .units
            .iter()
            .all(|u| u.fragment_recipient.is_some() == fragment));
        assert!(spawned.owners.iter().all(|o| o.next_spawn_ms == 10000));
        let first = spawned.units[0].unit;
        let second = spawned.units[1].unit;
        hit(&mut core, a, first, 1000, 5000);
        assert_eq!(
            (magazine(&core, a).remaining, magazine(&core, a).enhanced),
            (6, true)
        );
        assert_eq!((bounty(&core, a, false), bounty(&core, a, true)), (1, 0));
        let before = core.units[&a].stats.current_health;
        hit(&mut core, enemy, a, 100, 5000);
        assert_eq!(core.units[&a].stats.current_health, before - 70);
        assert_eq!(
            core.equipment_defense
                .deferred
                .values()
                .filter(|d| d.owner == a)
                .map(|d| d.remaining)
                .sum::<u32>(),
            30
        );
        hit(&mut core, a, second, 1000, 5000);
        assert_eq!(
            core.equipment_defense
                .deferred
                .values()
                .filter(|d| d.owner == a)
                .map(|d| d.remaining)
                .sum::<u32>(),
            15
        );
        assert_eq!((bounty(&core, a, false), bounty(&core, b, false)), (2, 0));
        let stock = core.input_plans_snapshot().unwrap();
        assert_eq!(
            stock.owners.iter().map(|o| o.enhanced).sum::<u32>(),
            if fragment { 2 } else { 0 }
        );
        core.step_battle_execution_until(&mut state, 6000).unwrap();
        assert_eq!(core.units[&a].stats.current_health, before - 73);
        Arc::make_mut(&mut core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(6100);
        core.step_battle_execution_until(&mut state, 6100).unwrap();
        let result = core.finalize_battle_execution(&mut state).unwrap();
        assert_eq!(result.verdict, BattleVerdict::Victory);
        assert_eq!((bounty(&core, a, false), bounty(&core, a, true)), (0, 2));
        assert!(core.finalize_battle_execution(&mut state).is_err());
        assert_eq!(bounty(&core, a, true), 2);
    }
}
