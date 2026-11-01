use super::*;

#[test]
fn shield_beneficiary_world_actual_ring_reward_personal_grants_and_damage() {
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    assert!(equipment.get_by_id("porccubus_pleasure_ring").is_some());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path,fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"porccubus_pleasure_ring\"),",1));
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut enemy = enemies
        .profiles
        .iter()
        .find(|p| p.id == "corroded_rusher")
        .unwrap()
        .clone();
    enemy.id = "shield_beneficiary_enemy".into();
    enemy.uuid = uuid::Uuid::from_u128(0xe890);
    enemy.max_health = 10000;
    enemy.attack = 1;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    enemies.profiles.push(enemy);
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
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
            profile_id: "shield_beneficiary_enemy".into(),
            tier: crate::game::enums::Tier::I,
            count: 1,
        },
    ]);
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    fixture.author_trial_facility();
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
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(28 - preparation);
    for _ in 0..100 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source=include_str!("../../../../tests/fixtures/skill_composition/personal_shield.skill.yaml")
        .replace("version: 1","version: 2")
        .replace("authored-personal-shield","fragment_freischutz_black_round")
        .replace("capacity: 100","capacity: 25")
        .replace("values:\n","equipment_supports:\n  - equipment: porccubus_pleasure_ring\n    program: { Shared: shield-beneficiary-support }\nvalues:\n  - name: shield-beneficiary-support\n    value:\n      SupportProgram:\n        heal_percent: 0\n        shield_percent: 0\n        shield_beneficiary_damage_percent: 20\n");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [20u64, 35] {
        let current = source.replace("damage_percent: 20", &format!("damage_percent: {percent}"));
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
        let mut shields = Vec::new();
        let mut release_max_hp = 0;
        let mut latest = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("shield beneficiary World step{pending}: {e:?}"))
                .index();
            let public = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if [14, 26, 27].contains(&index) {
                assert_eq!(public["disposition"], "accepted", "step{index}: {public}");
            }
            if index < 28 {
                continue;
            }
            let update = &public["payload"]["battle_update"];
            let cp = &update["checkpoint"];
            if let Some(active) = cp["sustained_effects"]["active"].as_array() {
                let pools: Vec<_> = active
                    .iter()
                    .filter(|p| p["personal_grant"].is_object())
                    .cloned()
                    .collect();
                if !pools.is_empty() && shields.is_empty() {
                    let provider = &pools[0]["caster"];
                    release_max_hp = cp["units"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|u| &u["unit_instance_id"] == provider)
                        .unwrap()["stats"]["max_health"]
                        .as_u64()
                        .unwrap();
                    shields = pools;
                }
            }
            let Some(units) = cp["units"].as_array() else {
                continue;
            };
            let enemies: std::collections::BTreeSet<_> = units
                .iter()
                .filter(|u| u["owner"] == "Opponent")
                .filter_map(|u| u["unit_instance_id"].as_str())
                .collect();
            if let Some(events) = update["events_delta"]["events"].as_array() {
                for event in events {
                    if event["event"]["type"] == "HpChanged"
                        && event["event"]["damage_source"] == "BasicAttack"
                        && event["event"]["target_instance_id"]
                            .as_str()
                            .is_some_and(|t| enemies.contains(t))
                    {
                        hits.insert(event["seq"].as_u64().unwrap(), event.clone());
                    }
                }
            }
            let obs = trial.session().observation().unwrap();
            if let Some(w) = obs
                .active()
                .and_then(|o| o.latest_shield_beneficiary_damage())
            {
                assert_eq!(u64::from(w.percent), percent);
                let origin = trial.latest_shield_beneficiary_damage_origin().unwrap();
                assert!(origin.definition.ends_with("SupportProgram"));
                latest = Some((
                    serde_json::to_value(w).unwrap(),
                    serde_json::to_value(origin).unwrap(),
                ));
            }
        }
        assert_eq!(shields.len(), 2, "release grants both real employees");
        for pool in &shields {
            assert_eq!(pool["shield"]["capacity"], release_max_hp * 20 / 100);
        }
        let (witness, origin) =
            latest.expect("real equipped personal shield must increase actual damage");
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let item = &grant["payload"]["inventory_diff"]["added"][0]["Equipment"];
        assert_eq!(item["definition_id"], "porccubus_pleasure_ring");
        assert_eq!(witness["equipment"]["persistent_copy_uuid"], item["uuid"]);
        let expiry = shields[0]["expires_at_ms"].as_u64().unwrap();
        let begins = expiry - 6000;
        let enhanced: Vec<_> = hits
            .values()
            .filter(|e| (begins..expiry).contains(&e["time_ms"].as_u64().unwrap()))
            .collect();
        let ended: Vec<_> = hits
            .values()
            .filter(|e| e["time_ms"].as_u64().unwrap() >= expiry)
            .collect();
        assert!(
            !enhanced.is_empty() && !ended.is_empty(),
            "need active and expired public hits: {hits:?}"
        );
        for event in enhanced.iter().chain(ended.iter()) {
            let raw = event["event"]["raw_damage"].as_u64().unwrap();
            let rate = if event["time_ms"].as_u64().unwrap() < expiry {
                percent
            } else {
                0
            };
            assert_eq!(
                event["event"]["final_damage"],
                raw * (100 + rate) / 100,
                "{event}; shields={shields:?}; witness={witness}"
            );
        }
        expected.push(json!({"percent":percent,"witness":witness,"origin":origin,"shields":shields,"damage":hits.values().collect::<Vec<_>>(),"enhanced_hits":enhanced.len(),"expired_hits":ended.len()}));
    }
    if let Some(directory) = std::env::var_os("SHIELD_BENEFICIARY_COMPOSITION_FIXTURE_DIR") {
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
