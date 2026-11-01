#[test]
fn composition_shared_cohort_world_actual_accessory_activates_without_deployment_or_hood_fragment()
{
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
    accessory.id = "shared_cohort_fixture_accessory".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe500);
    accessory.name = "공유 사냥감 실제 출현·미배치 주기 시험 장신구".into();
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
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"shared_cohort_fixture_accessory\"),", 1));
    fixture.author_trial_facility();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    // The ordinary first victory grants the actual item. End after the second
    // encounter entry; the recipient remains on the undeployed roster.
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(21 - preparation);
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
        "clock":{"mode":"elapsed","delta_ms":6000,"after_seq":0}}));
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/shared_cohort.skill.yaml")
            .replace(
                "equipment: r11_accessory",
                "equipment: shared_cohort_fixture_accessory",
            );
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for hp in [300u32, 450] {
        let current = source.replace("hp: 300", &format!("hp: {hp}"));
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
        trial.run_to_end(&cancel).unwrap();
        let granted = trial.session().step(13).unwrap().public_result().unwrap();
        let equipped = trial.session().step(14).unwrap().public_result().unwrap();
        assert_eq!(equipped["disposition"], "accepted");
        let copy = &granted["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"];
        assert_eq!(equipped["payload"]["requested_item_uuid"], *copy);
        let entry = trial.session().step(21).unwrap().public_result().unwrap();
        let checkpoint = &entry["payload"]["battle_update"]["checkpoint"];
        assert!(checkpoint["input_plans"]["owners"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(
            checkpoint["shared_cohorts"]["owners"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(checkpoint["units"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["unit_source"]["kind"] != "employee"));
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let cohort = observed
            .latest_shared_cohort()
            .expect("equipped undeployed roster must spawn an actual hostile");
        assert_eq!(cohort.initial_hp, hp);
        assert_eq!(cohort.time_ms, 5000);
        assert_eq!(cohort.next_spawn_ms, 10000);
        assert_eq!(cohort.provider_count, 1);
        assert_eq!(cohort.fragment_recipient, None);
        assert_eq!(serde_json::to_value(cohort.equipment_copy).unwrap(), *copy);
        let origin = trial.latest_shared_cohort_origin().unwrap();
        assert!(origin.definition.contains("AttackProgram"));
        expected.push(json!({"hp":hp,"cohort":cohort,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("SHARED_COHORT_COMPOSITION_FIXTURE_DIR") {
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
