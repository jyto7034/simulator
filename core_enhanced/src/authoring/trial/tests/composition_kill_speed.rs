#[test]
fn composition_kill_speed_world_armor_reward_equip_and_actual_cycle_change() {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.id == "standard_armor")
        .unwrap()
        .clone();
    accessory.id = "kill_speed_fixture_coat".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe980);
    accessory.name = "배달부 외투 기술 검증값".into();
    accessory.triggered_effects = ron::de::from_str(r#"{Permanent:[(target:SelfUnit,effect:Modifier((stat:MaxHealth,kind:Flat,value:80))),(target:SelfUnit,effect:Modifier((stat:MagicResist,kind:Flat,value:10)))]}"#).unwrap();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    // Hold the three authored victims on the interception cell; a one-block
    // sword lets two walk behind its forward range to the harmless objective.
    equipment
        .items
        .iter_mut()
        .find(|i| i.id == "standard_training_sword")
        .unwrap()
        .weapon_profile
        .as_mut()
        .unwrap()
        .block_capacity = 3;
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"kill_speed_fixture_coat\"),", 1));
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.max_health = 24;
    enemy.attack = 0;
    enemy.defense = 0;
    enemy.magic_resist = 0;
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
    fixture.author_trial_facility();
    // Preserve the opening victory and actual copy grant. Insert equip before
    // the next encounter; remap every later authority reference together.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(24 - preparation);
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
    document["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"clock",
        "clock":{"mode":"elapsed","delta_ms":30000,"after_seq":0}}));
    let source = include_str!("../../../../tests/fixtures/skill_composition/kill_speed.skill.yaml")
        .replace(
            "equipment: r11_weapon",
            "equipment: kill_speed_fixture_coat",
        );
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for points in [50, 100] {
        let current = source.replace(
            "attack_speed_points: 50",
            &format!("attack_speed_points: {points}"),
        );
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
        let mut copy = Value::Null;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("kill speed World {pending}: {e:?}"))
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 24 {
                assert_eq!(result["disposition"], "accepted");
                let units = result["payload"]["battle_update"]["checkpoint"]["units"]
                    .as_array()
                    .unwrap();
                let wearer = units
                    .iter()
                    .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                    .unwrap();
                assert_eq!(wearer["stats"]["max_health"], 180);
                assert_eq!(wearer["stats"]["magic_resist"], 10);
            }
            if index == 13 {
                let item = &result["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "kill_speed_fixture_coat");
                copy = item["uuid"].clone();
            }
            if index == 14 {
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(result["payload"]["requested_item_uuid"], copy);
            }
        }
        let observation = trial.session().observation().unwrap();
        assert!(
            observation.active().is_none(),
            "fixture must finish the equipped second encounter"
        );
        let terminal = observation
            .last_terminal()
            .expect("real equipped armor encounter must finish");
        let latest = terminal
            .mandatory()
            .latest_sourced_sustained()
            .expect("real wearer kill must grant state");
        let equipment = latest.equipment_source().unwrap();
        assert_eq!(equipment.definition_uuid, uuid::Uuid::from_u128(0xe980));
        assert_eq!(json!(equipment.persistent_copy_uuid), copy);
        let authored = latest.authored_source().unwrap();
        assert!(authored.field_path().contains("handlers[0].body[0]"));
        let origin = trial.origin_for_observed(authored).unwrap();
        assert!(origin.definition.ends_with("StateProgram"));
        assert!(latest.source().is_none());
        expected.push(json!({"points":points,"actual_time_ms":terminal.actual_time_ms(),"verdict":terminal.verdict(),"latest":latest,"origin":origin,"copy":copy}));
    }
    assert!(
        expected[1]["actual_time_ms"].as_u64().unwrap()
            < expected[0]["actual_time_ms"].as_u64().unwrap(),
        "more speed must shorten real cycles: {expected:?}"
    );
    if let Some(directory) = std::env::var_os("KILL_SPEED_COMPOSITION_FIXTURE_DIR") {
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
