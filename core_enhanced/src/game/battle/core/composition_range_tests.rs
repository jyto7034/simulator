mod composition_range {
    use super::composition_duration_modifiers::equip_named;
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact, DiagnosticCode};
    use crate::game::battle::tile_range::FacingDirection;
    use serde_json::{json, Value};

    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/tile_range.skill.yaml"
    ));
    const RELIC: &str = "widened-horizon";

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
        let (mut c, s, a, b) = fixture_from_native_with_catalog(
            native,
            false,
            false,
            |_| {},
            |relics| {
                let mut r = relics.relics[0].clone();
                r.id = RELIC.into();
                r.uuid = Uuid::from_u128(0xd280);
                relics.relics.push(r);
            },
        );
        let e = c
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for (id, x, y) in [(a, 2, 1), (b, 3, 2), (e, 3, 1)] {
            place(&mut c, id, x, y);
            let u = c.units.get_mut(&id).unwrap();
            u.stats.current_health = 900;
            u.stats.max_health = 1000;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
            u.stats.move_speed_units_per_ms = 0;
            u.body.move_speed = 0.0;
            u.next_basic_attack_ms = 10000;
            u.action_locks = Default::default();
        }
        (c, s, a, b, e)
    }

    #[test]
    fn composition_range_shared_parameter_calls_roundtrip_and_origins() {
        let mut doc = parse_yaml(YAML).unwrap();
        let first = doc["skills"][0]["body"].clone();
        let mut second = first.clone();
        second["Call"]["id"] = json!("other-mask");
        second["Call"]["arguments"][1]["value"] =
            json!({"Literal":{"TilePattern":{"include_anchor_tile":true,"rows":["@X","XX"]}}});
        doc["skills"][0]["body"] = json!({"Sequence":{"id":"twice","children":[first,second]}});
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let native = artifact.load_native().unwrap();
        let skill = &native.skills[0];
        let changes = &skill.action_policy.as_ref().unwrap().modifiers;
        assert_eq!(changes.len(), 2);
        assert_ne!(changes[0].id, changes[1].id);
        for i in 0..2 {
            let crate::game::ability::action::ActionChange::Range { steps, pattern } =
                &changes[i].change
            else {
                panic!()
            };
            assert_eq!(steps, &[skill.steps[i].id.clone()]);
            assert_eq!(
                pattern.rows,
                if i == 0 {
                    vec!["XXX", "X@X", "XXX"]
                } else {
                    vec!["@X", "XX"]
                }
            );
            let source = artifact
                .sources()
                .get(&format!(
                    "skills[0].action_policy.modifiers[{i}].change.Range.pattern"
                ))
                .unwrap();
            assert_eq!(
                source.definition,
                if i == 0 {
                    "values[1].value.TilePattern"
                } else {
                    "skills[0].body.children[1].arguments[1].value.Literal.TilePattern"
                }
            );
            assert!(source
                .calls
                .iter()
                .any(|c| c == &format!("skills[0].body.children[{i}]")));
            let origin = artifact
                .origin_for(skill.steps[i].effects[0].source().unwrap())
                .unwrap();
            assert!(origin
                .inputs
                .iter()
                .any(|p| p == "definitions[0].body.modifiers[0].change.Range.pattern"));
            if i == 0 {
                assert!(origin.inputs.iter().any(|p| p == "values[1].value"));
            }
        }
    }

    #[test]
    fn composition_range_rejects_bad_unused_masks_types_unresolved_and_delivery() {
        for unused in [false, true] {
            for rows in [
                json!([]),
                json!([""]),
                json!(["XX"]),
                json!(["@@"]),
                json!(["@X", "X"]),
                json!(["@?"]),
            ] {
                let mut doc = parse_yaml(YAML).unwrap();
                doc["values"][1]["value"]["TilePattern"]["rows"] = rows;
                if unused {
                    doc["skills"][0]["body"]["Call"]["arguments"][1]["value"] = json!({"Literal":{"TilePattern":{"include_anchor_tile":true,"rows":["@"]}}});
                }
                let error = compile_json(&doc.to_string()).unwrap_err();
                assert_eq!(error.code, DiagnosticCode::Native);
                assert_eq!(error.source.definition, "values[1].value.TilePattern");
            }
        }
        for (input, code) in [
            (json!({"Literal":{"Count":3}}), DiagnosticCode::Type),
            (
                json!({"Unresolved":"확대 형상 미정"}),
                DiagnosticCode::Unresolved,
            ),
        ] {
            let mut doc = parse_yaml(YAML).unwrap();
            doc["skills"][0]["body"]["Call"]["arguments"][1]["value"] = input;
            assert_eq!(compile_json(&doc.to_string()).unwrap_err().code, code);
        }
        let mut doc = parse_yaml(YAML).unwrap();
        doc["definitions"][0]["body"]["Action"]["delivery"] =
            json!({"Literal":{"Delivery":"Instant"}});
        let error = compile_json(&doc.to_string()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native);
        assert!(error.source.definition.ends_with("change.Range.pattern"));
        assert!(error.reason.contains("TileArea"));
    }

    #[test]
    fn composition_range_equipped_owner_once_changes_control_targets_not_lifetime_or_reach() {
        for mode in ["none", "owned", "other", "one", "two", "outside", "immune"] {
            let (mut c, mut s, a, b, e) = fixture(|_| {});
            if mode == "outside" {
                place(&mut c, e, 4, 1);
            }
            if mode == "immune" {
                c.units
                    .get_mut(&e)
                    .unwrap()
                    .target_traits
                    .push(crate::game::battle::types::UnitTargetTrait::TauntImmune);
            }
            let owners = match mode {
                "owned" => vec![None],
                "other" => vec![Some(b)],
                "none" => vec![],
                "two" => vec![Some(a), Some(a)],
                _ => vec![Some(a)],
            };
            equip_named(&mut c, a, &owners, false, RELIC);
            let original_range = c.units[&a].basic_attack.clone();
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let hit = matches!(mode, "one" | "two");
            assert_eq!(c.taunt_source(e), hit.then_some(a), "{mode}");
            assert_eq!(c.taunt_source(b), None);
            assert_eq!(c.units[&e].stats.current_health, 900);
            assert_eq!(
                serde_json::to_value(&c.units[&a].basic_attack).unwrap(),
                serde_json::to_value(&original_range).unwrap()
            );
            if hit {
                let active = c.sustained_effect_snapshot().unwrap().active;
                let effect = active.iter().find(|r| r.target == e).unwrap();
                assert_eq!(effect.expires_at_ms, 1502);
                assert!(!c.channel_active(a));
                c.step_battle_execution_until(&mut s, 1502).unwrap();
                assert_eq!(c.taunt_source(e), None);
            }
        }
    }

    #[test]
    fn composition_range_direct_damage_respects_explicit_direction_holes_and_other_action() {
        for (facing, x, y, hits) in [
            (FacingDirection::Up, 3, 1, true),
            (FacingDirection::Right, 2, 2, true),
            (FacingDirection::Down, 1, 1, true),
            (FacingDirection::Left, 2, 0, true),
            (FacingDirection::Up, 3, 2, false),
        ] {
            let (mut c, mut s, a, b, e) = fixture(|doc| {
                doc["values"][1]["value"]["TilePattern"] =
                    json!({"include_anchor_tile":false,"rows":["@X",".."]});
                doc["definitions"][0]["body"]["Action"]["effects"] = json!([{"Damage":{"id":"hit","tags":[],"amount":{"Literal":{"Amount":{"moment":"Snapshot","root":{"Const":{"value":20,"unit":"Magnitude"}}}}},"damage_type":"Physical"}}]);
                let area = doc["skills"][0]["body"].clone();
                doc["skills"][0]["body"] = json!({"Sequence":{"id":"area-and-self","children":[area,{"Action":{"id":"self-heal","target":{"Literal":{"Target":"SelfUnit"}},"delivery":{"Literal":{"Delivery":"Instant"}},"effects":[{"Heal":{"id":"heal","tags":[],"amount":{"Literal":{"Amount":{"moment":"Snapshot","root":{"Const":{"value":7,"unit":"Magnitude"}}}}}}}],"wait":"Released"}}]}});
            });
            place(&mut c, e, x, y);
            c.units.get_mut(&a).unwrap().facing_direction = Some(facing);
            equip_named(&mut c, a, &[Some(a)], false, RELIC);
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(
                c.units[&e].stats.current_health,
                if hits { 880 } else { 900 },
                "{facing:?}/{x},{y}"
            );
            assert_eq!(c.units[&b].stats.current_health, 900);
            assert_eq!(c.units[&a].stats.current_health, 907);
            assert_eq!(c.taunt_source(e), None);
        }
    }

    #[test]
    fn composition_range_conflicting_masks_reject_catalog_before_publication() {
        let (c, _, _, _, _) = fixture(|_| {});
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
        );
        let data: ValidatedGameData = c.game_data.clone().into();
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        let mut modifier = doc["definitions"][0]["body"]["Action"]["modifiers"][0].clone();
        modifier["id"] = json!("different-mask");
        modifier["change"]["Range"]["pattern"] =
            json!({"Literal":{"TilePattern":{"include_anchor_tile":true,"rows":["@X"]}}});
        doc["definitions"][0]["body"]["Action"]["modifiers"]
            .as_array_mut()
            .unwrap()
            .push(modifier);
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
            .expect("conflicting legal loadout rejected");
        assert_eq!(error.code, "relic_structural_write_conflict");
        assert_eq!(error.field_path.as_deref(), Some("legal_coequip_plans"));
        assert_eq!(
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot()
            ),
            before
        );
    }
}
