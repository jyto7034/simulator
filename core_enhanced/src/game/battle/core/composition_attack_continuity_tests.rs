mod composition_attack_continuity {
    use super::*;
    use crate::game::battle::{
        damage::{BattleCommand, DamageSource, DamageType},
        runtime_control,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/attack_continuity.skill.yaml"
    ));
    fn setup(rate: u32, cap: u32) -> Fixture {
        setup_with_armor_provider(rate, cap, false)
    }
    fn setup_with_armor_provider(rate: u32, cap: u32, armor_provider: bool) -> Fixture {
        use crate::game::{
            battle::types::BattleEquipmentBinding,
            data::equipment_data::EquipmentType,
            resources::item_slot::{EquipmentSlotAddress as Slot, SlotLayoutKind},
        };
        // The released effect belongs to an accessory. A distinct test-only
        // Armor provider exercises shared non-stacking selection in legal
        // Weapon/Armor/Accessory slots; it is not another released content item.
        let mut f = fixture_copies_with_equipment(0, false, false, |doc| {
            *doc = parse_yaml(SOURCE).unwrap();
            program(doc)["continuity"]["per_second_percent"] = json!(rate);
            program(doc)["continuity"]["max_percent"] = json!(cap);
            doc["equipment_attacks"].as_array_mut().unwrap().push(json!({
                "equipment":"continuity_fixture_armor", "program":{"Shared":"attack-clock"}
            }));
        }, |_| {}, |scenario| {
            for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                spawn.draft.equipment_layout = Some(SlotLayoutKind::ByType);
                spawn.draft.equipped_items[0].slot = Some(Slot::Weapon);
                spawn.draft.equipped_items[0].persistent_copy_uuid = Some(Uuid::from_u128(0xed00 + i as u128));
                spawn.draft.equipped_items.push(BattleEquipmentBinding {
                    weapon_stage: Default::default(), base_uuid: Uuid::from_u128(0x1102),
                    persistent_copy_uuid: Some(Uuid::from_u128(0xe100 + i as u128)), slot: Some(Slot::Accessory),
                });
                if armor_provider {
                    spawn.draft.equipped_items.push(BattleEquipmentBinding {
                        weapon_stage: Default::default(), base_uuid: Uuid::from_u128(0x1103),
                        persistent_copy_uuid: Some(Uuid::from_u128(0xe102 + i as u128)), slot: Some(Slot::Armor),
                    });
                }
            }
        }, None, &[], |items| {
            for (id, uuid, kind) in [
                ("placeholder_schadenfreude_accessory", 0x1102, EquipmentType::Accessory),
                ("continuity_fixture_armor", 0x1103, EquipmentType::Armor),
            ] {
                let mut item = items[0].clone();
                item.id = id.into(); item.uuid = Uuid::from_u128(uuid);
                item.name = id.into(); item.equipment_type = kind; item.weapon_profile = None;
                items.push(item);
            }
        });
        if armor_provider {
            let removed = Arc::make_mut(f.core.units.get_mut(&f.a).unwrap().equipment_slots.as_mut().unwrap()).slots[1].take().unwrap();
            f.core.items.remove(&removed.runtime_item_instance_id);
        }
        for u in f.core.units.values_mut() {
            u.stats.current_health = 100_000;
            u.stats.max_health = 100_000;
            u.stats.magic_resist = 0;
        }
        // The shared fixture has already advanced its initial event bucket.
        // Begin these isolated clock scenarios after that setup activity.
        f.core.equipment_attack_state.continuity.clear();
        f.core.enable_authoring_observation();
        f.core
            .bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0x9200),
                },
            )
            .unwrap();
        f
    }
    fn move_armor_provider(f: &mut Fixture) -> Uuid {
        // Component lifetime injection only: preserve a representable loadout.
        // Actual game equipment changes still belong to the World equip path.
        let slot = Arc::make_mut(f.core.units.get_mut(&f.b).unwrap().equipment_slots.as_mut().unwrap()).slots[1].take().unwrap();
        assert_eq!(slot.binding.base_uuid, Uuid::from_u128(0x1103));
        let slots = Arc::make_mut(f.core.units.get_mut(&f.a).unwrap().equipment_slots.as_mut().unwrap());
        assert!(slots.slots[1].is_none());
        slots.slots[1] = Some(slot);
        f.core.items.get_mut(&slot.runtime_item_instance_id).unwrap().owner_unit_instance = f.a;
        for actor in [f.a, f.b] {
            let weapon = f.core.basic_attack_weapon_source(actor).unwrap().unwrap();
            assert_eq!(weapon.definition_uuid, Uuid::from_u128(0x1101));
            assert_eq!(f.core.items.values().filter(|i| i.owner_unit_instance == actor && i.base_uuid == Uuid::from_u128(0x1101)).count(), 1);
        }
        slot.runtime_item_instance_id
    }
    fn hit(
        f: &mut Fixture,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        kind: DamageType,
        source: DamageSource,
        now: u64,
        reaction: bool,
    ) -> u32 {
        let before = f.core.units[&target].stats.current_health;
        f.core
            .runtime_transaction(|c| {
                let mut s = c
                    .damage_source_snapshot_for_unit(
                        actor,
                        target,
                        source,
                        kind,
                        100,
                        Default::default(),
                        0,
                        now,
                        false,
                    )
                    .unwrap();
                s.fact_source.reaction = reaction;
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: target,
                        source_snapshot: s,
                    }],
                    now,
                );
                runtime_control::poll()
            })
            .unwrap();
        before - f.core.units[&target].stats.current_health
    }
    fn damage(f: &mut Fixture, now: u64) -> u32 {
        hit(
            f,
            f.a,
            f.enemy,
            DamageType::Physical,
            DamageSource::Ability,
            now,
            false,
        )
    }
    fn witness(
        f: &Fixture,
    ) -> &crate::game::battle::core::attack_continuity::AttackContinuityWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_attack_continuity
            .as_ref()
            .unwrap()
    }
    fn snap(c: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn composition_attack_continuity_roundtrip_and_precise_bounds() {
        let a = compile_json(&parse_yaml(SOURCE).unwrap().to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let p = a.equipment_attacks()[0].program.continuity.unwrap();
        assert_eq!((p.per_second_percent, p.max_percent), (1, 100));
        for field in ["per_second_percent", "max_percent"] {
            for bad in [0, 10_001] {
                let mut d = parse_yaml(SOURCE).unwrap();
                program(&mut d)["continuity"][field] = json!(bad);
                let e = compile_json(&d.to_string()).unwrap_err();
                assert!(format!("{e:?}").contains(field), "{e:?}");
            }
        }
    }
    #[test]
    fn composition_attack_continuity_linear_time_cap_hits_and_independent_owner() {
        let mut f = setup(1, 100);
        attack(&mut f, 0, true);
        for (now, expected) in [
            (0, 100),
            (999, 100),
            (1000, 101),
            (2000, 102),
            (10_000, 110),
            (60_000, 160),
            (100_000, 200),
        ] {
            assert_eq!(damage(&mut f, now), expected, "at {now}");
            let w = witness(&f);
            assert_eq!(
                (w.started_ms, w.elapsed_ms, w.percent),
                (0, now, expected - 100)
            );
            assert!(w.source_file.ends_with(".yaml") || !w.source_file.is_empty());
            assert!(w.field_path.ends_with("continuity"));
        }
        // Many additional hits and another victim neither renew nor accelerate time.
        for _ in 0..5 {
            assert_eq!(damage(&mut f, 100_000), 200);
        }
        let (a, b, t) = (f.a, f.b, f.secondary);
        assert_eq!(
            hit(
                &mut f,
                a,
                t,
                DamageType::Magic,
                DamageSource::BasicAttack,
                100_000,
                false
            ),
            200
        );
        assert_eq!(
            hit(
                &mut f,
                b,
                t,
                DamageType::Magic,
                DamageSource::Ability,
                100_000,
                false
            ),
            100
        );
        assert_eq!(damage(&mut f, 100_001), 100);
        assert!(f.core.equipment_attack_state.continuity.is_empty());
    }
    #[test]
    fn composition_attack_continuity_regular_admission_exact_deadline_and_restart() {
        let mut f = setup(2, 5);
        f.core.units.get_mut(&f.a).unwrap().stats.attack_interval_ms = 1000;
        attack(&mut f, 0, true);
        assert_eq!(damage(&mut f, 999), 100);
        attack(&mut f, 1000, true);
        assert_eq!(damage(&mut f, 1000), 102);
        attack(&mut f, 2000, true);
        assert_eq!(damage(&mut f, 2000), 104);
        attack(&mut f, 3000, true);
        assert_eq!(damage(&mut f, 3000), 105);
        attack(&mut f, 4001, true);
        assert_eq!(damage(&mut f, 4001), 100);
        assert_eq!(witness(&f).started_ms, 4001);
        attack(&mut f, 5001, false);
        assert_eq!(damage(&mut f, 5002), 100);
    }
    #[test]
    fn composition_attack_continuity_damage_sources_and_non_offensive_values() {
        let mut f = setup(1, 100);
        attack(&mut f, 0, true);
        let (a, t) = (f.a, f.enemy);
        for source in [
            DamageSource::BasicAttack,
            DamageSource::Ability,
            DamageSource::BuffTick,
        ] {
            for kind in [DamageType::Physical, DamageType::Magic] {
                assert_eq!(hit(&mut f, a, t, kind, source, 10_000, false), 110);
            }
            assert_eq!(
                hit(&mut f, a, t, DamageType::True, source, 10_000, false),
                100
            );
        }
        assert_eq!(
            hit(
                &mut f,
                a,
                t,
                DamageType::Magic,
                DamageSource::Environment,
                10_000,
                false
            ),
            100
        );
        assert_eq!(
            hit(
                &mut f,
                a,
                t,
                DamageType::Magic,
                DamageSource::Ability,
                10_000,
                true
            ),
            100
        );
        let attack_stat = f.core.units[&a].stats.attack;
        assert_eq!(damage(&mut f, 20_000), 120);
        assert_eq!(f.core.units[&a].stats.attack, attack_stat);
    }
    #[test]
    fn composition_attack_continuity_no_target_corrosion_unequip_and_terminal() {
        for mode in 0..3 {
            let mut f = setup(1, 100);
            attack(&mut f, 0, true);
            assert_eq!(damage(&mut f, 1000), 101);
            match mode {
                0 => f.core.units.get_mut(&f.a).unwrap().corroded = true,
                1 => {
                    let removed = Arc::make_mut(f.core.units.get_mut(&f.a).unwrap().equipment_slots.as_mut().unwrap()).slots[2].take().unwrap();
                    f.core.items.remove(&removed.runtime_item_instance_id);
                }
                _ => {
                    f.core.units.get_mut(&f.enemy).unwrap().owner = Side::Player;
                    f.core.units.get_mut(&f.secondary).unwrap().owner = Side::Player;
                    attack(&mut f, 1100, true);
                }
            }
            f.core
                .runtime_transaction(|c| c.reconcile_attack_continuity(1100))
                .unwrap();
            assert!(
                f.core.equipment_attack_state.continuity.is_empty(),
                "mode {mode}"
            );
        }
    }
    #[test]
    fn composition_attack_continuity_cancellation_preserves_clock_and_damage() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let token = CancellationToken::default();
        let mut f = setup(1, 100);
        f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 0;
        let before = snap(&f.core);
        let result = crate::authoring::control::with_test_cancellation(
            "attack_continuity_started",
            1,
            || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        f.core.runtime_transaction(|c| {
                            c.process_event(
                                BattleEvent::AttackStart {
                                    time_ms: 0,
                                    attacker_instance_id: f.a,
                                    target_instance_id: Some(f.enemy),
                                    schedule_next: true,
                                    cause: Default::default(),
                                },
                                0,
                            )
                        })
                    },
                )
            },
        );
        assert!(result.is_err());
        assert_eq!(snap(&f.core), before);
        attack(&mut f, 0, true);
        let before = snap(&f.core);
        let token = CancellationToken::default();
        let result = crate::authoring::control::with_test_cancellation(
            "attack_continuity_evaluated",
            1,
            || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        f.core.runtime_transaction(|c| {
                            let s = c
                                .damage_source_snapshot_for_unit(
                                    f.a,
                                    f.enemy,
                                    DamageSource::Ability,
                                    DamageType::Magic,
                                    100,
                                    Default::default(),
                                    0,
                                    1000,
                                    false,
                                )
                                .unwrap();
                            c.process_commands(
                                vec![BattleCommand::ApplyDamage {
                                    target_id: f.enemy,
                                    source_snapshot: s,
                                }],
                                1000,
                            );
                            runtime_control::poll()
                        })
                    },
                )
            },
        );
        assert!(result.is_err());
        assert_eq!(snap(&f.core), before);
        assert_eq!(damage(&mut f, 1000), 101);
    }
    #[test]
    fn composition_attack_continuity_long_windup_and_target_change_preserve_time() {
        let mut f = setup(1, 100);
        f.core.units.get_mut(&f.a).unwrap().stats.attack_interval_ms = 1000;
        f.core.units.get_mut(&f.a).unwrap().basic_attack.windup_ms = 1500;
        attack(&mut f, 0, true);
        assert_eq!(damage(&mut f, 1499), 101);
        f.core.units.get_mut(&f.enemy).unwrap().owner = Side::Player;
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 1);
        attack(&mut f, 1500, true);
        assert_eq!(f.core.units[&f.a].current_target, Some(f.secondary));
        assert_eq!(damage(&mut f, 1500), 101);
        assert_eq!(witness(&f).started_ms, 0);
        assert_eq!(witness(&f).through_ms, 3000);
    }
    #[test]
    fn composition_attack_continuity_stun_withdrawal_and_terminal_clear() {
        use crate::game::ability::sustained::{
            HitStatusKind, HitStatusProfile, SustainedEffectDef,
        };
        use crate::game::battle::core::sim::BattleLiveCommand;
        let mut f = setup(1, 100);
        attack(&mut f, 0, true);
        let (a, e) = (f.a, f.enemy);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    e,
                    a,
                    SustainedEffectDef::HitStatus {
                        duration_ms: 1000,
                        profile: HitStatusProfile {
                            definition: "continuity-stun".into(),
                            kind: HitStatusKind::Stun,
                            modifier_source: None,
                            vulnerability: None,
                        },
                    },
                    1000,
                )
            })
            .unwrap();
        assert_eq!(damage(&mut f, 1000), 100);
        assert!(f.core.equipment_attack_state.continuity.is_empty());
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(2001))
            .unwrap();
        attack(&mut f, 2001, true);
        assert_eq!(damage(&mut f, 3001), 101);
        assert_eq!(witness(&f).started_ms, 2001);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 3002,
            })
            .unwrap();
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(3002))
            .unwrap();
        assert!(f.core.equipment_attack_state.continuity.is_empty());
        let mut f = setup(1, 100);
        attack(&mut f, 0, true);
        f.core
            .runtime_transaction(|c| c.clear_sustained(1000))
            .unwrap();
        assert!(f.core.equipment_attack_state.continuity.is_empty());
    }
    #[test]
    fn composition_attack_continuity_copy_tie_and_later_copy_do_not_stack() {
        let mut f = setup_with_armor_provider(1, 100, true);
        attack(&mut f, 0, true);
        let other = move_armor_provider(&mut f);
        attack(&mut f, 1000, true);
        assert_eq!(damage(&mut f, 2000), 102);
        let selected = witness(&f).equipment.runtime_item_instance_id;
        assert_ne!(selected, other);
        assert_eq!(Arc::make_mut(f.core.units.get_mut(&f.a).unwrap().equipment_slots.as_mut().unwrap()).slots[2].take().unwrap().runtime_item_instance_id, selected);
        f.core.items.remove(&selected);
        assert_eq!(damage(&mut f, 2000), 101);
        assert_eq!(witness(&f).equipment.runtime_item_instance_id, other);
        let mut f = setup_with_armor_provider(1, 100, true);
        let expected = f.core.units[&f.a].equipment_slots.as_ref().unwrap().slots[2].unwrap().binding.persistent_copy_uuid;
        move_armor_provider(&mut f);
        attack(&mut f, 0, true);
        assert_eq!(damage(&mut f, 2000), 102);
        assert_eq!(
            witness(&f).equipment.persistent_copy_uuid,
            expected
        );
    }
    #[test]
    fn composition_attack_continuity_retained_refusal_and_observer_off_clone() {
        let mut f = setup_with_armor_provider(1, 100, true);
        // The second legal copy allocates after the first clock is installed,
        // above the temporary summary-copy peak earlier in admission.
        move_armor_provider(&mut f);
        f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 0;
        let event = BattleEvent::AttackStart {
            time_ms: 0,
            attacker_instance_id: f.a,
            target_instance_id: Some(f.enemy),
            schedule_next: true,
            cause: Default::default(),
        };
        let mut probe = f.core.clone();
        let (result, admissions) = runtime_control::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| c.process_event(event.clone(), 0))
        });
        result.unwrap();
        assert!(admissions
            .iter()
            .any(|(site, _, _)| *site == "attack_continuity_state"));
        // A lower clock-allocation limit is masked by an earlier temporary
        // provider/summary peak. Refuse the first actual maximum of the whole
        // admitted action and verify its precise site plus complete rollback.
        let needed = admissions.iter().map(|(_, _, n)| *n).max().unwrap();
        let expected_site = admissions.iter().find(|(_, _, n)| *n == needed).unwrap().0;
        f.core.runtime_facts.limits.max_retained_bytes = needed - 1;
        let before = snap(&f.core);
        let error = f
            .core
            .runtime_transaction(|c| c.process_event(event.clone(), 0))
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(runtime_control::BattleRuntimeFault {
                    code: runtime_control::RuntimeFaultCode::RetainedByteLimit,
                    site,
                    ..
                }) if site == expected_site
            ),
            "{error:?}"
        );
        assert_eq!(before, snap(&f.core));
        f.core.runtime_facts.limits = Default::default();
        let mut off = f.core.clone();
        off.authoring_observation = None;
        f.core
            .runtime_transaction(|c| c.process_event(event.clone(), 0))
            .unwrap();
        off.runtime_transaction(|c| c.process_event(event, 0))
            .unwrap();
        assert_eq!(snap(&f.core), snap(&off));
        assert_eq!(damage(&mut f, 1000), 101);
        let (a, t) = (f.a, f.enemy);
        off.runtime_transaction(|c| {
            let s = c
                .damage_source_snapshot_for_unit(
                    a,
                    t,
                    DamageSource::Ability,
                    DamageType::Physical,
                    100,
                    Default::default(),
                    0,
                    1000,
                    false,
                )
                .unwrap();
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: t,
                    source_snapshot: s,
                }],
                1000,
            );
            runtime_control::poll()
        })
        .unwrap();
        assert_eq!(snap(&f.core), snap(&off));
    }
    #[test]
    fn composition_attack_continuity_missing_copy_refuses_entire_start() {
        let mut f = setup(1, 100);
        for item in f
            .core
            .items
            .values_mut()
            .filter(|i| i.owner_unit_instance == f.a)
        {
            item.persistent_copy_uuid = None;
        }
        f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 0;
        let before = snap(&f.core);
        let error = f
            .core
            .runtime_transaction(|c| {
                c.process_event(
                    BattleEvent::AttackStart {
                        time_ms: 0,
                        attacker_instance_id: f.a,
                        target_instance_id: Some(f.enemy),
                        schedule_next: true,
                        cause: Default::default(),
                    },
                    0,
                )
            })
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(runtime_control::BattleRuntimeFault {
                    site: "attack_continuity_source",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(snap(&f.core), before);
    }
    #[test]
    fn composition_attack_continuity_fast_and_slow_actions_have_equal_elapsed_bonus() {
        for interval in [250, 1000] {
            let mut f = setup(1, 100);
            f.core.units.get_mut(&f.a).unwrap().stats.attack_interval_ms = interval;
            for now in (0..=4000).step_by(interval as usize) {
                attack(&mut f, now, true);
            }
            assert_eq!(damage(&mut f, 4000), 104, "interval={interval}");
            assert_eq!(
                (
                    witness(&f).started_ms,
                    witness(&f).elapsed_ms,
                    witness(&f).percent
                ),
                (0, 4000, 4)
            );
        }
    }
    #[test]
    fn composition_attack_continuity_appended_components_keep_type_and_actor_filter() {
        use crate::game::battle::{cooldown::CooldownSource, damage::DamageBonusSnapshot};
        for base in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup(1, 100);
            attack(&mut f, 0, true);
            let before = f.core.units[&f.enemy].stats.current_health;
            f.core
                .runtime_transaction(|c| {
                    let mut shot = c
                        .damage_source_snapshot_for_unit(
                            f.a,
                            f.enemy,
                            DamageSource::Ability,
                            base,
                            100,
                            Default::default(),
                            0,
                            10_000,
                            false,
                        )
                        .unwrap();
                    for (actor, kind) in [
                        (f.a, DamageType::Physical),
                        (f.a, DamageType::Magic),
                        (f.a, DamageType::True),
                        (f.b, DamageType::Physical),
                    ] {
                        shot.on_attack_bonus_damage.push(DamageBonusSnapshot {
                            source: CooldownSource::Unit {
                                unit_instance_id: actor,
                            },
                            equipment_source: None,
                            flat: 100,
                            percent: 0,
                            damage_type: kind,
                        });
                    }
                    c.process_commands(
                        vec![BattleCommand::ApplyDamage {
                            target_id: f.enemy,
                            source_snapshot: shot,
                        }],
                        10_000,
                    );
                    runtime_control::poll()
                })
                .unwrap();
            // Ten elapsed seconds add10% to only this actor's Physical/Magic components.
            assert_eq!(
                before - f.core.units[&f.enemy].stats.current_health,
                if base == DamageType::True { 520 } else { 530 },
                "{base:?}"
            );
        }
    }
}
