#[test]
fn composition_resonance_gain_world_actual_accessory_other_fragment_earlier_full_and_cast() {
    let (fixture, _, mut document) = state_world_fixture();
    let accessory = "placeholder_laetitia_accessory";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let metadata = equipment.get_by_id(accessory).unwrap();
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
            &format!("GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: {accessory:?}),"),
            1,
        ),
    );
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(enemy_path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.attack = 4;
    enemy.max_health = 10000;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    let fragment_path = "official/data/skill_fragments/base.ron";
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(&fixture.read(fragment_path)).unwrap();
    let mut updated_forms = 0;
    for fragment in &mut fragments.fragments {
        if let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = &mut fragment.effect
        {
            for form in forms.values_mut() {
                if form.skill_id.as_str() == "fragment_freischutz_black_round" {
                    form.resonance_max = 10;
                    updated_forms += 1;
                }
            }
        }
    }
    assert!(updated_forms > 0);
    fixture.write(fragment_path, ron::ser::to_string(&fragments).unwrap());
    let employee_path = "official/data/employees/definitions.ron";
    let mut employees: crate::game::data::employee_data::EmployeeDefinitionDatabase =
        ron::de::from_str(&fixture.read(employee_path)).unwrap();
    for employee in &mut employees.definitions {
        employee.profile.base_combat_stats.max_hp = 1000;
    }
    fixture.write(employee_path, ron::ser::to_string(&employees).unwrap());
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
    let token = fixture.load();
    let cancel = CancellationToken::default();
    let probe = simulate(
        fixture_content(&token),
        checked(&document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let units = probe.step(24).unwrap().public_result().unwrap()["payload"]["battle_update"]
        ["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let player = units
        .iter()
        .position(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
        .unwrap();
    drop(probe);
    for _ in 0..30 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let last = preparation + document["steps"].as_array().unwrap().len() - 1;
    let mut begin = authority_step("begin_skill_targeting", last);
    begin["command"]["employee_uuid"] = Value::Null;
    begin["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    begin["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut activate = authority_step("activate_skill", last + 1);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["command"]["target"] = json!({"type":"Unit","unit_instance_id":null});
    activate["bindings"].as_array_mut().unwrap().extend([
      json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),
      json!({"from":{"step":24,"pointer":format!("/payload/battle_update/checkpoint/units/{player}/unit_instance_id")},"to":"/command/target/unit_instance_id","value_type":"string"})]);
    document["steps"].as_array_mut().unwrap().extend([
        begin,
        activate,
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}),
    ]);
    let source=include_str!("../../../../tests/fixtures/skill_composition/resonance_gain.skill.yaml")
      .replace("equipment: r11_weapon",&format!("equipment: {accessory}"))
      .replace("skills: []",r#"skills:
  - id: fragment_freischutz_black_round
    name: 공명 획득 장신구의 다른 파편 검사
    kind: Targeted
    cast_targeting:
      Explicit: { selection: player_unit, target: AllySingle, reach: global, air_capable: true }
    prepare_ms: 0
    focus_permissions: { allows_move: true, allows_basic_attack: true }
    admission: { capacity: 10, requirement: Full, cost: All }
    body:
      Action:
        id: recover
        target: { Literal: { Target: CastTarget } }
        delivery: { Literal: { Delivery: Instant } }
        effects:
          - Heal:
              id: small-heal
              tags: []
              amount: { Literal: { Amount: { moment: Snapshot, root: { Const: { value: 1, unit: Magnitude } } } } }
        wait: EffectsSettled
"#);
    let mut expected = Vec::new();
    for percent in [20, 40] {
        let current = source.replace("percent: 20", &format!("percent: {percent}"));
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
        let mut activated = false;
        let mut observations = std::collections::BTreeMap::new();
        let mut initial = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("resonance gain World step {pending}: {e:?}"))
                .index();
            let public = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 13 {
                let gear = &public["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(gear["definition_id"], accessory);
                copy = Some(
                    gear["uuid"]
                        .as_str()
                        .unwrap()
                        .parse::<uuid::Uuid>()
                        .unwrap(),
                );
            }
            if index == 14 {
                assert_eq!(public["disposition"], "accepted");
                assert_eq!(
                    public["payload"]["requested_item_uuid"],
                    json!(copy.unwrap())
                );
                equipped = true;
            }
            if index == 24 {
                let u = &public["payload"]["battle_update"]["checkpoint"]["units"][player];
                initial = Some(u.clone());
            }
            if index == last + 2 {
                assert_eq!(public["disposition"], "accepted", "{public}");
                activated = true;
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(w) = observed.and_then(|o| o.latest_resonance_gain()) {
                assert_eq!(w.contributors.len(), 1);
                let provider = &w.contributors[0];
                assert_eq!(provider.equipment.definition_uuid, definition);
                assert_eq!(provider.equipment.persistent_copy_uuid, copy);
                assert_eq!(w.percent, u64::from(percent));
                assert_eq!(provider.percent, percent);
                assert_eq!(w.max, 10);
                assert_eq!(w.actual_gain, w.after - w.before);
                assert!(w.remainder_before < 100 && w.remainder_after < 100);
                let origins = trial.latest_resonance_gain_origins().unwrap();
                assert!(origins[0]
                    .as_ref()
                    .unwrap()
                    .definition
                    .ends_with("SupportProgram"));
                observations.insert(w.time_ms, json!({"witness":w,"origins":origins}));
            }
        }
        assert!(equipped && activated && !observations.is_empty());
        let full = observations
            .values()
            .find(|v| v["witness"]["after"] == 10)
            .expect("authored increase must reach full");
        assert_eq!(
            observations.values().next().unwrap()["witness"]["before"],
            2,
            "initial20% is not amplified"
        );
        expected.push(json!({"percent":percent,"first_full_ms":full["witness"]["time_ms"],"initial":initial,"observations":observations.values().collect::<Vec<_>>() }));
    }
    assert_eq!(expected[0]["first_full_ms"], 7000);
    assert_eq!(expected[1]["first_full_ms"], 6000);
    if let Some(directory) = std::env::var_os("RESONANCE_GAIN_WORLD_FIXTURE_DIR") {
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
