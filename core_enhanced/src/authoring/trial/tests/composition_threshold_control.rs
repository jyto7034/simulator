use super::*;

#[test]
fn threshold_control_world_actual_frost_sword_reward_equip_area_and_origin() {
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let profile = equipment
        .get_by_id("frost_splinter")
        .unwrap()
        .weapon_profile
        .as_ref()
        .unwrap();
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"frost_splinter\"),",
            1,
        ),
    );
    let enemies = "official/data/enemies/corroded_employees.ron";
    let old = fixture.read(enemies);
    assert!(old.contains("max_health: 300,"));
    fixture.write(
        enemies,
        old.replacen("max_health: 300,", "max_health: 500,", 1),
    );
    fixture.author_trial_facility();
    let path = "official/data/map/battlefield_templates.ron";
    let mut templates: Vec<crate::game::combat_preview::BattlefieldTemplateDefinition> =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let template = templates
        .iter_mut()
        .find(|t| t.id == "corridor_hook_control_01")
        .unwrap();
    let mut authored = serde_json::to_value(&*template).unwrap();
    assert_eq!(authored["rows"][5], "     ... ");
    authored["rows"][5] = json!("    T... ");
    *template = serde_json::from_value(authored).unwrap();
    fixture.write(path, ron::ser::to_string(&templates).unwrap());
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(26 - preparation);
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
    for (index, employee) in [(24, 10), (26, 9)] {
        for b in document["steps"][index - preparation]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if b["to"] == "/command/employee_uuid" {
                b["from"]["step"] = json!(employee);
            }
        }
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/threshold_control.skill.yaml")
            .to_string();
    let cancel = CancellationToken::default();
    let mut pre = document.clone();
    pre["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(26 - preparation);
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
    let units = probe.session().step(25).unwrap().public_result().unwrap()["payload"]
        ["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let enemy = units
        .iter()
        .find(|u| u["owner"] == "Opponent")
        .expect("living enemy for five real strikes");
    let position: crate::game::resources::Position =
        serde_json::from_value(enemy["position"].clone()).unwrap();
    let cell = zones[platform]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .position(|cell| {
            !units.iter().any(|u| u["position"] == *cell)
                && profile
                    .defense_tile_range
                    .affected_tiles(
                        serde_json::from_value(cell.clone()).unwrap(),
                        crate::game::battle::tile_range::FacingDirection::Right,
                    )
                    .unwrap()
                    .contains(&position)
        })
        .unwrap();
    for b in document["steps"][26 - preparation]["bindings"]
        .as_array_mut()
        .unwrap()
    {
        if b["to"] == "/command/position" {
            b["from"]["pointer"] = json!(format!(
                "/payload/battle_setup_snapshot/deployment_zones/{platform}/cells/{cell}"
            ));
        }
    }
    drop(probe);
    for _ in 0..23 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    for threshold in [5, 3] {
        let current = source.replace("threshold: 5", &format!("threshold: {threshold}"));
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
        let mut triggered = false;
        let mut short = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("frost sword World step {pending}: {e:?}"))
                .index();
            if [14, 24, 26].contains(&index) {
                assert_eq!(
                    trial
                        .session()
                        .step(index)
                        .unwrap()
                        .public_result()
                        .unwrap()["disposition"],
                    "accepted",
                    "step {index}"
                );
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(w) = observed.and_then(|o| o.latest_sourced_hit_counter()) {
                let crate::authoring::ApplicationChange::HitCounter { outcome } = w.change() else {
                    panic!("counter")
                };
                triggered |= outcome.control_applied;
                if index == 38 {
                    short = Some(
                        json!({"threshold":threshold,"time_ms":w.time_ms(),"target_unit":w.target_unit(),"outcome":outcome,"equipment":w.equipment_source(),"origin":trial.origin_for_observed(w.authored_source().unwrap())}),
                    );
                }
            }
        }
        assert!(
            triggered,
            "actual weapon must consume a threshold in ordinary World attacks"
        );
        let obs = trial.session().observation().unwrap();
        let observed = obs
            .active()
            .or_else(|| obs.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let w = observed.latest_sourced_hit_counter().unwrap();
        let crate::authoring::ApplicationChange::HitCounter { outcome } = w.change() else {
            panic!("counter")
        };
        assert_eq!(outcome.after, if threshold == 5 { 0 } else { 2 });
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        assert_eq!(
            serde_json::to_value(w.equipment_source().unwrap().persistent_copy_uuid).unwrap(),
            grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
        );
        assert!(trial
            .origin_for_observed(w.authored_source().unwrap())
            .is_some());
        expected.push(short.expect("third strike observation"));
    }
    if let Some(directory) = std::env::var_os("THRESHOLD_CONTROL_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        document["steps"]
            .as_array_mut()
            .unwrap()
            .truncate(39 - preparation);
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
