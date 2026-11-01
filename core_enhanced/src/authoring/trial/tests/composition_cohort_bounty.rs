#[test]
fn composition_cohort_bounty_world_ui_actual_direct_kill_and_threshold_source() {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    // This encounter already occupies one block with an ordinary enemy. Keep
    // the first cohort in reach for a real queued attack instead of letting it
    // cross the one-cell sword range between attack starts.
    equipment
        .items
        .iter_mut()
        .find(|i| i.id == "standard_training_sword")
        .unwrap()
        .weapon_profile
        .as_mut()
        .unwrap()
        .block_capacity = 2;
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.id == "crimson_goggles")
        .unwrap()
        .clone();
    accessory.id = "cohort_bounty_fixture_accessory".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe500);
    accessory.name = "수금·임계치 성장 실제 처치 시험 장신구".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    assert_eq!(
        accessory.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Accessory
    );
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"cohort_bounty_fixture_accessory\"),", 1));
    fixture.author_trial_facility();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    // Preserve the real first reward, equip, second encounter and deployed wearer.
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
    document["steps"][25 - preparation]["clock"]["delta_ms"] = json!(100);
    for _ in 0..200 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock",
            "clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/cohort_bounty.skill.yaml")
            .replace(
                "equipment: r11_accessory",
                "equipment: cohort_bounty_fixture_accessory",
            )
            .replace("hp: 300", "hp: 1")
            .replace("power: 20", "power: 1");
    let cancel = CancellationToken::default();
    let mut probe = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&document),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let stop = loop {
        assert!(
            !probe.session().is_complete(),
            "a real direct cohort kill must occur within the fixed 20s fixture"
        );
        let pending = probe.session().next_step_index();
        probe
            .advance(&cancel)
            .unwrap_or_else(|error| panic!("bounty preparation step {pending}: {error:?}"));
        let observation = probe.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
        if pending == 14 || pending == 24 {
            let result = probe
                .session()
                .step(pending)
                .unwrap()
                .public_result()
                .unwrap();
            assert_eq!(
                result["disposition"], "accepted",
                "required setup step {pending}: {result}"
            );
        }
        if pending == 75 {
            let cohort = observed
                .and_then(|o| o.latest_shared_cohort())
                .expect("equipped provider must have spawned at 5000ms");
            assert_eq!(
                (cohort.time_ms, cohort.initial_hp, cohort.provider_count),
                (5000, 1, 1)
            );
        }
        if observed
            .and_then(|o| o.latest_bounty())
            .is_some_and(|b| b.pending > 0 || b.confirmed > 0)
        {
            break probe.session().next_step_index();
        }
    };
    drop(probe);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(stop - preparation);
    let mut expected = Vec::new();
    for threshold in [2u32, 3] {
        let current = source.replace("threshold: 2", &format!("threshold: {threshold}"));
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
        trial.run_to_end(&cancel).unwrap();
        let equipped = trial.session().step(14).unwrap().public_result().unwrap();
        assert_eq!(equipped["disposition"], "accepted");
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let bounty = observed
            .latest_bounty()
            .expect("real direct kill must retain a source witness");
        assert!(bounty.pending > 0 || bounty.confirmed > 0);
        assert_eq!(bounty.next_threshold, Some(threshold));
        assert_eq!(
            bounty.confirmed, 0,
            "fixture ends at the first earned, unconfirmed bounty"
        );
        assert_eq!(bounty.tier_attack, 0);
        assert_eq!(
            serde_json::to_value(bounty.equipment_copy).unwrap(),
            equipped["payload"]["requested_item_uuid"]
        );
        let origin = trial.latest_bounty_origin().unwrap();
        assert!(origin.definition.contains("AttackProgram"));
        expected.push(json!({"threshold":threshold,"bounty":bounty,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("COHORT_BOUNTY_COMPOSITION_FIXTURE_DIR") {
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
