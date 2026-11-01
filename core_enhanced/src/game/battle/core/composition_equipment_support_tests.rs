mod composition_equipment_support {
    use super::*;
    use crate::game::battle::core::equipment_support::SupportChannel;
    use crate::game::battle::event_log::HpChangeReason;
    const SUPPORT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_support.skill.yaml"
    ));
    fn setup() -> Fixture {
        fixture_extra_with_defense(
            "Heal(amount:19)",
            false,
            None,
            false,
            SUPPORT,
            |_| {},
            |_| {},
        )
    }
    fn heal(f: &mut Fixture, source: UnitInstanceId, target: UnitInstanceId, flat: i32) {
        f.core
            .runtime_transaction(|c| {
                c.process_commands(
                    vec![crate::game::battle::damage::BattleCommand::ApplyHeal {
                        target_id: target,
                        flat,
                        percent: 0,
                        source_id: Some(source),
                    }],
                    1,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
    }
    fn shield(f: &mut Fixture, source: UnitInstanceId, target: UnitInstanceId, independent: bool) {
        use crate::game::ability::sustained::SustainedEffectDef;
        let definition = if independent {
            SustainedEffectDef::IndependentShield {
                duration_ms: 100,
                amount: 19,
                priority: 0,
            }
        } else {
            SustainedEffectDef::Shield {
                duration_ms: 100,
                amount: 19,
                priority: 0,
                shared_radius_units: None,
            }
        };
        f.core
            .runtime_transaction(|c| c.begin_sustained(source, target, definition, 1))
            .unwrap();
    }
    fn witness(
        f: &Fixture,
    ) -> &crate::game::battle::core::equipment_support::EquipmentSupportWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_support
            .as_ref()
            .unwrap()
    }
    #[test]
    fn composition_equipment_support_authored_roundtrip_shared_origin_and_bounds() {
        let doc = parse_yaml(SUPPORT).unwrap();
        let artifact = compile_json(&doc.to_string()).unwrap();
        let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(loaded.equipment_supports()[0].program.heal_percent, 25);
        assert!(loaded
            .sources()
            .get("equipment_supports[0].program")
            .unwrap()
            .definition
            .ends_with("SupportProgram"));
        for n in [0, 10000] {
            let mut d = doc.clone();
            d["values"][0]["value"]["SupportProgram"]["heal_percent"] = json!(n);
            compile_json(&d.to_string()).unwrap();
        }
        for field in ["heal_percent", "shield_percent"] {
            let mut d = doc.clone();
            d["values"][0]["value"]["SupportProgram"][field] = json!(10001);
            assert!(compile_json(&d.to_string())
                .unwrap_err()
                .source
                .definition
                .ends_with(field));
        }
        let mut duplicate = doc.clone();
        duplicate["equipment_supports"]
            .as_array_mut()
            .unwrap()
            .push(doc["equipment_supports"][0].clone());
        assert!(compile_json(&duplicate.to_string()).is_err());
        let mut unresolved = doc;
        unresolved["equipment_supports"][0]["program"] = json!({"Unresolved":"pending"});
        assert!(compile_json(&unresolved.to_string()).is_err());
    }
    #[test]
    fn composition_equipment_support_heal_floor_cap_and_source_ownership() {
        let mut f = setup();
        let source = f.caster;
        let target = f.ally;
        f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
        // Capture authority before the operation; do not call the production
        // source_snapshot helper to build the expected observation.
        let item = f.core.items.values().find(|i| i.owner_unit_instance == source).unwrap();
        let expected_equipment = crate::game::battle::runtime_facts::EquipmentSourceSnapshot {
            definition_uuid: Uuid::from_u128(0x1101),
            runtime_item_instance_id: item.instance_id,
            persistent_copy_uuid: item.persistent_copy_uuid,
        };
        heal(&mut f, source, target, 19);
        assert_eq!(f.core.units[&target].stats.current_health, 923);
        assert_eq!(
            (
                witness(&f).base_amount,
                witness(&f).increased_amount,
                witness(&f).percent
            ),
            (19, 23, 25)
        );
        assert_eq!(
            witness(&f).contributors[0].equipment,
            expected_equipment
        );
        f.core.units.get_mut(&target).unwrap().stats.current_health = 990;
        heal(&mut f, source, target, 19);
        assert_eq!(f.core.units[&target].stats.current_health, 1000);
        assert_eq!(
            f.core
                .runtime_fact_summary()
                .latest_heal
                .as_ref()
                .unwrap()
                .requested_delta,
            23
        );
    }
    #[test]
    fn composition_equipment_support_self_enemy_negative_and_unsourced_excluded() {
        let mut f = setup();
        let source = f.caster;
        let enemy = f.target;
        for target in [source, enemy] {
            f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
            heal(&mut f, source, target, 19);
            assert_eq!(f.core.units[&target].stats.current_health, 919);
        }
        let target = f.ally;
        f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
        heal(&mut f, source, target, -19);
        assert_eq!(f.core.units[&target].stats.current_health, 881);
        f.core
            .apply_hp_delta_and_record(Some(source), target, 19, 1, HpChangeReason::Command);
        assert_eq!(
            f.core.units[&target].stats.current_health, 900,
            "generic HP restoration is not a direct heal"
        );
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_support
            .is_none());
    }
    #[test]
    fn composition_equipment_support_corrosion_inactive_and_object_excluded() {
        for boundary in 0..4 {
            let mut f = setup();
            let source = f.caster;
            let target = f.ally;
            f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
            match boundary {
                0 => f.core.units.get_mut(&source).unwrap().corroded = true,
                1 => f.core.units.get_mut(&target).unwrap().corroded = true,
                2 => {
                    f.core.units.get_mut(&source).unwrap().lifecycle =
                        crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
                }
                _ => {
                    f.core.units.get_mut(&target).unwrap().role =
                        crate::game::battle::types::BattleUnitRole::IndependentWall
                }
            }
            heal(&mut f, source, target, 19);
            assert_eq!(
                f.core.units[&target].stats.current_health,
                if boundary == 3 { 900 } else { 919 }
            );
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_equipment_support
                .is_none());
        }
    }
    #[test]
    fn composition_equipment_support_only_source_equipment_and_additive_copies() {
        let mut f = setup();
        let source = f.caster;
        let target = f.ally;
        f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
        let owned: Vec<_> = f
            .core
            .items
            .values()
            .filter(|i| i.owner_unit_instance == source)
            .map(|i| i.instance_id)
            .collect();
        let mut item = f.core.items.remove(&owned[0]).unwrap();
        heal(&mut f, source, target, 19);
        assert_eq!(
            f.core.units[&target].stats.current_health, 919,
            "recipient's equipment must not amplify incoming healing"
        );
        let first_runtime = item.instance_id;
        item.persistent_copy_uuid = Some(Uuid::from_u128(0xabc41));
        f.core.items.insert(item.instance_id, item.clone());
        item.instance_id = Uuid::from_u128(0xabc43);
        item.persistent_copy_uuid = Some(Uuid::from_u128(0xabc42));
        f.core.items.insert(item.instance_id, item);
        heal(&mut f, source, target, 20);
        assert_eq!(
            f.core.units[&target].stats.current_health, 949,
            "20 with +50% is 30; sequential multiplication would give 31"
        );
        assert_eq!(witness(&f).contributors.len(), 2);
        assert_eq!(witness(&f).percent, 50);
        let expected = [
            (first_runtime, Some(Uuid::from_u128(0xabc41))),
            (Uuid::from_u128(0xabc43), Some(Uuid::from_u128(0xabc42))),
        ];
        let matches_input = |contributors: &[crate::game::battle::core::equipment_support::SupportContribution]| {
            // Legacy fixture providers are a hash map. Require a bijection of
            // complete identities without inventing an order for this sum.
            contributors.len() == expected.len() && expected.iter().all(|(runtime, copy)| {
                contributors.iter().filter(|c| {
                c.equipment.definition_uuid == Uuid::from_u128(0x1101)
                    && c.equipment.runtime_item_instance_id == *runtime
                    && c.equipment.persistent_copy_uuid == *copy
                    && c.percent == 25 && c.definition_id == "r11_weapon"
                    && c.field_path == "equipment_supports[0].program.heal_percent"
                }).count() == 1
            })
        };
        let actual = &witness(&f).contributors;
        assert!(matches_input(actual), "actual contributions must match distinct input copies: {actual:?}");
        // Verifier counterexamples, not claims about a supported corruption path.
        let mut mixed = actual.clone();
        mixed[0].equipment.persistent_copy_uuid = actual[1].equipment.persistent_copy_uuid;
        mixed[1].equipment.persistent_copy_uuid = actual[0].equipment.persistent_copy_uuid;
        assert!(!matches_input(&mixed), "copy/runtime cross-wiring must be detected");
        mixed[0] = actual[1].clone();
        mixed[1] = actual[1].clone();
        assert!(!matches_input(&mixed), "two copies of one witness must be detected");
        assert!(!matches_input(&actual[..1]), "missing contributor must be detected");
        mixed = actual.clone();
        mixed[0].field_path = "equipment_supports[0].program.shield_percent".into();
        assert!(!matches_input(&mixed), "another valid channel must not pass as heal");
        assert!(matches_input(actual), "negative checks must not change the real observation");
    }
    #[test]
    fn composition_equipment_support_new_shields_and_self_exclusion() {
        for independent in [false, true] {
            let mut f = setup();
            let source = f.caster;
            let target = f.ally;
            shield(&mut f, source, target, independent);
            let snapshot = f.core.sustained_effect_snapshot().unwrap();
            let pool = snapshot
                .active
                .iter()
                .find(|e| e.target == target)
                .unwrap()
                .shield
                .as_ref()
                .unwrap();
            assert_eq!((pool.capacity, pool.remaining), (23, 23));
            assert_eq!(witness(&f).channel, SupportChannel::Shield);
            shield(&mut f, source, source, independent);
            let snapshot = f.core.sustained_effect_snapshot().unwrap();
            let pool = snapshot
                .active
                .iter()
                .find(|e| e.target == source)
                .unwrap()
                .shield
                .as_ref()
                .unwrap();
            assert_eq!((pool.capacity, pool.remaining), (19, 19));
        }
    }
    #[test]
    fn composition_equipment_support_cancellation_restores_heal_shield_and_observation() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        let source = f.caster;
        let target = f.ally;
        f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
        let before = f.core.transaction_debug_snapshot();
        let observation = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("direct_heal_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core.runtime_transaction(|c| {
                        c.apply_direct_heal_and_record(Some(source), target, 19, 1)
                    })
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            observation
        );
        heal(&mut f, source, target, 19);
        assert_eq!(f.core.units[&target].stats.current_health, 923);
    }
    #[test]
    fn composition_equipment_support_observation_does_not_change_authority() {
        let mut observed = setup();
        let source = observed.caster;
        let target = observed.ally;
        observed
            .core
            .units
            .get_mut(&target)
            .unwrap()
            .stats
            .current_health = 900;
        let mut plain = observed.core.clone();
        plain.authoring_observation = None;
        plain
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(source), target, 19, 1))
            .unwrap();
        heal(&mut observed, source, target, 19);
        assert_eq!(
            plain.transaction_debug_snapshot(),
            observed.core.transaction_debug_snapshot()
        );
        assert_eq!(
            plain
                .runtime_fact_summary()
                .latest_heal
                .as_ref()
                .unwrap()
                .hp_after,
            923
        );
    }
    #[test]
    fn composition_equipment_support_retained_failure_restores_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        let source = f.caster;
        let target = f.ally;
        f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| {
                c.apply_direct_heal_and_record(Some(source), target, 19, 1)
            })
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "equipment_support_observation")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let error = f
            .core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(source), target, 19, 1))
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.runtime_facts.limits = Default::default();
        heal(&mut f, source, target, 19);
        assert_eq!(f.core.units[&target].stats.current_health, 923);
    }
    #[test]
    fn composition_equipment_support_shield_cancellation_preserves_prior_pool() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        let source = f.caster;
        let target = f.ally;
        shield(&mut f, source, target, true);
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_support_evaluated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core.runtime_transaction(|c|c.begin_sustained(source,target,crate::game::ability::sustained::SustainedEffectDef::IndependentShield{duration_ms:100,amount:50,priority:0},2))
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        shield(&mut f, source, target, true);
        let snapshot = f.core.sustained_effect_snapshot().unwrap();
        assert_eq!(
            snapshot
                .active
                .iter()
                .filter(|p| p.target == target)
                .count(),
            2
        );
        assert!(snapshot
            .active
            .iter()
            .filter(|p| p.target == target)
            .all(|p| p.shield.as_ref().unwrap().capacity == 23));
    }
    #[test]
    fn composition_equipment_support_large_amounts_saturate_at_channel_bounds() {
        let mut f = setup();
        let source = f.caster;
        let target = f.ally;
        f.core.units.get_mut(&target).unwrap().stats.current_health = 900;
        heal(&mut f, source, target, i32::MAX);
        assert_eq!(f.core.units[&target].stats.current_health, 1000);
        assert_eq!(
            f.core
                .runtime_fact_summary()
                .latest_heal
                .as_ref()
                .unwrap()
                .requested_delta,
            i64::from(i32::MAX)
        );
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    source,
                    target,
                    crate::game::ability::sustained::SustainedEffectDef::IndependentShield {
                        duration_ms: 100,
                        amount: u32::MAX,
                        priority: 0,
                    },
                    2,
                )
            })
            .unwrap();
        let snapshot = f.core.sustained_effect_snapshot().unwrap();
        let pool = snapshot
            .active
            .iter()
            .find(|p| p.target == target)
            .unwrap()
            .shield
            .as_ref()
            .unwrap();
        assert_eq!((pool.capacity, pool.remaining), (u32::MAX, u32::MAX));
    }
}
