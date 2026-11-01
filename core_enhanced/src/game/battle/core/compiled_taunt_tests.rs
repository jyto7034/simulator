mod compiled_taunt {
    use super::*;
    use crate::{
        authoring::node_graph::{compile, tests::taunt_fixture, PortableGraph},
        game::{
            battle::{
                core::{
                    relic_numeric::RelicNumericOwner,
                    sim::{BattleExecutionState, BattleLiveCommand},
                },
                damage::{BattleCommand, DamageSource, DamageType},
                types::BattleUnitThreatClass,
            },
            data::{
                equipment_data::EquipmentDatabase, relic_data::RelicDatabase, ValidatedGameData,
            },
            managers::uuid_manager::UuidManager,
            relic::{
                structural::{compile_effective_plan, EffectivePlanInput, ProspectiveProvider},
                RelicInventory,
            },
            resource::{ResourceCatalog, ResourceOwner, RunResourceState},
        },
    };
    const RELICS: [&str; 5] = [
        "widened-horizon",
        "long-breath",
        "taunted-enemy-damage-reduction",
        "taunted-enemy-basic-attack-boost",
        "double-harvest",
    ];
    fn fixture(
        equipped: &[usize],
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let artifact = compile(&taunt_fixture().to_string(), "r11_aura".into()).unwrap();
        let content = PortableGraph::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_content()
            .unwrap();
        let mut skills = SkillDatabase::from_ron("fixture", R08_SKILLS)
            .unwrap()
            .skills;
        skills.extend(content.skills.skills.clone());
        let authored = ron::ser::to_string(&SkillDatabase::new(skills)).unwrap();
        let fragments = ron::de::from_str(
            &include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r09/fragments.ron"
            ))
            .replace("r08_strike", "r11_aura"),
        )
        .unwrap();
        let rows:Vec<_>=RELICS.iter().enumerate().map(|(i,id)|format!(r#"(id:"{id}",uuid:"{}",name:"Modifier fixture",description:"Graph owns this declaration",rarity:"fixture",allow_duplicates:true,effects:[(id:"unused",selector:(all_tags:["unused_fixture"]),percent:1)],dismantle_equipment_dust:1)"#,Uuid::from_u128(0xe200+i as u128))).collect();
        let relics: RelicDatabase =
            ron::de::from_str(&format!("(relics:[{}])", rows.join(","))).unwrap();
        let weapon = std::cell::RefCell::new(None);
        let (mut core, state, actor, ally) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |s| {
                s.terminal_outcomes = BattleScenario::fixture_survival_outcomes(8000);
                s.groups[0].enemy_movement_plan = None;
                for (i, spawn) in s.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipment_layout =
                        Some(crate::game::resources::item_slot::SlotLayoutKind::ByType);
                    spawn.draft.equipped_items[0].slot =
                        Some(crate::game::resources::item_slot::EquipmentSlotAddress::Weapon);
                    spawn.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe300 + i as u128));
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0xe400 + i as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                }
            },
            |w| {
                w.id = "laughter".into();
                *weapon.borrow_mut() = Some(w.clone());
            },
            Some(fragments),
            |b| {
                let catalogs = content
                    .bind_catalogs(
                        &SkillDatabase::from_ron("fixture", R08_SKILLS).unwrap(),
                        &EquipmentDatabase::new(vec![weapon.borrow().as_ref().unwrap().clone()]),
                        &ResourceCatalog { resources: vec![] },
                    )
                    .unwrap();
                b.with_skills(catalogs.skills)
                    .with_equipment_data(std::sync::Arc::new(catalogs.equipment))
                    .with_resources(catalogs.resources)
                    .with_relics(relics)
            },
        );
        let enemy = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for u in core.units.values_mut() {
            u.stats.current_health = 1000;
            u.stats.max_health = 1000;
            u.stats.defense = 0;
            u.next_basic_attack_ms = 100_000;
            u.stats.attack_interval_ms = 100_000;
            u.stats.move_speed_units_per_ms = 0;
            u.body.move_speed = 0.;
        }
        core.units.get_mut(&enemy).unwrap().threat_class = BattleUnitThreatClass::Normal;
        for (id, x, y) in [(actor, 2, 1), (enemy, 3, 1), (ally, 7, 3)] {
            place(&mut core, id, x, y);
        }
        let owners = core
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        core.mount_run_resources(&RunResourceState::default(), &owners)
            .unwrap();
        if !equipped.is_empty() {
            let data: ValidatedGameData = core.game_data.clone().into();
            let employee = core.units[&actor].source_owned_uuid;
            let mut inventory = RelicInventory::default();
            let mut ids = UuidManager::new(0xe500);
            for (slot, index) in equipped.iter().enumerate() {
                let copy = inventory
                    .grant(&data.relic_data.relics[*index], &mut ids)
                    .unwrap();
                inventory
                    .equip(copy, employee, slot, &data.relic_data)
                    .unwrap();
            }
            let providers = [ProspectiveProvider::Fragment(
                core.units[&actor]
                    .fragment_source
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .clone(),
            )];
            if let Some(plan) = compile_effective_plan(EffectivePlanInput {
                data: &data,
                employee_uuid: employee,
                inventory: &inventory,
                providers: &providers,
            })
            .unwrap()
            {
                let owner = crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(
                    std::collections::BTreeMap::from([(employee, plan)]),
                )
                .unwrap();
                core.set_effective_owner(owner).unwrap();
            }
            let owner = RelicNumericOwner::capture(
                &inventory,
                core.game_data.clone(),
                core.content_identity.as_ref().unwrap().clone(),
            )
            .unwrap();
            core.install_relic_numeric_owner(owner).unwrap();
        }
        (core, state, actor, enemy, ally)
    }
    fn place(core: &mut BattleCore, id: UnitInstanceId, x: i32, y: i32) {
        crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary(
            core, id, x, y,
        );
    }
    fn cast(core: &mut BattleCore, actor: UnitInstanceId, now: u64) {
        core.apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: actor,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: now,
        })
        .unwrap();
    }
    fn mass(core: &BattleCore, actor: UnitInstanceId) -> u32 {
        let copy = core
            .items
            .values()
            .find(|i| i.owner_unit_instance == actor)
            .unwrap()
            .persistent_copy_uuid
            .unwrap();
        let view = core.typed_resources().unwrap();
        let id = &view.catalog().resources[0].id;
        let scope = view
            .scopes()
            .find(|(_, l, _, _)| *l == crate::game::resource::ResourceLifetime::Run)
            .unwrap()
            .0;
        view.value(id, ResourceOwner::EquipmentCopy(copy), scope)
            .unwrap()
            .balance
    }
    fn hit(
        core: &mut BattleCore,
        source: UnitInstanceId,
        target: UnitInstanceId,
        kind: DamageSource,
        reaction: bool,
    ) -> u32 {
        let hp = core.units[&target].stats.current_health;
        core.runtime_transaction(|c| {
            let mut snapshot = c
                .damage_source_snapshot_for_unit(
                    source,
                    target,
                    kind,
                    DamageType::Physical,
                    100,
                    Default::default(),
                    0,
                    1,
                    false,
                )
                .unwrap();
            snapshot.fact_source.reaction = reaction;
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: snapshot,
                }],
                1,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
        hp - core.units[&target].stats.current_health
    }
    #[test]
    fn compiled_taunt_release_applies_state_without_lock_or_growth_and_long_breath_extends_bundle()
    {
        for (equipped, deadline) in [(vec![], 1500), (vec![1], 1875)] {
            let (mut core, mut state, a, enemy, _) = fixture(&equipped);
            cast(&mut core, a, 0);
            core.step_battle_execution_until(&mut state, 1).unwrap();
            assert_eq!(core.taunt_source(enemy), Some(a));
            assert!(!core.resonance_use_has_obligations(a));
            assert_eq!(mass(&core, a), 0);
            core.step_battle_execution_until(&mut state, deadline - 1)
                .unwrap();
            assert_eq!(core.taunt_source(enemy), Some(a));
            core.step_battle_execution_until(&mut state, deadline + 1)
                .unwrap();
            assert_eq!(core.taunt_source(enemy), None);
        }
    }
    #[test]
    fn compiled_taunt_equipped_range_uses_release_query_and_immune_bundle_is_atomic() {
        for equipped in [vec![], vec![0]] {
            let (mut core, mut state, a, enemy, _) = fixture(&equipped);
            place(&mut core, enemy, 4, 1);
            cast(&mut core, a, 0);
            core.step_battle_execution_until(&mut state, 1).unwrap();
            assert_eq!(
                core.taunt_source(enemy),
                (!equipped.is_empty()).then_some(a)
            );
        }
        let (mut core, mut state, a, enemy, _) = fixture(&[0, 1]);
        core.units
            .get_mut(&enemy)
            .unwrap()
            .target_traits
            .push(crate::game::battle::types::UnitTargetTrait::TauntImmune);
        cast(&mut core, a, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.taunt_source(enemy), None);
        assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    }
    #[test]
    fn compiled_taunt_damage_relations_require_equipped_relic_and_exclude_environment_reflection_corrosion(
    ) {
        let (mut core, mut state, a, enemy, ally) = fixture(&[2, 3]);
        cast(&mut core, a, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(hit(&mut core, enemy, a, DamageSource::Ability, false), 80);
        assert_eq!(
            hit(&mut core, enemy, ally, DamageSource::Ability, false),
            100
        );
        assert_eq!(
            hit(&mut core, a, enemy, DamageSource::BasicAttack, false),
            130
        );
        assert_eq!(hit(&mut core, a, enemy, DamageSource::Ability, false), 100);
        assert_eq!(
            hit(&mut core, enemy, a, DamageSource::Environment, false),
            100
        );
        assert_eq!(hit(&mut core, enemy, a, DamageSource::Ability, true), 100);
        core.units.get_mut(&enemy).unwrap().corroded = true;
        assert_eq!(hit(&mut core, enemy, a, DamageSource::Ability, false), 100);
    }
    #[test]
    fn compiled_taunt_double_harvest_only_changes_new_primary_kill_gain_and_duplicate_is_once() {
        for equipped in [vec![], vec![4], vec![4, 4]] {
            let (mut core, mut state, a, enemy, ally) = fixture(&equipped);
            cast(&mut core, a, 0);
            core.step_battle_execution_until(&mut state, 1).unwrap();
            assert_eq!(mass(&core, a), 0);
            core.units.get_mut(&enemy).unwrap().stats.current_health = 10;
            core.units.get_mut(&a).unwrap().next_basic_attack_ms = 10;
            core.process_event(
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: a,
                    target_instance_id: Some(enemy),
                    schedule_next: true,
                    cause: Default::default(),
                },
                10,
            )
            .unwrap();
            core.step_battle_execution_until(&mut state, 12).unwrap();
            assert_eq!(mass(&core, a), if equipped.is_empty() { 2 } else { 4 });
            assert_eq!(mass(&core, ally), 0);
        }
    }
    #[test]
    fn compiled_taunt_companion_catalog_binding_rejects_conflicts_without_mutating_inputs() {
        let (core, _, _, _, _) = fixture(&[]);
        let graph = compile(&taunt_fixture().to_string(), "another_taunt".into()).unwrap();
        let content = graph.load_content().unwrap();
        let skills = SkillDatabase::new(vec![]);
        let before = ron::ser::to_string(core.game_data.equipment_data.as_ref()).unwrap();
        assert!(content
            .bind_catalogs(
                &skills,
                &EquipmentDatabase::new(vec![]),
                &core.game_data.resource_data
            )
            .is_err());
        let mut equipment = (*core.game_data.equipment_data).clone();
        equipment.items[0]
            .weapon_profile
            .as_mut()
            .unwrap()
            .kill_growth
            .as_mut()
            .unwrap()
            .normal += 1;
        assert!(content
            .bind_catalogs(&skills, &equipment, &core.game_data.resource_data)
            .is_err());
        let mut resources = (*core.game_data.resource_data).clone();
        resources.resources[0].cap = 999;
        assert!(content
            .bind_catalogs(&skills, &core.game_data.equipment_data, &resources)
            .is_err());
        assert_eq!(
            ron::ser::to_string(core.game_data.equipment_data.as_ref()).unwrap(),
            before
        );
        let merged = content
            .bind_catalogs(
                &skills,
                &core.game_data.equipment_data,
                &core.game_data.resource_data,
            )
            .unwrap();
        assert_eq!(merged.resources.resources.len(), 1);
        assert_eq!(
            merged.equipment.items[0]
                .weapon_profile
                .as_ref()
                .unwrap()
                .kill_growth,
            core.game_data.equipment_data.items[0]
                .weapon_profile
                .as_ref()
                .unwrap()
                .kill_growth
        );
    }
}
