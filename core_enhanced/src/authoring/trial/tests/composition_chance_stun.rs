use super::*;

#[test]
fn composition_chance_stun_world_accessory_reward_equip_and_authored_probability() {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut item = equipment
        .get_by_id("placeholder_clouded_monk_accessory")
        .unwrap()
        .clone();
    item.id = "h99_gift_sack".into();
    item.uuid = uuid::Uuid::from_u128(0xe990);
    item.name = "커다란 선물자루 기술 검증값".into();
    item.triggered_effects.clear();
    item.ability_activations.clear();
    assert_eq!(
        item.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Accessory
    );
    equipment.items.push(item);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"h99_gift_sack\"),",
            1,
        ),
    );
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.attack = 0;
    enemy.max_health = 2000;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
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
    document["steps"].as_array_mut().unwrap().insert(5,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
        {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    for _ in 0..110 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/chance_stun.skill.yaml");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for chance in [0, 100] {
        let current = source.replace("chance_percent: 100", &format!("chance_percent: {chance}"));
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
        let mut copy = Value::Null;
        let mut saw_control = false;
        let mut saw_damage = false;
        let mut latest = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("chance stun World {chance} step{pending}: {e:?}"))
                .index();
            let public = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 13 {
                let item = &public["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "h99_gift_sack");
                copy = item["uuid"].clone();
            }
            if [14, 24].contains(&index) {
                assert_eq!(public["disposition"], "accepted");
            }
            if index < 25 {
                continue;
            }
            let cp = &public["payload"]["battle_update"]["checkpoint"];
            if let Some(units) = cp["units"].as_array() {
                for enemy in units.iter().filter(|u| u["owner"] == "Opponent") {
                    saw_damage |= enemy["stats"]["current_health"].as_u64().unwrap()
                        < enemy["stats"]["max_health"].as_u64().unwrap();
                    saw_control |= enemy["control"]["hard_cc_active"] == true;
                }
                for status in cp["sustained_effects"]["active"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|s| s["hit_status"]["definition"] == "gift-sack-stun")
                {
                    assert_eq!(chance, 100);
                    assert_eq!(status["hit_status"]["kind"], "Stun");
                    let target = units
                        .iter()
                        .find(|u| u["unit_instance_id"] == status["target"])
                        .unwrap();
                    assert_eq!(target["control"]["hard_cc_active"], true);
                }
            }
            let observation = trial.session().observation().unwrap();
            let active = observation
                .active()
                .expect("high HP second encounter must remain active");
            if let Some(w) = active.latest_sourced_sustained() {
                assert_eq!(chance, 100);
                let e = w.equipment_source().unwrap();
                assert_eq!(e.definition_uuid, uuid::Uuid::from_u128(0xe990));
                assert_eq!(json!(e.persistent_copy_uuid), copy);
                let origin = trial
                    .origin_for_observed(w.authored_source().unwrap())
                    .unwrap();
                assert!(origin.definition.ends_with("AttackProgram"));
                assert!(w.source().is_none());
                latest = Some(json!({"observed":w,"origin":origin}));
            }
        }
        assert!(saw_damage, "real ordinary weapon must hit");
        assert_eq!(saw_control, chance == 100);
        assert_eq!(latest.is_some(), chance == 100);
        expected.push(json!({"chance":chance,"copy":copy,"control":saw_control,"latest":latest}));
    }
    if let Some(directory) = std::env::var_os("CHANCE_STUN_COMPOSITION_FIXTURE_DIR") {
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
