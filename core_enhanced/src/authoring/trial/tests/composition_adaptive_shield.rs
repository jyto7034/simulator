use super::*;

#[test]
fn composition_adaptive_shield_world_actual_armor_copy_and_normal_attack() {
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
    accessory.id = "adaptive_fixture_armor".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe450);
    accessory.name = "미미크리 적응형 보호막 연결 시험용".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"adaptive_fixture_armor\"),", 1));
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
    document["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"clock",
        "clock":{"mode":"elapsed","delta_ms":20000,"after_seq":0}}));
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/adaptive_shield.skill.yaml")
            .replace("equipment: r11_weapon", "equipment: adaptive_fixture_armor");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [20u16, 35] {
        let current = source.replace("max_hp_percent: 20", &format!("max_hp_percent: {percent}"));
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
        let mut granted_copy = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("adaptive shield World step {pending}: {e:?}"))
                .index();
            if index == 14 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                assert_eq!(result["disposition"], "accepted");
                let granted = trial.session().step(13).unwrap().public_result().unwrap();
                let item = &granted["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "adaptive_fixture_armor");
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
                granted_copy = Some(item["uuid"].clone());
            }
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let shield = observed
            .latest_adaptive_shield()
            .expect("equipped employee must land a normal primary attack");
        assert_eq!(json!(shield.equipment.persistent_copy_uuid),
            granted_copy.expect("actual reward must be equipped before this effect"));
        assert_eq!(shield.capacity, shield.max_hp * u32::from(percent) / 100);
        assert_eq!(shield.expires_at_ms - shield.time_ms, 5000);
        assert_eq!(shield.ready_at_ms - shield.time_ms, 15000);
        assert_eq!(shield.definition_id, "adaptive_fixture_armor");
        assert!(shield.attack_source.action.is_some());
        assert_ne!(
            shield.attack_source.equipment,
            Some(shield.equipment),
            "attack weapon and defensive armor are separate copies"
        );
        let origin = trial.latest_adaptive_shield_origin().unwrap();
        assert!(origin.definition.ends_with("DefenseProgram"), "{origin:?}");
        expected.push(json!({"percent":percent,"shield":shield,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("ADAPTIVE_SHIELD_COMPOSITION_FIXTURE_DIR") {
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
