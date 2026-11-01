#[test]
fn composition_channel_opening_world_actual_armor_and_damage_source() {
    let (fixture, source, mut document, _, _) = channel_support_world_fixture();
    let armor = "placeholder_burrowing_heaven_armor";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let metadata = equipment.get_by_id(armor).unwrap();
    assert_eq!(
        metadata.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Armor
    );
    let armor_uuid = metadata.uuid;
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            &format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"{armor}\"),"),
            1,
        ),
    );
    // Keep both coefficient runs alive and above the engine's minimum damage.
    // This isolated enemy uses four true damage; the catalog armor is unchanged.
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(enemy_path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.attack = 4;
    enemy.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
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
    document["steps"].as_array_mut().unwrap().insert(5,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    for _ in 0..12 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source=source.replace("duration_ms: 1500","duration_ms: 5000")
        .replace("definitions:\n", "  - name: opening-protection\n    value:\n      DefenseProgram:\n        ChannelOpeningGuard:\n          duration_ms: 2000\n          reduction_percent: 50\ndefinitions:\n")
        + &format!("\nequipment_defenses:\n  - equipment: {armor}\n    program: {{ Shared: opening-protection }}\n");
    let mut expected = Vec::new();
    for percent in [50, 75] {
        let current = source.replace(
            "reduction_percent: 50",
            &format!("reduction_percent: {percent}"),
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
        let mut equipped = false;
        let mut copy = None;
        let mut events = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let previous = trial
                .session()
                .step(pending.saturating_sub(1))
                .and_then(|s| s.public_result())
                .cloned();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| {
                    panic!("opening World step {pending}: {e:?}; previous: {previous:?}")
                })
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
            if let Some(entries) =
                result["payload"]["battle_update"]["events_delta"]["events"].as_array()
            {
                for e in entries {
                    if e["event"]["type"] == "HpChanged" {
                        events.insert(e["seq"].as_u64().unwrap(), e.clone());
                    }
                }
            }
        }
        assert!(equipped);
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let witness = observed
            .latest_channel_opening()
            .expect("actual hostile hit during the equipped armor opening window");
        assert_eq!(witness.equipment.definition_uuid, armor_uuid);
        assert_eq!(
            serde_json::to_value(witness.equipment.persistent_copy_uuid).unwrap(),
            copy.unwrap()
        );
        assert_eq!(witness.reduction_percent, percent);
        assert_eq!(
            (witness.started_at_ms, witness.window_end_ms),
            (23001, 25001)
        );
        let origin = trial.latest_channel_opening_origin().unwrap();
        assert!(origin.definition.ends_with(".DefenseProgram"), "{origin:?}");
        assert_eq!(witness.source_file, trial.identity().artifact_path);
        assert_eq!(
            witness.field_path,
            "equipment_defenses[0].program.ChannelOpeningGuard"
        );
        let target = serde_json::to_value(witness.target).unwrap();
        let damage: Vec<_> = events
            .values()
            .filter(|e| {
                e["event"]["target_instance_id"] == target
                    && e["event"]["delta"].as_i64().is_some_and(|n| n < 0)
            })
            .cloned()
            .collect();
        assert!(
            damage.iter().any(|e| e["time_ms"]
                .as_u64()
                .is_some_and(|t| t >= 23001 && t < 25001)),
            "{damage:?}"
        );
        assert!(
            damage
                .iter()
                .any(|e| e["time_ms"].as_u64().is_some_and(|t| t >= 25001)),
            "post-window hit: {damage:?}"
        );
        expected.push(json!({"percent":percent,"witness":witness,"origin":origin,"damage":damage}));
    }
    let loss = |v: &Value| {
        v["damage"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                e["time_ms"]
                    .as_u64()
                    .is_some_and(|t| t >= 23001 && t < 25001)
            })
            .map(|e| -e["event"]["delta"].as_i64().unwrap())
            .sum::<i64>()
    };
    assert!(
        loss(&expected[0]) > loss(&expected[1]),
        "actual HP loss must follow the authored coefficient: {expected:?}"
    );
    if let Some(path) = std::env::var_os("CHANNEL_OPENING_COMPOSITION_FIXTURE_DIR") {
        let path = std::path::PathBuf::from(path);
        assert!(!path.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &path.join("content"));
        std::fs::write(path.join("source.skill.yaml"), source).unwrap();
        std::fs::write(path.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":path.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            path.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
