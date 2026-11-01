mod composition_taunt_damage {
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact, DiagnosticCode};
    use crate::game::battle::damage::{BattleCommand, DamageSource, DamageType};
    use serde_json::{json, Value};
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/taunt_damage.skill.yaml"
    ));
    const TAKEN: &str = "taunted-enemy-damage-reduction";
    const BASIC: &str = "taunted-enemy-basic-attack-boost";

    fn fixture(
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let native = Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap();
        let (mut c, s, a, b) = fixture_from_native(native, false, false);
        let e = c
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for (id, x, y) in [(a, 2, 1), (b, 3, 2), (e, 3, 1)] {
            place(&mut c, id, x, y);
            let u = c.units.get_mut(&id).unwrap();
            u.stats.current_health = 10_000;
            u.stats.max_health = 10_000;
            u.stats.defense = 0;
            u.next_basic_attack_ms = 10_000;
            u.action_locks = Default::default();
        }
        (c, s, a, b, e)
    }
    fn plan(
        c: &BattleCore,
        a: UnitInstanceId,
        copies: &[(&str, Option<UnitInstanceId>)],
    ) -> Result<
        Option<std::sync::Arc<crate::game::relic::structural::EmployeeEffectivePlan>>,
        crate::game::relic::structural::RelicPlanError,
    > {
        let data: ValidatedGameData = c.game_data.clone().into();
        let employee = c.units[&a].source_owned_uuid;
        let mut inventory = RelicInventory::default();
        let mut ids = UuidManager::new(0xd270);
        for (slot, (name, owner)) in copies.iter().enumerate() {
            let copy = inventory
                .grant(data.relic_data.get_by_id(name).unwrap(), &mut ids)
                .unwrap();
            if let Some(owner) = owner {
                inventory
                    .equip(
                        copy,
                        c.units[owner].source_owned_uuid,
                        slot,
                        &data.relic_data,
                    )
                    .unwrap();
            }
        }
        let providers = [ProspectiveProvider::Fragment(
            c.units[&a]
                .fragment_source
                .as_ref()
                .unwrap()
                .as_ref()
                .clone(),
        )];
        compile_effective_plan(EffectivePlanInput {
            data: &data,
            employee_uuid: employee,
            inventory: &inventory,
            providers: &providers,
        })
    }
    fn equip(c: &mut BattleCore, a: UnitInstanceId, copies: &[(&str, Option<UnitInstanceId>)]) {
        if let Some(plan) = plan(c, a, copies).unwrap() {
            c.set_effective_owner(
                crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(
                    std::collections::BTreeMap::from([(c.units[&a].source_owned_uuid, plan)]),
                )
                .unwrap(),
            )
            .unwrap();
        }
    }
    fn hit(
        c: &mut BattleCore,
        source: UnitInstanceId,
        target: UnitInstanceId,
        kind: DamageSource,
        reaction: bool,
        now: u64,
    ) -> u32 {
        let hp = c.units[&target].stats.current_health;
        c.runtime_transaction(|c| {
            let mut snapshot = c
                .damage_source_snapshot_for_unit(
                    source,
                    target,
                    kind,
                    DamageType::Physical,
                    100,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .unwrap();
            snapshot.fact_source.reaction = reaction;
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: snapshot,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
        hp - c.units[&target].stats.current_health
    }

    #[test]
    fn composition_taunt_damage_typed_inputs_calls_source_roundtrip_and_rejections() {
        let doc = parse_yaml(YAML).unwrap();
        let mut twice = doc.clone();
        let first = twice["skills"][0]["body"].clone();
        let mut second = first.clone();
        second["Call"]["id"] = json!("second");
        second["Call"]["arguments"][2]["value"] = json!({"Literal":{"Count":50}});
        twice["skills"][0]["body"] = json!({"Sequence":{"id":"twice","children":[first,second]}});
        let artifact = compile_json(&twice.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let native = artifact.load_native().unwrap();
        let skill = &native.skills[0];
        let modifiers = &skill.action_policy.as_ref().unwrap().modifiers;
        assert_eq!(modifiers.len(), 4);
        for (i, expected) in [20, 30, 20, 50].into_iter().enumerate() {
            let crate::game::ability::action::ActionChange::TauntDamage {
                step,
                effect,
                taken_percent,
                basic_percent,
            } = &modifiers[i].change
            else {
                panic!()
            };
            assert_eq!(step, &skill.steps[i / 2].id);
            assert_eq!(effect, "control");
            assert_eq!(
                (*taken_percent, *basic_percent),
                if i % 2 == 0 {
                    (expected, 0)
                } else {
                    (0, expected)
                }
            );
        }
        let sources = artifact.sources();
        let key = "skills[0].action_policy.modifiers[3].change.TauntDamage.basic_percent";
        assert_eq!(
            sources[key].definition,
            "skills[0].body.children[1].arguments[2].value.Literal.Count"
        );
        assert!(sources[key]
            .calls
            .contains(&"skills[0].body.children[1]".into()));
        assert!(sources
            .values()
            .any(|o| o.inputs.contains(&"values[1].value".into())
                && o.inputs.contains(&"values[2].value".into())));
        for (index, value, code) in [
            (1, json!({"Milliseconds":20}), DiagnosticCode::Type),
            (1, json!({"Count":101}), DiagnosticCode::Native),
            (2, json!({"Count":2147483648u32}), DiagnosticCode::Native),
        ] {
            let mut bad = doc.clone();
            bad["values"][index]["value"] = value;
            let error = compile_json(&bad.to_string()).unwrap_err();
            assert_eq!(error.code, code, "{error:?}");
            if code == DiagnosticCode::Native {
                assert_eq!(
                    error.source.definition,
                    format!("values[{index}].value.Count")
                );
            }
        }
        for unused in [false, true] {
            let mut bad = doc.clone();
            if unused {
                bad["skills"][0]["body"] =
                    json!({"Wait":{"id":"unused","duration":{"Literal":{"Milliseconds":1}}}});
            }
            bad["definitions"][0]["body"]["Action"]["modifiers"][0]["change"]["TauntDamage"]
                ["taken_percent"] = json!({"Literal":{"Count":101}});
            assert!(compile_json(&bad.to_string())
                .unwrap_err()
                .source
                .definition
                .ends_with("taken_percent.Literal.Count"));
            bad["definitions"][0]["body"]["Action"]["modifiers"][0]["change"]["TauntDamage"]
                ["taken_percent"] = json!({"Literal":{"Count":0}});
            assert!(compile_json(&bad.to_string())
                .unwrap_err()
                .reason
                .contains("positive coefficient"));
        }
        for missing in [false, true] {
            let mut bad = doc.clone();
            if missing {
                bad["definitions"][0]["body"]["Action"]["modifiers"][0]["change"]["TauntDamage"]
                    ["effect"] = json!("absent");
            } else {
                bad["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                    ["Control"]["kind"] = json!("Root");
            }
            let e = compile_json(&bad.to_string()).unwrap_err();
            assert_eq!(
                e.code,
                if missing {
                    DiagnosticCode::Reference
                } else {
                    DiagnosticCode::Native
                }
            );
            assert!(e.source.definition.ends_with("TauntDamage.effect"), "{e:?}");
        }
    }

    #[test]
    fn composition_taunt_damage_equipped_owner_once_and_pair_source_eligibility() {
        for mode in ["none", "owned", "other", "one", "two"] {
            let (mut c, mut s, a, b, e) = fixture(|_| {});
            let owner = match mode {
                "other" => Some(b),
                "one" | "two" => Some(a),
                _ => None,
            };
            let mut copies = if mode == "none" {
                vec![]
            } else {
                vec![(TAKEN, owner), (BASIC, owner)]
            };
            if mode == "two" {
                copies.push((BASIC, owner));
            }
            equip(&mut c, a, &copies);
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(c.taunt_source(e), Some(a));
            let active = matches!(mode, "one" | "two");
            assert_eq!(
                hit(&mut c, e, a, DamageSource::Ability, false, 1),
                if active { 80 } else { 100 },
                "{mode}"
            );
            assert_eq!(
                hit(&mut c, a, e, DamageSource::BasicAttack, false, 1),
                if active { 130 } else { 100 },
                "{mode}"
            );
            assert_eq!(
                hit(&mut c, e, a, DamageSource::BuffTick, false, 1),
                if active { 80 } else { 100 }
            );
            assert_eq!(hit(&mut c, a, e, DamageSource::BuffTick, false, 1), 100);
            assert_eq!(hit(&mut c, e, b, DamageSource::Ability, false, 1), 100);
            assert_eq!(hit(&mut c, b, e, DamageSource::BasicAttack, false, 1), 100);
            assert_eq!(hit(&mut c, a, e, DamageSource::Ability, false, 1), 100);
            assert_eq!(hit(&mut c, a, e, DamageSource::BasicAttack, true, 1), 100);
            assert_eq!(hit(&mut c, e, a, DamageSource::Environment, false, 1), 100);
            c.units.get_mut(&e).unwrap().corroded = true;
            assert_eq!(hit(&mut c, e, a, DamageSource::Ability, false, 1), 100);
            assert_eq!(hit(&mut c, a, e, DamageSource::BasicAttack, false, 1), 100);
        }
    }

    #[test]
    fn composition_taunt_damage_replacement_expiry_and_inflight_damage_use_live_relation() {
        for mode in ["expire", "replace", "withdraw", "immune"] {
            let (mut c, mut s, a, b, e) = fixture(|_| {});
            equip(&mut c, a, &[(TAKEN, Some(a)), (BASIC, Some(a))]);
            if mode == "immune" {
                c.units
                    .get_mut(&e)
                    .unwrap()
                    .target_traits
                    .push(crate::game::battle::types::UnitTargetTrait::TauntImmune);
            }
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let snapshot = c
                .damage_source_snapshot_for_unit(
                    a,
                    e,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    100,
                    Default::default(),
                    0,
                    1,
                    false,
                )
                .unwrap();
            let now = match mode {
                "expire" => {
                    c.step_battle_execution_until(&mut s, 1502).unwrap();
                    1502
                }
                "replace" => {
                    cast(&mut c, b, None, 1);
                    c.step_battle_execution_until(&mut s, 2).unwrap();
                    assert_eq!(c.taunt_source(e), Some(b));
                    2
                }
                "withdraw" => {
                    c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: 2,
                    })
                    .unwrap();
                    c.step_battle_execution_until(&mut s, 2).unwrap();
                    2
                }
                _ => 1,
            };
            if mode != "replace" {
                assert_eq!(c.taunt_source(e), None);
            }
            let hp = c.units[&e].stats.current_health;
            c.runtime_transaction(|c| {
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: e,
                        source_snapshot: snapshot,
                    }],
                    now,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
            assert_eq!(hp - c.units[&e].stats.current_health, 100, "{mode}");
            if mode != "withdraw" {
                assert_eq!(
                    hit(&mut c, e, a, DamageSource::Ability, false, now),
                    100,
                    "{mode}"
                );
            }
            if mode == "replace" {
                assert_eq!(
                    hit(&mut c, b, e, DamageSource::BasicAttack, false, now),
                    100,
                    "new source did not equip bonus"
                );
            }
        }
    }

    #[test]
    fn composition_taunt_damage_real_area_and_authored_replacement_check_each_target() {
        use crate::game::ability::{DeliveryDef, SkillAreaAnchorSource};
        use crate::game::battle::tile_range::TileRangePattern;
        for replacement in [false, true] {
            let (mut c, mut s, a, b, e) = fixture(|doc| {
                if replacement {
                    let mut state = parse_yaml(include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/tests/fixtures/skill_composition/manual_state.skill.yaml"
                    )))
                    .unwrap()["values"][0]
                        .clone();
                    let p = &mut state["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                        ["State"]["program"];
                    p["handlers"] = json!([{"id":"finish","event":"Expired","body":["End"]}]);
                    p["basic_attack"]["line"]["weapon_effects"] = json!(true);
                    p["basic_attack"]["line"]["piercing_relics"] = json!(false);
                    p["basic_attack"]["line"]
                        .as_object_mut()
                        .unwrap()
                        .remove("piercing");
                    doc["values"].as_array_mut().unwrap().push(state);
                    let taunt = doc["skills"][0]["body"].clone();
                    doc["skills"][0]["body"] = json!({"Sequence":{"id":"taunt-and-transform","children":[taunt,{"Action":{"id":"transform","target":{"Literal":{"Target":"SelfUnit"}},"delivery":{"Literal":{"Delivery":"Instant"}},"effects":[{"Append":{"effects":{"Shared":"transformation"}}}],"wait":"Released"}}]}});
                }
            });
            c.units.get_mut(&b).unwrap().owner = Side::Opponent;
            c.units.get_mut(&b).unwrap().fragment_source = None;
            c.units.get_mut(&b).unwrap().skill_id = None;
            c.units
                .get_mut(&b)
                .unwrap()
                .target_traits
                .push(crate::game::battle::types::UnitTargetTrait::TauntImmune);
            place(&mut c, b, 4, 1);
            c.units.get_mut(&a).unwrap().stats.attack = 100;
            c.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                    include_anchor_tile: true,
                },
                area: crate::game::ability::SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tile_origin: crate::game::ability::SkillTileAreaOrigin::Anchor,
                    ..Default::default()
                },
            };
            equip(&mut c, a, &[(BASIC, Some(a))]);
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(c.taunt_source(e), Some(a));
            assert_eq!(c.taunt_source(b), None);
            let before_e = c.units[&e].stats.current_health;
            let before_b = c.units[&b].stats.current_health;
            c.units.get_mut(&a).unwrap().next_basic_attack_ms = 10;
            c.process_event(
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: a,
                    target_instance_id: Some(e),
                    schedule_next: true,
                    cause: Default::default(),
                },
                10,
            )
            .unwrap();
            c.step_battle_execution_until(&mut s, 11).unwrap();
            assert_eq!(
                before_e - c.units[&e].stats.current_health,
                if replacement { 52 } else { 130 },
                "taunted primary, replacement={replacement}"
            );
            assert_eq!(
                before_b - c.units[&b].stats.current_health,
                if replacement { 40 } else { 100 },
                "immune secondary, replacement={replacement}"
            );
        }
    }

    #[test]
    fn composition_taunt_damage_combined_overflow_rejects_catalog_without_publication() {
        let (c, _, _, _, _) = fixture(|_| {});
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
        );
        let data: ValidatedGameData = c.game_data.clone().into();
        for basic in [false, true] {
            let mut doc = parse_yaml(YAML).unwrap();
            doc["skills"][0]["id"] = json!("r11_aura");
            let profile = &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]
                ["definition"]["Control"]["kind"]["ApproachTaunt"]["profile"];
            profile[if basic {
                "basic_damage_bonus_percent"
            } else {
                "damage_reduction_percent"
            }] = if basic {
                json!(2147483647u32)
            } else {
                json!(90)
            };
            let artifact = compile_json(&doc.to_string()).unwrap();
            let mut skills = (*data.skill_data).clone();
            skills.skills.retain(|s| s.id != "r11_aura");
            skills.skills.extend(artifact.load_native().unwrap().skills);
            let error = crate::game::data::GameDataBuilder::from_validated(&data)
                .with_skills(skills)
                .build_with_runtime_safety(
                    crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
                )
                .err()
                .expect("legal coequip profile must be rejected before catalog publication");
            assert_eq!(error.code, "relic_structural_out_of_range");
            assert_eq!(error.field_path.as_deref(), Some("legal_coequip_plans"));
            assert!(
                error
                    .cause
                    .contains(if basic { "overflow" } else { "exceeds 100" }),
                "{error:?}"
            );
            assert_eq!(
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot()
                ),
                before
            );
        }
    }
}
