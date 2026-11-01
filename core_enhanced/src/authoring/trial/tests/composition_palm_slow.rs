use super::*;

#[test]
fn palm_slow_world_actual_catalog_platform_area_copy_and_movement() {
    let definition_id = "placeholder_clouded_monk_weapon";
    let expected_action = json!({"DamageArea":{"footprint":{"include_anchor_tile":true,"rows":["XXX","X@X","XXX"]},"secondary_percent":100}});
    let expected_damage_type = "Physical";
    let export_env = "PALM_SLOW_WORLD_FIXTURE_DIR";
    use crate::game::{battle::tile_range::FacingDirection, resources::Position};
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let profile = equipment
        .items
        .iter()
        .find(|w| w.id == definition_id)
        .unwrap()
        .weapon_profile
        .clone()
        .unwrap();
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            &format!(
                "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: {definition_id:?}),"
            ),
            1,
        ),
    );
    // Slow, durable enemies expose repeated area hits through the real World scheduler.
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
    enemy.movement.speed_units_per_ms = 500;
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
                        *v = json!(v.as_u64().unwrap() + 1)
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
    document["steps"].as_array_mut().unwrap().insert(5,json!({"kind":"command",
        "command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    let token = fixture.load();
    let cancel = CancellationToken::default();
    // Select a real platform from the public setup once; the final scenario has
    // explicit bindings and the runner never invents a deployment position.
    let mut probe_document = document.clone();
    probe_document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(22 - preparation);
    let mut probe = TrialSession::prepare(
        fixture_content(&token),
        checked(&probe_document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    probe.run_to_end(&cancel).unwrap();
    let entry = probe.step(21).unwrap().public_result().unwrap();
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
    let source = include_str!("../../../../tests/fixtures/skill_composition/palm_slow.skill.yaml");
    let mut expected = Vec::new();
    for (percent, speed) in [(20, 400), (35, 325)] {
        let current = source.replace("percent: 20", &format!("percent: {percent}"));
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
        let mut actor = Value::Null;
        let mut granted = Value::Null;
        let mut statuses = std::collections::BTreeMap::new();
        let mut slow_effects = std::collections::BTreeMap::new();
        let mut hits = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("palm World step {pending}: {e:?}"))
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 13 {
                granted = result["payload"]["inventory_diff"]["added"][0]["Equipment"].clone();
                assert_eq!(granted["definition_id"], definition_id);
                assert_eq!(granted["weapon_profile"]["action"], expected_action);
                assert_eq!(
                    granted["weapon_profile"]["damage_type"],
                    expected_damage_type
                );
            }
            if index == 14 {
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(result["payload"]["requested_item_uuid"], granted["uuid"]);
            }
            if index == 24 {
                assert_eq!(result["disposition"], "accepted");
                let u = result["payload"]["battle_update"]["checkpoint"]["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                    .unwrap();
                actor = u["unit_instance_id"].clone();
                assert_eq!(u["tactical_state"]["base_block_capacity"], 0);
            }
            if index < 25 {
                continue;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(w) = observed.and_then(|o| o.latest_sourced_sustained()) {
                let value = serde_json::to_value(w).unwrap();
                if value["source_unit"] == actor {
                    let authored = w.authored_source().unwrap();
                    assert!(authored.field_path().ends_with("program.on_hit_status"));
                    let origin = trial.origin_for_observed(authored).unwrap();
                    assert!(origin.definition.ends_with("AttackProgram"));
                    assert_eq!(
                        json!(w.equipment_source().unwrap().persistent_copy_uuid),
                        granted["uuid"]
                    );
                    assert!(w.source().is_none());
                    if value["change"]["ended"].is_null() {
                        assert_eq!(
                            value["change"]["after"]["move_speed_units_per_ms"], speed,
                            "every actual reapplication keeps movement slow: {value:?}"
                        );
                        assert_eq!(
                            value["change"]["before"]["attack_interval_ms"],
                            value["change"]["after"]["attack_interval_ms"]
                        );
                    }
                    statuses.insert(
                        value["ordinal"].as_u64().unwrap(),
                        json!({"witness":value,"origin":origin}),
                    );
                }
            }
            if let Some(effects) = result["payload"]["battle_update"]["checkpoint"]
                ["sustained_effects"]["active"]
                .as_array()
            {
                let mut seen = std::collections::BTreeSet::new();
                for effect in effects.iter().filter(|e| {
                    e["caster"] == actor && e["hit_status"]["definition"] == "palm-movement-slow"
                }) {
                    assert_eq!(effect["hit_status"]["kind"]["Slow"]["percent"], percent);
                    let target = effect["target"].as_str().unwrap().to_owned();
                    assert!(
                        seen.insert(target.clone()),
                        "one live contribution per target and equipped copy"
                    );
                    slow_effects.insert(target, effect.clone());
                }
            }
            if let Some(facts) = observation
                .active_runtime_facts()
                .or_else(|| observation.last_terminal().map(|t| t.runtime_facts()))
            {
                for (_, row) in facts.actors() {
                    if let Some(hit) = row
                        .caused
                        .latest_damage
                        .as_ref()
                        .filter(|h| h.actual_hp_loss() > 0 && json!(h.source.actor) == actor)
                    {
                        hits.insert(
                            (hit.time_ms, hit.target),
                            json!({"hit":hit,"loss":hit.actual_hp_loss()}),
                        );
                    }
                }
            }
        }
        assert_eq!(granted["weapon_profile"]["weapon_archetype"], "Staff");
        assert!(
            slow_effects.len() >= 2,
            "multiple actual area targets: {slow_effects:?}"
        );
        assert!(hits.len() >= 3, "repeated native basic damage: {hits:?}");
        assert!(
            statuses.values().any(
                |s| s["witness"]["change"]["before"]["move_speed_units_per_ms"] == 500
                    && s["witness"]["change"]["after"]["move_speed_units_per_ms"] == speed
            ),
            "actual speed: {statuses:?}"
        );
        expected.push(json!({"percent":percent,"speed":speed,"equipment":granted,"actor":actor,
            "hits":hits.values().collect::<Vec<_>>(),"statuses":statuses,"slow_effects":slow_effects}));
    }
    if let Some(directory) = std::env::var_os(export_env) {
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
