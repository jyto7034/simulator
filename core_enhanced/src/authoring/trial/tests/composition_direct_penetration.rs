use super::*;

#[test]
fn direct_penetration_world_actual_accessory_reward_copy_and_authored_increase() {
    penetration_world(false);
}
#[test]
fn magic_penetration_world_test_pack_reward_copy_and_authored_increase() {
    penetration_world(true);
}
fn penetration_world(magic: bool) {
    let (fixture, _, mut document) = state_world_fixture();
    let armor = if magic {
        "mp01_probe_accessory"
    } else {
        "placeholder_clouded_monk_accessory"
    };
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    if magic {
        // A local test-pack definition; never assign magic penetration to the
        // official physical-only accessory or introduce release balance.
        let mut probe = equipment
            .get_by_id("placeholder_clouded_monk_accessory")
            .unwrap()
            .clone();
        probe.id = armor.into();
        probe.uuid = uuid::Uuid::from_u128(0x9601);
        probe.name = "MP01 technical probe".into();
        equipment.items.push(probe);
        equipment
            .items
            .iter_mut()
            .find(|e| e.id == "standard_training_sword")
            .unwrap()
            .weapon_profile
            .as_mut()
            .unwrap()
            .damage_type = crate::game::battle::damage::DamageType::Magic;
        fixture.write(
            "official/data/equipments/base.ron",
            ron::ser::to_string(&equipment).unwrap(),
        );
    }
    // get_by_id's immutable index predates the test-only insertion.
    let metadata = equipment.items.iter().find(|e| e.id == armor).unwrap();
    assert_eq!(
        metadata.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Accessory
    );
    let definition = metadata.uuid;
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            &format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: {armor:?}),"),
            1,
        ),
    );
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(enemy_path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.attack = 0;
    enemy.max_health = 2000;
    enemy.defense = 100;
    enemy.magic_resist = if magic { 100 } else { 0 };
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    fixture.author_trial_facility();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(24 - preparation);
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= 14) {
                        *v = json!(v.as_u64().unwrap() + 1);
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
    for _ in 0..110 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source = if magic {
        include_str!("../../../../tests/fixtures/skill_composition/magic_penetration.skill.yaml")
    } else {
        include_str!("../../../../tests/fixtures/skill_composition/direct_penetration.skill.yaml")
    };
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [25, 50] {
        let current = source.replace("percent: 25", &format!("percent: {percent}"));
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
        let mut copy = None;
        let mut equipped = false;
        let mut deployed = false;
        let mut observations = std::collections::BTreeMap::new();
        let mut hits = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("direct penetration World step {pending}: {e:?}"))
                .index();
            let public = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 13 {
                let gear = &public["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(gear["definition_id"], armor);
                copy = Some(
                    gear["uuid"]
                        .as_str()
                        .unwrap()
                        .parse::<uuid::Uuid>()
                        .unwrap(),
                );
            }
            if index == 14 {
                assert_eq!(public["disposition"], "accepted");
                equipped = true;
            }
            if index == 24 {
                assert_eq!(public["disposition"], "accepted");
                deployed = true;
                let units = public["payload"]["battle_update"]["checkpoint"]["units"]
                    .as_array()
                    .unwrap();
                let wearer = units
                    .iter()
                    .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                    .unwrap();
                assert_eq!(wearer["stats"]["max_health"], 180);
                assert_eq!(wearer["stats"]["defense"], 5);
                assert_eq!(wearer["stats"]["attack"], 12);
            }
            if index < 24 {
                continue;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(w) = observed.and_then(|o| o.latest_direct_penetration()) {
                assert_eq!(w.contributors.len(), if magic { 2 } else { 1 });
                assert_eq!(w.contributors[0].equipment.definition_uuid, definition);
                assert_eq!(w.contributors[0].equipment.persistent_copy_uuid, copy);
                // The selected Easy encounter applies the policy's90% enemy defense scale.
                assert_eq!(w.target_defense, 90);
                if magic {
                    assert_eq!(w.percent, 0);
                    assert_eq!(w.magic_percent, percent);
                    assert_eq!(w.magic_flat, 10);
                    // Easy defense scaling does not scale magic resistance.
                    assert_eq!(w.target_magic_resist, 100);
                    assert!(w.contributors.iter().all(|c| c.damage_type
                        == crate::game::battle::damage::DamageType::Magic
                        && c.equipment.persistent_copy_uuid == copy));
                } else {
                    assert_eq!(w.percent, percent);
                    assert_eq!(w.magic_percent, 0);
                }
                let origins = trial.latest_direct_penetration_origins().unwrap();
                assert_eq!(origins.len(), w.contributors.len());
                assert!(origins
                    .iter()
                    .all(|o| o.as_ref().is_some_and(|o| !o.calls.is_empty())));
                let origin = origins[0].as_ref().unwrap();
                assert!(origin.definition.ends_with("AttackProgram"), "{origin:?}");
                assert!(!origin.calls.is_empty());
                observations.insert(w.time_ms, json!({"witness":w,"origin":origin}));
                if let Some(facts) = observation
                    .active_runtime_facts()
                    .or_else(|| observation.last_terminal().map(|t| t.runtime_facts()))
                {
                    for (_, row) in facts.actors() {
                        if let Some(hit) = row.caused.latest_damage.as_ref().filter(|h| {
                            h.actual_hp_loss() > 0
                                && h.source.actor == Some(w.actor)
                                && h.time_ms == w.time_ms
                        }) {
                            assert_eq!(hit.actual_hp_loss(), if percent == 25 { 7 } else { 8 });
                            hits.insert(
                                (hit.time_ms, hit.target),
                                json!({"hit":hit,"loss":hit.actual_hp_loss()}),
                            );
                        }
                    }
                }
            }
        }
        assert!(equipped && deployed);
        assert!(
            observations
                .values()
                .any(|v| v["witness"][if magic { "magic_percent" } else { "percent" }] == percent),
            "{observations:?}"
        );
        assert!(hits.len() >= 3, "{hits:?}");
        expected.push(json!({"percent":percent,"observations":observations.values().collect::<Vec<_>>(),"hits":hits.values().collect::<Vec<_>>()}));
    }
    assert!(expected[0]["hits"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| expected[1]["hits"]
            .as_array()
            .unwrap()
            .iter()
            .any(|u| l["hit"]["time_ms"] == u["hit"]["time_ms"]
                && l["loss"] == 7
                && u["loss"] == 8)));
    if let Some(directory) = std::env::var_os(if magic {
        "MAGIC_PENETRATION_WORLD_FIXTURE_DIR"
    } else {
        "DIRECT_PENETRATION_WORLD_FIXTURE_DIR"
    }) {
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
