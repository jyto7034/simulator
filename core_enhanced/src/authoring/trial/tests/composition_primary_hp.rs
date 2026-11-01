#[test]
fn composition_primary_hp_world_actual_accessory_reward_equip_and_target_damage() {
    let (fixture, _, mut document) = state_world_fixture();
    let weapon = "placeholder_mountain_smiling_bodies_accessory";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let metadata = equipment.get_by_id(weapon).unwrap();
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
            &format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"{weapon}\"),"),
            1,
        ),
    );
    // Fixed isolated enemy values keep both runs alive and allow real HP loss.
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(enemy_path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.attack = 4;
    enemy.max_health = 500;
    enemy.defense = 0;
    enemy.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    fixture.author_trial_facility();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= 14) {
                        *v = json!(v.as_u64().unwrap() + 1);
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
    for _ in 0..20 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/primary_hp.skill.yaml")
        .replace("equipment: r11_weapon", &format!("equipment: {weapon}"));
    let mut expected = Vec::new();
    for coefficient in [100, 200] {
        let current = source.replace(
            "value: 100, unit: Scalar",
            &format!("value: {coefficient}, unit: Scalar"),
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
        let mut copy = None;
        let mut equipped = false;
        let mut observations = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("primary HP World {pending}: {e:?}"))
                .index();
            if index == 13 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                let gear = &result["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(gear["definition_id"], weapon);
                copy = Some(
                    gear["uuid"]
                        .as_str()
                        .unwrap()
                        .parse::<uuid::Uuid>()
                        .unwrap(),
                );
            }
            if index == 14 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(
                    result["payload"]["requested_item_uuid"],
                    json!(copy.unwrap())
                );
                equipped = true;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(witness) = observed.and_then(|o| o.latest_primary_damage()) {
                assert_eq!(witness.contributors.len(), 1);
                let contribution = &witness.contributors[0];
                assert_eq!(contribution.equipment.definition_uuid, definition);
                assert_eq!(contribution.equipment.persistent_copy_uuid, copy);
                assert_ne!(witness.actor, witness.target);
                assert!(
                    witness.target_current_hp > 0
                        && witness.target_current_hp <= witness.target_max_hp
                );
                let wanted = (u64::from(witness.target_max_hp - witness.target_current_hp)
                    * coefficient
                    / u64::from(witness.target_max_hp))
                .min(50);
                assert_eq!(u64::from(witness.percent), wanted);
                let origins = trial.latest_primary_damage_origins().unwrap();
                assert!(origins[0]
                    .as_ref()
                    .unwrap()
                    .definition
                    .ends_with("AttackProgram"));
                observations.insert(
                    witness.time_ms,
                    json!({"witness":witness,"origins":origins}),
                );
            }
        }
        assert!(
            equipped
                && observations
                    .values()
                    .any(|v| v["witness"]["percent"].as_u64().unwrap() > 0),
            "{coefficient}: {observations:?}"
        );
        expected.push(json!({"coefficient":coefficient,"observations":observations.values().collect::<Vec<_>>()}));
    }
    let first = expected[0]["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["witness"]["percent"].as_u64().unwrap() > 0)
        .unwrap();
    let second = expected[1]["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["witness"]["time_ms"] == first["witness"]["time_ms"])
        .unwrap();
    assert!(
        second["witness"]["percent"].as_u64().unwrap()
            > first["witness"]["percent"].as_u64().unwrap(),
        "{expected:?}"
    );
    if let Some(directory) = std::env::var_os("PRIMARY_HP_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve prior evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
