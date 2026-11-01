use super::*;

#[test]
fn pleasure_world_actual_reward_ground_magic_area_and_nonstack_weakness() {
    let (fixture, _, mut document) = state_world_fixture();
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    assert!(equipment.get_by_id("porccubus_pleasure").is_some());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"porccubus_pleasure\"),",
            1,
        ),
    );
    // Isolate the second encounter while preserving the prerequisite reward battle.
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut striker = enemies
        .profiles
        .iter()
        .find(|p| p.id == "corroded_rusher")
        .unwrap()
        .clone();
    striker.id = "pleasure_striker".into();
    striker.uuid = uuid::Uuid::from_u128(0x9187);
    striker.name = "Pleasure trial striker".into();
    striker.max_health = 1000;
    striker.attack = 1;
    striker.defense = 0;
    striker.magic_resist = 0;
    striker.basic_attack.interval_ms = 1000;
    enemies.profiles.push(striker);
    // Deserialization resolves range presets; avoid serializing both authored authorities.
    for profile in &mut enemies.profiles {
        if profile.basic_attack_range_preset.is_some() {
            profile.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let path = "official/data/pve/encounters.ron";
    let mut encounters: crate::game::data::pve_data::PveEncounterDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let encounter = encounters
        .encounters
        .iter_mut()
        .find(|e| e.id == "normal_breach_response")
        .unwrap();
    encounter.waves.truncate(1);
    encounter.waves[0].source = crate::game::data::pve_data::PveWaveSource::Manual(vec![
        crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
            profile_id: "pleasure_striker".into(),
            tier: crate::game::enums::Tier::I,
            count: 2,
        },
    ]);
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    fixture.author_trial_facility();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= 14) {
                        *v = json!(v.as_u64().unwrap() + 1)
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
    document["steps"].as_array_mut().unwrap().insert(5,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    document["steps"][25 - preparation]["clock"]["delta_ms"] = json!(1);
    // Observe initial normal damage, then later Magic hits under the refreshed weakness.
    for _ in 0..160 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/pleasure.skill.yaml")
        .to_string();
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [20u64, 35] {
        let current = source.replace("percent: 20", &format!("percent: {percent}"));
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
        let mut first_applied = None;
        let mut seen_targets = std::collections::BTreeSet::new();
        let mut hits = std::collections::BTreeMap::new();
        let mut latest = None;
        let mut saw_block = false;
        while !trial.session().is_complete() {
            let index = trial.advance(&cancel).unwrap().index();
            if [14, 24].contains(&index) {
                assert_eq!(
                    trial
                        .session()
                        .step(index)
                        .unwrap()
                        .public_result()
                        .unwrap()["disposition"],
                    "accepted"
                );
            }
            if index < 25 {
                continue;
            }
            let public = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            let update = &public["payload"]["battle_update"];
            let cp = &update["checkpoint"];
            let Some(units) = cp["units"].as_array() else {
                continue;
            };
            let wearer = units
                .iter()
                .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                .unwrap();
            assert_eq!(wearer["tactical_state"]["base_block_capacity"], 3);
            saw_block |= wearer["basic_action"]["blocking"]
                .as_array()
                .is_some_and(|a| !a.is_empty());
            let enemies: std::collections::BTreeSet<_> = units
                .iter()
                .filter(|u| u["owner"] == "Opponent")
                .map(|u| u["unit_instance_id"].as_str().unwrap().to_string())
                .collect();
            if let Some(events) = update["events_delta"]["events"].as_array() {
                for event in events {
                    if event["event"]["type"] == "HpChanged"
                        && event["event"]["source_instance_id"] == wearer["unit_instance_id"]
                        && event["event"]["target_instance_id"]
                            .as_str()
                            .is_some_and(|t| enemies.contains(t))
                    {
                        assert_eq!(
                            event["event"]["damage_source"], "BasicAttack",
                            "weapon must not generate poison ticks"
                        );
                        assert_eq!(event["event"]["damage_type"], "Magic");
                        hits.insert(event["seq"].as_u64().unwrap(), event.clone());
                    }
                }
            }
            if let Some(active) = cp["sustained_effects"]["active"].as_array() {
                for state in active {
                    if state["hit_status"]["definition"] == "pleasure-weakness" {
                        assert_eq!(
                            state["hit_status"]["kind"]["DamageTaken"]["percent"],
                            percent
                        );
                        seen_targets.insert(state["target"].as_str().unwrap().to_string());
                    }
                }
            }
            let observation = trial.session().observation().unwrap();
            let Some(observed) = observation.active() else {
                panic!("second encounter ended before the interval observation");
            };
            if let Some(w) = observed.latest_sourced_sustained() {
                let crate::authoring::ApplicationChange::Sustained {
                    damage_taken_increase_percent: Some(actual),
                    ended: None,
                    expires_at_ms,
                    ..
                } = w.change()
                else {
                    continue;
                };
                assert_eq!(u64::from(actual), percent);
                first_applied.get_or_insert(w.time_ms());
                let grant = trial.session().step(13).unwrap().public_result().unwrap();
                assert_eq!(
                    grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["definition_id"],
                    "porccubus_pleasure"
                );
                assert_eq!(
                    serde_json::to_value(w.equipment_source().unwrap().persistent_copy_uuid)
                        .unwrap(),
                    grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
                );
                assert_eq!(
                    w.source_unit().unwrap().to_string(),
                    wearer["unit_instance_id"].as_str().unwrap()
                );
                let origin = trial
                    .origin_for_observed(w.authored_source().unwrap())
                    .unwrap();
                assert!(origin.definition.ends_with("AttackProgram"));
                latest = Some(
                    json!({"percent":percent,"time_ms":w.time_ms(),"expires_at_ms":expires_at_ms,"target_unit":w.target_unit(),"equipment":w.equipment_source(),"origin":origin,"step":index}),
                );
            }
        }
        assert!(saw_block);
        assert_eq!(
            seen_targets.len(),
            2,
            "both ordinary enemies must receive the actual area basic status"
        );
        let first = first_applied.unwrap();
        let mut grouped = std::collections::BTreeMap::<String, Vec<Value>>::new();
        for (_, event) in hits {
            grouped
                .entry(
                    event["event"]["target_instance_id"]
                        .as_str()
                        .unwrap()
                        .into(),
                )
                .or_default()
                .push(event);
        }
        assert_eq!(grouped.len(), 2);
        for (target, events) in &grouped {
            assert!(
                events.len() >= 3,
                "{target} expected repeated real area basics: {events:?}"
            );
            for (i, e) in events.iter().enumerate() {
                let raw = e["event"]["raw_damage"].as_u64().unwrap();
                assert!(raw > 0);
                let actual = -e["event"]["delta"].as_i64().unwrap();
                let expected = if i == 0 {
                    raw
                } else {
                    raw * (100 + percent) / 100
                };
                assert_eq!(actual, expected as i64, "target{target} hit{i}: {e}");
                assert!(e["time_ms"].as_u64().unwrap() >= first);
            }
        }
        let mut value = latest.expect("actual Pleasure weakness witness");
        value["public_magic_hits"] = json!(grouped);
        value["affected_targets"] = json!(seen_targets);
        expected.push(value);
    }
    if let Some(directory) = std::env::var_os("PLEASURE_COMPOSITION_FIXTURE_DIR") {
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
