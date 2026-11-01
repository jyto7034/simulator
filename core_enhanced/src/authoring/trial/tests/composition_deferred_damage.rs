use super::*;

#[test]
fn composition_deferred_damage_world_actual_armor_and_scheduled_repayment() {
    let (fixture, _, mut document) = state_world_fixture();
    let enemies = "official/data/enemies/corroded_employees.ron";
    let enemy_source = fixture.read(enemies);
    assert!(enemy_source.contains("attack: 1,"));
    fixture.write(
        enemies,
        enemy_source.replacen("attack: 1,", "attack: 8,", 1),
    );
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.equipment_type == crate::game::data::equipment_data::EquipmentType::Armor)
        .unwrap()
        .clone();
    accessory.id = "deferred_damage_fixture_armor".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe480);
    accessory.name = "붉은 흉터 독립 피해 유예·상환 시험용".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"deferred_damage_fixture_armor\"),", 1));
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
    for _ in 0..200 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock", "clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/deferred_damage.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: deferred_damage_fixture_armor",
            );
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [30u8, 50] {
        let current = source.replace("defer_percent: 30", &format!("defer_percent: {percent}"));
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
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("deferred damage World step {pending}: {e:?}"))
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
                assert_eq!(item["definition_id"], "deferred_damage_fixture_armor");
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
            }
            let observation = trial.session().observation().unwrap();
            if observation
                .active()
                .and_then(|o| o.latest_deferred_damage())
                .is_some_and(|d| {
                    matches!(
                        d.operation,
                        crate::game::battle::core::equipment_defense::DeferredOperation::Repay
                    ) && d.actual_hp_loss > 0
                })
            {
                if percent == 30 {
                    document["steps"]
                        .as_array_mut()
                        .unwrap()
                        .truncate(index + 1 - preparation);
                    break;
                }
            }
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let deferred = observed
            .latest_deferred_damage()
            .expect("equipped employee must receive hostile damage then repay");
        assert!(matches!(
            deferred.operation,
            crate::game::battle::core::equipment_defense::DeferredOperation::Repay
        ));
        assert!(deferred.actual_hp_loss > 0);
        assert_eq!(u64::from(deferred.actual_hp_loss), deferred.amount);
        assert!(deferred.equipment.persistent_copy_uuid.is_some());
        let granted = trial.session().step(13).unwrap().public_result().unwrap();
        assert_eq!(
            serde_json::to_value(deferred.equipment.persistent_copy_uuid).unwrap(),
            granted["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
        );
        assert_eq!(deferred.definition_id, "deferred_damage_fixture_armor");
        assert_ne!(deferred.incoming.actor, Some(deferred.target));
        let origin = trial.latest_deferred_damage_origin().unwrap();
        assert!(origin.definition.ends_with("DefenseProgram"), "{origin:?}");
        expected.push(json!({"percent":percent,"deferred":deferred,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("DEFERRED_DAMAGE_COMPOSITION_FIXTURE_DIR") {
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
