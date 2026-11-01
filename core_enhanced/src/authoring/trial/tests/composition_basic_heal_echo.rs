use super::*;

#[test]
fn composition_basic_heal_echo_world_granted_pendant_focused_staff_and_origin() {
    use crate::game::data::equipment_data::{EquipmentDatabase, WeaponCombatProfile};
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    let path = "official/data/equipments/base.ron";
    let mut equipment: EquipmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    let mut pendant = equipment
        .items
        .iter()
        .find(|i| i.id == "crimson_goggles")
        .unwrap()
        .clone();
    pendant.id = "basic_heal_echo_fixture_pendant".into();
    pendant.uuid = uuid::Uuid::from_u128(0xe460);
    pendant.name = "검은·흰 펜던트 실제 기본 치료 자기 회복 시험용".into();
    pendant.triggered_effects.clear();
    pendant.ability_activations.clear();
    equipment.items.push(pendant);
    let mut staff = equipment
        .items
        .iter()
        .find(|i| i.id == "standard_training_healing_staff")
        .unwrap()
        .clone();
    staff.id = "yin_yang_fixture_staff".into();
    staff.uuid = uuid::Uuid::from_u128(0xe461);
    staff.name = "음양 최저 HP 아군 한 명 집중 치료 시험용 지팡이".into();
    staff.triggered_effects.clear();
    staff.ability_activations.clear();
    let mut profile: WeaponCombatProfile = ron::de::from_str(include_str!(
        "../../../../tests/fixtures/skill_composition/yin-yang-healing.weapon.ron"
    ))
    .unwrap();
    // Small World fixture amount distinguishes both authored pendant ratios.
    profile.action = crate::game::data::abnormality_data::BasicAttackAction::HealLowestHealth {
        flat: 8,
        spell_power_percent: 0,
        max_targets: 1,
    };
    staff.weapon_profile = Some(profile);
    equipment.items.push(staff);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),", "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"basic_heal_echo_fixture_pendant\"), GrantEquipment(equipment_id: \"yin_yang_fixture_staff\"),", 1));
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
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
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
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/basic_heal_echo.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: basic_heal_echo_fixture_pendant",
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
    for _ in 0..3 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    for (ratio, amount) in [(25, 2), (50, 4)] {
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
                .unwrap_or_else(|e| panic!("basic heal echo World step {pending}: {e:?}"))
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
        }
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let copy = &grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"];
        let staff_copy = &grant["payload"]["inventory_diff"]["added"][1]["Equipment"]["uuid"];
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let echo = observed
            .latest_basic_heal_echo()
            .expect("equipped pendant must react to normal allied basic heal");
        assert_eq!(
            (
                echo.actual_other_healed,
                echo.requested_self_heal,
                echo.actual_self_healed
            ),
            (8, amount, 0)
        );
        assert_eq!(
            serde_json::to_value(echo.equipment.persistent_copy_uuid).unwrap(),
            *copy
        );
        assert_eq!(
            serde_json::to_value(echo.healing_source.equipment.unwrap().persistent_copy_uuid)
                .unwrap(),
            *staff_copy
        );
        assert!(
            echo.healing_source.cast.is_none()
                && echo.healing_source.fragment.is_none()
                && echo.healing_source.action.is_some()
        );
        assert_ne!(echo.equipment, echo.healing_source.equipment.unwrap());
        let origin = trial.latest_basic_heal_echo_origin().unwrap();
        assert!(origin.definition.ends_with("SupportProgram"));
        expected.push(json!({"ratio":ratio,"echo":echo,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("BASIC_HEAL_ECHO_COMPOSITION_FIXTURE_DIR") {
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
