fn last_will_world_fixture() -> (ExternalFixture, Value) {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.equipment_type == crate::game::data::equipment_data::EquipmentType::Armor)
        .unwrap()
        .clone();
    accessory.id = "last_will_fixture_armor".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe540);
    accessory.name = "마지막 유언 실제 치명 피해와 퇴장 시험 방어구".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"last_will_fixture_armor\"),", 1));
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(enemy_path)).unwrap();
    let profile = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    profile.max_health = 5000;
    profile.attack = 10000;
    profile.basic_attack.interval_ms = 100000;
    profile.magic_resist = 0;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    fixture.author_trial_facility();
    // Preserve the opening victory and actual copy grant. Insert equip before
    // the next encounter; remap every later authority reference together.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
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
    (fixture, document)
}

#[test]
fn composition_last_will_world_preflight_actual_armor_lethal_blast_and_ko() {
    let (fixture, mut document) = last_will_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_last_will.skill.yaml");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    let mut stop = None;
    for percent in [300u32, 450] {
        let current = source.replace("attack_percent: 300", &format!("attack_percent: {percent}"));
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
        loop {
            if trial.session().is_complete() {
                let snapshot = trial
                    .session()
                    .reconnect(0, &crate::authoring::ExecutionLimits::default(), &cancel)
                    .unwrap();
                eprintln!("last will World unreachable: {}", snapshot.document());
                panic!("actual lethal armor reaction must occur within20second fixture");
            }
            let step = trial.session().next_step_index();
            trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("last will World step{step}: {e:?}"));
            if step == 14 || step == 24 {
                let result = trial.session().step(step).unwrap().public_result().unwrap();
                assert_eq!(
                    result["disposition"], "accepted",
                    "required step{step}: {result}"
                );
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            let Some(w) = observed.and_then(|o| o.latest_last_will()) else {
                continue;
            };
            assert_eq!((w.hp_protected, w.hp_after, w.incapacitated), (1, 0, true));
            assert!(w.targets > 0);
            assert_eq!(w.base_damage, w.attack * percent / 100);
            let hp = observed.unwrap().latest_nonzero_sourced_hp().unwrap();
            assert!(hp
                .authored_source()
                .unwrap()
                .field_path()
                .ends_with("program.LastWill"));
            assert_eq!(hp.change().hp_delta(), -i64::from(w.base_damage));
            let origin = trial.latest_last_will_origin().unwrap();
            assert!(origin.definition.ends_with("DefenseProgram"));
            let equipped = trial.session().step(14).unwrap().public_result().unwrap();
            assert_eq!(
                json!(w.equipment.persistent_copy_uuid),
                equipped["payload"]["requested_item_uuid"]
            );
            let result = trial.session().step(step).unwrap().public_result().unwrap();
            let deployment = &result["payload"]["battle_update"]["checkpoint"]["deployment"];
            let roster = deployment["redeploying_units"]
                .as_array()
                .expect("KO must reach real World deployment consumer");
            let employee = trial.session().step(9).unwrap().public_result().unwrap()["payload"]
                ["employee_uuid"]
                .clone();
            let waiting = roster
                .iter()
                .find(|row| row["employee_uuid"] == employee)
                .expect("wearer must wait for redeployment");
            assert_eq!(waiting["ready_at_ms"], json!(w.time_ms + 90000));
            assert!(!deployment["deployed_units"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["unit_instance_id"] == json!(w.actor)));
            expected.push(json!({"percent":percent,"witness":w,"hp":hp,"origin":origin,"deployment":deployment}));
            if let Some(stop) = stop {
                assert_eq!(trial.session().next_step_index(), stop);
            } else {
                stop = Some(trial.session().next_step_index());
            }
            break;
        }
    }
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(stop.unwrap() - preparation);
    if let Some(directory) = std::env::var_os("LAST_WILL_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
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

#[test]
fn composition_last_will_world_same_copy_redeployment_does_not_refill_and_injuries_accumulate() {
    let (fixture, mut document) = last_will_world_fixture();
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(enemy_path)).unwrap();
    let profile = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    profile.max_health = 800;
    profile.basic_attack.interval_ms = 500000;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    let path = "official/data/pve/encounters.ron";
    let mut encounters: crate::game::data::pve_data::PveEncounterDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let encounter = encounters
        .encounters
        .iter_mut()
        .find(|e| e.id == "normal_breach_response")
        .unwrap();
    use crate::game::data::pve_data::{PveWaveActivationData, PveWaveEnemyData, PveWaveSource};
    encounter.waves.truncate(1);
    encounter.waves[0].source = PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
        profile_id: "corroded_rusher".into(),
        tier: crate::game::enums::Tier::I,
        count: 1,
    }]);
    let mut next = encounter.waves[0].clone();
    next.id = "after_redeployment".into();
    next.activation = PveWaveActivationData::at_time("after_redeployment", 100000);
    encounter.waves.push(next);
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    let original_deploy = document["steps"][24 - preparation].clone();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    // Keep a second real employee alive: the encounter ends when all deployed
    // players are incapacitated even if a future required wave remains.
    let cancel_probe = CancellationToken::default();
    let token = fixture.load();
    let probe = simulate(
        fixture_content(&token),
        checked(&document),
        TrialLimits::default(),
        &cancel_probe,
    )
    .unwrap();
    let entry = probe.step(21).unwrap().public_result().unwrap();
    let setup = &entry["payload"]["battle_setup_snapshot"];
    let zones = setup["deployment_zones"].as_array().unwrap();
    let ground = zones.iter().position(|z| z["kind"] == "Ground").unwrap();
    let cells = zones[ground]["cells"].as_array().unwrap();
    let pointer = original_deploy["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["to"] == "/command/position")
        .unwrap()["from"]["pointer"]
        .as_str()
        .unwrap();
    let wearer = entry.pointer(pointer).unwrap();
    let occupied = entry["payload"]["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let route = setup["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "defense_main")
        .unwrap()["cells"]
        .as_array()
        .unwrap();
    let guard_index = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| *c != wearer && !occupied.iter().any(|u| u["position"] == **c))
        .max_by_key(|(_, c)| {
            (
                !route.contains(c),
                (c["x"].as_i64().unwrap() - wearer["x"].as_i64().unwrap()).abs()
                    + (c["y"].as_i64().unwrap() - wearer["y"].as_i64().unwrap()).abs(),
            )
        })
        .unwrap()
        .0;
    let mut guard = original_deploy.clone();
    for b in guard["bindings"].as_array_mut().unwrap() {
        if b["to"].as_str().unwrap().starts_with("/context/") {
            b["from"]["step"] = json!(24);
        }
        if b["to"] == "/command/employee_uuid" {
            b["from"]["step"] = json!(10);
        }
        if b["to"] == "/command/position" {
            b["from"]["pointer"] = json!(format!(
                "/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{guard_index}"
            ));
        }
    }
    drop(probe);
    let deploy = |authority| {
        let mut d = original_deploy.clone();
        for b in d["bindings"].as_array_mut().unwrap() {
            if b["to"].as_str().unwrap().starts_with("/context/") {
                b["from"]["step"] = json!(authority);
            }
        }
        d
    };
    let clock =
        |delta| json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":delta,"after_seq":0}});
    document["steps"].as_array_mut().unwrap().extend([
        guard,
        clock(9000),
        json!({"kind":"snapshot","snapshot":"employee_roster"}),
        deploy(26),
        clock(60000),
        clock(30000),
        deploy(30),
        clock(10000),
        json!({"kind":"snapshot","snapshot":"employee_roster"}),
    ]);
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_last_will.skill.yaml")
            .replace("attack_percent: 300", "attack_percent: 10000");
    let cancel = CancellationToken::default();
    // An application-level cooldown error aborts a speculative trial step;
    // validate that real rejection in a separate finite trial, preserving the
    // continuation scenario and its step references.
    let mut early = document.clone();
    early["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(29 - preparation);
    let mut cooldown = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&early),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    while cooldown.session().next_step_index() < 28 {
        cooldown.advance(&cancel).unwrap();
    }
    let before = serde_json::to_value(cooldown.session().observation().unwrap()).unwrap();
    let failure = cooldown.advance(&cancel).unwrap_err();
    match failure.error {
        AuthoringError::Application { diagnostic, .. } => {
            assert_eq!(diagnostic.code(), "redeploy_cooldown")
        }
        other => panic!("expected actual deployment cooldown, got {other:?}"),
    }
    assert_eq!(cooldown.session().next_step_index(), 28);
    assert_eq!(
        serde_json::to_value(cooldown.session().observation().unwrap()).unwrap(),
        before
    );
    drop(cooldown);
    document["steps"][28 - preparation] = json!({"kind":"snapshot","snapshot":"employee_roster"});
    let mut trial = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&document),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    while !trial.session().is_complete() {
        let step = trial.session().next_step_index();
        trial
            .advance(&cancel)
            .unwrap_or_else(|e| panic!("last will redeployment step{step}: {e:?}"));
        if [14, 24, 25, 31].contains(&step) {
            let result = trial.session().step(step).unwrap().public_result().unwrap();
            assert_eq!(result["disposition"], "accepted", "step{step}: {result}");
        }
        if step == 26 {
            let o = trial.session().observation().unwrap();
            let w = o
                .active()
                .expect("future required wave must keep battle active")
                .latest_last_will()
                .unwrap();
            assert_eq!(w.base_damage, 1200);
            assert!(w.incapacitated);
        }
    }
    let employee = trial.session().step(9).unwrap().public_result().unwrap()["payload"]
        ["employee_uuid"]
        .clone();
    let original = trial.session().step(24).unwrap().public_result().unwrap();
    let redeployed = trial.session().step(31).unwrap().public_result().unwrap();
    let actor = redeployed["payload"]["unit_instance_id"].clone();
    assert_ne!(actor, original["payload"]["unit_instance_id"]);
    let observation = trial.session().observation().unwrap();
    let w = observation.active().unwrap().latest_last_will().unwrap();
    assert_ne!(
        json!(w.actor),
        actor,
        "second lethal hit must not publish another blast"
    );
    let last = trial.session().step(32).unwrap().public_result().unwrap();
    let deployment = &last["payload"]["battle_update"]["checkpoint"]["deployment"];
    assert!(!deployment["deployed_units"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v["unit_instance_id"] == actor));
    assert!(deployment["redeploying_units"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v["employee_uuid"] == employee));
    for (step, injuries) in [(27, 1), (33, 2)] {
        let roster = trial.session().step(step).unwrap().public_result().unwrap();
        let row = roster["read_model"]["employees"]
            .as_array()
            .expect("actual roster snapshot")
            .iter()
            .find(|e| e["uuid"] == employee)
            .unwrap();
        assert_eq!(row["condition"]["physical_damage"], json!(injuries));
        assert!(row["condition"]["trauma"].as_u64().unwrap() > 0);
        let armor = row["equipped_items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["definition_id"] == "last_will_fixture_armor")
            .unwrap();
        assert_eq!(
            armor["instance_uuid"],
            json!(w.equipment.persistent_copy_uuid)
        );
    }
}
