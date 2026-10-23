//! Public external representative values. These are not official balance data.
use super::*;

fn family_text(name: &str) -> String {
    // Runtime fixture bytes are explicitly included in the runner input manifest.
    // Authored-data fixes can reuse this public probe binary without recompiling
    // unrelated native test functions.
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/goals/core_relic_structural_activation/execution/single_goal/test_parts/runtime_validation001/fixtures").join(name)).unwrap()
}

fn append_entries(root: &Path, file: &str, key: &str, extra: &str) {
    let tail = extra.trim().strip_prefix(&format!("({key}: [")).unwrap();
    let entries = if key == "rewards" {
        tail.split_once("], pools:").unwrap().0
    } else {
        tail.strip_suffix("])").unwrap()
    };
    let entries = format!("{},\n", entries.trim_end().trim_end_matches(','));
    let path = root.join(file);
    let mut original = fs::read_to_string(&path).unwrap();
    let start = original.find(&format!("{key}:")).unwrap();
    let insert = start + original[start..].find('[').unwrap() + 1;
    original.insert_str(insert, &entries);
    fs::write(path, original).unwrap();
}
fn shift_from(value: &mut Value, first: u64) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "step" && value.as_u64().is_some_and(|n| n >= first) {
                    *value = json!(value.as_u64().unwrap() + 1);
                } else {
                    shift_from(value, first);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                shift_from(value, first);
            }
        }
        _ => {}
    }
}
fn equip(copy_index: usize, slot: usize) -> Value {
    json!({"kind":"command","command":{"type":"equip_relic","copy_uuid":null,"employee_uuid":null,"slot":slot},"bindings":[{"from":{"step":15,"pointer":format!("/payload/copies/{copy_index}/copy_uuid")},"to":"/command/copy_uuid","value_type":"string"},{"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]})
}
fn public<'a>(session: &'a TrialSession, index: usize) -> &'a Value {
    &session.step(index).unwrap().public_result().unwrap()["payload"]
}
fn checkpoint(session: &TrialSession, index: usize) -> Value {
    public(session, index)["battle_update"]["checkpoint"].clone()
}
fn props(plan: &Value, property: &str) -> Vec<i64> {
    let mut values: Vec<_> = plan["effects"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|e| e["properties"].as_array().unwrap())
        .filter(|p| p["property"] == property)
        .filter_map(|p| p["final_value"].as_object())
        .flat_map(|p| p.values())
        .map(|v| v.as_i64().unwrap())
        .collect();
    values.sort();
    values
}
fn hp(unit: &Value) -> u64 {
    unit["stats"]["current_health"].as_u64().unwrap()
}
fn unit<'a>(checkpoint: &'a Value, id: &Value) -> &'a Value {
    checkpoint["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["unit_instance_id"] == *id)
        .unwrap()
}

#[test]
fn rv003_external_representatives_and_cross_relic_closure_consume_normal_public_commands() {
    let export = std::env::var_os("U09_PUBLIC_CLI_EXPORT").map(PathBuf::from);
    let dir = export
        .as_ref()
        .map(|p| {
            p.with_file_name(format!(
                "{}_families",
                p.file_name().unwrap().to_str().unwrap()
            ))
        })
        .unwrap_or_else(|| std::env::temp_dir().join(format!("u09-families-{}", Uuid::new_v4())));
    fs::create_dir(&dir).unwrap();
    let (_owned, base, mut original, selection) =
        if let Some(input) = std::env::var_os("U09_PUBLIC_INPUT") {
            let input = PathBuf::from(input);
            let doc: Value =
                serde_json::from_slice(&fs::read(input.join("scenario.json")).unwrap()).unwrap();
            let selection: crate::game::data::content_pack::ContentSelection =
                serde_json::from_slice(&fs::read(input.join("selection.json")).unwrap()).unwrap();
            let expected: Value =
                serde_json::from_slice(&fs::read(input.join("content_manifest.json")).unwrap())
                    .unwrap();
            assert_eq!(manifest(&input.join("content")), expected);
            (None, input.join("content"), doc, selection)
        } else {
            let (fixture, token, doc) = fixture();
            let doc = scenario(&token, doc);
            let root = fixture.root().to_path_buf();
            let selection = fixture.selection();
            (Some(fixture), root, doc, selection)
        };
    let content = dir.join("content");
    copy_tree(&base, &content);
    append_entries(
        &content,
        "official/data/skills/base.ron",
        "skills",
        &family_text("family_skills.ron"),
    );
    append_entries(
        &content,
        "official/data/skill_fragments/base.ron",
        "fragments",
        &family_text("family_fragments.ron"),
    );
    append_entries(
        &content,
        "official/data/relics/base.ron",
        "relics",
        &family_text("family_relics.ron"),
    );
    append_entries(
        &content,
        "official/data/events/rewards/base.ron",
        "rewards",
        &family_text("family_rewards.ron"),
    );
    let path = content.join("official/data/map/node_assignment_profiles.ron");
    let mut assignments: NodeAssignmentProfileDatabase =
        ron::de::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assignments
        .profiles
        .iter_mut()
        .find(|p| p.id == "standard_floor_1")
        .unwrap()
        .fixed_slot_assignments
        .iter_mut()
        .find(|s| s.slot_id.as_str() == "intake_corridor")
        .unwrap()
        .authored_combat
        .as_mut()
        .unwrap()
        .outcome_policies
        .iter_mut()
        .find(|p| p.outcome_id == "victory")
        .unwrap()
        .reward_ids
        .push("u09_family_supply".into());
    fs::write(path, ron::ser::to_string(&assignments).unwrap()).unwrap();
    // Authored external mitigation baseline permits literal Physical40 oracle.
    let path = content.join("official/data/enemies/corroded_employees.ron");
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for enemy in &mut enemies.profiles {
        enemy.defense = 0;
        if enemy.id == "corroded_rusher" {
            // One normal auto-cast releases an independent 10s area at the
            // nearest player (the defended objective); each 1s tick hits the
            // surrounding employees. Start one normal basic-release gain below
            // full: auto-cast is scheduled by the below-max -> max transition,
            // not by spawning with an already-full meter (pending_cast=false).
            enemy.skill_id = Some("u09_family_enemy_pressure".into());
            enemy.resonance.start = 90;
            enemy.resonance.max = 100;
        }
        // This deserializer expands presets. Keep the authored preset when
        // serializing a fixture; publishing both reference and expansion is invalid.
        if enemy.basic_attack_range_preset.is_some() {
            enemy.basic_attack.defense_tile_range = None;
        }
    }
    fs::write(path, ron::ser::to_string(&enemies).unwrap()).unwrap();
    let token =
        crate::game::data::content_pack::load(&content, &selection, &Default::default()).unwrap();
    write_json(&dir.join("selection.json"), &selection);
    write_json(&dir.join("generation.json"), token.identity());
    write_json(&dir.join("content_manifest.json"), &manifest(&content));
    // Discover only normal reward copy indices. No amount, damage or protection
    // result is harvested to manufacture an expected value.
    let full = original.clone();
    original["steps"].as_array_mut().unwrap().truncate(7);
    let discovery = simulate(
        fixture_content(&token),
        checked(&original),
        TrialLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let fragments = public(&discovery, 13)["skill_fragment_diffs"]
        .as_array()
        .unwrap();
    let relics = public(&discovery, 15)["copies"].as_array().unwrap();
    let fragment_index = |name: &str| {
        fragments
            .iter()
            .position(|r| r["fragment_id"] == format!("u09_family_{name}_fragment"))
            .unwrap()
    };
    let relic_index = |name: &str| {
        relics
            .iter()
            .position(|r| r["definition_id"] == format!("u09_family_{name}_relic"))
            .unwrap()
    };
    let selected = std::env::var("U09_FAMILY_CASES").ok();
    if let Some(selected) = &selected {
        assert!(selected
            .split(',')
            .all(|case| ["closure", "area", "single", "support", "monk"].contains(&case)));
    }
    let mut observed = vec![];
    for (case, skill, relic, two_units) in [
        ("closure", "closure", "seed", false),
        ("area", "personal", "area", true),
        ("single", "personal", "single", true),
        ("support", "support", "support", true),
        ("monk", "monk", "monk", false),
    ] {
        if selected
            .as_ref()
            .is_some_and(|list| !list.split(',').any(|name| name == case))
        {
            continue;
        }
        let mut doc = full.clone();
        doc["steps"][5]["bindings"][0]["from"]["pointer"] = json!(format!(
            "/payload/skill_fragment_diffs/{}/copy_uuid",
            fragment_index(skill)
        ));
        doc["steps"][7] = equip(relic_index(relic), 2);
        for step in doc["steps"].as_array_mut().unwrap() {
            if step["command"]["skill_id"] == "u09_public_strike" {
                step["command"]["skill_id"] = json!(format!("u09_family_{skill}"));
            }
            if ["closure", "area", "monk"].contains(&case)
                && step["command"]["type"] == "begin_skill_targeting"
            {
                // Automatic casts expose the same battle authority checkpoint
                // through a normal query; BeginSkillTargeting requires selection.
                step["command"] = json!({"type":"request_battle_state"});
                step.as_object_mut().unwrap().remove("bindings");
                step.as_object_mut().unwrap().remove("context");
            }
            if step["command"]["type"] == "activate_skill" {
                step["command"]["target"] = Value::Null;
                step["bindings"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|b| b["to"] != "/command/target/unit_instance_id");
            }
        }
        let mut query_index = 17;
        let mut ready = 30;
        let mut resolved = 33;
        if case == "closure" {
            shift_from(&mut doc, 17);
            doc["steps"]
                .as_array_mut()
                .unwrap()
                .insert(8, equip(relic_index("enhancer"), 1));
            query_index += 1;
            ready += 1;
            resolved += 1;
        }
        if two_units {
            let first_deploy = 29;
            let mut deploy = authority_step("deploy_unit", first_deploy);
            deploy["command"]["employee_uuid"] = Value::Null;
            deploy["command"]["position"] = Value::Null;
            deploy["command"]["facing"] = json!("right");
            let first = doc["steps"][first_deploy - 9]["bindings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|b| b["to"] == "/command/position")
                .unwrap()["from"]
                .clone();
            // The defended objective is at (4,6), at the end of the north-south
            // route. Personal recipients sit within its authored pressure area;
            // the supported ally at (3,6) can attack right toward the route end.
            let (caster_cell, ally_cell) = if case == "support" { (2, 3) } else { (6, 5) };
            let original_pointer = first["pointer"].as_str().unwrap();
            assert!(original_pointer.ends_with("/cells/3"));
            let pointer = original_pointer.replace("/cells/3", &format!("/cells/{ally_cell}"));
            let caster_position = doc["steps"][first_deploy - 9]["bindings"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|b| b["to"] == "/command/position")
                .unwrap();
            caster_position["from"]["pointer"] =
                json!(original_pointer.replace("/cells/3", &format!("/cells/{caster_cell}")));
            deploy["bindings"].as_array_mut().unwrap().extend([json!({"from":{"step":10,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),json!({"from":{"step":first["step"],"pointer":pointer},"to":"/command/position","value_type":"object"})]);
            shift_from(&mut doc, 30);
            doc["steps"].as_array_mut().unwrap().insert(21, deploy);
            ready += 1;
            resolved += 1;
            if case == "single" || case == "support" {
                let source = if case == "single" { 29 } else { 30 };
                let activate = doc["steps"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|s| s["command"]["type"] == "activate_skill")
                    .unwrap();
                activate["command"]["target"] = json!({"type":"Unit","unit_instance_id":null});
                activate["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":source,"pointer":"/payload/unit_instance_id"},"to":"/command/target/unit_instance_id","value_type":"string"}));
            }
        }
        if two_units {
            doc["steps"].as_array_mut().unwrap().push(
                json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":if case == "support" { 4000 } else { 2000 },"after_seq":0}}),
            );
        }
        write_json(&dir.join(format!("{case}.scenario.json")), &doc);
        let cancel = CancellationToken::default();
        let mut session = TrialSession::prepare(
            fixture_content(&token),
            checked(&doc),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        while !session.is_complete() {
            let index = session.next_step_index();
            if let Err(error) = session.advance(&cancel) {
                let mut raw = session.receipt_documents().collect::<Vec<_>>().join("\n");
                raw.push('\n');
                write_new(
                    &dir.join(format!("{case}.failed.trial.jsonl")),
                    raw.as_bytes(),
                );
                panic!("family {case} step {index}: {error:?}");
            }
        }
        let plan = public(&session, query_index).clone();
        let before = checkpoint(&session, ready);
        let after = checkpoint(&session, resolved);
        let actor = public(&session, 29)["unit_instance_id"].clone();
        let facts = serde_json::to_value(
            session
                .state
                .application
                .battle_runtime_facts()
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        let mut raw = session.receipt_documents().collect::<Vec<_>>().join("\n");
        raw.push('\n');
        write_new(&dir.join(format!("{case}.trial.jsonl")), raw.as_bytes());
        write_json(
            &dir.join(format!("{case}.observed.json")),
            &json!({"case":case,"query_index":query_index,"ready":ready,"resolved":resolved,"plan":plan,"before":before,"after":after,"facts":facts}),
        );
        match case {
            "closure" => {
                assert_eq!(props(&plan, "DamageAmount"), vec![30, 75]);
                let contributions: Vec<_> = plan["rules"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| r["contribution"]["definition_id"].as_str().unwrap())
                    .collect();
                assert!(
                    contributions.contains(&"u09_family_seed_relic")
                        && contributions.contains(&"u09_family_enhancer_relic")
                );
                let delta: u64 = before["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|u| u["owner"] == "Opponent")
                    .map(|u| hp(u) - hp(unit(&after, &u["unit_instance_id"])))
                    .sum();
                assert_eq!(delta, 105);
            }
            "area" | "single" => {
                assert_eq!(
                    props(&plan, "PersonalShieldInitialAmount"),
                    if case == "area" { vec![300] } else { vec![550] }
                );
                let pools: Vec<_> = after["sustained_effects"]["active"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r.get("personal_grant").is_some())
                    .collect();
                let (count, amount) = if case == "area" { (2, 450) } else { (1, 825) };
                assert_eq!(pools.len(), count);
                for pool in pools {
                    assert_eq!(pool["shield"]["capacity"], amount);
                    assert_eq!(pool["shield"]["remaining"], amount);
                    assert_eq!(pool["is_channel"], false);
                    if case == "single" {
                        assert_eq!(pool["target"], actor);
                    }
                }
                let later = checkpoint(&session, resolved + 1);
                let remaining: u64 = later["sustained_effects"]["active"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r.get("personal_grant").is_some())
                    .map(|r| r["shield"]["remaining"].as_u64().unwrap())
                    .sum();
                assert!(
                    remaining < count as u64 * amount as u64,
                    "authored enemy pressure must consume some personal protection"
                );
                assert_eq!(hp(unit(&later, &actor)), hp(unit(&after, &actor)));
                let incoming = &facts["actors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|row| row["actor"] == actor)
                    .unwrap()["summary"]["received"]["latest_damage"];
                assert_eq!(
                    incoming["source"]["cast"]["skill_id"], "u09_family_enemy_pressure",
                    "{incoming}"
                );
                assert_eq!(incoming["requested_damage"], 1);
                assert_eq!(incoming["hp_before"], incoming["hp_after"]);
            }
            "support" => {
                assert_eq!(props(&plan, "RegularBasicDamagePercent"), vec![40]);
                assert_eq!(props(&plan, "RegularBasicSpeedPoints"), vec![20]);
                let ally = public(&session, 30)["unit_instance_id"].clone();
                let row = after["sustained_effects"]["active"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r.get("fixed_connection").is_some())
                    .unwrap();
                assert_eq!(row["caster"], actor);
                assert_eq!(row["target"], actor);
                assert_eq!(row["members"], json!([ally]));
                assert_eq!(row["is_channel"], true);
                let damage = facts["actors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| &a["summary"]["caused"]["latest_damage"])
                    .find(|h| {
                        h["source"]["actor"] == ally && h["source"].get("regular_support").is_some()
                    })
                    .expect("real regular attack by selected ally carries support");
                let contributors = damage["source"]["regular_support"]["contributors"]
                    .as_array()
                    .unwrap();
                assert_eq!(contributors.len(), 1);
                assert_eq!(contributors[0]["damage_percent"], 40);
                assert_eq!(contributors[0]["speed_points"], 20);
                assert!(contributors[0]["source"]["action_ordinal"].is_null());
                assert_eq!(
                    damage["requested_damage"].as_u64().unwrap(),
                    unit(&before, &ally)["stats"]["attack"].as_u64().unwrap() * 140 / 100
                );
                assert!(
                    damage["hp_after"].as_u64().unwrap() < damage["hp_before"].as_u64().unwrap()
                );
            }
            "monk" => {
                let mut targets = 0;
                for enemy in before["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|u| u["owner"] == "Opponent")
                {
                    assert_eq!(enemy["stats"]["defense"], 0);
                    assert_eq!(hp(enemy) - hp(unit(&after, &enemy["unit_instance_id"])), 40);
                    targets += 1;
                }
                assert!(targets > 0);
                assert_eq!(hp(unit(&before, &actor)) - hp(unit(&after, &actor)), 3);
                assert_eq!(facts["summary"]["heal_mutations"], 1);
                assert_eq!(facts["summary"]["healing_delta"], 7);
                let latest = &facts["summary"]["latest_damage"];
                assert_eq!(latest["segments"][0]["damage_type"], "Physical");
                assert_eq!(latest["requested_damage"], 40);
            }
            _ => unreachable!(),
        }
        observed.push(json!({"case":case,"query_index":query_index,"ready":ready,"resolved":resolved,"plan":plan,"before":before,"after":after,"facts":facts}));
        println!(
            "U09_EXTERNAL_FAMILY {case} normal_reward_equip_query_cast_and_real_consumer=true"
        );
    }
    assert!(!observed.is_empty());
    write_json(&dir.join("observed.json"), &observed);
    write_json(&dir.join("export_manifest.json"), &manifest(&dir));
    if export.is_none() {
        fs::remove_dir_all(dir).unwrap();
    }
}
