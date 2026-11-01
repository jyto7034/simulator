mod composition_magazine {
    use super::*;
    use crate::game::{
        battle::{
            core::equipment_magazine::MagazineOperation,
            damage::{BattleCommand, DamageSource, DamageType},
        },
        data::equipment_data::{WeaponArchetype, WeaponRangeRole},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/magazine.skill.yaml"
    ));
    struct GunFixture {
        core: BattleCore,
        state: BattleExecutionState,
        a: UnitInstanceId,
        b: UnitInstanceId,
        enemy: UnitInstanceId,
    }
    fn setup_source(source: &str) -> GunFixture {
        setup_source_profile(source, false)
    }
    fn setup_source_profile(source: &str, area: bool) -> GunFixture {
        use crate::authoring::node_graph::{compile, PortableGraph};
        let artifact = crate::authoring::composition::compile_yaml(source).unwrap();
        let graph_input = if area {
            crate::authoring::node_graph::tests::scheduled_area_fixture()
        } else {
            crate::authoring::node_graph::tests::input_plan_fixture()
        };
        let graph = compile(
            &graph_input.to_string(),
            "r11_aura".into(),
        )
        .unwrap();
        let native = PortableGraph::from_ron(&graph.to_ron().unwrap())
            .unwrap()
            .load_content()
            .unwrap();
        let relics = [
            "double-harvest",
            "little-red-puncture-wound",
            "deep-imprint",
            "little-red-slug-round",
            "broken-balance",
            "little-red-incendiary-round",
        ];
        let (mut core, state, a, enemy, b) =
            super::super::compiled_action_fixture::fixture_content_with_data(
                &relics,
                &[],
                native,
                |w| {
                    w.content_family = Some("little-red".into());
                    let p = w.weapon_profile.as_mut().unwrap();
                    p.range_role = WeaponRangeRole::Ranged;
                    p.weapon_archetype = WeaponArchetype::Gun;
                    p.air_capable = true;
                    p.windup_ms = 200;
                    p.defense_tile_range = crate::game::battle::tile_range::TileRangePattern {
                        include_anchor_tile: true,
                        rows: (0..13)
                            .map(|y| {
                                if y == 6 {
                                    "XXXXXX@XXXXXX".into()
                                } else {
                                    "XXXXXXXXXXXXX".into()
                                }
                            })
                            .collect(),
                    };
                    p.delivery = ron::de::from_str("Projectile(speed_units_per_ms:10000,hit_policy:DirectionalCollision,max_range_tiles:Some(6),max_lifetime_ms:Some(1500),collision:(radius_units:125000,piercing:false))").unwrap();
                    if area {
                        p.action = crate::game::data::abnormality_data::BasicAttackAction::DamageArea {
                            footprint: crate::game::battle::tile_range::TileRangePattern {
                                include_anchor_tile: true, rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                            }, secondary_percent: 50,
                        };
                        p.delivery = ron::de::from_str("Projectile(speed_units_per_ms:10000)").unwrap();
                    }
                },
                ResourceCatalog { resources: vec![] },
                |b| b.with_equipment_attacks(artifact.equipment_attacks().to_vec())
                    .with_equipment_states(artifact.equipment_states().to_vec()),
                |scenario| {
                    for spawn in &mut scenario.groups[1].spawns {
                        spawn.draft.equipped_items[0].persistent_copy_uuid =
                            Some(Uuid::from_u128(0xe490 + spawn.instance_salt as u128));
                    }
                },
            );
        Arc::make_mut(&mut core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(120000);
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
        // Keep this actual fragment provider, but isolate ordinary magazine tests
        // before the first cohort tick. Cohort tests intentionally advance to it.
        core.units.get_mut(&b).unwrap().fragment_source = None;
        super::super::compiled_action_fixture::place(&mut core, a, 2, 1);
        super::super::compiled_action_fixture::place(&mut core, enemy, 4, 1);
        super::super::compiled_action_fixture::place(&mut core, b, 7, 3);
        core.enable_authoring_observation();
        GunFixture {
            core,
            state,
            a,
            b,
            enemy,
        }
    }
    fn setup() -> GunFixture {
        setup_source(SOURCE)
    }
    fn copy(
        f: &GunFixture,
        actor: UnitInstanceId,
    ) -> crate::game::battle::core::equipment_magazine::MagazineKey {
        use crate::game::battle::core::equipment_magazine::MagazineKey;
        f.core
            .items
            .values()
            .find(|i| i.owner_unit_instance == actor)
            .map(|i| {
                MagazineKey::for_equipment(
                    f.core.units[&actor].source_owned_uuid,
                    i.source_snapshot(),
                )
            })
            .or_else(|| {
                f.core
                    .equipment_attack_state
                    .magazines
                    .iter()
                    .find(|(_, m)| m.owner == actor)
                    .map(|(k, _)| *k)
            })
            .unwrap()
    }
    fn rounds(f: &GunFixture, actor: UnitInstanceId) -> u32 {
        f.core
            .equipment_attack_state
            .magazines
            .get(&copy(f, actor))
            .map_or(6, |m| m.remaining)
    }
    fn advance(f: &mut GunFixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn start(f: &mut GunFixture, now: u64, regular: bool) {
        if regular {
            f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = now;
        }
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: now,
                    attacker_instance_id: f.a,
                    target_instance_id: Some(f.enemy),
                    schedule_next: regular,
                    cause: Default::default(),
                },
                now,
            )
            .unwrap();
    }
    fn reward(f: &mut GunFixture, now: u64) {
        let a = f.a;
        f.core
            .runtime_transaction(|c| c.reward_cohort_magazine(a, "little-red", now))
            .unwrap();
    }
    fn shot(f: &mut GunFixture, now: u64) {
        start(f, now, false);
        advance(f, now + 800);
    }
    fn witness(f: &GunFixture) -> &crate::game::battle::core::equipment_magazine::MagazineWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_magazine
            .as_ref()
            .unwrap()
    }
    #[test]
    fn composition_magazine_authored_roundtrip_bounds_and_source() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert_eq!(
            a.equipment_attacks()[0]
                .program
                .magazine
                .as_ref()
                .unwrap()
                .capacity,
            6
        );
        assert!(a.sources().get("equipment_attacks[0].program").is_some());
        for (key, value) in [
            ("capacity", 0),
            ("capacity", 1001),
            ("reload_ms", 0),
            ("normal_percent", 0),
            ("enhanced_primary_percent", 10001),
            ("enhanced_max_hits", 0),
            ("enhanced_max_hits", 256),
        ] {
            let mut d = parse_yaml(SOURCE).unwrap();
            d["values"][0]["value"]["AttackProgram"]["magazine"][key] = json!(value);
            assert!(compile_json(&d.to_string()).is_err(), "{key}={value}");
        }
        let f = setup();
        assert!(f
            .core
            .game_data
            .equipment_attack_catalog()
            .by_equipment
            .values()
            .any(|p| p.magazine.is_some()));
    }
    #[test]
    fn composition_magazine_six_rounds_empty_and_exact_reload_boundary() {
        let mut f = setup();
        for i in 0..6 {
            shot(&mut f, 1 + i * 1000);
            assert_eq!(rounds(&f, f.a), 5 - i as u32);
        }
        let due = 5201 + 3000;
        let key = copy(&f, f.a);
        assert_eq!(
            f.core.equipment_attack_state.magazines[&key].reload_at,
            Some(due)
        );
        start(&mut f, 6001, false);
        advance(&mut f, 6801);
        assert_eq!(rounds(&f, f.a), 0);
        advance(&mut f, due - 1);
        assert_eq!(rounds(&f, f.a), 0);
        advance(&mut f, due);
        assert_eq!(rounds(&f, f.a), 6);
        assert!(matches!(witness(&f).operation, MagazineOperation::Reloaded));
        assert!(!witness(&f).enhanced);
    }
    #[test]
    fn composition_magazine_enhanced_first_collision_and_rear_hit_once_each() {
        for enhanced in [false, true] {
            let mut f = setup();
            f.core.units.get_mut(&f.b).unwrap().owner = Side::Opponent;
            super::super::compiled_action_fixture::place(&mut f.core, f.b, 6, 1);
            if enhanced {
                reward(&mut f, 0);
            }
            shot(&mut f, 1);
            assert_eq!(
                f.core.units[&f.enemy].stats.current_health,
                if enhanced { 9850 } else { 9900 }
            );
            assert_eq!(
                f.core.units[&f.b].stats.current_health,
                if enhanced { 9900 } else { 10000 }
            );
            assert!(f.core.projectiles.is_empty());
            assert_eq!(rounds(&f, f.a), 5);
        }
    }
    #[test]
    fn composition_magazine_reward_replaces_and_stale_reload_cannot_downgrade() {
        let mut f = setup_source(&SOURCE.replace("capacity: 6", "capacity: 1"));
        shot(&mut f, 1);
        assert_eq!(rounds(&f, f.a), 0);
        reward(&mut f, 1000);
        assert_eq!(rounds(&f, f.a), 1);
        assert!(witness(&f).enhanced);
        reward(&mut f, 1100);
        assert_eq!(rounds(&f, f.a), 1);
        advance(&mut f, 3201);
        assert_eq!(rounds(&f, f.a), 1);
        assert!(f.core.equipment_attack_state.magazines[&copy(&f, f.a)].enhanced);
        shot(&mut f, 3300);
        advance(&mut f, 6500);
        assert_eq!(rounds(&f, f.a), 1);
        assert!(!f.core.equipment_attack_state.magazines[&copy(&f, f.a)].enhanced);
    }
    #[test]
    fn composition_magazine_round_is_chosen_at_release_and_released_carrier_is_immutable() {
        let mut f = setup();
        start(&mut f, 1, false);
        assert_eq!(rounds(&f, f.a), 6);
        reward(&mut f, 100);
        advance(&mut f, 201);
        assert_eq!(rounds(&f, f.a), 5);
        assert!(f.core.projectiles.values().all(|p| p
            .source_snapshot
            .magazine
            .as_ref()
            .unwrap()
            .enhanced));
        for p in f.core.projectiles.values() {
            assert_eq!(
                p.source_snapshot.fact_source.equipment,
                Some(p.source_snapshot.magazine.as_ref().unwrap().equipment)
            );
        }
        reward(&mut f, 220);
        advance(&mut f, 801);
        assert_eq!(rounds(&f, f.a), 6);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9850);
        let mut normal = setup();
        start(&mut normal, 1, false);
        advance(&mut normal, 201);
        reward(&mut normal, 220);
        advance(&mut normal, 801);
        assert_eq!(normal.core.units[&normal.enemy].stats.current_health, 9900);
    }
    #[test]
    fn composition_magazine_target_loss_before_release_does_not_spend() {
        let mut f = setup();
        start(&mut f, 1, false);
        f.core.units.get_mut(&f.enemy).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        advance(&mut f, 201);
        assert_eq!(rounds(&f, f.a), 6);
        assert!(f.core.projectiles.is_empty());
    }
    #[test]
    fn composition_magazine_corrosion_preserves_mechanical_ammo_but_denies_reward() {
        let mut f = setup();
        shot(&mut f, 1);
        assert_eq!(rounds(&f, f.a), 5);
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        reward(&mut f, 900);
        assert_eq!(rounds(&f, f.a), 5);
        shot(&mut f, 1001);
        assert_eq!(rounds(&f, f.a), 4);
    }
    #[test]
    fn composition_magazine_withdraw_keeps_reload_death_unequip_and_terminal_clear() {
        let mut f = setup_source(&SOURCE.replace("capacity: 6", "capacity: 1"));
        shot(&mut f, 1);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 900,
            })
            .unwrap();
        advance(&mut f, 3201);
        assert_eq!(rounds(&f, f.a), 1);
        let key = copy(&f, f.a);
        assert_eq!(f.core.equipment_attack_state.magazines[&key].remaining, 1);
        let mut unequipped = setup();
        shot(&mut unequipped, 1);
        let removed_key = copy(&unequipped, unequipped.a);
        let retained_key = copy(&unequipped, unequipped.b);
        unequipped
            .core
            .items
            .retain(|_, i| i.owner_unit_instance != unequipped.a);
        unequipped
            .core
            .runtime_transaction(|c| c.reconcile_equipment_magazines())
            .unwrap();
        assert!(!unequipped
            .core
            .equipment_attack_state
            .magazines
            .contains_key(&removed_key));
        assert!(unequipped
            .core
            .equipment_attack_state
            .magazines
            .contains_key(&retained_key));
        let mut dead = setup();
        shot(&mut dead, 1);
        let removed_key = copy(&dead, dead.a);
        dead.core.units.get_mut(&dead.a).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        dead.core
            .runtime_transaction(|c| c.reconcile_equipment_magazines())
            .unwrap();
        assert!(!dead
            .core
            .equipment_attack_state
            .magazines
            .contains_key(&removed_key));
        let mut terminal = setup();
        shot(&mut terminal, 1);
        terminal
            .core
            .runtime_transaction(|c| c.clear_sustained(900))
            .unwrap();
        assert!(terminal.core.equipment_attack_state.magazines.is_empty());
    }
    #[test]
    fn composition_magazine_actual_redeployment_keeps_copy_ammo_and_reload_clock() {
        let mut f = setup();
        shot(&mut f, 1);
        let key = copy(&f, f.a);
        let employee = f.core.units[&f.a].source_owned_uuid;
        let draft = f
            .core
            .scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .find(|s| s.draft.owned_uuid == employee)
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
        f.core
            .runtime_transaction(|c| c.reconcile_equipment_magazines())
            .unwrap();
        assert_ne!(new, f.a);
        assert_eq!(copy(&f, new), key);
        assert_eq!(rounds(&f, new), 5);
        assert_eq!(f.core.equipment_attack_state.magazines[&key].owner, new);
    }
    #[test]
    fn composition_magazine_actual_hostile_cohort_direct_kill_keeps_fragment_reward_separate() {
        let mut f = setup();
        advance(&mut f, 5000);
        let before = f.core.input_plans_snapshot().unwrap();
        let (wolf, owner) = before.cohort_units[0];
        let stock = before
            .owners
            .iter()
            .find(|o| o.copy == owner)
            .unwrap()
            .enhanced;
        let a = f.a;
        f.core
            .runtime_transaction(|c| {
                let source = c
                    .damage_source_snapshot_for_unit(
                        a,
                        wolf,
                        DamageSource::Ability,
                        DamageType::True,
                        10000,
                        Default::default(),
                        0,
                        5000,
                        false,
                    )
                    .unwrap();
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: wolf,
                        source_snapshot: source,
                    }],
                    5000,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert!(f.core.units[&wolf].lifecycle.is_dead());
        assert_eq!(rounds(&f, f.a), 6);
        assert!(witness(&f).enhanced);
        assert!(!f
            .core
            .equipment_attack_state
            .magazines
            .get(&copy(&f, f.b))
            .is_some_and(|m| m.enhanced));
        assert_eq!(
            f.core
                .input_plans_snapshot()
                .unwrap()
                .owners
                .iter()
                .find(|o| o.copy == owner)
                .unwrap()
                .enhanced,
            stock + 1
        );
    }
    #[test]
    fn composition_magazine_public_cancel_restores_full_release() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        start(&mut f, 1, false);
        advance(&mut f, 200);
        let snap = |f: &GunFixture| {
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state),
            )
        };
        let before = snap(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("magazine_fired", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 201),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snap(&f));
        f.core
            .step_battle_execution_until(&mut f.state, 201)
            .unwrap();
        assert_eq!(rounds(&f, f.a), 5);
    }
    #[test]
    fn composition_magazine_public_retained_failure_restores_release_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        start(&mut f, 1, false);
        advance(&mut f, 200);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 201)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "magazine_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
        );
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 201)
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "magazine_observation",
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
                serde_json::to_value(&f.core.authoring_observation).unwrap()
            )
        );
        f.core.runtime_facts.limits = Default::default();
        advance(&mut f, 201);
        assert_eq!(rounds(&f, f.a), 5);
    }
    #[test]
    fn composition_magazine_automatic_wait_reward_wake_and_old_timer_are_single_owner() {
        let mut f = setup_source(&SOURCE.replace("capacity: 6", "capacity: 1"));
        f.core.units.get_mut(&f.a).unwrap().stats.attack_interval_ms = 1000;
        start(&mut f, 1, true);
        advance(&mut f, 1201);
        let key = copy(&f, f.a);
        assert_eq!(rounds(&f, f.a), 0);
        assert!(f.core.equipment_attack_state.magazines[&key].waiting);
        reward(&mut f, 1500);
        advance(&mut f, 1700);
        assert_eq!(rounds(&f, f.a), 0);
        assert_eq!(
            f.core.equipment_attack_state.magazines[&key].reload_at,
            Some(4700)
        );
        advance(&mut f, 3201);
        assert_eq!(
            f.core.equipment_attack_state.magazines[&key].reload_at,
            Some(4700)
        );
        assert_eq!(rounds(&f, f.a), 0);
    }
    #[test]
    fn composition_magazine_ordinary_kill_and_wrong_family_do_not_reload() {
        let mut f = setup();
        shot(&mut f, 1);
        let a = f.a;
        let enemy = f.enemy;
        f.core
            .runtime_transaction(|c| {
                c.reward_cohort_magazine(a, "unrelated", 900)?;
                let source = c
                    .damage_source_snapshot_for_unit(
                        a,
                        enemy,
                        DamageSource::Ability,
                        DamageType::True,
                        10000,
                        Default::default(),
                        0,
                        900,
                        false,
                    )
                    .unwrap();
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: enemy,
                        source_snapshot: source,
                    }],
                    900,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(rounds(&f, f.a), 5);
        assert!(!f.core.equipment_attack_state.magazines[&copy(&f, f.a)].enhanced);
    }
    #[test]
    fn composition_magazine_observer_is_passive_and_cloned_carriers_keep_ammunition() {
        let mut f = setup();
        reward(&mut f, 0);
        start(&mut f, 1, false);
        advance(&mut f, 201);
        let mut c = f.core.clone();
        let mut state = f.state.clone();
        c.authoring_observation = None;
        advance(&mut f, 801);
        c.step_battle_execution_until(&mut state, 801).unwrap();
        assert_eq!(
            f.core.units[&f.enemy].stats.current_health,
            c.units[&f.enemy].stats.current_health
        );
        assert_eq!(
            format!("{:?}", f.core.equipment_attack_state),
            format!("{:?}", c.equipment_attack_state)
        );
        assert_eq!(f.core.projectiles.len(), c.projectiles.len());
    }
    include!("composition_plain_magazine_tests.inc.rs");
}
