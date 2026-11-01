use super::*;

#[test]
fn composition_idle_hp_guard_world_armor_reward_equip_and_hp_damage_edit() {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut item = equipment.get_by_id("standard_armor").unwrap().clone();
    item.id = "idle_guard_fixture_coat".into();
    item.uuid = uuid::Uuid::from_u128(0xe1010);
    item.name = "짚 완충 외투 기술 검증값".into();
    item.triggered_effects = ron::de::from_str(r#"{Permanent:[(target:SelfUnit,effect:Modifier((stat:MaxHealth,kind:Flat,value:80))),(target:SelfUnit,effect:Modifier((stat:Defense,kind:Flat,value:10)))]}"#).unwrap();
    item.ability_activations.clear();
    assert_eq!(
        item.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Armor
    );
    equipment.items.push(item);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"idle_guard_fixture_coat\"),",
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
    enemy.attack = 100;
    enemy.basic_attack.interval_ms = 3000;
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
        include_str!("../../../../tests/fixtures/skill_composition/idle_hp_guard.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: idle_guard_fixture_coat",
            );
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for reduction in [50, 25] {
        let current = source.replace(
            "reduction_percent: 50",
            &format!("reduction_percent: {reduction}"),
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
        let mut copy = Value::Null;
        let mut latest = None;
        let mut last_index = 0;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("idle guard World {reduction} step{pending}: {e:?}"))
                .index();
            last_index = index;
            let public = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 13 {
                let item = &public["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "idle_guard_fixture_coat");
                copy = item["uuid"].clone();
            }
            if index == 14 {
                assert_eq!(public["disposition"], "accepted");
                assert_eq!(public["payload"]["requested_item_uuid"], copy);
            }
            if index == 24 {
                assert_eq!(public["disposition"], "accepted");
                let units = public["payload"]["battle_update"]["checkpoint"]["units"]
                    .as_array()
                    .unwrap();
                let wearer = units
                    .iter()
                    .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                    .unwrap();
                assert_eq!(wearer["stats"]["max_health"], 180);
                assert_eq!(wearer["stats"]["defense"], 10);
            }
            if index < 25 {
                continue;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
                .unwrap();
            if let Some(w) = observed.latest_idle_hp_guard() {
                assert_eq!(w.equipment.definition_uuid, uuid::Uuid::from_u128(0xe1010));
                assert_eq!(json!(w.equipment.persistent_copy_uuid), copy);
                assert_eq!(w.reduction_percent, reduction);
                assert!(w.time_ms - w.charged_since_ms >= u64::from(w.recharge_ms));
                assert!(w.prevented > 0 && w.incoming_hp_damage > w.prevented);
                let origin = trial.latest_idle_hp_guard_origin().unwrap();
                assert!(origin.definition.ends_with("DefenseProgram"));
                let units = public["payload"]["battle_update"]["checkpoint"]["units"]
                    .as_array()
                    .unwrap();
                let wearer = units
                    .iter()
                    .find(|u| u["unit_instance_id"] == json!(w.target))
                    .unwrap();
                assert_eq!(
                    wearer["stats"]["current_health"].as_u64().unwrap(),
                    180 - u64::from(w.incoming_hp_damage - w.prevented)
                );
                latest = Some(
                    json!({"reduction_percent":reduction,"observed":w,"origin":origin,"copy":copy,"wearer":wearer}),
                );
                break;
            }
        }
        assert!(
            latest.is_some(),
            "actual enemy damage must consume charged armor"
        );
        expected.push(latest.unwrap());
        if reduction == 50 {
            document["steps"]
                .as_array_mut()
                .unwrap()
                .truncate(last_index + 1 - preparation);
        }
    }
    assert_eq!(
        expected[0]["observed"]["time_ms"],
        expected[1]["observed"]["time_ms"]
    );
    assert!(
        expected[0]["wearer"]["stats"]["current_health"]
            .as_u64()
            .unwrap()
            > expected[1]["wearer"]["stats"]["current_health"]
                .as_u64()
                .unwrap()
    );
    if let Some(directory) = std::env::var_os("IDLE_HP_GUARD_COMPOSITION_FIXTURE_DIR") {
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
