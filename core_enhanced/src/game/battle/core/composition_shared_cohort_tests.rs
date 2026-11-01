mod composition_shared_cohort {
    include!("composition_revival_tests.rs");
    include!("composition_cohort_bounty_tests.rs");
    include!("composition_equipment_forms_tests.rs");
    include!("composition_equipment_death_burst_tests.rs");
    include!("composition_last_will_tests.rs");
    use super::super::compiled_action_fixture::{fixture_content_with_data, hit, place};
    use super::*;
    use crate::game::{
        ability::hostile_cohort::*,
        data::equipment_data::{EquipmentType, WeaponRangeRole},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/shared_cohort.skill.yaml"
    ));
    const ACCESSORY: Uuid = Uuid::from_u128(0xe500);
    const SHARED_COHORT_INPUT_RELICS: &[&str] = &[
        "double-harvest", "little-red-puncture-wound", "deep-imprint",
        "little-red-slug-round", "broken-balance", "little-red-incendiary-round",
    ];
    struct Hunt {
        core: BattleCore,
        state: BattleExecutionState,
        a: UnitInstanceId,
        b: UnitInstanceId,
        enemy: UnitInstanceId,
    }
    fn setup(fragment: bool, accessory: bool) -> Hunt {
        setup_source(fragment, accessory, SOURCE)
    }
    fn setup_source(fragment: bool, accessory: bool, source: &str) -> Hunt {
        setup_source_with(fragment, accessory, source, |_| {})
    }
    fn setup_source_with(fragment: bool, accessory: bool, source: &str,
        configure: impl FnOnce(&mut BattleScenario)) -> Hunt {
        setup_source_with_graph(fragment, accessory, source, &crate::authoring::node_graph::tests::input_plan_fixture(), SHARED_COHORT_INPUT_RELICS, configure)
    }
    fn setup_source_with_graph(fragment: bool, accessory: bool, source: &str, graph: &serde_json::Value, relics: &[&str],
        configure: impl FnOnce(&mut BattleScenario)) -> Hunt {
        setup_source_with_graph_and_data(fragment, accessory, source, graph, relics, |b| b, configure)
    }
    fn setup_source_with_graph_and_data(fragment: bool, accessory: bool, source: &str, graph: &serde_json::Value, relics: &[&str],
        configure_data: impl FnOnce(crate::game::data::GameDataBuilder) -> crate::game::data::GameDataBuilder,
        configure: impl FnOnce(&mut BattleScenario)) -> Hunt {
        use crate::authoring::node_graph::{compile, PortableGraph};
        let artifact = crate::authoring::composition::compile_yaml(source).unwrap();
        let graph = compile(
            &graph.to_string(),
            "r11_aura".into(),
        )
        .unwrap();
        let mut native = PortableGraph::from_ron(&graph.to_ron().unwrap())
            .unwrap()
            .load_content()
            .unwrap();
        native.skills.skills.extend(artifact.load_native().unwrap().skills);
        let last_will_armor = artifact.equipment_defenses().iter().find(|b|
            matches!(b.program, crate::game::ability::equipment_defense::EquipmentDefenseProgram::LastWill { .. }))
            .map(|b| b.equipment.clone());
        let has_magazine = artifact
            .equipment_attacks()
            .iter()
            .any(|b| b.program.magazine.is_some());
        let mark_weapon = native.mark_weapons.first().map(|binding| binding.definition.clone());
        let captured = std::cell::RefCell::new(None);
        let (mut core, state, a, enemy, b) = fixture_content_with_data(
            relics,
            &[],
            native,
            |weapon| {
                if has_magazine {
                    weapon.weapon_profile = Some(
                        ron::de::from_str(include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/tests/fixtures/skill_composition/handcannon.weapon.ron"
                        )))
                        .unwrap(),
                    );
                }
                let p = weapon.weapon_profile.as_mut().unwrap();
                if let Some(mark_weapon) = &mark_weapon { p.mark_weapon = Some(mark_weapon.clone()); }
                p.range_role = WeaponRangeRole::Ranged;
                p.windup_ms = 200;
                p.defense_tile_range = crate::game::battle::tile_range::TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec![
                        "XXXXX".into(),
                        "XXXXX".into(),
                        "XX@XX".into(),
                        "XXXXX".into(),
                        "XXXXX".into(),
                    ],
                };
                captured.replace(Some(weapon.clone()));
            },
            ResourceCatalog { resources: artifact.resources().to_vec() },
            |builder| {
                let weapon = captured.borrow().as_ref().unwrap().clone();
                let mut item = weapon.clone();
                item.uuid = ACCESSORY;
                item.id = "r11_accessory".into();
                item.name = "공유 사냥감 출현 시험 장신구".into();
                item.equipment_type = EquipmentType::Accessory;
                item.weapon_profile = None;
                let mut equipment = vec![weapon, item];
                if let Some(id) = &last_will_armor {
                    let mut armor = equipment[1].clone();
                    armor.uuid = Uuid::from_u128(0xe540);
                    armor.id = id.clone();
                    armor.equipment_type = EquipmentType::Armor;
                    equipment.push(armor);
                }
                configure_data(builder)
                    .with_equipment(equipment)
                    .with_equipment_defenses(artifact.equipment_defenses().to_vec())
                    .with_equipment_attacks(artifact.equipment_attacks().to_vec())
            },
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(120000);
                for spawn in &mut scenario.groups[1].spawns {
                    use crate::game::resources::item_slot::{EquipmentSlotAddress, SlotLayoutKind};
                    spawn.draft.equipment_layout = Some(SlotLayoutKind::ByType);
                    spawn.draft.equipped_items[0].slot = Some(EquipmentSlotAddress::Weapon);
                    spawn.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe590 + spawn.instance_salt as u128));
                    if !fragment {
                        let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                            panic!()
                        };
                        profile.fragment_source = None;
                        profile.skill_id = None;
                    }
                    if last_will_armor.is_some() {
                        spawn.draft.equipped_items.push(crate::game::battle::types::BattleEquipmentBinding {
                            weapon_stage: Default::default(), base_uuid: Uuid::from_u128(0xe540),
                            persistent_copy_uuid: Some(Uuid::from_u128(0xe550 + spawn.instance_salt as u128)),
                            slot: Some(EquipmentSlotAddress::Armor),
                        });
                    }
                    if accessory {
                        spawn.draft.equipped_items.push(
                            crate::game::battle::types::BattleEquipmentBinding {
                                weapon_stage: Default::default(),
                                base_uuid: ACCESSORY,
                                persistent_copy_uuid: Some(Uuid::from_u128(
                                    0xe510 + spawn.instance_salt as u128,
                                )),
                                slot: Some(EquipmentSlotAddress::Accessory),
                            },
                        );
                    }
                }
                configure(scenario);
            },
        );
        for unit in core.units.values_mut() {
            unit.stats.max_health = 10000;
            unit.stats.current_health = 10000;
            unit.stats.attack_interval_ms = 100000;
            unit.next_basic_attack_ms = 100000;
            unit.basic_attack_epoch += 1;
            unit.action_locks = Default::default();
            unit.basic_attack_windup_until_ms = None;
            unit.pending_basic_attack = false;
        }
        assert!(core.units[&a].equipment_slots.is_some());
        core.enable_authoring_observation();
        Hunt {
            core,
            state,
            a,
            b,
            enemy,
        }
    }
    fn advance(f: &mut Hunt, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn owner(f: &Hunt, actor: UnitInstanceId) -> SharedCohortOwnerView {
        f.core
            .shared_cohorts_snapshot()
            .unwrap()
            .owners
            .into_iter()
            .find(|o| o.employee == f.core.units[&actor].source_owned_uuid)
            .unwrap()
    }
    fn wolf(f: &Hunt, actor: UnitInstanceId) -> UnitInstanceId {
        let spawner = owner(f, actor).id;
        f.core
            .shared_cohorts_snapshot()
            .unwrap()
            .units
            .into_iter()
            .find(|u| u.spawner == spawner)
            .unwrap()
            .unit
    }
    #[test]
    fn composition_shared_cohort_single_or_both_providers_keep_one_clock_per_employee() {
        for (fragment, accessory, providers) in
            [(true, false, 1), (false, true, 1), (true, true, 2)]
        {
            // Capture intended providers before admission/registration. Neither
            // the runtime owner list nor its single display witness is the oracle.
            let mut expected = Vec::new();
            let mut f = setup_source_with(fragment, accessory, SOURCE, |scenario| {
                for spawn in &scenario.groups[1].spawns {
                    let BattleUnitSource::Employee(profile) = &spawn.draft.source else {
                        panic!("expected employee fixture");
                    };
                    let fragment_copy = profile.fragment_source.as_ref().map(|f| f.copy_uuid);
                    let mut providers = Vec::new();
                    if let Some(copy) = fragment_copy {
                        providers.push(HostileCohortProvider::Fragment { copy });
                    }
                    for item in &spawn.draft.equipped_items {
                        if item.base_uuid == ACCESSORY {
                            providers.push(HostileCohortProvider::Equipment {
                                copy: item.persistent_copy_uuid.unwrap(),
                                definition: ACCESSORY,
                            });
                        }
                    }
                    expected.push((spawn.draft.owned_uuid, providers, fragment_copy));
                }
            });
            let matches_providers = |employee, rows: &[HostileCohortProvider]| {
                let wanted = &expected.iter().find(|(id, _, _)| *id == employee).unwrap().1;
                rows.len() == wanted.len()
                    && wanted.iter().all(|p| rows.iter().filter(|row| *row == p).count() == 1)
            };
            let before = f.core.shared_cohorts_snapshot().unwrap();
            assert_eq!(expected.len(), 2);
            for (index, (employee, _, _)) in expected.iter().enumerate() {
                let owners: Vec<_> = before.owners.iter().filter(|o| o.employee == *employee).collect();
                assert_eq!(owners.len(), 1);
                let rows = &owners[0].providers;
                assert!(matches_providers(*employee, rows));
                let mut wrong = rows.clone();
                wrong[0] = expected[1 - index].1[0];
                assert!(!matches_providers(*employee, &wrong), "other employee's copy");
                wrong = rows.clone();
                wrong.pop();
                assert!(!matches_providers(*employee, &wrong), "missing provider");
                wrong = rows.clone();
                wrong.push(wrong[0]);
                assert!(!matches_providers(*employee, &wrong), "duplicate provider");
                if accessory {
                    wrong = rows.clone();
                    let equipment = wrong.iter_mut().find(|p| matches!(p, HostileCohortProvider::Equipment { .. })).unwrap();
                    let HostileCohortProvider::Equipment { definition, .. } = equipment else { unreachable!() };
                    *definition = Uuid::nil();
                    assert!(!matches_providers(*employee, &wrong), "wrong equipment definition");
                }
            }
            assert_eq!(before.owners.len(), 2);
            assert!(before
                .owners
                .iter()
                .all(|o| o.providers.len() == providers && o.next_spawn_ms == 5000));
            assert_eq!(
                f.core.input_plans_snapshot().unwrap().owners.len(),
                if fragment { 2 } else { 0 }
            );
            advance(&mut f, 4999);
            assert!(f.core.shared_cohorts_snapshot().unwrap().units.is_empty());
            advance(&mut f, 5000);
            let after = f.core.shared_cohorts_snapshot().unwrap();
            assert_eq!(after.units.len(), 2);
            assert_eq!(after.owners.len(), expected.len());
            for owner in &after.owners {
                assert!(matches_providers(owner.employee, &owner.providers));
                let wanted = expected.iter().find(|(id, _, _)| *id == owner.employee).unwrap();
                let members: Vec<_> = after.units.iter().filter(|u| u.spawner == owner.id).collect();
                assert_eq!(members.len(), 1);
                assert_eq!(members[0].fragment_recipient, wanted.2);
            }
            assert!(after
                .units
                .iter()
                .all(|u| u.fragment_recipient.is_some() == fragment));
            assert!(after.owners.iter().all(|o| o.next_spawn_ms == 10000));
            let wa = wolf(&f, f.a);
            let wb = wolf(&f, f.b);
            assert_ne!(wa, wb);
            assert_eq!(f.core.units[&wa].owner, Side::Opponent);
            assert_eq!(f.core.units[&wa].stats.current_health, 300);
            let observation = f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_shared_cohort
                .as_ref()
                .unwrap();
            assert_eq!(observation.initial_hp, 300);
            assert_eq!(observation.provider_count, providers);
            let wanted = expected.iter().find(|(id, _, _)| *id == observation.employee).unwrap();
            let equipment_copy = wanted.1.iter().find_map(|p| match p {
                HostileCohortProvider::Equipment { copy, .. } => Some(*copy),
                HostileCohortProvider::Fragment { .. } => None,
            });
            assert_eq!(observation.equipment_copy, equipment_copy);
            assert_eq!(observation.fragment_recipient, wanted.2);
            assert_eq!(observation.time_ms, 5000);
            assert_eq!(observation.next_spawn_ms, 10000);
            if !fragment {
                assert!(f
                    .core
                    .input_plans_snapshot()
                    .unwrap()
                    .cohort_units
                    .is_empty());
            }
        }
    }
    #[test]
    fn composition_shared_cohort_ally_kill_credits_original_real_fragment_only() {
        for fragment in [false, true] {
            let mut f = setup(fragment, true);
            advance(&mut f, 5000);
            let target = wolf(&f, f.a);
            let killer = f.b;
            hit(&mut f.core, killer, target, 1000, 5000);
            assert_eq!(f.core.shared_cohorts_snapshot().unwrap().units.len(), 1);
            let stocks = f.core.input_plans_snapshot().unwrap();
            if fragment {
                let employee = f.core.units[&f.a].source_owned_uuid;
                assert_eq!(
                    stocks
                        .owners
                        .iter()
                        .find(|o| o.employee == employee)
                        .unwrap()
                        .enhanced,
                    1
                );
                assert_eq!(
                    stocks
                        .owners
                        .iter()
                        .find(|o| o.employee != employee)
                        .unwrap()
                        .enhanced,
                    0
                );
            } else {
                assert!(stocks.owners.is_empty());
            }
        }
    }
    #[test]
    fn composition_shared_cohort_actual_accessory_priority_keeps_range_and_spawn_ties() {
        let mut f = setup(false, true);
        advance(&mut f, 5000);
        let wa = wolf(&f, f.a);
        let wb = wolf(&f, f.b);
        place(&mut f.core, f.a, 2, 1);
        place(&mut f.core, f.enemy, 3, 1);
        place(&mut f.core, wa, 1, 1);
        place(&mut f.core, wb, 1, 1);
        let first = f.core.choose_attack_target_in_range(f.a).unwrap();
        let expected = if f.core.units[&wa].spawn_order < f.core.units[&wb].spawn_order {
            wa
        } else {
            wb
        };
        assert_eq!(first, expected);
        assert_eq!(
            f.core
                .select_basic_attack_target(f.a, Some(f.enemy), Some(f.enemy)),
            Some(first)
        );
        assert!(f.core.equipment_prioritizes_cohort(f.a, first));
        assert!(!f.core.equipment_prioritizes_cohort(f.a, f.enemy));
        // Move both actual cohorts beyond the authored weapon range.
        place(&mut f.core, wa, 7, 1);
        place(&mut f.core, wb, 7, 2);
        assert_eq!(f.core.choose_attack_target_in_range(f.a), Some(f.enemy));
    }
    #[test]
    fn composition_shared_cohort_conflicting_provider_profile_is_rejected_atomically() {
        let donor = setup(true, false);
        let fragment = donor.core.units[&donor.a]
            .fragment_source
            .as_ref()
            .unwrap()
            .clone();
        let mut f = setup_source(
            false,
            true,
            &SOURCE.replace("interval_ms: 5000", "interval_ms: 6000"),
        );
        let before = f.core.shared_cohorts_snapshot().unwrap();
        let stocks = f.core.input_plans_snapshot().unwrap();
        let employee = f.core.units[&f.a].source_owned_uuid;
        assert_eq!(employee, donor.core.units[&donor.a].source_owned_uuid);
        let result = f
            .core
            .runtime_transaction(|core| core.register_input_plan_copy(employee, fragment, 0));
        assert!(
            matches!(result, Err(GameError::InvalidStaticData(ref message)) if message.contains("conflicting shared hostile cohort providers")),
            "{result:?}"
        );
        assert_eq!(f.core.shared_cohorts_snapshot().unwrap(), before);
        assert_eq!(f.core.input_plans_snapshot().unwrap(), stocks);
        advance(&mut f, 6000);
        assert_eq!(f.core.shared_cohorts_snapshot().unwrap().units.len(), 2);
    }
    #[test]
    fn composition_shared_cohort_authored_roundtrip_and_native_boundaries() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert_eq!(
            a.equipment_attacks()[0]
                .program
                .cohort
                .as_ref()
                .unwrap()
                .spawn
                .hp,
            300
        );
        assert!(a.sources().get("equipment_attacks[0].program").is_some());
        for (old, new) in [
            ("interval_ms: 5000", "interval_ms: 0"),
            ("hp: 300", "hp: 0"),
            ("denominator: 1", "denominator: 0"),
            ("damage_type: Physical", "damage_type: True"),
        ] {
            let error = crate::authoring::composition::compile_yaml(&SOURCE.replacen(old, new, 1))
                .unwrap_err();
            assert!(
                error
                    .source
                    .definition
                    .contains("AttackProgram.cohort.spawn"),
                "{error:?}"
            );
        }
    }
    #[test]
    fn composition_shared_cohort_withdraw_keeps_clock_and_real_hostile_attack() {
        let mut f = setup(false, true);
        let before = owner(&f, f.a);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 0,
            })
            .unwrap();
        advance(&mut f, 5000);
        let snapshot = f.core.shared_cohorts_snapshot().unwrap();
        assert_eq!(snapshot.owners.len(), 2);
        assert_eq!(
            snapshot
                .owners
                .iter()
                .find(|o| o.id == before.id)
                .unwrap()
                .next_spawn_ms,
            10000
        );
        assert_eq!(snapshot.units.len(), 2);
        assert!(f.core.input_plans_snapshot().unwrap().owners.is_empty());
        place(&mut f.core, f.b, 1, 1);
        f.core.units.get_mut(&f.b).unwrap().block_capacity = 2;
        advance(&mut f, 7000);
        assert!(f.core.units[&f.b].stats.current_health < 10000);
    }
    #[test]
    fn composition_shared_cohort_arrival_changes_next_attack_without_retargeting_windup() {
        let mut f = setup(false, true);
        // The shared fixture has already stepped through time zero. Changing
        // readiness alone does not replace its queued/pending initial attack.
        let actor = f.core.units.get_mut(&f.a).unwrap();
        actor.stats.attack_interval_ms = 1000;
        assert_eq!(actor.stats.attack, 100);
        assert_eq!(actor.basic_attack.windup_ms, 200);
        f.core.event_queue.discard_channel_periodic_starts(f.a).unwrap();
        f.core.schedule_initial_attack_for_unit(f.a, 4900);
        assert_eq!(f.core.units[&f.a].next_basic_attack_ms, 4900);
        assert!(f.core.event_queue.iter().any(|event| matches!(event.as_ref(),
            BattleEvent::AttackStart { time_ms: 4900, attacker_instance_id,
                schedule_next: true, .. } if *attacker_instance_id == f.a)));
        advance(&mut f, 4900);
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, Some(5100));
        assert_eq!(f.core.units[&f.a].current_target, Some(f.enemy));
        assert!(f.core.shared_cohorts_snapshot().unwrap().units.is_empty());
        advance(&mut f, 5100);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        let members = f.core.shared_cohorts_snapshot().unwrap().units;
        assert_eq!(members.len(), 2);
        assert!(members
            .iter()
            .all(|m| f.core.units[&m.unit].stats.current_health == 300));
        assert_eq!(f.core.units[&f.a].next_basic_attack_ms, 5900);
        advance(&mut f, 5900);
        let target = f.core.units[&f.a].current_target.unwrap();
        assert!(members.iter().any(|m| m.unit == target));
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, Some(6100));
        advance(&mut f, 6100);
        assert_eq!(f.core.units[&target].stats.current_health, 200);
        assert!(members
            .iter()
            .filter(|m| m.unit != target)
            .all(|m| f.core.units[&m.unit].stats.current_health == 300));
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
    }
    #[test]
    fn composition_shared_cohort_real_population_delays_victory_and_terminal_clears_clock() {
        let mut f = setup(false, true);
        Arc::make_mut(&mut f.core.scenario).terminal_outcomes =
            BattleScenario::fixture_required_group_outcomes();
        advance(&mut f, 5000);
        hit(&mut f.core, f.a, f.enemy, 20000, 5000);
        advance(&mut f, 5001);
        assert!(!f.state.is_finished());
        let members = f.core.shared_cohorts_snapshot().unwrap().units;
        for m in members {
            hit(&mut f.core, f.a, m.unit, 1000, 5001);
        }
        advance(&mut f, 5002);
        assert!(f.state.is_finished());
        f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert_eq!(
            f.core.shared_cohorts_snapshot().unwrap(),
            SharedCohortsSnapshot::default()
        );
    }
    #[test]
    fn composition_shared_cohort_redeployment_reuses_persistent_provider_and_clock() {
        let mut f = setup(false, true);
        let before = owner(&f, f.a);
        let draft = f
            .core
            .scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .find(|s| s.draft.owned_uuid == before.employee)
            .unwrap()
            .draft
            .clone();
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 900,
            })
            .unwrap();
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: new } =
            f.core
                .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                    draft,
                    position: crate::game::resources::Position::new(2, 1),
                    tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                    facing: crate::game::battle::tile_range::FacingDirection::Right,
                    instance_salt: 4242,
                    time_ms: 900,
                    current_hp_policy: None,
                })
                .unwrap()
        else {
            panic!()
        };
        assert_ne!(new, f.a);
        f.core
            .runtime_transaction(|c| c.reconcile_equipment_cohorts(900))
            .unwrap();
        assert_eq!(owner(&f, new), before);
        advance(&mut f, 5000);
        assert_eq!(f.core.shared_cohorts_snapshot().unwrap().units.len(), 2);
    }
    #[test]
    fn composition_shared_cohort_accessory_only_kill_reloads_actual_handcannon_without_fragment_stock(
    ) {
        let magazine = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/magazine.skill.yaml"
        ));
        let values = magazine
            .split_once("values:\n")
            .unwrap()
            .1
            .split_once("equipment_attacks:\n")
            .unwrap()
            .0;
        let binding = magazine
            .split_once("equipment_attacks:\n")
            .unwrap()
            .1
            .split_once("definitions:")
            .unwrap()
            .0;
        let source = SOURCE.replace(
            "equipment_attacks:\n",
            &format!("{values}equipment_attacks:\n{binding}"),
        );
        let mut f = setup_source(false, true, &source);
        advance(&mut f, 5000);
        let target = wolf(&f, f.b);
        hit(&mut f.core, f.a, target, 1000, 5000);
        let m = f
            .core
            .equipment_attack_state
            .magazines
            .values()
            .find(|m| m.owner == f.a)
            .unwrap();
        assert_eq!(m.remaining, 6);
        assert!(m.enhanced);
        assert!(m.equipment.persistent_copy_uuid.is_some());
        assert!(f.core.input_plans_snapshot().unwrap().owners.is_empty());
        assert_eq!(f.core.shared_cohorts_snapshot().unwrap().units.len(), 1);
    }
    #[test]
    fn composition_shared_cohort_public_cancel_restores_spawn_ids_queue_and_shared_owner() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(false, true);
        advance(&mut f, 4999);
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
            f.core.shared_cohorts_snapshot().unwrap(),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("input_plan_hostile_spawn", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 5000),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state),
                f.core.shared_cohorts_snapshot().unwrap()
            )
        );
        advance(&mut f, 5000);
        assert_eq!(f.core.shared_cohorts_snapshot().unwrap().units.len(), 2);
    }
    #[test]
    fn composition_shared_cohort_retained_failure_restores_late_spawn_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(false, true);
        advance(&mut f, 4999);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 5000)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "shared_cohort_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
            f.core.shared_cohorts_snapshot().unwrap(),
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
        );
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 5000)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "shared_cohort_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state),
                f.core.shared_cohorts_snapshot().unwrap(),
                serde_json::to_value(&f.core.authoring_observation).unwrap()
            )
        );
        f.core.runtime_facts.limits = Default::default();
        advance(&mut f, 5000);
        assert_eq!(f.core.shared_cohorts_snapshot().unwrap().units.len(), 2);
    }
    #[test]
    fn composition_shared_cohort_observation_and_clone_do_not_change_population() {
        let mut f = setup(false, true);
        let mut core = f.core.clone();
        let mut state = f.state.clone();
        core.authoring_observation = None;
        advance(&mut f, 5000);
        core.step_battle_execution_until(&mut state, 5000).unwrap();
        assert_eq!(
            f.core.shared_cohorts_snapshot().unwrap(),
            core.shared_cohorts_snapshot().unwrap()
        );
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            core.event_queue_debug_snapshot()
        );
        for m in f.core.shared_cohorts_snapshot().unwrap().units {
            assert_eq!(
                f.core.units[&m.unit].stats.current_health,
                core.units[&m.unit].stats.current_health
            );
        }
    }
}
