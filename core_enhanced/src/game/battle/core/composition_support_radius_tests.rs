mod composition_support_radius {
    use super::composition_duration_modifiers::equip_named;
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact};
    use serde_json::{json, Value};

    fn fixture(
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut doc = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/support_radius.skill.yaml"
        )))
        .unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        fixture_from_native(artifact.load_native().unwrap(), false, false)
    }

    #[test]
    fn composition_support_radius_actual_boundary_membership_owner_once_conversion_and_stop() {
        for mode in ["none", "owned", "other", "one", "two", "single"] {
            let (mut c, mut s, a, b) = fixture(|_| {});
            let owners = match mode {
                "owned" => vec![None],
                "other" => vec![Some(b)],
                "two" => vec![Some(a), Some(a)],
                "one" | "single" => vec![Some(a)],
                _ => vec![],
            };
            equip_named(&mut c, a, &owners, mode == "single", "resonator");
            cast(&mut c, a, (mode == "single").then_some(b), 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(b, true).unwrap(),
                if mode == "single" { (40, 20) } else { (20, 15) }
            );
            place(&mut c, b, 4, 1); // exactly 2 world units: 1.6 * 1.25
            c.reconcile_sustained(2).unwrap();
            let extended = matches!(mode, "one" | "two");
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(b, true).unwrap(),
                if mode == "single" {
                    (40, 20)
                } else if extended {
                    (20, 15)
                } else {
                    (0, 0)
                },
                "{mode}"
            );
            place(&mut c, b, 4, 2); // same mask row, outside the circle: sqrt(5)
            c.reconcile_sustained(3).unwrap();
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(b, true).unwrap(),
                if mode == "single" { (40, 20) } else { (0, 0) },
                "{mode}"
            );
            place(&mut c, b, 3, 1);
            c.reconcile_sustained(4).unwrap();
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(b, true).unwrap(),
                if mode == "single" { (40, 20) } else { (20, 15) }
            );
            if mode == "single" {
                place(&mut c, b, 6, 1);
                c.reconcile_sustained(5).unwrap();
            } else {
                cast(&mut c, a, None, 5);
            }
            assert!(!c.channel_active(a));
            assert_eq!(c.aura_basic_bonus_for_occurrence(b, true).unwrap(), (0, 0));
            c.step_battle_execution_until(&mut s, 202).unwrap();
            assert!(!c.channel_active(a));
        }
    }

    #[test]
    fn composition_support_radius_adds_contributions_once_and_rounds_native_units_up() {
        let (mut c, mut s, a, b) = fixture(|doc| {
            doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                ["ChannelAura"]["aura"]["radius_units"] = json!(1600001);
            let mut extra = doc["definitions"][0]["body"]["Action"]["modifiers"][0].clone();
            extra["id"] = json!("second-radius-contribution");
            extra["change"]["SupportRadius"]["percent"] = json!({"Literal":{"Count":50}});
            doc["definitions"][0]["body"]["Action"]["modifiers"]
                .as_array_mut()
                .unwrap()
                .push(extra);
        });
        equip_named(&mut c, a, &[Some(a), Some(a)], false, "resonator");
        let plan = &c.effective_owner.as_ref().unwrap().plans[&c.units[&a].source_owned_uuid];
        let effect = &plan.execution[0].skill.steps[0].effects[0].op;
        assert_eq!(
            crate::game::ability::action::support_radius(effect),
            Some(2800002)
        );
        let radius = plan
            .effects
            .iter()
            .flat_map(|e| &e.properties)
            .find(|p| {
                p.property
                    == crate::game::relic::structural::authoring::RelicProperty::SupportAreaRadius
            })
            .unwrap();
        assert_eq!(
            radius.final_value,
            Some(crate::game::relic::structural::authoring::TypedRelicValue::DataUnits(2800002))
        );

        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        let x = c.units[&a].body.position.x;
        crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary_x(
            &mut c,
            b,
            x + 2.85,
        );
        c.reconcile_sustained(2).unwrap();
        assert_eq!(c.aura_basic_bonus_for_occurrence(b, true).unwrap(), (0, 0));
        crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary_x(
            &mut c,
            b,
            x + 2.75,
        );
        c.reconcile_sustained(3).unwrap();
        assert_eq!(
            c.aura_basic_bonus_for_occurrence(b, true).unwrap(),
            (20, 15)
        );
    }

    #[test]
    fn composition_support_radius_personal_shield_uses_release_snapshot_not_live_membership() {
        for mode in ["none", "equipped", "single"] {
            let equipped = mode != "none";
            let single = mode == "single";
            let (mut c, mut s, a, b) = fixture(|doc| {
                doc["definitions"][0]["body"]["Action"]["wait"] = json!("Released");
                doc["values"][0]["value"]["Effects"] = json!([{"id":"support","op":{"PersonalShieldGrant":{"definition":{
                    "amount":{"moment":"Snapshot","root":{"Const":{"value":200,"unit":"Magnitude"}}},"duration_ms":201,"priority":0,
                    "recipients":{"PlayerEmployeesAtRelease":{"radius_units":1600000}},"support":{},"replace_only_recipients":true
                }}}}]);
                doc["values"][0]["value"]["Effects"][0]["op"]["PersonalShieldGrant"]
                    ["definition"]["support"]["conversion"] = json!({
                    "relic_definition":"single-target-conversion","variant":"Normal",
                    "selection_range":{"include_anchor_tile":true,"rows":["XXXXX","XXXXX","XX@XX","XXXXX","XXXXX"]},
                    "replacement":{"PersonalShield":{"amount":{"moment":"Snapshot","root":{"Const":{"value":300,"unit":"Magnitude"}}}}}
                });
            });
            if equipped {
                equip_named(&mut c, a, &[Some(a)], single, "resonator");
            }
            place(&mut c, b, 4, 1);
            cast(&mut c, a, single.then_some(b), 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(shields(&c, a), if single { vec![] } else { vec![200] });
            assert_eq!(
                shields(&c, b),
                if single {
                    vec![300]
                } else if equipped {
                    vec![200]
                } else {
                    vec![]
                }
            );
            assert!(!c.resonance_use_has_obligations(a));
            place(&mut c, b, 6, 1);
            c.reconcile_sustained(2).unwrap();
            assert_eq!(
                shields(&c, b),
                if single {
                    vec![300]
                } else if equipped {
                    vec![200]
                } else {
                    vec![]
                }
            );
            place(&mut c, b, 3, 1);
            c.reconcile_sustained(3).unwrap();
            assert_eq!(
                shields(&c, b),
                if single {
                    vec![300]
                } else if equipped {
                    vec![200]
                } else {
                    vec![]
                }
            );
            c.step_battle_execution_until(&mut s, 202).unwrap();
            assert!(shields(&c, a).is_empty() && shields(&c, b).is_empty());
        }
    }
    #[test]
    fn composition_support_radius_conflicts_and_overflow_reject_without_publication() {
        use crate::game::{ability::action::ActionChange, data::GameDataBuilder};
        let (c, _, _, _) = fixture(|_| {});
        let data: ValidatedGameData = c.game_data.clone().into();
        let before = ron::ser::to_string(&data.skill_data).unwrap();
        for mode in ["overflow", "radius-first", "mask-first"] {
            let mut skills = data.skill_data.skills.clone();
            let skill = skills.iter_mut().find(|s| s.id == "r11_aura").unwrap();
            if mode == "overflow" {
                let crate::game::ability::SkillEffectDef::Sustained {
                    definition:
                        crate::game::ability::sustained::SustainedEffectDef::ChannelAura {
                            aura, ..
                        },
                } = &mut skill.steps[0].effects[0].op
                else {
                    unreachable!()
                };
                aura.radius_units = u32::MAX;
            } else {
                // A legacy native mask modifier and the new radius modifier
                // are each valid on the circular source, but conflict together.
                let policy = std::sync::Arc::make_mut(skill.action_policy.as_mut().unwrap());
                let mut mask = policy.modifiers[0].clone();
                let ActionChange::SupportRadius { step, effect, .. } = &mask.change else {
                    unreachable!()
                };
                mask.change = ActionChange::SupportRange {
                    step: step.clone(),
                    effect: effect.clone(),
                    pattern: serde_json::from_value(
                        json!({"include_anchor_tile":true,"rows":["@X"]}),
                    )
                    .unwrap(),
                };
                mask.id = "replace-mask".into();
                policy.modifiers.push(mask);
                if mode == "mask-first" {
                    policy.modifiers.reverse();
                }
            }
            let native = SkillDatabase::new(skills);
            let wire = ron::ser::to_string(&native).unwrap();
            // Publication checks every legal coequip plan, so the bad input is
            // rejected before an employee can install such a plan.
            for _ in 0..2 {
                let loaded = SkillDatabase::from_ron("radius-negative", &wire).unwrap();
                let result = GameDataBuilder::from_validated(&data)
                    .with_skills(loaded)
                    .build_with_runtime_safety(
                        crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
                    );
                let error = match result {
                    Err(error) => error,
                    Ok(_) => panic!("{mode} unexpectedly published"),
                };
                assert_eq!(
                    error.code,
                    if mode == "overflow" {
                        "relic_structural_out_of_range"
                    } else {
                        "relic_structural_write_conflict"
                    },
                    "{mode}: {error:?}"
                );
                assert_eq!(error.field_path.as_deref(), Some("legal_coequip_plans"));
                assert_eq!(ron::ser::to_string(&data.skill_data).unwrap(), before);
                assert_eq!(ron::ser::to_string(&native).unwrap(), wire);
            }
        }
    }
}
