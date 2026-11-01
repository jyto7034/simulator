use super::*;

#[test]
fn composition_heal_barrier_world_actual_armor_copy_and_fragment_heal() {
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut armor = equipment
        .items
        .iter()
        .find(|i| i.id == "crimson_goggles")
        .unwrap()
        .clone();
    armor.id = "heal_barrier_fixture_armor".into();
    armor.uuid = uuid::Uuid::from_u128(0xe440);
    armor.name = "정의의 결계 실제 회복 반응 시험용".into();
    armor.equipment_type = crate::game::data::equipment_data::EquipmentType::Armor;
    armor.triggered_effects.clear();
    armor.ability_activations.clear();
    equipment.items.push(armor);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path,fixture.read(path).replacen("GrantEnkephalin(amount: 100),","GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"heal_barrier_fixture_armor\"),",1));
    fixture.author_trial_facility();
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
    // Deploy the recipient on the enemy route first and the healer beside it
    // after the existing combat clock. This creates a real allied wound.
    // The UI fixture uses 100%; exact 50% is covered by Core and basic-heal World.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    for (index, employee_step) in [(24, 10), (26, 9)] {
        for binding in document["steps"][index - preparation]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if binding["to"] == "/command/employee_uuid" {
                binding["from"]["step"] = json!(employee_step);
            }
        }
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/heal_barrier.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: heal_barrier_fixture_armor",
            )
            .replace("hp_threshold_percent: 50", "hp_threshold_percent: 100")
            .replace(
                "skills: []",
                r#"skills:
  - id: fragment_freischutz_black_round
    name: 회복 결계 실제 장비 검사
    kind: Targeted
    cast_targeting:
      Explicit: { selection: player_unit, target: AllySingle, reach: global, air_capable: true }
    prepare_ms: 0
    focus_permissions: { allows_move: true, allows_basic_attack: true }
    admission: { capacity: 25, requirement: None, cost: Zero }
    body:
      Action:
        id: actual-allied-heal
        target: { Literal: { Target: CastTarget } }
        delivery: { Literal: { Delivery: Instant } }
        effects:
          - Heal:
              id: direct-recovery
              tags: []
              amount: { Literal: { Amount: { moment: Snapshot, root: { Const: { value: 8, unit: Magnitude } } } } }
        wait: EffectsSettled
"#,
            );
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(27 - preparation);
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
    probe.run_to_end(&cancel).unwrap();
    let result = probe.session().step(26).unwrap().public_result().unwrap();
    let units = result["payload"]["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let employee = probe.session().step(10).unwrap().public_result().unwrap()["payload"]
        ["employee_uuid"]
        .clone();
    let target = units
        .iter()
        .position(|u| u["unit_source"]["employee_uuid"] == employee)
        .unwrap();
    let mut begin = authority_step("begin_skill_targeting", 26);
    begin["command"]["employee_uuid"] = Value::Null;
    begin["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    begin["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut activate = authority_step("activate_skill", 27);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["command"]["target"] = json!({"type":"Unit","unit_instance_id":null});
    activate["bindings"].as_array_mut().unwrap().extend([
        json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),
        json!({"from":{"step":26,"pointer":format!("/payload/battle_update/checkpoint/units/{target}/unit_instance_id")},"to":"/command/target/unit_instance_id","value_type":"string"})]);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .extend([begin, activate]);
    for _ in 0..3 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    drop(probe);
    let mut expected = Vec::new();
    for (ratio, capacity) in [(25, 2), (50, 6)] {
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
        trial.run_to_end(&cancel).unwrap();
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let copy = grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"].clone();
        assert_eq!(
            trial.session().step(14).unwrap().public_result().unwrap()["disposition"],
            "accepted"
        );
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let barrier = observed
            .latest_heal_barrier()
            .expect("actual armor must react to allied fragment healing");
        assert_eq!(barrier.actual_healed, 10);
        assert_eq!(barrier.increased_capacity, capacity);
        assert_eq!(barrier.remaining, capacity);
        assert_ne!(barrier.actor, barrier.target);
        assert!(u64::from(barrier.hp_before) * 100 <= u64::from(barrier.max_hp) * 100);
        assert_eq!(
            serde_json::to_value(barrier.equipment.persistent_copy_uuid).unwrap(),
            copy
        );
        assert!(barrier.healing_source.cast.is_some() && barrier.healing_source.action.is_some());
        let origin = trial.latest_heal_barrier_origin().unwrap();
        assert!(origin.definition.ends_with("SupportProgram"));
        expected.push(json!({"ratio":ratio,"barrier":barrier,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("HEAL_BARRIER_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve prior fixture evidence");
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
