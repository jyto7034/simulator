#[test]
fn composition_stun_end_world_actual_accessory_ground_and_independent_root() {
    let (fixture, _, mut document, _, _) = channel_support_world_fixture();
    let accessory = "placeholder_burrowing_heaven_accessory";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let metadata = equipment.get_by_id(accessory).unwrap();
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
            &format!(
                "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"{accessory}\"),"
            ),
            1,
        ),
    );
    fixture.author_trial_facility();
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= 14) {
                        *v = json!(v.as_u64().unwrap() + 1);
                    } else {
                        shift(v);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    shift(v);
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().insert(5,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[{"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},{"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    for _ in 0..7 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let gear =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_stun_end.skill.yaml");
    let value = gear
        .split("values:\n")
        .nth(1)
        .unwrap()
        .split("equipment_states:\n")
        .next()
        .unwrap();
    let mut source=include_str!("../../../../tests/fixtures/skill_composition/channel_ground.skill.yaml")
        .replace("version: 1","version: 2").replace("authored-channel-ground","fragment_freischutz_black_round")
        .replace("capacity: 100","capacity: 25").replace("duration_ms: 2001","duration_ms: 2000")
        .replace("rows: [XXX, X@X, XXX]","rows: [XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXX@XXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX]")
        .replace("definitions:\n",&format!("{value}definitions:\n"));
    let first = source.find("        modifiers:\n").unwrap();
    let last = source[first..].find("        wait:").unwrap() + first;
    source.replace_range(first..last, "");
    source+=&format!("\nequipment_states:\n  - equipment: {accessory}\n    program: {{ Shared: lingering-gaze }}\n");
    let mut expected = Vec::new();
    for duration in [1000, 1500] {
        let current = source.replace("value: 1000", &format!("value: {duration}"));
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
        let mut roots = Vec::new();
        let mut after = false;
        let mut stunned = false;
        let end = 25001 + duration;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("stun-end World {pending}: {e:?}"))
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 13 {
                copy = Some(
                    result["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"].clone(),
                );
            }
            if index == 14 {
                assert_eq!(result["disposition"], "accepted");
                equipped = true;
            }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            let Some(time) = cp["battle_time_ms"].as_u64() else {
                continue;
            };
            if (24001..25001).contains(&time) {
                stunned |=
                    cp["units"].as_array().unwrap().iter().any(|u| {
                        u["owner"] == "Opponent" && u["control"]["hard_cc_active"] == true
                    });
            }
            let current_roots: Vec<_> = cp["sustained_effects"]["active"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["hit_status"]["kind"] == "Root")
                .cloned()
                .collect();
            if time >= 25001 && time < end {
                assert!(!current_roots.is_empty(), "expected root at {time}: {cp}");
                for root in &current_roots {
                    assert_eq!(root["expires_at_ms"], end);
                    let unit = cp["units"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|u| u["unit_instance_id"] == root["target"])
                        .unwrap();
                    assert_eq!(unit["control"]["hard_cc_active"], false);
                }
                assert!(cp["sustained_effects"]["active"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|r| r["is_channel"] == false));
                roots.push(json!({"step":index,"time_ms":time,"effects":current_roots,"states":cp["sustained_effects"]["equipment_states"]}));
            }
            if time >= end {
                assert!(current_roots.is_empty(), "root must expire at {end}: {cp}");
                after = true;
            }
        }
        assert!(equipped && stunned && !roots.is_empty() && after);
        let observed = trial.session().observation().unwrap().active().unwrap();
        let witness = observed.latest_sourced_sustained().unwrap();
        let equipment = witness
            .equipment_source()
            .expect("real accessory attribution");
        assert_eq!(equipment.definition_uuid, definition);
        assert_eq!(
            serde_json::to_value(equipment.persistent_copy_uuid).unwrap(),
            copy.unwrap()
        );
        let authored = witness.authored_source().unwrap();
        assert_eq!(authored.source_file(), trial.identity().artifact_path);
        let origin = trial.origin_for_observed(authored).unwrap();
        assert!(origin.definition.contains("StateProgram"), "{origin:?}");
        expected.push(json!({"duration":duration,"expiry":end,"equipment":equipment,"roots":roots,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("STUN_END_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
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
