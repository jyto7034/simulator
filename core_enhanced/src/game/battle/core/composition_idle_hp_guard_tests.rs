mod composition_idle_hp_guard {
    use super::*;
    use crate::game::{
        ability::sustained::{SurvivalPolicy, SustainedEffectDef},
        battle::runtime_control,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/idle_hp_guard.skill.yaml"
    ));
    fn setup(percent: u8) -> Fixture {
        let source = SOURCE.replace(
            "reduction_percent: 50",
            &format!("reduction_percent: {percent}"),
        );
        let mut f = fixture_extra_with_defense(
            "Damage(amount:100,damage_type:True)",
            false,
            None,
            false,
            &source,
            |_| {},
            |scenario| {
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe10100 + i as u128));
                }
            },
        );
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(0))
            .unwrap();
        f
    }
    fn hit(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        amount: u32,
        source: DamageSource,
        kind: DamageType,
        now: u64,
    ) -> Result<u32, GameError> {
        let before = core.units[&target].stats.current_health;
        core.runtime_transaction(|c| {
            let snapshot = c
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    source,
                    kind,
                    amount,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .unwrap();
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: snapshot,
                }],
                now,
            );
            runtime_control::poll()
        })?;
        Ok(before - core.units[&target].stats.current_health)
    }
    fn damage(f: &mut Fixture, amount: u32, now: u64) -> u32 {
        hit(
            &mut f.core,
            f.target,
            f.caster,
            amount,
            DamageSource::Ability,
            DamageType::True,
            now,
        )
        .unwrap()
    }
    fn effect(f: &mut Fixture, def: SustainedEffectDef, now: u64) {
        let a = f.caster;
        f.core
            .runtime_transaction(|c| c.begin_sustained(a, a, def, now))
            .unwrap();
    }
    fn witness(f: &Fixture) -> &crate::game::battle::core::equipment_defense::IdleHpGuardWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_idle_hp_guard
            .as_ref()
            .unwrap()
    }
    fn snapshot(c: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn composition_idle_hp_guard_schema_shared_roundtrip_and_invalid_bounds() {
        let doc = parse_yaml(SOURCE).unwrap();
        let a = compile_json(&doc.to_string()).unwrap();
        let b = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert!(matches!(
            b.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::IdleHpGuard {
                recharge_ms: 2000,
                reduction_percent: 50
            }
        ));
        for (key, value) in [
            ("recharge_ms", json!(0)),
            ("recharge_ms", json!(-1)),
            ("reduction_percent", json!(0)),
            ("reduction_percent", json!(101)),
            ("reduction_percent", json!(50.5)),
            ("reduction_percent", json!({"Unresolved":"balance"})),
        ] {
            let mut bad = doc.clone();
            bad["values"][0]["value"]["DefenseProgram"]["IdleHpGuard"][key] = value;
            let err = compile_json(&bad.to_string()).unwrap_err();
            assert!(err.to_string().contains(key), "{err:?}");
        }
    }
    #[test]
    fn composition_idle_hp_guard_exact_charge_damage_reset_no_stack_and_rounding() {
        let mut f = setup(50);
        assert_eq!(damage(&mut f, 100, 1999), 100);
        assert_eq!(damage(&mut f, 100, 3998), 100);
        assert_eq!(damage(&mut f, 101, 5998), 50);
        assert_eq!(witness(&f).charged_since_ms, 3998);
        assert_eq!(
            (witness(&f).incoming_hp_damage, witness(&f).prevented),
            (101, 51)
        );
        assert_eq!(damage(&mut f, 100, 5998), 100);
        assert_eq!(damage(&mut f, 100, 20000), 50);
        assert_eq!(damage(&mut f, 100, 20000), 100);
    }
    #[test]
    fn composition_idle_hp_guard_all_damage_kinds_and_independent_wearers() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for source in [
                DamageSource::BasicAttack,
                DamageSource::Ability,
                DamageSource::BuffTick,
                DamageSource::Environment,
            ] {
                let mut f = setup(50);
                assert_eq!(
                    hit(&mut f.core, f.target, f.caster, 100, source, kind, 2000).unwrap(),
                    50
                );
                assert_eq!(
                    hit(&mut f.core, f.target, f.ally, 100, source, kind, 2000).unwrap(),
                    50
                );
                assert_eq!(
                    hit(&mut f.core, f.target, f.caster, 100, source, kind, 2001).unwrap(),
                    100
                );
            }
        }
        let mut f = setup(50);
        assert_eq!(
            hit(
                &mut f.core,
                f.caster,
                f.caster,
                100,
                DamageSource::Ability,
                DamageType::True,
                2000
            )
            .unwrap(),
            50
        );
    }
    #[test]
    fn composition_idle_hp_guard_shields_before_reduction_zero_preserves_and_receipt_conserves() {
        let mut f = setup(50);
        effect(
            &mut f,
            SustainedEffectDef::Shield {
                duration_ms: 10000,
                amount: 140,
                priority: 0,
                shared_radius_units: None,
            },
            1,
        );
        assert_eq!(damage(&mut f, 100, 2000), 0);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_idle_hp_guard
            .is_none());
        assert_eq!(damage(&mut f, 0, 2001), 0);
        assert_eq!(damage(&mut f, 100, 2002), 30);
        assert_eq!(witness(&f).charged_since_ms, 0);
        assert_eq!(
            (witness(&f).incoming_hp_damage, witness(&f).prevented),
            (60, 30)
        );
        let facts = serde_json::to_value(f.core.runtime_fact_view()).unwrap();
        let text = serde_json::to_string(&facts).unwrap();
        assert!(text.contains("idle_hp_guard"));
        assert!(text.contains("IdleHpGuard"));
        assert_eq!(damage(&mut f, 100, 2003), 100);
    }
    #[test]
    fn composition_idle_hp_guard_full_prevention_recharges_and_lethal_is_not_survival() {
        let mut f = setup(100);
        assert_eq!(damage(&mut f, 100, 2000), 0);
        assert_eq!(damage(&mut f, 100, 2001), 100);
        assert_eq!(damage(&mut f, 100, 4001), 0);
        let mut f = setup(50);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 40;
        assert_eq!(damage(&mut f, 100, 2000), 40);
        assert!(f.core.units[&f.caster].lifecycle.is_dead());
        assert_eq!(witness(&f).prevented, 50);
    }
    #[test]
    fn composition_idle_hp_guard_survival_prevention_is_not_subtracted_twice() {
        let mut f = setup(50);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 40;
        effect(
            &mut f,
            SustainedEffectDef::Survival {
                duration_ms: 10000,
                policy: SurvivalPolicy::PreventDeathOnce,
            },
            1,
        );
        assert_eq!(damage(&mut f, 100, 2000), 39);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 1);
        assert_eq!(damage(&mut f, 100, 2001), 1);
        assert!(f.core.units[&f.caster].lifecycle.is_dead());
    }
    #[test]
    fn composition_idle_hp_guard_cost_and_attack_do_not_consume_or_reset() {
        let mut f = setup(50);
        let a = f.caster;
        let e = f.target;
        let def=ron::de::from_str(r#"ChannelAura(duration_ms:1000,aura:(family:"idle-cost",priority:0,radius_units:10000000,basic_damage_percent:1,basic_speed_points:0,health_upkeep:Some((interval_ms:250,max_health_percent:10))))"#).unwrap();
        effect(&mut f, def, 1800);
        f.core
            .step_battle_execution_until(&mut f.state, 2050)
            .unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 900);
        hit(
            &mut f.core,
            a,
            e,
            10,
            DamageSource::BasicAttack,
            DamageType::True,
            2051,
        )
        .unwrap();
        assert_eq!(damage(&mut f, 100, 2052), 50);
        assert_eq!(witness(&f).charged_since_ms, 0);
    }
    #[test]
    fn composition_idle_hp_guard_remove_corrosion_and_terminal_clear_charge() {
        let mut f = setup(50);
        let a = f.caster;
        f.core.units.get_mut(&a).unwrap().corroded = true;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(2000))
            .unwrap();
        f.core.units.get_mut(&a).unwrap().corroded = false;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(3000))
            .unwrap();
        assert_eq!(damage(&mut f, 100, 4999), 100);
        assert_eq!(damage(&mut f, 100, 6999), 50);
        let ids: Vec<_> = f
            .core
            .items
            .values()
            .filter(|i| i.owner_unit_instance == a)
            .map(|i| i.instance_id)
            .collect();
        for id in ids {
            f.core.items.remove(&id);
        } // Legacy fixture has no slot table.
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(9000))
            .unwrap();
        assert_eq!(damage(&mut f, 100, 9000), 100);
        f.core
            .runtime_transaction(|c| c.clear_sustained(9001))
            .unwrap();
        assert!(f.core.equipment_defense.is_empty());
    }
    #[test]
    fn composition_idle_hp_guard_cancellation_restores_state_facts_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in ["idle_hp_guard_consumed", "idle_hp_guard_damage_reset"] {
            let mut f = setup(50);
            let before = snapshot(&f.core);
            let token = CancellationToken::default();
            let res = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        hit(
                            &mut f.core,
                            f.target,
                            f.caster,
                            100,
                            DamageSource::Ability,
                            DamageType::True,
                            2000,
                        )
                    },
                )
            });
            assert!(res.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(snapshot(&f.core), before);
            assert_eq!(damage(&mut f, 100, 2000), 50);
        }
    }
    #[test]
    fn composition_idle_hp_guard_actual_clone_without_observation_matches_ordered_facts() {
        let mut f = setup(50);
        damage(&mut f, 100, 1000);
        let mut clone = f.core.clone();
        clone.authoring_observation = None;
        for time in [2999, 4999, 5000, 7000] {
            assert_eq!(
                hit(
                    &mut clone,
                    f.target,
                    f.caster,
                    101,
                    DamageSource::Ability,
                    DamageType::True,
                    time
                )
                .unwrap(),
                damage(&mut f, 101, time)
            );
        }
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            clone.event_queue_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            serde_json::to_value(clone.runtime_fact_view()).unwrap()
        );
    }
    #[test]
    fn composition_idle_hp_guard_parry_and_invulnerability_preserve_preparation() {
        let mut f = setup(50);
        effect(
            &mut f,
            SustainedEffectDef::Parry {
                duration_ms: 5000,
                window_ms: 1000,
                invulnerable_ms: 1500,
                miss_recovery_ms: 100,
                reduction_percent: 0,
                counter_skill: "r11_aura".into(),
                end_on_resolution: false,
                fragment_guard: None,
                guard_charge: None,
            },
            1999,
        );
        assert_eq!(damage(&mut f, 100, 2000), 0);
        assert_eq!(damage(&mut f, 100, 2001), 0);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_idle_hp_guard
            .is_none());
        assert_eq!(damage(&mut f, 100, 3500), 50);
        assert_eq!(witness(&f).charged_since_ms, 0);
    }
    #[test]
    fn composition_idle_hp_guard_packet_components_share_guard_but_next_hit_does_not() {
        use crate::game::battle::{cooldown::CooldownSource, damage::DamageBonusSnapshot};
        let mut f = setup(50);
        let a = f.caster;
        let e = f.target;
        f.core
            .runtime_transaction(|c| {
                let mut shot = c
                    .damage_source_snapshot_for_unit(
                        e,
                        a,
                        DamageSource::BasicAttack,
                        DamageType::Physical,
                        100,
                        Default::default(),
                        0,
                        2000,
                        false,
                    )
                    .unwrap();
                shot.on_attack_bonus_damage.push(DamageBonusSnapshot {
                    source: CooldownSource::Unit {
                        unit_instance_id: e,
                    },
                    equipment_source: None,
                    flat: 50,
                    percent: 0,
                    damage_type: DamageType::Magic,
                });
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: a,
                        source_snapshot: shot,
                    }],
                    2000,
                );
                runtime_control::poll()
            })
            .unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 925);
        assert_eq!(
            (witness(&f).incoming_hp_damage, witness(&f).prevented),
            (150, 75)
        );
        assert_eq!(damage(&mut f, 100, 2000), 100);
        // Ordinary incoming mitigation precedes the one-use post-shield guard.
        f.core
            .units
            .get_mut(&a)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 20;
        assert_eq!(damage(&mut f, 100, 4000), 40);
    }
    #[test]
    fn composition_idle_hp_guard_withdraw_redeploy_starts_empty_with_same_persistent_copy() {
        let mut f = setup(50);
        let a = f.caster;
        let e = f.target;
        let unit = &f.core.units[&a];
        let owned = unit.source_owned_uuid;
        let position = unit.body.projected_tile();
        let draft = f
            .core
            .scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .find(|s| s.draft.owned_uuid == owned)
            .unwrap()
            .draft
            .clone();
        assert_eq!(damage(&mut f, 100, 2000), 50);
        let copy = witness(&f).equipment.persistent_copy_uuid;
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 4000,
            })
            .unwrap();
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed {
            unit_id: fresh,
        } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe1011,
                time_ms: 5000,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        assert_ne!(a, fresh);
        let u = f.core.units.get_mut(&fresh).unwrap();
        u.stats.max_health = 1000;
        u.stats.current_health = 1000;
        u.stats.defense = 0;
        u.stats.magic_resist = 0;
        u.next_basic_attack_ms = 100000;
        assert_eq!(
            hit(
                &mut f.core,
                e,
                fresh,
                100,
                DamageSource::Ability,
                DamageType::True,
                6999
            )
            .unwrap(),
            100
        );
        assert_eq!(
            hit(
                &mut f.core,
                e,
                fresh,
                100,
                DamageSource::Ability,
                DamageType::True,
                8999
            )
            .unwrap(),
            50
        );
        assert_eq!(witness(&f).equipment.persistent_copy_uuid, copy);
        assert_eq!(witness(&f).charged_since_ms, 6999);
    }
    #[test]
    fn composition_idle_hp_guard_retained_refusal_restores_consumed_charge_and_retries() {
        let mut f = setup(50);
        let mut probe = f.core.clone();
        let (result, admissions) = runtime_control::observe_retained_admissions(|| {
            hit(
                &mut probe,
                f.target,
                f.caster,
                100,
                DamageSource::Ability,
                DamageType::True,
                2000,
            )
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "idle_hp_guard_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snapshot(&f.core);
        let err = hit(
            &mut f.core,
            f.target,
            f.caster,
            100,
            DamageSource::Ability,
            DamageType::True,
            2000,
        )
        .unwrap_err();
        assert!(
            matches!(
                &err,
                GameError::BattleRuntimeFault(runtime_control::BattleRuntimeFault {
                    code: runtime_control::RuntimeFaultCode::RetainedByteLimit,
                    site: "idle_hp_guard_observation",
                    ..
                })
            ),
            "{err:?}"
        );
        assert_eq!(snapshot(&f.core), before);
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        assert_eq!(damage(&mut f, 100, 2000), 50);
    }
    fn paired_fixture(extra: serde_json::Value, prepared: bool, bonus: i32) -> Fixture {
        use crate::game::data::equipment_data::{EquipmentMetadata, EquipmentType};
        let mut doc = parse_yaml(SOURCE).unwrap();
        doc["values"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":"extra-defense","value":{"DefenseProgram":extra}}));
        doc["equipment_defenses"]
            .as_array_mut()
            .unwrap()
            .push(json!({"equipment":"idle_guard_extra","program":{"Shared":"extra-defense"}}));
        if prepared {
            let mut head = parse_yaml(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/skill_composition/scarecrow_headshot.skill.yaml"
            )))
            .unwrap();
            head["skills"][0]["id"] = json!("r11_aura");
            doc["values"]
                .as_array_mut()
                .unwrap()
                .extend(head["values"].as_array().unwrap().clone());
            doc["definitions"] = head["definitions"].clone();
            doc["skills"] = head["skills"].clone();
        }
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let mut skills = artifact.load_native().unwrap();
        // This cross-boundary probe uses a boss attacker; keep the compiled preparation
        // and reward but remove employee-only admission from its native fixture skill.
        // The real employee admission path is covered by H100's native and World tests.
        if prepared {
            skills.skills[0].resonance_admission = None;
        }
        let authored = if prepared {
            ron::ser::to_string(&skills).unwrap()
        } else {
            r12_control_ron("Damage(amount:100,damage_type:True)")
        };
        let saved = std::cell::RefCell::new(None::<EquipmentMetadata>);
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |s| {
                s.terminal_outcomes = BattleScenario::fixture_survival_outcomes(100000);
                s.groups[0].enemy_movement_plan = None;
                for (i, spawn) in s.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe10100 + i as u128));
                    spawn.draft.equipped_items.push(
                        crate::game::battle::types::BattleEquipmentBinding {
                            base_uuid: Uuid::from_u128(0x1102),
                            persistent_copy_uuid: Some(Uuid::from_u128(0xe10110 + i as u128)),
                            slot: None,
                            weapon_stage: Default::default(),
                        },
                    );
                }
                if prepared {
                    s.groups[0].spawns[0].draft.equipped_items =
                        vec![crate::game::battle::types::BattleEquipmentBinding {
                            base_uuid: Uuid::from_u128(0x1101),
                            persistent_copy_uuid: Some(Uuid::from_u128(0xe10120)),
                            slot: None,
                            weapon_stage: Default::default(),
                        }];
                }
            },
            |weapon| {
                if bonus > 0 {
                    use crate::game::stats::{
                        Effect, TriggerEffectTarget, TriggerType, TriggeredEffect,
                    };
                    weapon.triggered_effects.insert(
                        TriggerType::OnAttack,
                        vec![TriggeredEffect::targeted(
                            TriggerEffectTarget::CounterpartUnit,
                            Effect::BonusDamage {
                                flat: bonus,
                                percent: 0,
                                damage_type: DamageType::True,
                            },
                        )],
                    );
                }
                *saved.borrow_mut() = Some(weapon.clone());
            },
            None,
            |builder| {
                let weapon = saved.borrow_mut().take().unwrap();
                let mut extra = weapon.clone();
                extra.id = "idle_guard_extra".into();
                extra.uuid = Uuid::from_u128(0x1102);
                extra.equipment_type = EquipmentType::Armor;
                extra.weapon_profile = None;
                extra.triggered_effects.clear();
                builder
                    .with_equipment(vec![weapon, extra])
                    .with_equipment_defenses(artifact.equipment_defenses().to_vec())
            },
        );
        let target = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for u in core.units.values_mut() {
            u.stats.max_health = 1000;
            u.stats.current_health = 1000;
            u.stats.attack = 100;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
            u.stats.attack_interval_ms = 100000;
            u.next_basic_attack_ms = 100000;
            u.stats.move_speed_units_per_ms = 0;
            u.basic_attack_epoch += 1;
            u.action_locks = Default::default();
            u.basic_attack_windup_until_ms = None;
            u.pending_basic_attack = false;
        }
        core.enable_authoring_observation();
        core.bind_authoring_observation(
            crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 1,
                battle_uuid: Uuid::from_u128(0xe101),
            },
        )
        .unwrap();
        core.runtime_transaction(|c| c.reconcile_sustained(0))
            .unwrap();
        Fixture {
            core,
            state,
            caster,
            target,
            ally,
        }
    }
    #[test]
    fn composition_idle_hp_guard_multiple_ready_copies_choose_strongest_without_banked_hits() {
        let mut f = paired_fixture(
            json!({"IdleHpGuard":{"recharge_ms":2000,"reduction_percent":75}}),
            false,
            0,
        );
        assert_eq!(damage(&mut f, 100, 2000), 25);
        assert_eq!(
            witness(&f).equipment.definition_uuid,
            Uuid::from_u128(0x1102)
        );
        assert_eq!(
            damage(&mut f, 100, 2000),
            100,
            "other ready copy belongs to same packet"
        );
        assert_eq!(damage(&mut f, 100, 4000), 25);
    }
    #[test]
    fn composition_idle_hp_guard_deferral_uses_reduced_packet_and_repayments_reset_charge() {
        let mut f = paired_fixture(
            json!({"DeferredDamage":{"defer_percent":30,"duration_ms":5000,"tick_ms":1000,"kill_clear_percent":50}}),
            false,
            0,
        );
        assert_eq!(damage(&mut f, 100, 2000), 35);
        let balance = |f: &Fixture| {
            f.core
                .equipment_defense
                .deferred
                .values()
                .filter(|d| d.owner == f.caster)
                .map(|d| d.remaining)
                .sum::<u32>()
        };
        assert_eq!(balance(&f), 15);
        f.core
            .step_battle_execution_until(&mut f.state, 7000)
            .unwrap();
        assert_eq!(balance(&f), 0);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 950);
        // Last actual repayment was at7000; no second mitigation on debt, and no charge since2000.
        assert_eq!(damage(&mut f, 100, 8999), 70);
        assert_eq!(witness(&f).time_ms, 2000);
    }
    #[test]
    fn composition_idle_hp_guard_prepared_primary_kill_uses_reduced_base_not_appended_bonus() {
        use crate::game::battle::{
            event_log::SkillCastTarget,
            tile_range::{FacingDirection, TileRangePolicy},
        };
        for (hp, bonus, expected) in [(120, 0, 20), (150, 100, 0), (125, 100, 20)] {
            // Second provider does not change the expected strongest50% guard.
            let mut f = paired_fixture(
                json!({"IdleHpGuard":{"recharge_ms":2000,"reduction_percent":25}}),
                true,
                bonus,
            );
            let e = f.target;
            let a = f.caster;
            super::super::compiled_action_fixture::place(&mut f.core, a, 5, 1);
            super::super::compiled_action_fixture::place(&mut f.core, f.ally, 0, 3);
            let u = f.core.units.get_mut(&e).unwrap();
            u.skill_id = Some("r11_aura".into());
            u.resonance_current = 100;
            u.resonance_max = 100;
            u.next_action_time = 0;
            u.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
            u.basic_attack.delivery = crate::game::ability::DeliveryDef::Instant;
            u.basic_attack.windup_ms = 1;
            // An unblocked route melee enemy only attacks the objective. This probe
            // needs the ordinary ranged-enemy target selection for the armored employee.
            u.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
            u.basic_attack.ranged_reposition_ms = 0;
            u.current_target = Some(a);
            f.core
                .runtime_transaction(|c| {
                    c.start_boss_skill_action(
                        2000,
                        e,
                        &"r11_aura".into(),
                        SkillCastTarget::Unit {
                            unit_instance_id: e,
                        },
                        FacingDirection::Left,
                    )
                    .map(|_| ())
                })
                .unwrap();
            f.core
                .step_battle_execution_until(&mut f.state, 2001)
                .unwrap();
            let prepared = f.core.sustained_effect_snapshot().unwrap();
            if prepared.prepared_basics.len() != 1 {
                eprintln!("prepared diagnostic {}",serde_json::to_string(&json!({"sustained":prepared,"logs":f.core.event_log_entries_after_seq(0),"facts":f.core.runtime_fact_view(),"queue":f.core.event_queue_debug_snapshot(),"enemy":format!("{:?}",f.core.units[&e])})).unwrap());
            }
            assert_eq!(prepared.prepared_basics.len(), 1);
            // Boss admission intentionally does not spend employee resonance.
            // Isolate the kill reward from that different resource policy.
            f.core.units.get_mut(&e).unwrap().resonance_current = 0;
            f.core.units.get_mut(&a).unwrap().stats.current_health = hp;
            assert_eq!(f.core.select_basic_attack_target(e, Some(a), None), Some(a));
            f.core.units.get_mut(&e).unwrap().next_basic_attack_ms = 2100;
            f.core.event_queue.push(BattleEvent::AttackStart {
                time_ms: 2100,
                attacker_instance_id: e,
                target_instance_id: Some(a),
                schedule_next: true,
                cause: Default::default(),
            });
            f.core
                .step_battle_execution_until(&mut f.state, 2101)
                .unwrap();
            assert!(
                f.core.units[&a].lifecycle.is_dead(),
                "hp{hp} bonus{bonus}; victim={:?}; logs={:?}; queue={:?}",
                f.core.units[&a],
                f.core.event_log_entries_after_seq(9),
                f.core.event_queue_debug_snapshot()
            );
            assert_eq!(witness(&f).reduction_percent, 50);
            // Bosses also gain ordinary damage/kill resonance. Observe the authored
            // command separately so this assertion measures only prepared-kill credit.
            let reward = &f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_sourced_resonance;
            if expected == 0 {
                assert!(reward.is_none(), "hp{hp} bonus{bonus}: {reward:?}");
            } else {
                let reward = reward.as_ref().unwrap();
                assert_eq!(reward.target_unit, e);
                assert_eq!(reward.time_ms, 2101);
                let crate::game::battle::core::authoring_observation::AppliedChange::Resonance {
                    before,
                    after,
                    ..
                } = &reward.change
                else {
                    panic!("{reward:?}")
                };
                assert_eq!(*after - *before, expected, "hp{hp} bonus{bonus}");
                assert!(f.core.units[&e].resonance_current >= *after);
            }
        }
    }
}
