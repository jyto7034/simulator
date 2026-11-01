use super::*;

#[test]
fn high_hp_damage_world_actual_armor_reward_copy_and_authored_increase() {
    let (fixture, _, mut document) = state_world_fixture();
    let armor = "placeholder_clouded_monk_armor";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let metadata = equipment.get_by_id(armor).unwrap();
    assert_eq!(
        metadata.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Armor
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
    enemy.attack = 4;
    enemy.max_health = 2000;
    enemy.defense = 0;
    enemy.magic_resist = 0;
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
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/high_hp_damage.skill.yaml");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [20, 40] {
        let current = source.replace(
            "value: 20, unit: Scalar",
            &format!("value: {percent}, unit: Scalar"),
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
        let mut copy = None;
        let mut equipped = false;
        let mut deployed = false;
        let mut observations = std::collections::BTreeMap::new();
        let mut hits = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("high HP World step {pending}: {e:?}"))
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
                assert_eq!(wearer["stats"]["max_health"], 200);
                assert_eq!(wearer["stats"]["defense"], 20);
                assert_eq!(wearer["stats"]["attack"], 12);
            }
            if index < 24 {
                continue;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(w) = observed.and_then(|o| o.latest_conditional_damage()) {
                assert_eq!(w.contributors.len(), 1);
                assert_eq!(w.contributors[0].equipment.definition_uuid, definition);
                assert_eq!(w.contributors[0].equipment.persistent_copy_uuid, copy);
                assert_eq!(w.actor, w.wearer);
                assert_eq!(w.wearer_max_hp, 200);
                let rate = if w.wearer_hp >= 160 { percent } else { 0 };
                assert_eq!(w.percent, rate);
                assert_eq!(
                    (w.physical_percent, w.magic_percent, w.true_percent),
                    (rate, rate, 0)
                );
                let origins = trial.latest_conditional_damage_origins().unwrap();
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
                            assert_eq!(
                                hit.actual_hp_loss(),
                                if rate == 0 {
                                    12
                                } else if rate == 20 {
                                    14
                                } else {
                                    16
                                }
                            );
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
                .any(|v| v["witness"]["percent"] == percent),
            "{observations:?}"
        );
        assert!(
            observations.values().any(|v| v["witness"]["percent"] == 0),
            "ordinary hostile hits must cross below HP80%: {observations:?}"
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
                && l["loss"] == 14
                && u["loss"] == 16)));
    if let Some(directory) = std::env::var_os("HIGH_HP_DAMAGE_WORLD_FIXTURE_DIR") {
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
