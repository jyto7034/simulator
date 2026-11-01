use super::*;

#[test]
fn composition_equipment_speed_world_reward_equip_and_regular_attacks() {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment.items.iter().find(|i| i.id == "crimson_goggles").unwrap().clone();
    accessory.id = "speed_fixture_receiver".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe350);
    accessory.name = "울대 수신기 공속 연결 시험용".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"speed_fixture_receiver\"),", 1));
    fixture.author_trial_facility();
    // Preserve the opening victory and actual copy grant. Insert equip before
    // the next encounter; remap every later authority reference together.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    fn shift(value: &mut Value) {
        match value {
            Value::Object(map) => for (key, value) in map {
                if key == "step" && value.as_u64().is_some_and(|n| n >= 14) {
                    *value = json!(value.as_u64().unwrap() + 1);
                } else { shift(value); }
            },
            Value::Array(values) => for value in values { shift(value); },
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
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock",
        "clock":{"mode":"elapsed","delta_ms":20000,"after_seq":0}}));
    let source = include_str!("../../../../tests/fixtures/skill_composition/equipment_speed.skill.yaml")
        .replace("equipment: r11_weapon", "equipment: speed_fixture_receiver");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for cap in [0, 100] {
        let current = source.replace("max: { Const: { value: 100", &format!("max: {{ Const: {{ value: {cap}"));
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(),
            &current, checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        let mut granted_copy = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("speed World step {pending}: {e:?}")).index();
            if index == 14 {
                let result = trial.session().step(index).unwrap().public_result().unwrap();
                assert_eq!(result["disposition"], "accepted");
                let granted = trial.session().step(13).unwrap().public_result().unwrap();
                let item = &granted["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "speed_fixture_receiver");
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
                granted_copy = Some(item["uuid"].clone());
            }
        }
        let observation = trial.session().observation().unwrap();
        let terminal = observation.last_terminal().expect("actual equipped basic attacks must finish encounter");
        assert!(observation.active().is_none());
        let timing = terminal.mandatory().latest_equipment_speed().unwrap();
        let points = ((200u64 * u64::from(timing.max_hp - timing.current_hp)) / u64::from(timing.max_hp)).min(cap);
        assert_eq!(u64::from(timing.equipment_points), points);
        assert_eq!(timing.final_interval_ms, (timing.base_interval_ms * 100).div_ceil(100 + points));
        assert_eq!(timing.contributors.len(), 1);
        let contributor = &timing.contributors[0];
        assert_eq!(serde_json::to_value(contributor.equipment.persistent_copy_uuid).unwrap(),
            granted_copy.expect("the reward must be granted and equipped before battle"));
        assert_eq!(contributor.equipment.definition_uuid, uuid::Uuid::from_u128(0xe350));
        assert_eq!(contributor.definition_id, "equipment:speed_fixture_receiver");
        assert_eq!(contributor.field_path, "equipment_attacks[0].program.basic_speed_points");
        assert_eq!(u64::from(contributor.points), points);
        let origins = trial.latest_equipment_speed_origins().unwrap();
        assert_eq!(serde_json::to_value(&origins).unwrap(), json!([{
            "definition":"values[0].value.AttackProgram",
            "calls":["equipment_attacks[0]"],
            "inputs":["values[0].value", "equipment_attacks[0].program"]
        }]));
        expected.push(json!({"cap":cap,"actual_time_ms":terminal.actual_time_ms(),
            "verdict":terminal.verdict(),"applications":terminal.mandatory().applications(),
            "timing":timing,"origins":origins}));
    }
    eprintln!("equipment speed World: {expected:?}");
    assert!(expected[1]["actual_time_ms"].as_u64().unwrap() < expected[0]["actual_time_ms"].as_u64().unwrap(),
        "HP-dependent accessory must shorten actual attack cycles: {expected:?}");
    if let Some(directory) = std::env::var_os("EQUIPMENT_SPEED_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({
            "root":directory.join("content"),"selection":fixture.selection(),"scenario":document
        })).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}
