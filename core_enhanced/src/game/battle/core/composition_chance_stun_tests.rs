mod composition_chance_stun {
    use super::*;
    use crate::game::{
        ability::{
            sustained::{HitStatusKind, SustainedEffectView},
            DeliveryDef,
        },
        battle::{
            damage::{DamageSource, DamageSourceSnapshot, DamageType},
            types::UnitTargetTrait,
        },
        data::equipment_data::{EquipmentMetadata, EquipmentType},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/chance_stun.skill.yaml"
    ));
    fn setup(chance: u8) -> Fixture {
        setup_with(chance, |_| {})
    }
    fn setup_with(chance: u8, configure: impl FnOnce(&mut EquipmentMetadata)) -> Fixture {
        setup_custom(chance, configure, |_| {})
    }
    fn setup_custom(
        chance: u8,
        configure: impl FnOnce(&mut EquipmentMetadata),
        edit: impl FnOnce(&mut Value),
    ) -> Fixture {
        let mut doc = parse_yaml(SOURCE).unwrap();
        program(&mut doc)["on_hit_status"]["chance_percent"] = json!(chance);
        doc["values"].as_array_mut().unwrap().push(json!({"name":"weapon-hit", "value":{
            "AttackProgram":{"reactions":[],"basic_lifesteal":{"percent":10,"max_heal":1000},
                "on_hit_status":{"duration_ms":1500,"profile":{"definition":"weapon-slow","kind":{"BasicAttackSlow":{"percent":20}}}}}}}));
        doc["equipment_attacks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"equipment":"r11_weapon","program":{"Shared":"weapon-hit"}}));
        edit(&mut doc);
        let a = compile_json(&doc.to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let saved = std::cell::RefCell::new(None);
        let (mut core, state, player, ally) = r12_fragment_equipped_effect_core_with_data(
            &r12_control_ron("Damage(amount:10, damage_type:True)"),
            |s| {
                s.terminal_outcomes = BattleScenario::fixture_survival_outcomes(100000);
                s.groups[0].enemy_movement_plan = None;
                let mut other = s.groups[0].spawns[0].clone();
                other.unit_ref =
                    crate::game::battle::scenario::ScenarioUnitRef::new("h99_secondary");
                other.instance_salt += 100;
                other.position = crate::game::resources::Position::new(6, 3);
                s.groups[0].spawns.push(other);
                for (i, p) in s.groups[1].spawns.iter_mut().enumerate() {
                    p.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe990 + i as u128));
                    p.draft.equipped_items.push(
                        crate::game::battle::types::BattleEquipmentBinding {
                            base_uuid: Uuid::from_u128(0x1199),
                            persistent_copy_uuid: Some(Uuid::from_u128(0xe998 + i as u128)),
                            slot: None,
                            weapon_stage: Default::default(),
                        },
                    );
                }
            },
            |weapon| {
                configure(weapon);
                *saved.borrow_mut() = Some(weapon.clone());
            },
            None,
            |builder| {
                let weapon = saved.borrow_mut().take().unwrap();
                let mut accessory = weapon.clone();
                accessory.id = "h99_gift_sack".into();
                accessory.uuid = Uuid::from_u128(0x1199);
                accessory.equipment_type = EquipmentType::Accessory;
                accessory.weapon_profile = None;
                builder
                    .with_equipment(vec![weapon, accessory])
                    .with_equipment_attacks(a.equipment_attacks().to_vec())
                    .with_equipment_defenses(a.equipment_defenses().to_vec())
            },
        );
        let mut enemies: Vec<_> = core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .map(|u| (u.spawn_order, u.instance_id))
            .collect();
        enemies.sort();
        let enemy = enemies[0].1;
        let secondary = enemies[1].1;
        for u in core.units.values_mut() {
            u.stats.max_health = 10000;
            u.stats.current_health = 10000;
            u.stats.attack_interval_ms = 100000;
            u.next_basic_attack_ms = 100000;
            u.stats.move_speed_units_per_ms = 0;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
        }
        let u = core.units.get_mut(&player).unwrap();
        u.stats.current_health = 5000;
        u.stats.attack = 100;
        u.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        u.basic_attack.windup_ms = 80;
        u.basic_attack.delivery = DeliveryDef::Instant;
        for (id, x, y) in [
            (player, 2, 1),
            (ally, 7, 3),
            (enemy, 3, 1),
            (secondary, 6, 3),
        ] {
            super::super::compiled_action_fixture::place(&mut core, id, x, y);
        }
        core.enable_authoring_observation();
        Fixture {
            core,
            state,
            a: player,
            b: ally,
            enemy,
            secondary,
            run: RunResourceState {
                revision: 0,
                closed: false,
                entries: vec![],
            },
        }
    }
    fn step(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn hit(f: &mut Fixture, now: u64) {
        attack(f, now, true);
        step(f, now + 80);
    }
    fn stuns(core: &BattleCore) -> Vec<SustainedEffectView> {
        core.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|r| {
                r.hit_status
                    .as_ref()
                    .is_some_and(|p| p.kind == HitStatusKind::Stun)
            })
            .collect()
    }
    fn state(f: &Fixture) -> impl PartialEq + std::fmt::Debug {
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            format!("{:?}", f.state),
        )
    }
    fn captured(f: &mut Fixture) -> DamageSourceSnapshot {
        attack(f, 10, true);
        f.core
            .event_queue
            .iter()
            .find_map(|e| match e.as_ref() {
                BattleEvent::AttackResolve {
                    attacker_instance_id,
                    source_snapshot,
                    ..
                } if *attacker_instance_id == f.a => Some(source_snapshot.clone()),
                _ => None,
            })
            .unwrap()
    }
    fn committed(f: &mut Fixture, snapshot: DamageSourceSnapshot, now: u64) {
        use crate::game::battle::core::damage_runtime::{
            CommittedDamageInput, CommittedDamagePolicy,
        };
        f.core
            .runtime_transaction(|c| {
                c.execute_committed_damage(CommittedDamageInput {
                    target_id: f.enemy,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: CommittedDamagePolicy::GeneralCommand,
                })
                .ok_or(GameError::InvalidAction)
                .map(|_| ())
            })
            .unwrap();
    }
    #[test]
    fn composition_chance_stun_schema_legacy_default_roundtrip_and_bounds() {
        for chance in [0, 50, 100] {
            let source =
                SOURCE.replace("chance_percent: 100", &format!("chance_percent: {chance}"));
            let a = crate::authoring::composition::compile_yaml(&source).unwrap();
            let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
            assert_eq!(
                a.equipment_attacks()[0]
                    .program
                    .on_hit_status
                    .as_ref()
                    .unwrap()
                    .chance_percent,
                chance
            );
        }
        let source = SOURCE.replace("          chance_percent: 100\n", "");
        let a = crate::authoring::composition::compile_yaml(&source).unwrap();
        assert_eq!(
            a.equipment_attacks()[0]
                .program
                .on_hit_status
                .as_ref()
                .unwrap()
                .chance_percent,
            100
        );
        assert!(!a.to_ron().unwrap().contains("chance_percent"));
        for value in ["101", "-1", "0.5", "unresolved"] {
            let e = crate::authoring::composition::compile_yaml(
                &SOURCE.replace("chance_percent: 100", &format!("chance_percent: {value}")),
            )
            .unwrap_err();
            assert!(format!("{e:?}").contains("chance_percent"), "{e:?}");
        }
    }
    #[test]
    fn composition_chance_stun_accessory_coexists_with_weapon_status_lifesteal_and_types() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for chance in [0, 100] {
                let mut f = setup(chance);
                f.core.units.get_mut(&f.a).unwrap().basic_attack.damage_type = kind;
                let snapshot = captured(&mut f);
                step(&mut f, 90);
                assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
                assert_eq!(f.core.units[&f.a].stats.current_health, 5010);
                assert_eq!(f.core.sustained_basic_interval(f.enemy, 1000), 1250);
                let records = stuns(&f.core);
                assert_eq!(records.len(), usize::from(chance == 100));
                if let Some(r) = records.first() {
                    assert_eq!(r.expires_at_ms, 1090);
                    let provider = snapshot
                        .equipment_hits
                        .as_ref()
                        .unwrap()
                        .providers
                        .iter()
                        .find(|p| p.equipment.definition_uuid == Uuid::from_u128(0x1199))
                        .unwrap();
                    let e = provider.equipment;
                    assert_eq!(e.definition_uuid, Uuid::from_u128(0x1199));
                    assert!(e.persistent_copy_uuid.is_some());
                    let origin = &provider.program.on_hit_status.as_ref().unwrap().source;
                    assert_eq!(r.authored_site.as_ref(), Some(&origin.site));
                    assert!(origin.field_path.contains("on_hit_status"));
                    assert_eq!(
                        origin.origin_definition_id.as_str(),
                        "equipment:h99_gift_sack"
                    );
                }
            }
        }
    }
    #[test]
    fn composition_chance_stun_overlap_is_finite_and_does_not_sum_durations() {
        let mut f = setup(100);
        hit(&mut f, 10);
        hit(&mut f, 500);
        assert_eq!(stuns(&f.core).len(), 2);
        step(&mut f, 1090);
        assert_eq!(stuns(&f.core).len(), 1);
        step(&mut f, 1579);
        assert!(f
            .core
            .unit_control_state(f.enemy, 1579)
            .unwrap()
            .hard_cc_active());
        step(&mut f, 1580);
        assert!(stuns(&f.core).is_empty());
        assert!(!f
            .core
            .unit_control_state(f.enemy, 1580)
            .unwrap()
            .hard_cc_active());
    }
    #[test]
    fn composition_chance_stun_immunity_triggered_and_foreign_source_exclusions() {
        let mut f = setup(100);
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .target_traits
            .push(UnitTargetTrait::StunImmune);
        hit(&mut f, 10);
        assert!(stuns(&f.core).is_empty());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        let mut f = setup(100);
        attack(&mut f, 10, false);
        step(&mut f, 90);
        assert!(stuns(&f.core).is_empty());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        for source in [
            DamageSource::Ability,
            DamageSource::BuffTick,
            DamageSource::Environment,
            DamageSource::BasicAttack,
        ] {
            let mut f = setup(100);
            let mut s = captured(&mut f);
            s.source = source;
            if source == DamageSource::BasicAttack {
                s.fact_source.reaction = true;
            }
            committed(&mut f, s, 20);
            assert!(stuns(&f.core).is_empty(), "{source:?}");
        }
    }
    #[test]
    fn composition_chance_stun_cancel_mid_contribution_rolls_back_whole_hit_and_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(100);
        attack(&mut f, 10, true);
        step(&mut f, 89);
        let before = state(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_attack_hit_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 90),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(state(&f), before);
        step(&mut f, 90);
        assert_eq!(stuns(&f.core).len(), 1);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
    }
    #[test]
    fn composition_chance_stun_fifty_percent_sequence_is_clone_observer_independent() {
        let mut observed = setup(50);
        let mut plain = Fixture {
            core: observed.core.clone(),
            state: observed.state.clone(),
            a: observed.a,
            b: observed.b,
            enemy: observed.enemy,
            secondary: observed.secondary,
            run: observed.run.clone(),
        };
        plain.core.authoring_observation = None;
        let mut hits = 0;
        for i in 0..32 {
            let now = 10 + i * 2000;
            hit(&mut observed, now);
            hit(&mut plain, now);
            let n = stuns(&observed.core).len();
            assert_eq!(n, stuns(&plain.core).len());
            hits += n;
            assert_eq!(
                serde_json::to_value(&observed.core.event_log).unwrap(),
                serde_json::to_value(&plain.core.event_log).unwrap()
            );
            assert_eq!(
                serde_json::to_value(observed.core.runtime_fact_view()).unwrap(),
                serde_json::to_value(plain.core.runtime_fact_view()).unwrap()
            );
            assert_eq!(
                observed.core.event_queue_debug_snapshot(),
                plain.core.event_queue_debug_snapshot()
            );
        }
        assert!(hits > 0 && hits < 32, "expected both outcomes, got {hits}");
    }
    #[test]
    fn composition_chance_stun_shield_zero_parry_invulnerability_and_lethal_boundaries() {
        use crate::game::ability::sustained::SustainedEffectDef;
        let mut f = setup(100);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 2000,
                        amount: 1000,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        hit(&mut f, 10);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        assert_eq!(stuns(&f.core).len(), 1);
        let mut f = setup(100);
        let mut s = captured(&mut f);
        s.base_damage = 0;
        s.minimum_damage = 0;
        committed(&mut f, s, 20);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        assert_eq!(stuns(&f.core).len(), 1);
        let mut f = setup(100);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        hit(&mut f, 10);
        assert!(stuns(&f.core).is_empty());
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        let mut f = setup(100);
        let s = captured(&mut f);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Parry {
                        duration_ms: 200,
                        window_ms: 30,
                        invulnerable_ms: 40,
                        miss_recovery_ms: 20,
                        reduction_percent: 0,
                        counter_skill: "r11_aura".into(),
                        end_on_resolution: false,
                        fragment_guard: None,
                        guard_charge: None,
                    },
                    15,
                )
            })
            .unwrap();
        committed(&mut f, s.clone(), 20);
        assert!(stuns(&f.core).is_empty());
        committed(&mut f, s, 30);
        assert!(stuns(&f.core).is_empty());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
    }
    #[test]
    fn composition_chance_stun_area_and_multitarget_each_victim_once_and_each_strike() {
        use crate::game::data::abnormality_data::BasicAttackAction;
        for area in [false, true] {
            let mut f = setup(100);
            super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 4, 1);
            let u = f.core.units.get_mut(&f.a).unwrap();
            u.basic_attack.action = if area {
                BasicAttackAction::DamageArea {
                    footprint: crate::game::battle::tile_range::TileRangePattern {
                        include_anchor_tile: true,
                        rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                    },
                    secondary_percent: 50,
                }
            } else {
                BasicAttackAction::DamageTargets { max_targets: 3 }
            };
            u.basic_attack.delivery =
                ron::de::from_str("Projectile(speed_units_per_ms:6000)").unwrap();
            attack(&mut f, 10, true);
            step(&mut f, 800);
            let records = stuns(&f.core);
            assert_eq!(records.len(), 2);
            assert!(records.iter().any(|r| r.target == f.enemy));
            assert!(records.iter().any(|r| r.target == f.secondary));
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
            assert_eq!(
                f.core.units[&f.secondary].stats.current_health,
                if area { 9950 } else { 9900 }
            );
            attack(&mut f, 810, true);
            // The farther projectile arrives at1165ms; the first expires at1199ms.
            step(&mut f, 1180);
            assert_eq!(stuns(&f.core).len(), 4);
        }
        // Appended damage components are part of the same committed hit.
        let mut f = setup(100);
        let mut s = captured(&mut f);
        s.on_attack_bonus_damage
            .push(crate::game::battle::damage::DamageBonusSnapshot {
                source: crate::game::battle::cooldown::CooldownSource::Unit {
                    unit_instance_id: f.a,
                },
                equipment_source: None,
                flat: 30,
                percent: 0,
                damage_type: DamageType::Magic,
            });
        committed(&mut f, s, 20);
        assert_eq!(stuns(&f.core).len(), 1);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9870);
    }
    #[test]
    fn composition_chance_stun_released_carrier_keeps_removed_source_and_finite_deadline() {
        for withdraw in [false, true] {
            let mut f = setup(100);
            f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery =
                ron::de::from_str("Projectile(speed_units_per_ms:6000)").unwrap();
            attack(&mut f, 10, true);
            step(&mut f, 90);
            assert!(!f.core.projectiles.is_empty());
            if withdraw {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.a,
                        time_ms: 91,
                    })
                    .unwrap();
            } else {
                f.core.items.retain(|_, item| {
                    item.owner_unit_instance != f.a || item.base_uuid != Uuid::from_u128(0x1199)
                });
            }
            step(&mut f, 800);
            let records = stuns(&f.core);
            assert_eq!(records.len(), 1);
            let expiry = records[0].expires_at_ms;
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
            step(&mut f, expiry);
            assert!(stuns(&f.core).is_empty());
            if !withdraw {
                attack(&mut f, expiry + 1, true);
                step(&mut f, expiry + 800);
                assert!(stuns(&f.core).is_empty());
            }
        }
    }
    #[test]
    fn composition_chance_stun_capture_and_status_retained_refusal_are_atomic() {
        use crate::game::battle::runtime_control as native;
        for admission in [true, false] {
            let mut f = setup(100);
            if admission {
                f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 10;
                f.core.event_queue.push(BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: f.a,
                    target_instance_id: Some(f.enemy),
                    schedule_next: true,
                    cause: Default::default(),
                });
                step(&mut f, 9);
            } else {
                attack(&mut f, 10, true);
                step(&mut f, 89);
            }
            let now = if admission { 10 } else { 90 };
            let site = if admission {
                "equipment_hit_capture"
            } else {
                "sustained_record_storage"
            };
            let mut probe = f.core.clone();
            let mut execution = f.state.clone();
            let (result, admissions) = native::observe_retained_admissions(|| {
                probe.step_battle_execution_until(&mut execution, now)
            });
            result.unwrap();
            let required = admissions.iter().find(|(s, _, _)| *s == site).unwrap().2;
            let original = f.core.runtime_facts.limits.max_retained_bytes;
            f.core.runtime_facts.limits.max_retained_bytes = required - 1;
            let before = state(&f);
            let error = f
                .core
                .step_battle_execution_until(&mut f.state, now)
                .unwrap_err();
            assert!(
                matches!(&error,GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code:native::RuntimeFaultCode::RetainedByteLimit,site:actual,..}) if *actual==site),
                "{error:?}"
            );
            assert_eq!(state(&f), before);
            f.core.runtime_facts.limits.max_retained_bytes = original;
            step(&mut f, now);
            step(&mut f, 90);
            assert_eq!(stuns(&f.core).len(), 1);
        }
    }
    #[test]
    fn composition_chance_stun_directional_piercing_is_once_per_contact_target() {
        let mut f = setup(100);
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 4, 1);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery=ron::de::from_str("Projectile(speed_units_per_ms:6000,hit_policy:DirectionalCollision,max_range_tiles:Some(6),max_lifetime_ms:Some(2000),collision:(radius_units:125000,piercing:true,despawn_on_hit:Some(false),max_hits:Some(8)))").unwrap();
        attack(&mut f, 10, true);
        step(&mut f, 700);
        let records = stuns(&f.core);
        assert_eq!(records.len(), 2);
        for target in [f.enemy, f.secondary] {
            assert_eq!(records.iter().filter(|r| r.target == target).count(), 1);
            assert_eq!(f.core.units[&target].stats.current_health, 9900);
        }
    }
    #[test]
    fn composition_chance_stun_actual_evasion_prevents_damage_and_status() {
        let mut f = setup_custom(
            100,
            |_| {},
            |doc| {
                doc["values"].as_array_mut().unwrap().push(json!({"name":"evasion","value":{"DefenseProgram":{"BasicAttackEvasion":{"chance_percent":100}}}}));
                doc["equipment_defenses"] =
                    json!([{"equipment":"h99_gift_sack","program":{"Shared":"evasion"}}]);
            },
        );
        // Explicit fixture binding of a second copy to the defender.
        let key = f
            .core
            .items
            .iter()
            .find(|(_, item)| {
                item.owner_unit_instance == f.b && item.base_uuid == Uuid::from_u128(0x1199)
            })
            .map(|(key, _)| *key)
            .unwrap();
        f.core.items.get_mut(&key).unwrap().owner_unit_instance = f.enemy;
        hit(&mut f, 10);
        assert!(stuns(&f.core).is_empty());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        assert!(
            f.core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_basic_evasion
                .as_ref()
                .unwrap()
                .evaded
        );
    }
    #[test]
    fn composition_release_gap_weapon_and_accessory_choose_strongest_reset_both() {
        let mut f=setup_custom(100, |_|{}, |d| {
            *d=parse_yaml(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/skill_composition/release_gap.skill.yaml"))).unwrap();
            d["values"].as_array_mut().unwrap().push(json!({"name":"second-clock","value":{"AttackProgram":{
                "reactions":[],"release_gap":{"per_second_percent":40,"max_percent":100}}}}));
            d["equipment_attacks"].as_array_mut().unwrap().push(json!({"equipment":"h99_gift_sack","program":{"Shared":"second-clock"}}));
        });
        assert_eq!(f.core.equipment_attack_state.release_gaps.values().filter(|c|c.owner==f.a).count(),2);
        attack(&mut f,1000,true);f.core.step_battle_execution_until(&mut f.state,1080).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health,9857);
        let w=f.core.authoring_observation.as_ref().unwrap().latest_release_gap.as_ref().unwrap();
        assert_eq!(w.percent,43);assert_eq!(w.equipment.definition_uuid,Uuid::from_u128(0x1199));
        assert_eq!(f.core.release_gap_snapshot(f.a,1080).unwrap().unwrap().percent,0);
        attack(&mut f,2000,true);f.core.step_battle_execution_until(&mut f.state,2080).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health,9717);
        assert_eq!(f.core.authoring_observation.as_ref().unwrap().latest_release_gap.as_ref().unwrap().percent,40);
    }
}
