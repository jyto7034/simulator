#[test]
fn composition_nearby_ally_damage_world_actual_armor_two_employees_and_live_damage() {
    let (fixture, _, mut document) = state_world_fixture();
    let armor = "placeholder_laetitia_armor";
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
    let original = document["steps"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(original["command"]["type"], "deploy_unit");
    let token = fixture.load();
    let cancel = CancellationToken::default();
    let probe = simulate(
        fixture_content(&token),
        checked(&document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let entry = probe.step(21).unwrap().public_result().unwrap();
    let pointer = original["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["to"] == "/command/position")
        .unwrap()["from"]["pointer"]
        .as_str()
        .unwrap();
    let wearer = entry.pointer(pointer).unwrap();
    let setup = &entry["payload"]["battle_setup_snapshot"];
    let zones = setup["deployment_zones"].as_array().unwrap();
    let ground = zones.iter().position(|z| z["kind"] == "Ground").unwrap();
    let cells = zones[ground]["cells"].as_array().unwrap();
    let occupied = entry["payload"]["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let nearby = cells
        .iter()
        .enumerate()
        .filter(|(_, p)| *p != wearer && !occupied.iter().any(|u| u["position"] == **p))
        .filter_map(|(i, p)| {
            let dx = (p["x"].as_i64().unwrap() - wearer["x"].as_i64().unwrap()).abs();
            let dy = (p["y"].as_i64().unwrap() - wearer["y"].as_i64().unwrap()).abs();
            (dx <= 2 && dy <= 2).then_some((dx + dy, i))
        })
        .min()
        .expect("fixture needs a second deployable employee tile in the authored range")
        .1;
    let mut guard = original;
    for b in guard["bindings"].as_array_mut().unwrap() {
        if b["to"].as_str().unwrap().starts_with("/context/") {
            b["from"]["step"] = json!(24);
        }
        if b["to"] == "/command/employee_uuid" {
            b["from"]["step"] = json!(10);
        }
        if b["to"] == "/command/position" {
            b["from"]["pointer"] = json!(format!(
                "/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{nearby}"
            ));
        }
    }
    drop(probe);
    document["steps"].as_array_mut().unwrap().push(guard);
    for _ in 0..40 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/nearby_ally_damage.skill.yaml")
            .replace("equipment: r11_weapon", &format!("equipment: {armor}"));
    let mut expected = Vec::new();
    for coefficient in [10, 20] {
        let current = source.replace(
            "value: 10, unit: Scalar",
            &format!("value: {coefficient}, unit: Scalar"),
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
        let mut guard_deployed = false;
        let mut observations = std::collections::BTreeMap::new();
        let mut hits = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("nearby ally World step {pending}: {e:?}"))
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
                assert_eq!(
                    public["payload"]["requested_item_uuid"],
                    json!(copy.unwrap())
                );
                equipped = true;
            }
            if index == 25 {
                assert_eq!(public["disposition"], "accepted");
                guard_deployed = true;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(w) = observed.and_then(|o| o.latest_nearby_ally_damage()) {
                assert_eq!(w.contributors.len(), 1);
                let c = &w.contributors[0];
                assert_eq!(c.equipment.definition_uuid, definition);
                assert_eq!(c.equipment.persistent_copy_uuid, copy);
                assert_eq!(c.percent, c.count.min(3) * coefficient);
                assert_eq!(w.percent, c.percent);
                assert_ne!(w.wearer, w.target);
                assert_eq!(w.actor, w.wearer);
                let origins = trial.latest_nearby_ally_damage_origins().unwrap();
                assert!(origins[0]
                    .as_ref()
                    .unwrap()
                    .definition
                    .ends_with("AttackProgram"));
                observations.insert(w.time_ms, json!({"witness":w,"origins":origins}));
                if let Some(facts) = observation
                    .active_runtime_facts()
                    .or_else(|| observation.last_terminal().map(|t| t.runtime_facts()))
                {
                    for (_, row) in facts.actors() {
                        if let Some(hit) =
                            row.caused.latest_damage.as_ref().filter(|h| {
                                h.actual_hp_loss() > 0 && h.source.actor == Some(w.wearer)
                            })
                        {
                            hits.insert(
                                (hit.time_ms, hit.target),
                                json!({"hit":hit,"loss":hit.actual_hp_loss()}),
                            );
                        }
                    }
                }
            }
        }
        assert!(equipped && guard_deployed);
        assert!(
            observations
                .values()
                .any(|v| v["witness"]["contributors"][0]["count"] == 1),
            "{observations:?}"
        );
        assert!(!hits.is_empty(), "armor wearer must deal actual HP damage");
        expected.push(json!({"coefficient":coefficient,"observations":observations.values().collect::<Vec<_>>(),"hits":hits.values().collect::<Vec<_>>()}));
    }
    let first = &expected[0]["hits"][0];
    let second = &expected[1]["hits"][0];
    assert_eq!(first["hit"]["time_ms"], second["hit"]["time_ms"]);
    assert!(
        second["loss"].as_u64().unwrap() > first["loss"].as_u64().unwrap(),
        "{expected:?}"
    );
    if let Some(directory) = std::env::var_os("NEARBY_ALLY_DAMAGE_WORLD_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve prior evidence");
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
