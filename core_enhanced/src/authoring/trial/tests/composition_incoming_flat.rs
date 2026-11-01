use super::*;

#[test]
fn incoming_flat_world_actual_armor_reward_copy_and_hostile_physical_basics() {
    let (fixture, _, mut document) = state_world_fixture();
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    assert!(equipment.get_by_id("black_swan_nettle_coat").is_some());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"black_swan_nettle_coat\"),",
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
    striker.id = "incoming_flat_striker".into();
    striker.uuid = uuid::Uuid::from_u128(0x9185);
    striker.name = "Fixed reduction trial striker".into();
    striker.max_health = 5000;
    striker.attack = 30;
    striker.basic_attack.damage_type = crate::game::battle::damage::DamageType::Physical;
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
            profile_id: "incoming_flat_striker".into(),
            tier: crate::game::enums::Tier::I,
            count: 1,
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
    // Observe repeated hostile basics on the actual equipped armor wearer.
    for _ in 0..160 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/incoming_flat.skill.yaml")
            .to_string();
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [50, 75] {
        let current = source.replace("value: 50,", &format!("value: {percent},"));
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
        let mut events = std::collections::BTreeMap::new();
        let mut wearer = None;
        while !trial.session().is_complete() {
            let index = trial.advance(&cancel).unwrap().index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if [14, 24].contains(&index) {
                assert_eq!(result["disposition"], "accepted");
            }
            if index < 25 {
                continue;
            }
            let update = &result["payload"]["battle_update"];
            if let Some(units) = update["checkpoint"]["units"].as_array() {
                wearer = units
                    .iter()
                    .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                    .map(|u| u["unit_instance_id"].clone())
                    .or(wearer);
            }
            if let Some(entries) = update["events_delta"]["events"].as_array() {
                for event in entries {
                    if event["event"]["type"] == "HpChanged" {
                        events.insert(event["seq"].as_u64().unwrap(), event.clone());
                    }
                }
            }
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .expect("armor wearer remains alive through ordinary hits");
        let w = observed
            .latest_incoming_flat()
            .expect("actual hostile basic fixed-reduction witness");
        let wearer = wearer.unwrap();
        assert_eq!(serde_json::to_value(w.target).unwrap(), wearer);
        assert!(w.incoming.action.is_some());
        assert!(w.incoming.actor.is_some());
        assert!(w.defense >= 20, "actual armor contributes Defense20");
        let allowance = (u64::try_from(w.defense).unwrap() * percent / 100).min(30) as u32;
        assert_eq!(w.allowance, allowance);
        assert_eq!(w.remaining, w.before - w.used);
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        assert_eq!(
            grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["definition_id"],
            "black_swan_nettle_coat"
        );
        assert_eq!(
            serde_json::to_value(w.equipment.persistent_copy_uuid).unwrap(),
            grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
        );
        assert_eq!(
            w.equipment.definition_uuid,
            equipment.get_by_id("black_swan_nettle_coat").unwrap().uuid
        );
        let origin = trial.latest_incoming_flat_origin().unwrap();
        assert!(origin.definition.ends_with("DefenseProgram"), "{origin:?}");
        assert_eq!(w.source_file, trial.identity().artifact_path);
        assert_eq!(
            w.field_path,
            "equipment_defenses[0].program.PhysicalBasicFlatReduction.amount"
        );
        let damage: Vec<_> = events
            .values()
            .filter(|e| {
                e["event"]["target_instance_id"] == wearer
                    && e["event"]["delta"].as_i64().is_some_and(|d| d < 0)
            })
            .cloned()
            .collect();
        assert!(damage.len() >= 3, "{damage:?}");
        let last = damage
            .iter()
            .find(|e| e["time_ms"].as_u64() == Some(w.time_ms))
            .unwrap();
        // The fixed-seed Easy encounter scales the authored Attack30 by90%.
        // Compare against the actual released component, after World scaling.
        assert_eq!(last["event"]["damage_source"], "BasicAttack");
        assert_eq!(last["event"]["damage_type"], "Physical");
        let raw = last["event"]["raw_damage"].as_u64().unwrap() as u32;
        assert_eq!(raw, 27);
        let post_resistance = raw * 100 / (100 + w.defense as u32);
        assert_eq!(w.used, allowance.min(post_resistance));
        let loss = post_resistance.saturating_sub(allowance).max(1);
        assert_eq!(-last["event"]["delta"].as_i64().unwrap(), i64::from(loss));
        expected.push(json!({"percent":percent,"witness":w,"origin":origin,"damage":damage,"last_hp_loss":loss}));
    }
    assert!(
        expected[0]["last_hp_loss"].as_u64() > expected[1]["last_hp_loss"].as_u64(),
        "same-source coefficient edit must alter actual HP loss: {expected:?}"
    );
    if let Some(directory) = std::env::var_os("INCOMING_FLAT_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(
            directory.join("trial.json"),
            serde_json::to_vec_pretty(&json!({
            "root":directory.join("content"),"selection":fixture.selection(),"scenario":document}))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
