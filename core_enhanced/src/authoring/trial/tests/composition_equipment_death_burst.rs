#[test]
fn composition_equipment_death_burst_world_ui_actual_kill_and_damage_source() {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.id == "crimson_goggles")
        .unwrap()
        .clone();
    accessory.id = "death_burst_fixture_accessory".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe500);
    accessory.name = "장례의 잔향 실제 처치와 지연 폭발 시험 장신구".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    assert_eq!(
        accessory.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Accessory
    );
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"death_burst_fixture_accessory\"),", 1));
    let path = "official/data/enemies/corroded_employees.ron";
    let enemies = fixture.read(path);
    assert!(enemies.contains("max_health: 300,"));
    fixture.write(
        path,
        enemies.replacen("max_health: 300,", "max_health: 1,", 1),
    );
    // A supported manual wave keeps a durable second enemy alive beside the first victim.
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(enemy_path)).unwrap();
    let mut durable = enemies
        .profiles
        .iter()
        .find(|e| e.id == "corroded_rusher")
        .unwrap()
        .clone();
    durable.id = "echo_durable_target".into();
    durable.uuid = uuid::Uuid::from_u128(0xe580);
    durable.max_health = 1000;
    durable.magic_resist = 0;
    enemies.profiles.push(durable);
    // Deserialization resolves preset ranges; preserve the authored reference on write.
    for profile in &mut enemies.profiles {
        if profile.basic_attack_range_preset.is_some() {
            profile.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    let encounter_path = "official/data/pve/encounters.ron";
    let mut encounters: crate::game::data::pve_data::PveEncounterDatabase =
        ron::de::from_str(&fixture.read(encounter_path)).unwrap();
    let encounter = encounters
        .encounters
        .iter_mut()
        .find(|e| e.id == "normal_breach_response")
        .unwrap();
    assert_eq!(encounter.waves.len(), 1);
    assert_eq!(
        encounter.waves[0].activation.primary,
        crate::game::data::pve_data::PveWavePrimaryTriggerData::AtTimeMs { time_ms: 5000 }
    );
    use crate::game::data::pve_data::{PveWaveEnemyData, PveWaveSource};
    encounter.waves[0].source = PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
        profile_id: "corroded_rusher".into(),
        tier: crate::game::enums::Tier::I,
        count: 1,
    }]);
    // The first victim reaches the employee before the durable follower. A simultaneous
    // spawn can put the follower in the only blocking slot while the victim walks past.
    let mut follower = encounter.waves[0].clone();
    follower.id = "echo_follower".into();
    follower.activation =
        crate::game::data::pve_data::PveWaveActivationData::at_time("echo_follower", 5200);
    follower.source = PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
        profile_id: "echo_durable_target".into(),
        tier: crate::game::enums::Tier::I,
        count: 1,
    }]);
    encounter.waves.push(follower);
    fixture.write(encounter_path, ron::ser::to_string(&encounters).unwrap());
    fixture.author_trial_facility();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    // Preserve the real first reward, equip, second encounter and deployed wearer.
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    fn shift(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "step" && value.as_u64().is_some_and(|n| n >= 14) {
                        *value = json!(value.as_u64().unwrap() + 1);
                    } else {
                        shift(value);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    shift(value);
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().insert(5, json!({
        "kind":"command", "command":{"type":"equip_item","item_uuid":null,"target_unit":null},
        "bindings":[
            {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}
        ]
    }));
    document["steps"][25 - preparation]["clock"]["delta_ms"] = json!(100);
    for _ in 0..200 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock",
            "clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source = include_str!(
        "../../../../tests/fixtures/skill_composition/equipment_death_burst.skill.yaml"
    )
    .replace(
        "equipment: r11_accessory",
        "equipment: death_burst_fixture_accessory",
    );
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    let mut stop = None;
    for percent in [100u32, 150] {
        let current = source.replace("attack_percent: 100", &format!("attack_percent: {percent}"));
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
        let mut reserved = None;
        loop {
            if trial.session().is_complete() {
                let snapshot = trial
                    .session()
                    .reconnect(0, &crate::authoring::ExecutionLimits::default(), &cancel)
                    .unwrap();
                eprintln!("burst World unreachable snapshot: {}", snapshot.document());
                panic!("actual equipped basic kill and damaging burst must occur inside the 20-second fixture");
            }
            let step = trial.session().next_step_index();
            trial
                .advance(&cancel)
                .unwrap_or_else(|error| panic!("burst World step {step}: {error:?}"));
            if step == 14 || step == 24 {
                let result = trial.session().step(step).unwrap().public_result().unwrap();
                assert_eq!(
                    result["disposition"], "accepted",
                    "required step {step}: {result}"
                );
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            let Some(witness) = observed.and_then(|o| o.latest_equipment_death_burst()) else {
                continue;
            };

            use crate::game::battle::core::equipment_death_burst::DeathBurstOperation;
            match witness.operation {
                DeathBurstOperation::Reserved => {
                    assert_eq!(witness.due_ms - witness.time_ms, 500);
                    assert_eq!(witness.base_damage, witness.attack * percent / 100);
                    reserved = Some(witness.id);
                }
                DeathBurstOperation::Detonated if witness.targets > 0 => {
                    assert_eq!(
                        reserved,
                        Some(witness.id),
                        "must observe reservation before damage"
                    );
                    let hp = observed
                        .unwrap()
                        .latest_nonzero_sourced_hp()
                        .expect("burst must have a real HP consumer");
                    let authored = hp
                        .authored_source()
                        .expect("burst damage must retain its authored equipment source");
                    assert!(authored.field_path().ends_with("death_burst"));
                    assert_eq!(hp.change().hp_delta(), -i64::from(witness.base_damage));
                    let origin = trial.latest_equipment_death_burst_origin().unwrap();
                    assert!(origin.definition.contains("AttackProgram"));
                    let equipped = trial.session().step(14).unwrap().public_result().unwrap();
                    assert_eq!(
                        json!(witness.equipment_copy),
                        equipped["payload"]["requested_item_uuid"]
                    );
                    expected
                        .push(json!({"percent":percent,"witness":witness,"hp":hp,"origin":origin}));
                    if let Some(stop) = stop {
                        assert_eq!(trial.session().next_step_index(), stop);
                    } else {
                        stop = Some(trial.session().next_step_index());
                    }
                    break;
                }
                DeathBurstOperation::Cancelled => {
                    panic!("burst cancelled before real World delivery: {witness:?}")
                }
                _ => (),
            }
        }
    }
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(stop.unwrap() - preparation);
    if let Some(directory) = std::env::var_os("DEATH_BURST_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(
            directory.join("trial.json"),
            serde_json::to_vec_pretty(&json!({
                "root":directory.join("content"),"selection":fixture.selection(),"scenario":document
            }))
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
