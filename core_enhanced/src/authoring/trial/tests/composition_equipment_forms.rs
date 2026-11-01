#[test]
fn composition_equipment_forms_world_ui_actual_kill_and_form_source() {
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
    accessory.id = "equipment_forms_fixture_accessory".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe500);
    accessory.name = "양의 탈 형태 성장 실제 처치 시험 장신구".into();
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
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"equipment_forms_fixture_accessory\"),", 1));
    let path = "official/data/enemies/corroded_employees.ron";
    let enemies = fixture.read(path);
    assert!(enemies.contains("max_health: 300,"));
    fixture.write(path, enemies.replacen("max_health: 300,", "max_health: 1,", 1));
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
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_forms.skill.yaml")
            .replace(
                "equipment: r11_accessory",
                "equipment: equipment_forms_fixture_accessory",
            )
;
    let cancel = CancellationToken::default();
    let mut probe = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &source,
        checked(&document), None, TrialLimits::default(), &cancel).unwrap();
    let (stop, first) = loop {
        assert!(!probe.session().is_complete(), "actual eligible kill and victory must occur inside the fixed 20 second fixture");
        let step = probe.session().next_step_index();
        probe.advance(&cancel).unwrap_or_else(|error| panic!("form setup step {step}: {error:?}"));
        if step == 14 || step == 24 {
            let result = probe.session().step(step).unwrap().public_result().unwrap();
            assert_eq!(result["disposition"], "accepted", "required step {step}: {result}");
        }
        let observation = probe.session().observation().unwrap();
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()));
        if let Some(form) = observed.and_then(|o| o.latest_equipment_form()).filter(|f| f.confirmed == 1) {
            assert_eq!(form.pending, 0);
            assert_eq!((form.form.index, form.form.progress), (1, 0));
            let origin = probe.latest_equipment_form_origin().unwrap();
            assert!(origin.definition.contains("AttackProgram"));
            break (probe.session().next_step_index(), json!({"amount":10,"witness":form,"origin":origin}));
        }
    };
    let equipped = probe.session().step(14).unwrap().public_result().unwrap();
    assert_eq!(first["witness"]["equipment_copy"], equipped["payload"]["requested_item_uuid"]);
    drop(probe);
    document["steps"].as_array_mut().unwrap().truncate(stop - preparation);
    let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(),
        &source.replace("amount: 10 }", "amount: 15 }"), checked(&document), None, TrialLimits::default(), &cancel).unwrap();
    trial.run_to_end(&cancel).unwrap();
    let observation = trial.session().observation().unwrap();
    let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory())).unwrap();
    let form = observed.latest_equipment_form().unwrap();
    assert_eq!((form.pending, form.confirmed, form.form.index), (0, 1, 1));
    assert_eq!(form.form.effect, crate::game::ability::equipment_forms::EquipmentFormEffect::Attack { amount: 15 });
    let origin = trial.latest_equipment_form_origin().unwrap();
    let expected = json!([first, {"amount":15,"witness":form,"origin":origin}]);
    if let Some(directory) = std::env::var_os("EQUIPMENT_FORMS_COMPOSITION_FIXTURE_DIR") {
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
