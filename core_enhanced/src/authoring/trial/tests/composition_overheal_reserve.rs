use super::*;

#[test]
fn composition_overheal_reserve_world_actual_armor_received_overflow_and_release() {
    use crate::game::data::equipment_data::{EquipmentDatabase, WeaponCombatProfile};
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    // The inherited support fixture inflicts only one HP per hit: ten hits after
    // the heal left 90/100 HP, so it cannot exercise the 50% release threshold.
    // Increase only this isolated rusher's authored attack; keep the real hit path.
    let enemies = "official/data/enemies/corroded_employees.ron";
    let enemy_source = fixture.read(enemies);
    assert!(enemy_source.contains("attack: 1,"));
    fixture.write(
        enemies,
        enemy_source.replacen("attack: 1,", "attack: 8,", 1),
    );
    let path = "official/data/equipments/base.ron";
    let mut equipment: EquipmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    let mut armor = equipment
        .items
        .iter()
        .find(|i| i.equipment_type == crate::game::data::equipment_data::EquipmentType::Armor)
        .unwrap()
        .clone();
    armor.id = "overheal_reserve_fixture_armor".into();
    armor.uuid = uuid::Uuid::from_u128(0xe470);
    armor.name = "태극 받은 초과 회복 저장·방출 시험용".into();
    armor.triggered_effects.clear();
    armor.ability_activations.clear();
    equipment.items.push(armor);
    let mut staff = equipment
        .items
        .iter()
        .find(|i| i.id == "standard_training_healing_staff")
        .unwrap()
        .clone();
    staff.id = "yin_yang_fixture_staff".into();
    staff.uuid = uuid::Uuid::from_u128(0xe471);
    staff.name = "음양 최저 HP 아군 한 명 집중 치료 시험용 지팡이".into();
    staff.triggered_effects.clear();
    staff.ability_activations.clear();
    let mut profile: WeaponCombatProfile = ron::de::from_str(include_str!(
        "../../../../tests/fixtures/skill_composition/yin-yang-healing.weapon.ron"
    ))
    .unwrap();
    // One large basic heal creates received overflow; a slow interval lets later injury release it.
    profile.action = crate::game::data::abnormality_data::BasicAttackAction::HealLowestHealth {
        flat: 4000,
        spell_power_percent: 0,
        max_targets: 1,
    };
    profile.interval_ms = 60000;
    staff.weapon_profile = Some(profile);
    equipment.items.push(staff);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),", "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"overheal_reserve_fixture_armor\"), GrantEquipment(equipment_id: \"yin_yang_fixture_staff\"),", 1));
    fixture.author_trial_facility();
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
    for index in 0..2 {
        document["steps"].as_array_mut().unwrap().insert(5+index,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
            {"from":{"step":13,"pointer":format!("/payload/inventory_diff/added/{index}/Equipment/uuid")},"to":"/command/item_uuid","value_type":"string"},
            {"from":{"step":if index == 0 {10} else {9},"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    }
    for (index, employee) in [(25, 10), (27, 9)] {
        for binding in document["steps"][index - preparation]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if binding["to"] == "/command/employee_uuid" {
                binding["from"]["step"] = json!(employee);
            }
        }
    }
    // Attack eight needs the healer before the thirteenth hit (104 damage).
    // The 17-second arrival leaves the frontline injured and alive for storage.
    document["steps"][26 - preparation]["clock"]["delta_ms"] = json!(17000);
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/overheal_reserve.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: overheal_reserve_fixture_armor",
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
    let platform = zones
        .iter()
        .position(|z| z["kind"] == "Platform")
        .expect("a platform for the focused staff");
    let units = probe.session().step(26).unwrap().public_result().unwrap()["payload"]
        ["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let cell = zones[platform]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .position(|cell| !units.iter().any(|u| u["position"] == *cell))
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
    for _ in 0..40 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    for ratio in [50, 75] {
        let current = source.replace(
            "max_hp_cap_percent: 50",
            &format!("max_hp_cap_percent: {ratio}"),
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
                .unwrap_or_else(|e| panic!("overheal reserve World step {pending}: {e:?}"))
                .index();
            if [14, 15, 25, 27].contains(&index) {
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
            if index >= 26 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                let cp = &result["payload"]["battle_update"]["checkpoint"];
                if let Some(units) = cp["units"].as_array() {
                    let compact:Vec<_>=units.iter().map(|u|json!({"id":u["unit_instance_id"],"owner":u["owner"],"hp":u["stats"]["current_health"],"max":u["stats"]["max_health"],"attack":u["stats"]["attack"],"position":u["position"]})).collect();
                    eprintln!("reserve world step {index}: {compact:?}");
                }
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if observed
                .and_then(|o| o.latest_overheal_reserve())
                .is_some_and(|r| {
                    matches!(
                        r.operation,
                        crate::game::battle::core::equipment_defense::ReserveOperation::Release
                    )
                })
            {
                if ratio == 50 {
                    document["steps"]
                        .as_array_mut()
                        .unwrap()
                        .truncate(index + 1 - preparation);
                }
                break;
            }
        }
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let copy = &grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"];
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let reserve = observed
            .latest_overheal_reserve()
            .expect("armor must store received overflow then release after injury");
        assert!(
            matches!(
                reserve.operation,
                crate::game::battle::core::equipment_defense::ReserveOperation::Release
            ),
            "latest reserve: {reserve:?}"
        );
        assert!(reserve.actual_healed > 0);
        assert_eq!(reserve.capacity, ratio);
        assert_eq!(reserve.stored_before, ratio);
        assert_eq!(
            reserve.stored_before - reserve.stored_after,
            reserve.actual_healed
        );
        assert_eq!(
            serde_json::to_value(reserve.equipment.persistent_copy_uuid).unwrap(),
            *copy
        );
        let origin = trial.latest_overheal_reserve_origin().unwrap();
        assert!(origin.definition.ends_with("DefenseProgram"));
        expected.push(json!({"ratio":ratio,"reserve":reserve,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("OVERHEAL_RESERVE_COMPOSITION_FIXTURE_DIR") {
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
