use super::*;

#[test]
fn basic_cone_world_reward_equip_platform_and_regular_release() {
    native_area_weapon_world(
        Some(ron::de::from_str(include_str!("../../../../tests/fixtures/skill_composition/larynx.weapon.ron")).unwrap()),
        "larynx_fixture_weapon", json!({"DamageCone":{"length_tiles":5,"end_half_width_tiles":2}}),Some(400),"Magic","BASIC_CONE_WORLD_FIXTURE_DIR");
}

#[test]
fn basic_occupied_tiles_world_catalog_reward_equip_platform_and_tile_release() {
    native_area_weapon_world(None,"placeholder_burrowing_heaven_weapon",json!("DamageOccupiedTiles"),None,"Magic","OCCUPIED_TILES_WORLD_FIXTURE_DIR");
}

#[test]
fn saw_world_catalog_reward_equip_ground_and_regular_front_tile_release() {
    native_area_weapon_world(None,"placeholder_schadenfreude_weapon",json!("DamageOccupiedTiles"),Some(750),"Physical","SAW_WORLD_FIXTURE_DIR");
}

#[test]
fn basic_delayed_tile_world_catalog_reward_equip_platform_release_and_impact() {
    native_area_weapon_world(None,"placeholder_king_of_greed_weapon",json!({"DamageTileAfterDelay":{"delay_ms":400}}),None,"Physical","DELAYED_TILE_WORLD_FIXTURE_DIR");
}

fn native_area_weapon_world(
    authored_profile: Option<crate::game::data::equipment_data::WeaponCombatProfile>,
    definition_id: &str, expected_action: Value, interval: Option<u64>, expected_damage_type: &str, export_env: &str,
) {
    use crate::game::{battle::tile_range::FacingDirection, resources::Position};
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    if let Some(profile) = authored_profile {
        let mut weapon = equipment.items.iter().find(|i| i.id == "standard_training_sword").unwrap().clone();
        weapon.id = definition_id.into();
        weapon.uuid = uuid::Uuid::from_u128(0xe380);
        weapon.name = "기본 공격 연결 시험 무기".into();
        weapon.triggered_effects.clear();
        weapon.ability_activations.clear();
        weapon.weapon_profile = Some(profile);
        equipment.items.push(weapon);
    }
    let profile = equipment.items.iter().find(|w| w.id == definition_id).unwrap().weapon_profile.clone().unwrap();
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    if profile.deployment_affinity == crate::game::battle::types::DeploymentAffinity::GroundOnly {
        // A one-cell weapon needs a target that remains in that cell for more
        // than one period. Isolate only the second encounter; keep the reward
        // battle unchanged and retain the real catalog weapon profile.
        let path = "official/data/enemies/corroded_employees.ron";
        let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let mut striker = enemies.profiles.iter().find(|p| p.id == "corroded_rusher").unwrap().clone();
        striker.id = "front_tile_trial_striker".into();
        striker.uuid = uuid::Uuid::from_u128(0x9190);
        striker.max_health = 1000;
        striker.attack = 1;
        striker.defense = 0;
        // 0.5 tile/second reaches the deployed row within the 30s window and
        // spends 2s in each tile, allowing more than one 750ms attack.
        striker.movement.speed_units_per_ms = 500;
        enemies.profiles.push(striker);
        for p in &mut enemies.profiles {
            if p.basic_attack_range_preset.is_some() { p.basic_attack.defense_tile_range = None; }
        }
        fixture.write(path, ron::ser::to_string(&enemies).unwrap());
        let path = "official/data/pve/encounters.ron";
        let mut encounters: crate::game::data::pve_data::PveEncounterDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let encounter = encounters.encounters.iter_mut().find(|e| e.id == "normal_breach_response").unwrap();
        encounter.waves.truncate(1);
        encounter.waves[0].source = crate::game::data::pve_data::PveWaveSource::Manual(vec![
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id: "front_tile_trial_striker".into(), tier: crate::game::enums::Tier::I, count: 1,
            }
        ]);
        fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    }
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path,fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        &format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: {definition_id:?}),"),1));
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
    // Select a compatible cell from the public setup once; the final scenario has
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
        let expected_zone = match profile.deployment_affinity {
            crate::game::battle::types::DeploymentAffinity::GroundOnly => "Ground",
            crate::game::battle::types::DeploymentAffinity::PlatformOnly => "Platform",
            crate::game::battle::types::DeploymentAffinity::Any => panic!("fixture needs explicit affinity"),
        };
        if z["kind"] != expected_zone {
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
        .expect("fixture must offer a compatible deployment cell");
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
    let mut trial = TrialSession::prepare(
        fixture_content(&token),
        checked(&document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let mut actor = Value::Null;
    let mut hits = Vec::new();
    let mut last_ordinal = None;
    let mut granted = Value::Null;
    let mut tile_releases = Vec::new();
    let mut delayed_releases = Vec::new();
    let mut delayed_impacts = Vec::new();
    let mut last_positions = Vec::new();
    while !trial.is_complete() {
        let pending = trial.next_step_index();
        let index = trial
            .advance(&cancel)
            .unwrap_or_else(|e| panic!("{definition_id} World step {pending}: {e:?}"))
            .index();
        let result = trial.step(index).unwrap().public_result().unwrap();
        if index == 13 {
            granted = result["payload"]["inventory_diff"]["added"][0]["Equipment"].clone();
            assert_eq!(granted["definition_id"], definition_id);
            assert_eq!(granted["weapon_profile"]["action"], expected_action);
            assert_eq!(granted["weapon_profile"]["damage_type"], expected_damage_type);
        }
        if index == 14 {
            assert_eq!(result["disposition"], "accepted");
            assert_eq!(result["payload"]["requested_item_uuid"], granted["uuid"]);
        }
        if index == 24 {
            assert_eq!(result["disposition"], "accepted");
            let units = result["payload"]["battle_update"]["checkpoint"]["units"]
                .as_array()
                .unwrap();
            let u = units
                .iter()
                .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                .unwrap();
            actor = u["unit_instance_id"].clone();
            assert_eq!(u["tactical_state"]["base_block_capacity"], profile.block_capacity);
        }
        if index < 25 {
            continue;
        }
        if let Some(units) = result["payload"]["battle_update"]["checkpoint"]["units"].as_array() {
            last_positions = units.iter().map(|u| json!({"owner":u["owner"],"position":u["position"],"basic_action":u["basic_action"]})).collect();
        }
        if let Some(events) = result["payload"]["battle_update"]["events_delta"]["events"].as_array() {
            for entry in events {
                let event = &entry["event"];
                if event["attacker_instance_id"] == actor {
                    match event["type"].as_str() {
                        Some("BasicAttackTileReleased") if !delayed_releases.iter().any(|e: &Value| e["seq"] == entry["seq"]) => delayed_releases.push(entry.clone()),
                        Some("BasicAttackTileImpacted") if !delayed_impacts.iter().any(|e: &Value| e["seq"] == entry["seq"]) => delayed_impacts.push(entry.clone()),
                        _ => (),
                    }
                }
                if event["type"] == "BasicAttackTilesReleased" && event["attacker_instance_id"] == actor {
                    let tiles = event["affected_tiles"].as_array().expect("public fixed tiles");
                    assert!(!tiles.is_empty());
                    assert!(tiles.iter().all(|p| p["x"].is_i64() && p["y"].is_i64()));
                    tile_releases.push(entry.clone());
                }
            }
        }
        let observation = trial.observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
        if let Some(witness) = observed.and_then(|o| o.latest()) {
            let hit = serde_json::to_value(witness).unwrap();
            if hit["source_unit"] == actor && hit["change"]["reason"] == "BasicAttack" {
                let ordinal = hit["ordinal"].as_u64();
                if ordinal != last_ordinal {
                    last_ordinal = ordinal;
                    hits.push(hit);
                }
            }
        }
    }
    assert!(
        !hits.is_empty(),
        "actual native area weapon must damage enemies through World: {hits:?}; last positions={last_positions:?}"
    );
    if let Some(interval) = interval {
        assert!(hits.windows(2).any(|p| p[1]["time_ms"].as_u64().unwrap() - p[0]["time_ms"].as_u64().unwrap() == interval),
            "regular {interval}ms cadence: {hits:?}");
    }
    if expected_action == json!("DamageOccupiedTiles") {
        assert!(!tile_releases.is_empty(), "fixed tile events reach the World DTO consumer");
    }
    if let Some(delay) = expected_action["DamageTileAfterDelay"]["delay_ms"].as_u64() {
        assert!(!delayed_releases.is_empty(), "actual Bow must release from Platform");
        assert!(!delayed_impacts.is_empty(), "finite delayed impact must reach World consumer");
        assert_eq!(granted["weapon_profile"]["weapon_archetype"], "Bow");
        assert_eq!(granted["weapon_profile"]["block_capacity"], 0);
        for impact in &delayed_impacts {
            let release = delayed_releases.iter().find(|r| r["event"]["release_id"] == impact["event"]["release_id"])
                .expect("each impact must retain its release identity");
            assert_eq!(impact["event"]["tile"], release["event"]["tile"]);
            assert_eq!(impact["time_ms"].as_u64().unwrap(), release["time_ms"].as_u64().unwrap()+delay);
            assert_eq!(impact["time_ms"], release["event"]["impact_at_ms"]);
        }
        for hit in &hits {
            assert!(hit["change"]["before"].as_u64().unwrap() > hit["change"]["after"].as_u64().unwrap());
            assert!(delayed_impacts.iter().any(|i| i["time_ms"] == hit["time_ms"] && i["event"]["target_count"].as_u64().unwrap() > 0),
                "basic HP damage must happen at impact, never release: {hit:?}");
        }
    }
    if let Some(directory) = std::env::var_os(export_env) {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&json!({"equipment":granted,"actor":actor,"hits":hits,"tile_releases":tile_releases,"delayed_releases":delayed_releases,"delayed_impacts":delayed_impacts}))
                .unwrap(),
        )
        .unwrap();
    }
}
