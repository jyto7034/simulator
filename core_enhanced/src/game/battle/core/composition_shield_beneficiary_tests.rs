mod composition_shield_beneficiary {
    use super::*;
    use crate::game::{ability::sustained::SustainedEffectDef, battle::damage::DamageModifiers};
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/shield_beneficiary.skill.yaml"
    ));

    fn setup(percent: u32) -> Fixture {
        let source = SOURCE
            .replace("porccubus_pleasure_ring", "r11_weapon")
            .replace("damage_percent: 20", &format!("damage_percent: {percent}"));
        let mut f = fixture_extra_with_defense(
            "Damage(amount:100,damage_type:True)",
            false,
            None,
            false,
            &source,
            |_| {},
            |_| {},
        );
        f.core
            .items
            .retain(|_, item| item.owner_unit_instance != f.ally);
        for unit in f.core.units.values_mut() {
            unit.stats.max_health = 10000;
            unit.stats.current_health = 10000;
            unit.stats.defense = 0;
            unit.stats.magic_resist = 0;
        }
        f
    }
    fn grant(f: &mut Fixture, provider: UnitInstanceId, target: UnitInstanceId, shared: bool) {
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    provider,
                    target,
                    SustainedEffectDef::Shield {
                        duration_ms: 100,
                        amount: 50,
                        priority: 0,
                        shared_radius_units: shared.then_some(3_000_000),
                    },
                    1,
                )
            })
            .unwrap();
    }
    fn hit(f: &mut Fixture, actor: UnitInstanceId, now: u64) -> u32 {
        f.core
            .runtime_transaction(|c| {
                let snapshot = c
                    .damage_source_snapshot_for_unit(
                        actor,
                        f.target,
                        DamageSource::Ability,
                        DamageType::True,
                        100,
                        DamageModifiers::default(),
                        0,
                        now,
                        false,
                    )
                    .unwrap();
                let before = c.units[&f.target].stats.current_health;
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: f.target,
                        source_snapshot: snapshot,
                    }],
                    now,
                );
                crate::game::battle::runtime_control::poll()?;
                Ok(before - c.units[&f.target].stats.current_health)
            })
            .unwrap()
    }
    fn apply(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        source: DamageSource,
        kind: DamageType,
        now: u64,
    ) -> Result<u32, crate::game::behavior::GameError> {
        let snapshot = core
            .damage_source_snapshot_for_unit(
                actor,
                target,
                source,
                kind,
                100,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .unwrap();
        let before = core.units[&target].stats.current_health;
        core.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: target,
                source_snapshot: snapshot,
            }],
            now,
        );
        crate::game::battle::runtime_control::poll()?;
        Ok(before - core.units[&target].stats.current_health)
    }
    fn state(c: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
            serde_json::to_value(&c.authoring_observation).unwrap(),
        )
    }
    #[test]
    fn shield_beneficiary_source_pool_and_authored_rate() {
        for (percent, damage) in [(20, 120), (35, 135)] {
            let mut f = setup(percent);
            let a = f.caster;
            let b = f.ally;
            assert_eq!(hit(&mut f, b, 1), 100);
            grant(&mut f, a, b, false);
            assert_eq!(hit(&mut f, b, 2), damage);
            assert_eq!(
                f.core
                    .sustained_effect_snapshot()
                    .unwrap()
                    .active
                    .iter()
                    .filter_map(|r| r.shield)
                    .map(|p| p.remaining)
                    .sum::<u32>(),
                50
            );
            let w = f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_shield_beneficiary_damage
                .as_ref()
                .unwrap();
            assert_eq!(
                (w.provider, w.beneficiary, w.percent, w.shield_remaining),
                (a, b, percent, 50)
            );
        }
    }
    #[test]
    fn shield_beneficiary_self_and_exact_expiry() {
        let mut f = setup(20);
        let a = f.caster;
        grant(&mut f, a, a, false);
        assert_eq!(hit(&mut f, a, 100), 120);
        assert_eq!(hit(&mut f, a, 101), 100);
    }
    #[test]
    fn shield_beneficiary_does_not_use_recipient_gear_or_shared_pool() {
        let mut f = setup(20);
        let a = f.caster;
        let b = f.ally;
        // The ally has no matching support equipment. Recipient gear cannot own its grant.
        grant(&mut f, b, a, false);
        assert_eq!(hit(&mut f, a, 2), 100);
        let mut f = setup(20);
        let a = f.caster;
        grant(&mut f, a, a, true);
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .any(|r| r.members.contains(&a)
                && r.shield
                    .is_some_and(|s| s.remaining == 50 && s.shared_radius_units.is_some())));
        assert_eq!(hit(&mut f, a, 2), 100);
    }

    #[test]
    fn shield_beneficiary_individual_depletion_keeps_other_recipient_bonus() {
        let mut f = setup(20);
        let (a, b, t) = (f.caster, f.ally, f.target);
        grant(&mut f, a, a, false);
        grant(&mut f, a, b, false);
        assert_eq!(hit(&mut f, b, 2), 120);
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, t, b, DamageSource::Ability, DamageType::True, 3))
                .unwrap(),
            50
        );
        assert_eq!(hit(&mut f, b, 4), 100);
        assert_eq!(hit(&mut f, a, 4), 120);
        assert_eq!(
            f.core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .filter_map(|r| r.shield)
                .map(|p| p.remaining)
                .sum::<u32>(),
            50
        );
    }

    #[test]
    fn shield_beneficiary_live_equipment_removal_and_reequip_do_not_change_pool() {
        let mut f = setup(20);
        let (a, b) = (f.caster, f.ally);
        grant(&mut f, a, b, false);
        assert_eq!(hit(&mut f, b, 2), 120);
        let equipped = f.core.items.clone();
        f.core.items.clear();
        assert_eq!(hit(&mut f, b, 3), 100);
        f.core.items = equipped;
        assert_eq!(hit(&mut f, b, 4), 120);
        assert_eq!(
            f.core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .filter_map(|r| r.shield)
                .map(|p| p.remaining)
                .sum::<u32>(),
            50
        );
    }

    #[test]
    fn shield_beneficiary_provider_and_recipient_liveness_are_separate() {
        for end in ["corrosion", "death", "withdraw"] {
            let mut f = setup(20);
            let (a, b) = (f.caster, f.ally);
            f.core
                .runtime_transaction(|c| {
                    c.begin_sustained(
                        a,
                        b,
                        SustainedEffectDef::IndependentShield {
                            duration_ms: 100,
                            amount: 50,
                            priority: 0,
                        },
                        1,
                    )
                })
                .unwrap();
            assert_eq!(hit(&mut f, b, 2), 120);
            match end {
                "corrosion" => f.core.units.get_mut(&a).unwrap().corroded = true,
                "death" => f.core.units.get_mut(&a).unwrap().stats.current_health = 0,
                _ => {
                    f.core
                        .apply_live_command(BattleLiveCommand::WithdrawUnit {
                            unit_id: a,
                            time_ms: 3,
                        })
                        .unwrap();
                }
            }
            assert_eq!(hit(&mut f, b, 4), 100, "{end}");
        }
        let mut f = setup(20);
        let (a, b) = (f.caster, f.ally);
        grant(&mut f, a, b, false);
        f.core.units.get_mut(&b).unwrap().corroded = true;
        assert_eq!(hit(&mut f, b, 2), 100);
    }

    #[test]
    fn shield_beneficiary_normal_channels_types_and_environment_boundary() {
        for source in [
            DamageSource::BasicAttack,
            DamageSource::Ability,
            DamageSource::BuffTick,
            DamageSource::Environment,
        ] {
            for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
                let mut f = setup(35);
                let (a, b, t) = (f.caster, f.ally, f.target);
                grant(&mut f, a, b, false);
                let expected = if source == DamageSource::Environment {
                    100
                } else {
                    135
                };
                assert_eq!(
                    f.core
                        .runtime_transaction(|c| apply(c, b, t, source, kind, 2))
                        .unwrap(),
                    expected,
                    "{source:?}/{kind:?}"
                );
            }
        }
    }

    #[test]
    fn shield_beneficiary_zero_and_bounds_roundtrip_and_source_diagnostic() {
        for rate in [0, 20, 10000] {
            let source = SOURCE.replace("damage_percent: 20", &format!("damage_percent: {rate}"));
            let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
            Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        }
        let err = crate::authoring::composition::compile_yaml(
            &SOURCE.replace("damage_percent: 20", "damage_percent: 10001"),
        )
        .unwrap_err();
        let detail = format!("{err:?}");
        assert!(
            detail.contains("shield_beneficiary_damage_percent"),
            "{detail}"
        );
        let mut f = setup(0);
        let (a, b) = (f.caster, f.ally);
        grant(&mut f, a, b, false);
        assert_eq!(hit(&mut f, b, 2), 100);
    }

    #[test]
    fn shield_beneficiary_cancel_after_evaluation_and_damage_restores_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "shield_beneficiary_evaluated",
            "shield_beneficiary_test_after_damage",
        ] {
            let mut f = setup(20);
            let (a, b, t) = (f.caster, f.ally, f.target);
            grant(&mut f, a, b, false);
            let before = state(&f.core);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        f.core.runtime_transaction(|c| {
                            let damage =
                                apply(c, b, t, DamageSource::Ability, DamageType::True, 2)?;
                            if !token.is_cancelled() {
                                assert_eq!(damage, 120);
                            }
                            control::poll("shield_beneficiary_test_after_damage")?;
                            Ok(damage)
                        })
                    },
                )
            });
            assert!(
                result.is_err() && token.is_cancelled(),
                "{site}: {result:?}"
            );
            assert_eq!(state(&f.core), before);
            assert_eq!(hit(&mut f, b, 2), 120);
        }
    }

    #[test]
    fn shield_beneficiary_retained_refusal_restores_then_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(20);
        let (a, b, t) = (f.caster, f.ally, f.target);
        grant(&mut f, a, b, false);
        let action =
            |c: &mut BattleCore| apply(c, b, t, DamageSource::Ability, DamageType::True, 2);
        let mut probe = f.core.clone();
        let (result, admissions) =
            native::observe_retained_admissions(|| probe.runtime_transaction(action));
        assert_eq!(result.unwrap(), 120);
        let required = admissions
            .iter()
            .find(|(s, _, _)| *s == "shield_beneficiary_observation")
            .unwrap()
            .2;
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = state(&f.core);
        assert!(f.core.runtime_transaction(action).is_err());
        assert_eq!(state(&f.core), before);
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        assert_eq!(f.core.runtime_transaction(action).unwrap(), 120);
    }

    #[test]
    fn shield_beneficiary_observer_disabled_clone_keeps_authoritative_outcome() {
        let mut f = setup(35);
        let (a, b, t) = (f.caster, f.ally, f.target);
        grant(&mut f, a, b, false);
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        for (now, expected) in [(2, 135), (100, 135), (101, 100)] {
            assert_eq!(hit(&mut f, b, now), expected);
            assert_eq!(
                plain
                    .runtime_transaction(|c| apply(
                        c,
                        b,
                        t,
                        DamageSource::Ability,
                        DamageType::True,
                        now
                    ))
                    .unwrap(),
                expected
            );
            assert_eq!(
                f.core.transaction_debug_snapshot(),
                plain.transaction_debug_snapshot()
            );
            assert_eq!(
                f.core.event_queue_debug_snapshot(),
                plain.event_queue_debug_snapshot()
            );
            assert_eq!(
                serde_json::to_value(f.core.runtime_fact_summary()).unwrap(),
                serde_json::to_value(plain.runtime_fact_summary()).unwrap()
            );
        }
    }
    #[test]
    fn shield_beneficiary_mixed_components_and_guard_credit_use_only_owned_bonus() {
        use crate::game::{
            battle::{
                cooldown::{CooldownSource, SourcedEffect},
                damage::{
                    calculate_damage_with_source_support, reduction_credit_with_source_support,
                    DamageContext, DamageRequest,
                },
            },
            stats::{Effect, TriggerEffectTarget},
        };
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id: Uuid::from_u128(10).into(),
            target_id: Uuid::from_u128(20).into(),
            base_damage: 100,
            minimum_damage: 1,
            time_ms: 0,
        };
        let bonus = |actor, damage_type| SourcedEffect {
            source: CooldownSource::Unit {
                unit_instance_id: actor,
            },
            equipment_source: None,
            target: TriggerEffectTarget::CounterpartUnit,
            effect: Effect::BonusDamage {
                flat: 100,
                percent: 0,
                damage_type,
            },
        };
        let attack = [bonus(request.attacker_id, DamageType::Magic)];
        let on_hit = [
            bonus(request.target_id, DamageType::True),
            bonus(Uuid::from_u128(30).into(), DamageType::Physical),
        ];
        let ctx = DamageContext {
            attacker_side: Side::Opponent,
            target_side: Side::Player,
            attacker_attack: 100,
            target_armor: 0,
            target_magic_resist: 0,
            target_incoming_modifiers: DamageModifiers {
                damage_reduction_percent: 30,
                ..Default::default()
            },
            target_current_hp: 1000,
            target_max_hp: 1000,
            on_attack_effects: &attack,
            on_hit_effects: &on_hit,
        };
        let mut flat = 10;
        let support = crate::game::battle::damage::AttackerDamageSupport {
            all_types_percent: 20,
            ..Default::default()
        };
        let result = calculate_damage_with_source_support(&request, &ctx, &mut flat, 0, support);
        assert_eq!(flat, 0);
        assert_eq!(
            result
                .breakdown
                .iter()
                .map(|b| b.final_damage)
                .collect::<Vec<_>>(),
            [75, 84, 70, 70]
        );
        assert_eq!(result.final_damage, 299);
        // Removing the guard leaves108+120+100+100=428; its own credit is129.
        assert_eq!(
            reduction_credit_with_source_support(
                &request,
                &ctx,
                &result,
                DamageModifiers::default(),
                10,
                0,
                support
            ),
            129
        );
        let support = crate::game::battle::damage::AttackerDamageSupport {
            all_types_percent: 20,
            per_type_percent: [20, 40, 0],
            ..Default::default()
        };
        let result = calculate_damage_with_source_support(&request, &ctx, &mut 10, 0, support);
        // Source-owned Physical gets40%, Magic60%; other actors remain unchanged.
        assert_eq!(
            result
                .breakdown
                .iter()
                .map(|b| b.final_damage)
                .collect::<Vec<_>>(),
            [88, 112, 70, 70]
        );
        assert_eq!(result.final_damage, 340);
        // Without the guard:126+160+100+100=486, so only146 was prevented.
        assert_eq!(
            reduction_credit_with_source_support(
                &request,
                &ctx,
                &result,
                DamageModifiers::default(),
                10,
                0,
                support
            ),
            146
        );
    }

    #[test]
    fn shield_beneficiary_segmented_meat_shield_is_not_personal_hp_support() {
        let mut f = setup(20);
        let a = f.caster;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    a,
                    a,
                    SustainedEffectDef::SegmentedShield {
                        duration_ms: 100,
                        pool: crate::game::ability::sustained::SegmentedShieldDef {
                            pool_id: "meat".into(),
                            segment_health: 25,
                            max_segments: 2,
                            initial_health: 50,
                            block_per_segment: 1,
                            priority: 0,
                        },
                    },
                    1,
                )
            })
            .unwrap();
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .any(|r| r.segmented.is_some()));
        assert_eq!(hit(&mut f, a, 2), 100);
    }

    #[test]
    fn shield_beneficiary_checked_creator_and_reaction_add_to_existing_damage_increase() {
        use crate::game::battle::runtime_facts::{
            IndependentObjectOriginSnapshot, RuntimeFactSource,
        };
        let object = UnitInstanceId(Uuid::from_u128(0x8989));
        for (direct, matching, reaction, expected) in [
            (true, false, false, 145),
            (false, true, false, 145),
            (false, false, false, 125),
            (true, false, true, 145),
        ] {
            let mut f = setup(20);
            let (a, b, t) = (f.caster, f.ally, f.target);
            grant(&mut f, a, b, false);
            let before = f.core.units[&t].stats.current_health;
            f.core
                .runtime_transaction(|c| {
                    let mut shot = c
                        .damage_source_snapshot_for_unit(
                            b,
                            t,
                            if reaction {
                                DamageSource::Environment
                            } else {
                                DamageSource::Ability
                            },
                            DamageType::True,
                            100,
                            DamageModifiers {
                                damage_amp_percent: 25,
                                ..Default::default()
                            },
                            0,
                            2,
                            false,
                        )
                        .unwrap();
                    shot.fact_source.reaction = reaction;
                    if !direct {
                        shot.source_id = object;
                        shot.fact_source.actor = Some(object);
                    }
                    shot.fact_source.extended_observation =
                        RuntimeFactSource::extended_observation(
                            Some(std::sync::Arc::new(IndependentObjectOriginSnapshot {
                                object_id: if matching {
                                    object
                                } else {
                                    UnitInstanceId(Uuid::from_u128(0x8990))
                                },
                                definition_id:
                                    crate::game::data::independent_object_data::IndependentObjectId(
                                        "fixture-owned".into(),
                                    ),
                                creator: b,
                                creation_action: None,
                                creation_cast: None,
                                creation_fragment: None,
                                creation_equipment: None,
                                creation_authored: None,
                            })),
                            None,
                        )?;
                    c.process_commands(
                        vec![BattleCommand::ApplyDamage {
                            target_id: t,
                            source_snapshot: shot,
                        }],
                        2,
                    );
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            assert_eq!(before - f.core.units[&t].stats.current_health, expected);
        }
    }

    #[test]
    fn shield_beneficiary_multiple_copies_choose_strongest_then_fall_back() {
        check_copies(35);
    }
    #[test]
    fn shield_beneficiary_equal_copies_choose_lowest_runtime_uuid() {
        check_copies(20);
    }
    fn check_copies(second_percent: u32) {
        let catalog: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let mut weapon = catalog
            .get_by_id("standard_training_sword")
            .unwrap()
            .clone();
        weapon.id = "r11_weapon".into();
        weapon.uuid = Uuid::from_u128(0x1101);
        weapon.triggered_effects.clear();
        weapon.ability_activations.clear();
        let ring = catalog
            .get_by_id("porccubus_pleasure_ring")
            .unwrap()
            .clone();
        let ring_id = ring.uuid;
        let mut doc = parse_yaml(&SOURCE.replace("porccubus_pleasure_ring", "r11_weapon")).unwrap();
        let mut other = doc["values"][0].clone();
        other["name"] = json!("other");
        other["value"]["SupportProgram"]["shield_beneficiary_damage_percent"] =
            json!(second_percent);
        doc["values"].as_array_mut().unwrap().push(other);
        doc["equipment_supports"]
            .as_array_mut()
            .unwrap()
            .push(json!({"equipment":"porccubus_pleasure_ring","program":{"Shared":"other"}}));
        let artifact = compile_json(&doc.to_string()).unwrap();
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &r12_control_ron("Damage(amount:100,damage_type:True)"),
            |scenario| {
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let mut second = spawn.draft.equipped_items[0];
                    second.base_uuid = ring_id;
                    second.persistent_copy_uuid = Some(Uuid::from_u128(0xe890 + i as u128));
                    spawn.draft.equipped_items.push(second);
                }
            },
            |_| {},
            None,
            |builder| {
                builder
                    .with_equipment(vec![weapon, ring])
                    .with_equipment_supports(artifact.equipment_supports().to_vec())
            },
        );
        let target = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for u in core.units.values_mut() {
            u.stats.current_health = 10000;
            u.stats.max_health = 10000;
        }
        core.enable_authoring_observation();
        let mut f = Fixture {
            core,
            state,
            caster,
            target,
            ally,
        };
        grant(&mut f, caster, ally, false);
        assert_eq!(hit(&mut f, ally, 2), 100 + second_percent);
        let chosen = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_shield_beneficiary_damage
            .as_ref()
            .unwrap()
            .equipment;
        if second_percent == 20 {
            let least = f
                .core
                .items
                .iter()
                .filter(|(_, i)| i.owner_unit_instance == caster)
                .map(|(id, _)| *id)
                .min()
                .unwrap();
            assert_eq!(chosen.runtime_item_instance_id, least);
        } else {
            assert_eq!(chosen.definition_uuid, ring_id);
            f.core.items.remove(&chosen.runtime_item_instance_id);
            assert_eq!(hit(&mut f, ally, 3), 120);
        }
    }
}
