#[test]
fn composition_area_batch_world_actual_accessory_and_occupied_tiles_weapon() {
    let (fixture, _, mut document) = state_world_fixture();
    let weapon = "placeholder_king_of_greed_accessory";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let metadata = equipment.get_by_id(weapon).unwrap();
    assert_eq!(
        metadata.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Accessory
    );
    let definition = metadata.uuid;
    let area_weapon = "placeholder_burrowing_heaven_weapon";
    let profile = equipment
        .get_by_id(area_weapon)
        .unwrap()
        .weapon_profile
        .clone()
        .unwrap();
    use crate::game::{battle::tile_range::FacingDirection, resources::Position};
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            &format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: {weapon:?}), GrantEquipment(equipment_id: {area_weapon:?}),"),
            1,
        ),
    );
    // Fixed isolated enemy values keep both runs alive and allow real HP loss.
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
    enemy.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    let wave_path = "official/data/enemies/corroded_wave_presets.ron";
    let mut waves: crate::game::data::corroded_wave_data::CorrodedWavePresetDatabase =
        ron::de::from_str(&fixture.read(wave_path)).unwrap();
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
    fixture.write(wave_path, ron::ser::to_string(&waves).unwrap());
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
                        *v = json!(v.as_u64().unwrap() + 2);
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
    document["steps"].as_array_mut().unwrap().insert(6,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":"/payload/inventory_diff/added/1/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    let token = fixture.load();
    let cancel = CancellationToken::default();
    // Select a real platform from the public setup once; the final scenario has
    // explicit bindings and the runner never invents a deployment position.
    let mut probe_document = document.clone();
    probe_document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(23 - preparation);
    let mut probe = TrialSession::prepare(
        fixture_content(&token),
        checked(&probe_document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    probe.run_to_end(&cancel).unwrap();
    let entry = probe.step(22).unwrap().public_result().unwrap();
    let setup = &entry["payload"]["battle_setup_snapshot"];
    let routes: Vec<Position> = setup["routes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|r| r["cells"].as_array().unwrap())
        .map(|p| serde_json::from_value(p.clone()).unwrap())
        .collect();
    let occupied = entry["payload"]["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let mut candidates = Vec::new();
    for (zone, z) in setup["deployment_zones"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        if z["kind"] != "Platform" {
            continue;
        }
        for (cell, p) in z["cells"].as_array().unwrap().iter().enumerate() {
            if occupied.iter().any(|u| u["position"] == *p) {
                continue;
            }
            let position: Position = serde_json::from_value(p.clone()).unwrap();
            for (facing, direction) in [
                ("up", FacingDirection::Up),
                ("right", FacingDirection::Right),
                ("down", FacingDirection::Down),
                ("left", FacingDirection::Left),
            ] {
                let tiles = profile
                    .defense_tile_range
                    .affected_tiles(position, direction)
                    .unwrap();
                let coverage = routes.iter().filter(|p| tiles.contains(p)).count();
                candidates.push((coverage, zone, cell, facing));
            }
        }
    }
    candidates.sort();
    let (coverage, zone, cell, facing) = *candidates
        .last()
        .expect("fixture must offer an actual platform");
    assert!(coverage > 0);
    drop(probe);
    let deploy = document["steps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|s| s["command"]["type"] == "deploy_unit")
        .unwrap();
    deploy["command"]["facing"] = json!(facing);
    deploy["bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|b| b["to"] == "/command/position")
        .unwrap()["from"]["pointer"] = json!(format!(
        "/payload/battle_setup_snapshot/deployment_zones/{zone}/cells/{cell}"
    ));
    for _ in 0..150 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":200,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/area_batch.skill.yaml")
        .replace("equipment: r11_weapon", &format!("equipment: {weapon}"));
    let mut expected = Vec::new();
    for coefficient in [10, 20] {
        let current = source.replace(
            "value: 10, unit: Scalar",
            &format!("value: {coefficient}, unit: Scalar"),
        );
        let cancel = CancellationToken::default();
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
        let mut weapon_equipped = false;
        let mut hits = std::collections::BTreeMap::new();
        let mut observations = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("area batch World {pending}: {e:?}"))
                .index();
            if index == 13 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                let gear = &result["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(gear["definition_id"], weapon);
                copy = Some(
                    gear["uuid"]
                        .as_str()
                        .unwrap()
                        .parse::<uuid::Uuid>()
                        .unwrap(),
                );
            }
            if index == 14 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(
                    result["payload"]["requested_item_uuid"],
                    json!(copy.unwrap())
                );
                equipped = true;
            }
            if index == 15 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                assert_eq!(result["disposition"], "accepted");
                weapon_equipped = true;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(hit) = observed.and_then(|o| o.latest()) {
                let hit = serde_json::to_value(hit).unwrap();
                if hit["change"]["reason"] == "BasicAttack"
                    && hit["change"]["before"].as_u64() > hit["change"]["after"].as_u64()
                {
                    hits.insert(hit["ordinal"].as_u64().unwrap(), hit);
                }
            }
            if let Some(witness) = observed.and_then(|o| o.latest_area_damage()) {
                assert_eq!(witness.contributors.len(), 1);
                let contribution = &witness.contributors[0];
                assert_eq!(contribution.equipment.definition_uuid, definition);
                assert_eq!(contribution.equipment.persistent_copy_uuid, copy);
                let wanted =
                    (u64::from(witness.target_count.saturating_sub(1)) * coefficient).min(50);
                assert_eq!(u64::from(witness.percent), wanted);
                let origins = trial.latest_area_damage_origins().unwrap();
                assert!(origins[0]
                    .as_ref()
                    .unwrap()
                    .definition
                    .ends_with("AttackProgram"));
                observations.insert(
                    witness.time_ms,
                    json!({"witness":witness,"origins":origins}),
                );
            }
        }
        let actor = observations.values().next().unwrap()["witness"]["actor"].clone();
        hits.retain(|_, hit| hit["source_unit"] == actor);
        assert!(
            equipped
                && weapon_equipped
                && !hits.is_empty()
                && observations
                    .values()
                    .any(|v| v["witness"]["percent"].as_u64().unwrap() > 0),
            "{coefficient}: {observations:?}"
        );
        expected.push(json!({"coefficient":coefficient,"observations":observations.values().collect::<Vec<_>>(),"hits":hits.values().collect::<Vec<_>>()}));
    }
    let first = expected[0]["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["witness"]["percent"].as_u64().unwrap() > 0)
        .unwrap();
    let second = expected[1]["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["witness"]["time_ms"] == first["witness"]["time_ms"])
        .unwrap();
    assert!(
        second["witness"]["percent"].as_u64().unwrap()
            > first["witness"]["percent"].as_u64().unwrap(),
        "{expected:?}"
    );
    let first_hit = &expected[0]["hits"][0];
    let second_hit = &expected[1]["hits"][0];
    assert_eq!(first_hit["time_ms"], second_hit["time_ms"]);
    assert_eq!(first_hit["target_unit"], second_hit["target_unit"]);
    let hp_loss = |hit: &Value| {
        hit["change"]["before"].as_u64().unwrap() - hit["change"]["after"].as_u64().unwrap()
    };
    assert!(
        hp_loss(second_hit) > hp_loss(first_hit),
        "authored coefficient must change actual wearer HP damage"
    );
    if let Some(directory) = std::env::var_os("AREA_BATCH_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve prior evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
