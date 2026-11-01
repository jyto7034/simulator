#[test]
fn composition_kill_cast_world_actual_three_equipment_copies_and_public_casts() {
    let (fixture, _, mut document) = state_world_fixture();
    let armor = "placeholder_warm_hearted_woodsman_armor";
    let accessory = "placeholder_warm_hearted_woodsman_accessory";
    let weapon = "placeholder_warm_hearted_woodsman_weapon";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let profile = equipment
        .get_by_id(weapon)
        .unwrap()
        .weapon_profile
        .as_ref()
        .unwrap();
    assert_eq!(profile.block_capacity, 2);
    assert_eq!(
        profile.weapon_archetype,
        crate::game::data::equipment_data::WeaponArchetype::Sword
    );
    let armor_id = equipment.get_by_id(armor).unwrap().uuid;
    let accessory_id = equipment.get_by_id(accessory).unwrap().uuid;
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path,fixture.read(path).replacen("GrantEnkephalin(amount: 100),",&format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id:{armor:?}), GrantEquipment(equipment_id:{accessory:?}), GrantEquipment(equipment_id:{weapon:?}),"),1));
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.max_health = 160;
    enemy.attack = 4;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    enemy.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    enemy.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
    enemy.basic_attack.interval_ms = 400;
    enemy.basic_attack_range_preset = Some("ranged_center_5x5".into());
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let path = "official/data/enemies/corroded_wave_presets.ron";
    let mut waves: crate::game::data::corroded_wave_data::CorrodedWavePresetDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let wave = waves
        .presets
        .iter_mut()
        .find(|p| p.id == "breach_response_light")
        .unwrap();
    wave.budget = 3;
    wave.count_range =
        crate::game::data::corroded_wave_data::CorrodedWaveCountRange { min: 3, max: 3 };
    wave.role_mix[0].min_count = 3;
    wave.role_mix[0].max_count = Some(3);
    fixture.write(path, ron::ser::to_string(&waves).unwrap());
    let path = "official/data/skill_fragments/base.ron";
    fixture.write(
        path,
        fixture
            .read(path)
            .replace("resonance_max: 25", "resonance_max: 1"),
    );
    fixture.author_trial_facility();
    let prep = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(24 - prep);
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= 14) {
                        *v = json!(v.as_u64().unwrap() + 3);
                    } else {
                        shift(v)
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    shift(v)
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    for index in 0..3 {
        document["steps"].as_array_mut().unwrap().insert(5+index,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":format!("/payload/inventory_diff/added/{index}/Equipment/uuid")},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    }
    for tick in 1..=120 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
        if [60, 80].contains(&tick) {
            for kind in ["activate_skill"] {
                let previous = prep + document["steps"].as_array().unwrap().len() - 1;
                let mut command = authority_step(kind, previous);
                command["command"]["employee_uuid"] = Value::Null;
                command["command"]["skill_id"] = json!("fragment_freischutz_black_round");
                command["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
                document["steps"].as_array_mut().unwrap().push(command);
            }
        }
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/kill_cast_state.skill.yaml")
            .replace("equipment: r11_weapon", &format!("equipment: {accessory}"))
            .replace(
                "definitions: []",
                &format!(
            "  - equipment: {armor}\n    program: {{ Shared: direct-kill-heal }}\ndefinitions: []"
        ),
            );
    let mut children = String::new();
    for i in 0..3 {
        children.push_str(&format!(r#"          - Wait:
              id: wait-{i}
              duration: {{ Literal: {{ Milliseconds: 100 }} }}
          - Action:
              id: hit-{i}
              target: {{ Literal: {{ Target: CastTarget }} }}
              delivery: {{ Literal: {{ Delivery: Instant }} }}
              effects:
                - Damage:
                    id: damage
                    tags: []
                    damage_type: Physical
                    amount: {{ Literal: {{ Amount: {{ moment: Snapshot, root: {{ Const: {{ value: 80, unit: Magnitude }} }} }} }} }}
              wait: EffectsSettled
"#));
    }
    let source=source.replace("skills: []",&format!(r#"skills:
  - id: fragment_freischutz_black_round
    name: 장비 직접 처치와 다음 시전 검사
    kind: Targeted
    cast_targeting:
      Explicit: {{ selection: automatic_unit, target: {{ EnemySingle: {{ rule: Nearest }} }}, reach: global, air_capable: true }}
    prepare_ms: 10
    focus_permissions: {{ allows_move: false, allows_basic_attack: false }}
    admission: {{ capacity: 1, requirement: Full, cost: All }}
    body:
      Sequence:
        id: three-hits
        children:
{children}"#));
    let mut expected = Vec::new();
    for percent in [20, 40] {
        let current = source.replace(
            "value: 20, unit: Scalar",
            &format!("value: {percent}, unit: Scalar"),
        );
        let cancel = CancellationToken::default();
        let mut trial = CompositionTrial::prepare(
            fixture.root(),
            &fixture.selection(),
            &current,
            checked(&document),
            None,
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        let mut hits = std::collections::BTreeMap::new();
        let mut heals = std::collections::BTreeMap::new();
        let mut copies = Vec::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("kill/cast World {pending}: {e:?}"))
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if document["steps"]
                .get(index.saturating_sub(prep))
                .is_some_and(|s| s["command"]["type"] == "activate_skill")
            {
                assert_eq!(
                    result["disposition"], "accepted",
                    "public cast {index}: {result:?}"
                );
            }
            if index == 13 {
                for i in 0..3 {
                    copies.push(
                        result["payload"]["inventory_diff"]["added"][i]["Equipment"]["uuid"]
                            .clone(),
                    );
                }
            }
            if (14..=16).contains(&index) {
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(result["payload"]["requested_item_uuid"], copies[index - 14]);
            }
            let observation = trial.session().observation().unwrap();
            if let Some(facts) = observation
                .active_runtime_facts()
                .or_else(|| observation.last_terminal().map(|t| t.runtime_facts()))
            {
                for (_, row) in facts.actors() {
                    if let Some(hit) = row
                        .caused
                        .latest_damage
                        .as_ref()
                        .filter(|h| h.actual_hp_loss() > 0)
                    {
                        if let Some(capture) = hit.source.cast_damage() {
                            assert_eq!(capture.percent, percent);
                            assert_eq!(capture.contributors.len(), 1);
                            assert_eq!(
                                capture.contributors[0].equipment.definition_uuid,
                                accessory_id
                            );
                            assert_eq!(
                                json!(capture.contributors[0].equipment.persistent_copy_uuid),
                                copies[1]
                            );
                            assert!(capture.contributors[0]
                                .authored
                                .field_path
                                .contains("handlers[1]"));
                            hits.insert(
                                (hit.time_ms, hit.target),
                                json!({"hit":hit,"loss":hit.actual_hp_loss(),"capture":capture}),
                            );
                        }
                    }
                    if let Some(heal) = row.caused.latest_heal.as_ref().filter(|h| {
                        h.source
                            .equipment
                            .is_some_and(|e| e.definition_uuid == armor_id)
                    }) {
                        heals.insert(heal.time_ms, json!(heal));
                    }
                }
            }
        }
        if hits.is_empty() {
            eprintln!(
                "World final observation: {:?}",
                serde_json::to_value(trial.session().observation().unwrap()).unwrap()
            );
            for index in [26, 27, 50, 100] {
                eprintln!(
                    "World step {index}: {:?}",
                    trial.session().step(index).unwrap().public_result()
                );
            }
        }
        assert!(!hits.is_empty(), "no boosted actual cast: {heals:?}");
        assert!(
            !heals.is_empty(),
            "actual armor direct kill must heal; hits={hits:?}"
        );
        expected.push(json!({"percent":percent,"hits":hits.values().collect::<Vec<_>>(),"heals":heals.values().collect::<Vec<_>>(),"copies":copies}));
    }
    assert_eq!(
        expected[0]["hits"][0]["hit"]["time_ms"],
        expected[1]["hits"][0]["hit"]["time_ms"]
    );
    assert!(
        expected[1]["hits"][0]["loss"].as_u64().unwrap()
            > expected[0]["hits"][0]["loss"].as_u64().unwrap(),
        "{expected:?}"
    );
    if let Some(directory) = std::env::var_os("KILL_CAST_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
