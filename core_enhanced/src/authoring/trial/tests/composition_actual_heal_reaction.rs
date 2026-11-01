use super::*;

#[test]
fn composition_actual_heal_reaction_world_actual_queen_bee_three_copies_and_origin() {
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    for id in [
        "placeholder_queen_bee_armor",
        "placeholder_queen_bee_accessory",
        "placeholder_queen_bee_weapon",
    ] {
        assert!(equipment.get_by_id(id).is_some());
    }
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),", "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"placeholder_queen_bee_armor\"), GrantEquipment(equipment_id: \"placeholder_queen_bee_accessory\"), GrantEquipment(equipment_id: \"placeholder_queen_bee_weapon\"),", 1));
    fixture.author_trial_facility();
    // Author a nearby platform in this isolated map pack. Keep the catalog
    // staff's real 5x5 range and the enemy's existing ground route unchanged.
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
                        *v = json!(v.as_u64().unwrap() + 3);
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
    for index in 0..3 {
        document["steps"].as_array_mut().unwrap().insert(5+index,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
            {"from":{"step":13,"pointer":format!("/payload/inventory_diff/added/{index}/Equipment/uuid")},"to":"/command/item_uuid","value_type":"string"},
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    }
    for (index, employee) in [(26, 10), (28, 9)] {
        for binding in document["steps"][index - preparation]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if binding["to"] == "/command/employee_uuid" {
                binding["from"]["step"] = json!(employee);
            }
        }
    }
    let source = include_str!(
        "../../../../tests/fixtures/skill_composition/actual_heal_reaction.skill.yaml"
    )
    .to_string();
    let cancel = CancellationToken::default();
    let mut pre = document.clone();
    pre["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(28 - preparation);
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
    let entry = probe.session().step(23).unwrap().public_result().unwrap();
    let zones = entry["payload"]["battle_setup_snapshot"]["deployment_zones"]
        .as_array()
        .unwrap();
    let platform = zones
        .iter()
        .position(|z| z["kind"] == "Platform")
        .expect("a platform for the focused staff");
    let units = probe.session().step(27).unwrap().public_result().unwrap()["payload"]
        ["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let ally = units
        .iter()
        .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
        .unwrap();
    // The actual tank loses 13 HP on this route before the healer deploys.
    // The staff offers 14 HP, so only the missing 13 HP may trigger reactions.
    assert_eq!(ally["stats"]["max_health"], 180);
    assert_eq!(ally["stats"]["current_health"], 167);
    let ally_position: crate::game::resources::Position =
        serde_json::from_value(ally["position"].clone()).unwrap();
    let profile = equipment
        .get_by_id("placeholder_queen_bee_weapon")
        .unwrap()
        .weapon_profile
        .as_ref()
        .unwrap();
    let cell = zones[platform]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .position(|cell| {
            let position = serde_json::from_value(cell.clone()).unwrap();
            !units.iter().any(|u| u["position"] == *cell)
                && profile
                    .defense_tile_range
                    .affected_tiles(
                        position,
                        crate::game::battle::tile_range::FacingDirection::Right,
                    )
                    .unwrap()
                    .contains(&ally_position)
        })
        .unwrap_or_else(|| {
            panic!("actual staff requires nearby platform: units={units:?} zones={zones:?}")
        });
    for b in document["steps"][28 - preparation]["bindings"]
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
    for _ in 0..3 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    for (ratio, amount, remainder) in [(25, 3, 25), (50, 6, 50)] {
        let current = source.replace(
            "actual_heal_percent: 25",
            &format!("actual_heal_percent: {ratio}"),
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
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("actual heal reaction World step {pending}: {e:?}"))
                .index();
            if [14, 15, 16, 26, 28].contains(&index) {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                assert_eq!(
                    result["disposition"], "accepted",
                    "World step {index}: {result}"
                );
            }
        }
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let copies = &grant["payload"]["inventory_diff"]["added"];
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let reaction = observed.latest_actual_heal_reaction().unwrap_or_else(|| {
            panic!(
                "actual equipment must react to ordinary allied healing: final={:?}",
                trial
                    .session()
                    .step(trial.session().next_step_index() - 1)
                    .unwrap()
                    .public_result()
            )
        });
        assert_eq!(reaction.actual_healed, 13);
        assert_eq!(reaction.contributors.len(), 2);
        let shield = reaction
            .contributors
            .iter()
            .find(|c| c.self_shield.is_some())
            .unwrap();
        let haste = reaction
            .contributors
            .iter()
            .find(|c| c.target_haste.is_some())
            .unwrap();
        assert_eq!(
            shield.self_shield.unwrap().after - shield.self_shield.unwrap().before,
            amount
        );
        assert_eq!(shield.self_shield.unwrap().actual_heal_percent, ratio);
        assert_eq!(shield.self_shield.unwrap().remainder_after, remainder);
        assert_eq!(haste.target_haste.unwrap().attack_speed_points, 30);
        assert_eq!(
            serde_json::to_value(shield.equipment.persistent_copy_uuid).unwrap(),
            copies[0]["Equipment"]["uuid"]
        );
        assert_eq!(
            serde_json::to_value(haste.equipment.persistent_copy_uuid).unwrap(),
            copies[1]["Equipment"]["uuid"]
        );
        assert_eq!(
            serde_json::to_value(
                reaction
                    .healing_source
                    .equipment
                    .unwrap()
                    .persistent_copy_uuid
            )
            .unwrap(),
            copies[2]["Equipment"]["uuid"]
        );
        assert!(
            reaction.healing_source.cast.is_none()
                && reaction.healing_source.fragment.is_none()
                && reaction.healing_source.action.is_some()
        );
        assert_ne!(reaction.actor, reaction.target);
        let origins = trial.latest_actual_heal_reaction_origins().unwrap();
        assert_eq!(origins.len(), 2);
        assert!(origins.iter().all(|o| o
            .as_ref()
            .is_some_and(|o| o.definition.ends_with("SupportProgram"))));
        expected.push(json!({"ratio":ratio,"reaction":reaction,"origins":origins}));
    }
    if let Some(directory) = std::env::var_os("ACTUAL_HEAL_REACTION_COMPOSITION_FIXTURE_DIR") {
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
