use super::*;

#[test]
fn composition_equipment_support_world_reward_equip_and_personal_shields() {
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.id == "crimson_goggles")
        .unwrap()
        .clone();
    accessory.id = "support_fixture_brooch".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe430);
    accessory.name = "마법소녀의 브로치 지원 연결 시험용".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path,fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"support_fixture_brooch\"),",1));
    fixture.author_trial_facility();
    fn shift(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "step" && value.as_u64().is_some_and(|n| n >= 14) {
                        *value = json!(value.as_u64().unwrap() + 1)
                    } else {
                        shift(value)
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    shift(value)
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().insert(5,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}
    ]}));
    let source=include_str!("../../../../tests/fixtures/skill_composition/personal_shield.skill.yaml")
        .replace("authored-personal-shield","fragment_freischutz_black_round")
        .replace("capacity: 100","capacity: 25")
        .replace("values:\n", "equipment_supports:\n  - equipment: support_fixture_brooch\n    program: { Shared: other-ally-support }\nvalues:\n  - name: other-ally-support\n    value:\n      SupportProgram:\n        heal_percent: 25\n        shield_percent: 25\n");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [25, 50] {
        let current = source.replace("shield_percent: 25", &format!("shield_percent: {percent}"));
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
        let mut actual_pools = Vec::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("support World step {pending}: {e:?}"))
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 14 {
                assert_eq!(result["disposition"], "accepted");
                let grant = trial.session().step(13).unwrap().public_result().unwrap();
                let item = &grant["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "support_fixture_brooch");
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
                copy = Some(item["uuid"].clone());
            }
            if index >= 28 {
                if let Some(pools) = result["payload"]["battle_update"]["checkpoint"]
                    ["sustained_effects"]["active"]
                    .as_array()
                {
                    if pools.iter().any(|p| p["personal_grant"].is_object()) {
                        actual_pools.push(pools.clone());
                    }
                }
            }
        }
        assert!(
            !actual_pools.is_empty(),
            "actual granted pools must appear in checkpoints"
        );
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let support = observed
            .latest_equipment_support()
            .expect("other ally's personal shield must be amplified");
        assert_eq!(support.percent, percent);
        assert_ne!(support.actor, support.target);
        assert_eq!(
            support.increased_amount,
            u32::try_from(u64::from(support.base_amount) * (100 + percent) / 100).unwrap()
        );
        assert_eq!(
            serde_json::to_value(support.contributors[0].equipment.persistent_copy_uuid).unwrap(),
            copy.unwrap()
        );
        let pools = actual_pools.first().unwrap();
        let own = pools
            .iter()
            .find(|p| p["target"] == json!(support.actor) && p["personal_grant"].is_object())
            .unwrap();
        let other = pools
            .iter()
            .find(|p| p["target"] == json!(support.target) && p["personal_grant"].is_object())
            .unwrap();
        assert_eq!(own["shield"]["capacity"], support.base_amount);
        assert_eq!(other["shield"]["capacity"], support.increased_amount);
        let origins = trial.latest_equipment_support_origins().unwrap();
        assert!(origins[0]
            .as_ref()
            .unwrap()
            .definition
            .ends_with("SupportProgram"));
        assert!(
            support.source.cast.is_some(),
            "retain the authored fragment cast separately from the brooch"
        );
        expected.push(
            json!({"percent":percent,"support":support,"origins":origins,"own":own,"other":other}),
        );
    }
    if let Some(directory) = std::env::var_os("EQUIPMENT_SUPPORT_COMPOSITION_FIXTURE_DIR") {
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
