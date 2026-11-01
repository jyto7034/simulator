mod composition_channel_upkeep {
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact};
    use serde_json::{json, Value};
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/channel_upkeep.skill.yaml"
    ));
    fn source() -> Value {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        doc
    }
    fn aura(doc: &mut Value) -> &mut Value {
        &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["ChannelAura"]
    }
    fn fixture(
        variant: Option<crate::game::relic::RelicVariant>,
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut doc = source();
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let native = Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap();
        let (mut c, s, a, b) = fixture_from_native_with_catalog(
            native,
            false,
            false,
            |_| {},
            |relics| {
                relics.relics[0].corruption =
                    Some(crate::game::data::relic_data::RelicCorruptionVariant {
                        effects: vec![],
                        rules: vec![],
                        penalties: vec![
                            crate::game::data::relic_data::RelicPenalty::AuthoredSupportUpkeep,
                        ],
                    });
            },
        );
        if let Some(variant) = variant {
            let data: ValidatedGameData = c.game_data.clone().into();
            let employee = c.units[&a].source_owned_uuid;
            let mut inventory = RelicInventory::default();
            let copy = inventory
                .grant(&data.relic_data.relics[0], &mut UuidManager::new(0xe360))
                .unwrap();
            if variant == crate::game::relic::RelicVariant::Corrupt {
                inventory.corrupt(copy, &data.relic_data).unwrap();
            }
            inventory
                .equip(copy, employee, 0, &data.relic_data)
                .unwrap();
            let providers = [ProspectiveProvider::Fragment(
                c.units[&a]
                    .fragment_source
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .clone(),
            )];
            let plan = compile_effective_plan(EffectivePlanInput {
                data: &data,
                employee_uuid: employee,
                inventory: &inventory,
                providers: &providers,
            })
            .unwrap()
            .unwrap();
            c.set_effective_owner(
                crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(
                    std::collections::BTreeMap::from([(employee, plan)]),
                )
                .unwrap(),
            )
            .unwrap();
        }
        (c, s, a, b)
    }
    fn corrupt(
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        fixture(Some(crate::game::relic::RelicVariant::Corrupt), edit)
    }

    #[test]
    fn composition_channel_upkeep_variants_and_current_max_hp_rounding() {
        use crate::game::relic::RelicVariant::*;
        for (variant, bonus, cost) in [
            (None, (20, 15), 0),
            (Some(Normal), (40, 20), 0),
            (Some(Corrupt), (80, 30), 100),
        ] {
            let (mut c, mut s, a, b) = fixture(variant, |_| {});
            cast(&mut c, a, variant.map(|_| b), 0);
            c.step_battle_execution_until(&mut s, 1000).unwrap();
            assert_eq!(c.aura_basic_bonus_for_occurrence(b, true).unwrap(), bonus);
            assert_eq!(c.units[&a].stats.current_health, 1000);
            c.step_battle_execution_until(&mut s, 1001).unwrap();
            assert_eq!(c.units[&a].stats.current_health, 1000 - cost);
            assert_eq!(c.units[&b].stats.current_health, 1000);
            c.units.get_mut(&a).unwrap().stats.max_health = 1001;
            c.step_battle_execution_until(&mut s, 2001).unwrap();
            assert_eq!(
                c.units[&a].stats.current_health,
                if cost == 0 { 1000 } else { 799 }
            );
            c.units.get_mut(&a).unwrap().stats.current_health = 1001;
            c.step_battle_execution_until(&mut s, 3001).unwrap();
            assert_eq!(
                c.units[&a].stats.current_health,
                if cost == 0 { 1001 } else { 900 }
            );
        }
    }
    #[test]
    fn composition_channel_upkeep_stop_expiry_target_exit_and_lethal() {
        for ending in ["stop", "expiry", "target-exit", "lethal", "stun"] {
            let (mut c, mut s, a, b) = corrupt(|doc| {
                if ending == "expiry" {
                    aura(doc)["duration_ms"] = json!(2000);
                }
            });
            cast(&mut c, a, Some(b), 0);
            c.step_battle_execution_until(&mut s, 1001).unwrap();
            assert_eq!(c.units[&a].stats.current_health, 900);
            match ending {
                "stop" => cast(&mut c, a, None, 1002),
                "target-exit" => {
                    place(&mut c, b, 7, 3);
                    c.reconcile_sustained(1002).unwrap();
                }
                "lethal" => {
                    c.units.get_mut(&a).unwrap().stats.current_health = 60;
                }
                "stun" => {
                    let enemy = c
                        .units
                        .values()
                        .find(|u| u.owner == Side::Opponent)
                        .unwrap()
                        .instance_id;
                    c.process_event_for_test(
                        BattleEvent::ApplyBuff {
                            time_ms: 1002,
                            caster_instance_id: enemy,
                            target_instance_id: a,
                            buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"),
                            duration_ms: 50,
                            cause: Default::default(),
                        },
                        1002,
                    )
                    .unwrap();
                }
                _ => (),
            }
            c.step_battle_execution_until(&mut s, 3001).unwrap();
            assert!(!c.channel_active(a), "{ending}");
            assert_eq!(
                c.units[&a].stats.current_health,
                if ending == "lethal" { 0 } else { 900 },
                "{ending}"
            );
            assert_eq!(c.aura_basic_bonus_for_occurrence(b, true).unwrap(), (0, 0));
            assert!(!c
                .event_queue_debug_snapshot()
                .iter()
                .any(|e| e.contains("ChannelUpkeep")));
            assert!(!c.resonance_use_has_obligations(a));
        }
    }
    #[test]
    fn composition_channel_upkeep_bypasses_shield_survival_and_damage_facts() {
        use crate::game::ability::sustained::{SurvivalPolicy, SustainedEffectDef as D};
        let (mut c, mut s, a, b) = corrupt(|_| {});
        cast(&mut c, a, Some(b), 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        for definition in [
            D::Shield {
                duration_ms: 5000,
                amount: 600,
                priority: 0,
                shared_radius_units: None,
            },
            D::Survival {
                duration_ms: 5000,
                policy: SurvivalPolicy::FloorOne,
            },
            D::Survival {
                duration_ms: 5000,
                policy: SurvivalPolicy::PreventDeathOnce,
            },
        ] {
            c.begin_sustained(a, a, definition, 2).unwrap();
        }
        c.step_battle_execution_until(&mut s, 1001).unwrap();
        assert_eq!(c.units[&a].stats.current_health, 900);
        assert_eq!(
            c.sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .filter(|r| r.target == a)
                .filter_map(|r| r.shield.map(|s| s.remaining))
                .collect::<Vec<_>>(),
            vec![600]
        );
        let facts = c.runtime_fact_summary();
        assert_eq!(facts.latest_cost.as_ref().unwrap().actual_spent(), 100);
        // The supported ally legitimately attacks between payment ticks. Compare
        // only the next HP-cost boundary, not all damage over that full second.
        c.step_battle_execution_until(&mut s, 2000).unwrap();
        let facts = c.runtime_fact_summary();
        let damage_before = facts.damage_hp_lost;
        let heals_before = facts.heal_mutations;
        c.units.get_mut(&a).unwrap().stats.current_health = 50;
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert!(c.units[&a].lifecycle.is_dead());
        let facts = c.runtime_fact_summary();
        assert_eq!(
            (facts.damage_hp_lost, facts.heal_mutations),
            (damage_before, heals_before)
        );
        assert!(facts.latest_death.as_ref().unwrap().lethal_damage.is_none());
        assert_eq!(facts.latest_cost.as_ref().unwrap().actual_spent(), 50);
    }
    #[test]
    fn composition_channel_upkeep_cancel_payment_restores_hp_queue_and_source() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut c, mut s, a, b) = corrupt(|_| {});
        cast(&mut c, a, Some(b), 0);
        c.step_battle_execution_until(&mut s, 1000).unwrap();
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("health_cost_after_payment", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 1001),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(
            before,
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                format!("{s:?}")
            )
        );
        c.step_battle_execution_until(&mut s, 1001).unwrap();
        assert_eq!(c.units[&a].stats.current_health, 900);
    }
    #[test]
    fn composition_channel_upkeep_bad_policy_duplicate_variant_and_roundtrip() {
        for kind in ["interval", "percent", "duplicate"] {
            let mut doc = source();
            let support = &mut aura(&mut doc)["aura"]["support"];
            match kind {
                "interval" => {
                    support["conversions"][0]["replacement"]["RegularBasic"]["health_upkeep"]
                        ["interval_ms"] = json!(0)
                }
                "percent" => {
                    support["conversions"][0]["replacement"]["RegularBasic"]["health_upkeep"]
                        ["max_health_percent"] = json!(101)
                }
                _ => support["conversions"][0] = support["conversion"].clone(),
            }
            let error = compile_json(&doc.to_string()).unwrap_err();
            assert!(error.source.definition.contains("ChannelAura"), "{error:?}");
            assert!(
                error.reason.contains(if kind == "duplicate" {
                    "duplicate"
                } else {
                    "upkeep"
                }),
                "{error:?}"
            );
        }
    }
    #[test]
    fn composition_channel_upkeep_corruption_cannot_publish_unpaired_benefits() {
        let (c, _, _, _) = corrupt(|_| {});
        let data: ValidatedGameData = c.game_data.clone().into();
        for case in ["independent-benefit", "missing-cost", "missing-consumer"] {
            let mut relics = (*data.relic_data).clone();
            let mut skills = (*data.skill_data).clone();
            if case == "independent-benefit" {
                relics.relics[0].corruption.as_mut().unwrap().effects =
                    relics.relics[0].effects.clone();
            } else {
                let skill = skills
                    .skills
                    .iter_mut()
                    .find(|s| s.id.as_str() == "r11_aura")
                    .unwrap();
                let crate::game::ability::SkillEffectDef::Sustained {
                    definition:
                        crate::game::ability::sustained::SustainedEffectDef::ChannelAura {
                            aura, ..
                        },
                } = &mut skill.steps[0].effects[0].op
                else {
                    panic!("expected support")
                };
                let support = aura.support.as_mut().unwrap();
                if case == "missing-consumer" {
                    support.conversions.clear();
                } else {
                    let crate::game::relic::structural::support::SupportReplacementDef::RegularBasic{health_upkeep,..}=&mut support.conversions[0].replacement else {panic!("expected replacement")};
                    *health_upkeep = None;
                }
            }
            let error = crate::game::data::GameDataBuilder::from_validated(&data)
                .with_skills(skills)
                .with_relics(relics)
                .build_with_runtime_safety(
                    crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
                )
                .err()
                .expect("invalid paired contract must fail publication");
            assert!(
                error.cause.contains(if case == "independent-benefit" {
                    "paired provider"
                } else {
                    "upkeep"
                }),
                "{case}: {error:?}"
            );
        }
    }
}
