use super::*;

#[test]
fn conditional_damage_world_actual_crown_ally_frost_stun_and_origin() {
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
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"frost_splinter\"), GrantEquipment(equipment_id: \"ice_crown\"),",
            1,
        ),
    );
    let enemies = "official/data/enemies/corroded_employees.ron";
    let old = fixture.read(enemies);
    assert!(old.contains("max_health: 300,"));
    fixture.write(
        enemies,
        old.replacen("max_health: 300,", "max_health: 3000,", 1),
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
                        *v = json!(v.as_u64().unwrap() + 2);
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
    document["steps"].as_array_mut().unwrap().insert(6,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":"/payload/inventory_diff/added/1/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":10,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    for (index, employee) in [(25, 10), (27, 9)] {
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
        include_str!("../../../../tests/fixtures/skill_composition/conditional_damage.skill.yaml")
            .to_string();
    // Compose the actual H81 frostbite rule as an independent ally provider.
    let frost =
        include_str!("../../../../tests/fixtures/skill_composition/threshold_control.skill.yaml");
    let frost_value = frost
        .split("values:\n")
        .nth(1)
        .unwrap()
        .split("equipment_attacks:\n")
        .next()
        .unwrap();
    let source = source
        .replace(
            "equipment_attacks:\n",
            &format!("{frost_value}equipment_attacks:\n"),
        )
        .replace(
            "definitions: []",
            "  - equipment: frost_splinter\n    program: { Shared: frostbite }\ndefinitions: []",
        );
    let cancel = CancellationToken::default();
    let mut pre = document.clone();
    pre["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(27 - preparation);
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
    let entry = probe.session().step(22).unwrap().public_result().unwrap();
    let zones = entry["payload"]["battle_setup_snapshot"]["deployment_zones"]
        .as_array()
        .unwrap();
    let platform = zones.iter().position(|z| z["kind"] == "Platform").unwrap();
    let units = probe.session().step(26).unwrap().public_result().unwrap()["payload"]
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
    for b in document["steps"][27 - preparation]["bindings"]
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
    for _ in 0..110 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    let mut export_document = None;
    for percent in [20, 35] {
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
        let mut saw_counter_without_control = false;
        let mut triggered_by_ally = false;
        let mut positive = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("crown World step{pending}: {e:?}"))
                .index();
            if [14, 15, 25, 27].contains(&index) {
                assert_eq!(
                    trial
                        .session()
                        .step(index)
                        .unwrap()
                        .public_result()
                        .unwrap()["disposition"],
                    "accepted",
                    "step{index}"
                );
            }
            // The opening encounter ends before rewards and both deployments.
            if index < 27 {
                continue;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(counter) = observed.and_then(|o| o.latest_sourced_hit_counter()) {
                let crate::authoring::ApplicationChange::HitCounter { outcome } = counter.change()
                else {
                    panic!("counter");
                };
                saw_counter_without_control |= outcome.after > 0 && !outcome.control_applied;
                triggered_by_ally |= outcome.control_applied;
            }
            if let Some(w) = observed.and_then(|o| o.latest_conditional_damage()) {
                assert_eq!(w.actor, w.wearer);
                assert_eq!(w.contributors.len(), 1);
                if w.percent > 0 {
                    assert!(triggered_by_ally && saw_counter_without_control);
                    assert_eq!(w.percent, percent);
                    assert_eq!(w.target_movement_impaired, Some(true));
                    let grant = trial.session().step(13).unwrap().public_result().unwrap();
                    assert_eq!(
                        grant["payload"]["inventory_diff"]["added"][1]["Equipment"]
                            ["definition_id"],
                        "ice_crown"
                    );
                    assert_eq!(
                        serde_json::to_value(w.contributors[0].equipment.persistent_copy_uuid)
                            .unwrap(),
                        grant["payload"]["inventory_diff"]["added"][1]["Equipment"]["uuid"]
                    );
                    let counter = observed.unwrap().latest_sourced_hit_counter().unwrap();
                    assert_ne!(counter.source_unit(), Some(w.wearer));
                    let origins = trial.latest_conditional_damage_origins().unwrap();
                    assert!(origins[0]
                        .as_ref()
                        .unwrap()
                        .definition
                        .ends_with("AttackProgram"));
                    let facts = observation
                        .active_runtime_facts()
                        .or_else(|| observation.last_terminal().map(|t| t.runtime_facts()))
                        .unwrap();
                    let hit = facts
                        .actors()
                        .find_map(|(_, row)| {
                            row.caused.latest_damage.as_ref().filter(|h| {
                                h.source.actor == Some(w.wearer)
                                    && h.target == w.target
                                    && h.time_ms == w.time_ms
                            })
                        })
                        .unwrap();
                    assert!(hit.actual_hp_loss() > 0);
                    positive = Some(
                        json!({"percent":percent,"witness":w,"origins":origins,"damage":hit,"loss":hit.actual_hp_loss(),"step":index}),
                    );
                    if export_document.is_none() {
                        let mut doc = document.clone();
                        doc["steps"]
                            .as_array_mut()
                            .unwrap()
                            .truncate(index + 1 - preparation);
                        export_document = Some(doc);
                    }
                    break;
                }
            }
            if observation.active().is_none() && observation.last_terminal().is_some() {
                break;
            }
        }
        expected.push(positive.unwrap_or_else(|| {
            let observation = trial.session().observation().unwrap();
            let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            panic!("actual crown must boost a normal attack during the ally's actual frostbite stun; counter_before={saw_counter_without_control} triggered={triggered_by_ally}; conditional={:?}; counter={:?}; checkpoint={:?}",
                observed.and_then(|o| o.latest_conditional_damage()),
                observed.and_then(|o| o.latest_sourced_hit_counter()).map(|c| (c.source_unit(), c.change())),
                trial.session().step(trial.session().next_step_index().saturating_sub(1)).and_then(|s| s.public_result()).map(|v| v["payload"]["battle_update"]["checkpoint"].clone()));
        }));
    }
    assert_eq!(
        expected[0]["witness"]["time_ms"],
        expected[1]["witness"]["time_ms"]
    );
    assert!(expected[1]["loss"].as_u64().unwrap() > expected[0]["loss"].as_u64().unwrap());
    if let Some(directory) = std::env::var_os("CONDITIONAL_DAMAGE_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":export_document.unwrap()})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
