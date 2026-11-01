use super::*;

#[test]
fn composition_magazine_world_actual_gun_copy_and_released_ammunition() {
    magazine_world(false);
}
#[test]
fn composition_plain_magazine_world_actual_explosive_copy_and_data_change() {
    magazine_world(true);
}
fn magazine_world(area: bool) {
    let (fixture, _, mut document) = state_world_fixture();
    let equipment_id = if area { "plain_magazine_fixture_gun" } else { "magazine_fixture_gun" };
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.equipment_type == crate::game::data::equipment_data::EquipmentType::Weapon)
        .unwrap()
        .clone();
    accessory.id = equipment_id.into();
    accessory.uuid = uuid::Uuid::from_u128(if area { 0xe970 } else { 0xe490 });
    accessory.name = if area { "유탄 실제 탄창·폭발 시험용" } else { "핸드캐논 실제 탄창·발사 시험용" }.into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    accessory.weapon_profile = Some(
        ron::de::from_str(if area {
            include_str!("../../../../tests/fixtures/skill_composition/grenade.weapon.ron")
        } else {
            include_str!("../../../../tests/fixtures/skill_composition/handcannon.weapon.ron")
        })
        .unwrap(),
    );
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            &format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"{equipment_id}\"),"),
            1,
        ),
    );
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
    let source = (if area {
        include_str!("../../../../tests/fixtures/skill_composition/plain_magazine.skill.yaml")
    } else {
        include_str!("../../../../tests/fixtures/skill_composition/magazine.skill.yaml")
    }).replace("equipment: r11_weapon", &format!("equipment: {equipment_id}"));
    let cancel = CancellationToken::default();
    let mut pre = document.clone();
    pre["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(24 - preparation);
    let mut probe = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&pre),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    probe.run_to_end(&cancel).unwrap();
    let entry = probe.session().step(21).unwrap().public_result().unwrap();
    let zones = entry["payload"]["battle_setup_snapshot"]["deployment_zones"]
        .as_array()
        .unwrap();
    let platform = zones.iter().position(|z| z["kind"] == "Platform").unwrap();
    assert!(!zones[platform]["cells"].as_array().unwrap().is_empty());
    for b in document["steps"][24 - preparation]["bindings"]
        .as_array_mut()
        .unwrap()
    {
        if b["to"] == "/command/position" {
            b["from"]["pointer"] = json!(format!(
                "/payload/battle_setup_snapshot/deployment_zones/{platform}/cells/0"
            ));
        }
    }
    document["steps"][25 - preparation]["clock"]["delta_ms"] = json!(100);
    drop(probe);
    for _ in 0..200 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    let variants = if area { [(6u32, 100u32), (6, 200)] } else { [(6, 100), (9, 100)] };
    for (variant, (capacity, percent)) in variants.into_iter().enumerate() {
        let current = source.replace("capacity: 6", &format!("capacity: {capacity}"))
            .replace("normal_percent: 100", &format!("normal_percent: {percent}"));
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
                .unwrap_or_else(|e| panic!("magazine World step {pending}: {e:?}"))
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
                assert_eq!(item["definition_id"], equipment_id);
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
            }
            let observation = trial.session().observation().unwrap();
            if observation
                .active()
                .and_then(|o| o.latest_magazine())
                .is_some_and(|d| {
                    matches!(
                        d.operation,
                        crate::game::battle::core::equipment_magazine::MagazineOperation::Fired
                    ) && d.remaining < d.capacity
                        && observation.active_runtime_facts().is_some_and(|facts| {
                            facts.actors().any(|(actor, row)| {
                                *actor == d.actor
                                    && row.caused.latest_damage.as_ref().is_some_and(|hit| {
                                        hit.actual_hp_loss() > 0
                                            && hit.source.equipment == Some(d.equipment)
                                    })
                            })
                        })
                })
            {
                if variant == 0 {
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
        let magazine = observed
            .latest_magazine()
            .expect("actual equipped gun must release a round");
        assert!(matches!(
            magazine.operation,
            crate::game::battle::core::equipment_magazine::MagazineOperation::Fired
        ));
        assert!(!magazine.enhanced);
        assert!(magazine.remaining < capacity);
        assert_eq!(magazine.capacity, capacity);
        let granted = trial.session().step(13).unwrap().public_result().unwrap();
        assert_eq!(
            serde_json::to_value(magazine.equipment.persistent_copy_uuid).unwrap(),
            granted["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
        );
        assert_eq!(magazine.definition_id, format!("equipment:{equipment_id}"));
        let facts = observation
            .active_runtime_facts()
            .or_else(|| observation.last_terminal().map(|t| t.runtime_facts()))
            .unwrap();
        let damage = facts
            .actors()
            .find(|(actor, _)| **actor == magazine.actor)
            .unwrap()
            .1
            .caused
            .latest_damage
            .as_ref()
            .unwrap();
        assert!(damage.actual_hp_loss() > 0);
        assert_eq!(damage.source.equipment, Some(magazine.equipment));
        let origin = trial.latest_magazine_origin().unwrap();
        assert!(origin.definition.contains("AttackProgram"), "{origin:?}");
        expected
            .push(json!({"capacity":capacity,"percent":percent,"magazine":magazine,"origin":origin,"damage":damage,"actual_hp_loss":damage.actual_hp_loss()}));
    }
    if area {
        assert!(expected[1]["actual_hp_loss"].as_u64().unwrap() > expected[0]["actual_hp_loss"].as_u64().unwrap(), "{expected:?}");
    }
    if let Some(directory) = std::env::var_os(if area { "PLAIN_MAGAZINE_WORLD_FIXTURE_DIR" } else { "MAGAZINE_COMPOSITION_FIXTURE_DIR" }) {
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
