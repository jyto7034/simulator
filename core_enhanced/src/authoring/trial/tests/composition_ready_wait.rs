#[test]
fn composition_ready_wait_world_actual_armor_reward_cast_and_outgoing_damage() {
    let (fixture, _, mut document) = state_world_fixture();
    let weapon = "placeholder_king_of_greed_armor";
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let metadata = equipment.get_by_id(weapon).unwrap();
    assert_eq!(
        metadata.equipment_type,
        crate::game::data::equipment_data::EquipmentType::Armor
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
    enemy.max_health = 10000;
    enemy.defense = 0;
    enemy.basic_attack.damage_type = crate::game::battle::damage::DamageType::Physical;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(enemy_path, ron::ser::to_string(&enemies).unwrap());
    // Explicit fixture HP keeps the wearer alive through waiting and the
    // outgoing-damage comparison window.
    let employee_path = "official/data/employees/definitions.ron";
    let mut employees: crate::game::data::employee_data::EmployeeDefinitionDatabase =
        ron::de::from_str(&fixture.read(employee_path)).unwrap();
    for employee in &mut employees.definitions {
        employee.profile.base_combat_stats.max_hp = 1000;
    }
    fixture.write(employee_path, ron::ser::to_string(&employees).unwrap());
    fixture.author_trial_facility();
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
    let source = include_str!("../../../../tests/fixtures/skill_composition/ready_wait.skill.yaml")
        .replace("equipment: r11_weapon", &format!("equipment: {weapon}"));
    let source = source
        .replace("max_percent: 50", "max_percent: 500")
        .replace(
            "skills: []",
            r#"skills:
  - id: fragment_freischutz_black_round
    name: 시전 대기 방어구 검사
    kind: Targeted
    cast_targeting:
      Explicit: { selection: player_unit, target: AllySingle, reach: global, air_capable: true }
    prepare_ms: 0
    focus_permissions: { allows_move: true, allows_basic_attack: true }
    admission: { capacity: 25, requirement: Full, cost: All }
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
"#,
        );
    let mut expected = Vec::new();
    for coefficient in [10, 20] {
        let current = source.replace(
            "per_second_percent: 10",
            &format!("per_second_percent: {coefficient}"),
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
        let mut events = std::collections::BTreeMap::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let previous = trial
                .session()
                .step(pending.saturating_sub(1))
                .and_then(|s| s.public_result())
                .cloned();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| {
                    panic!("ready-wait World {pending}: {e:?}; previous={previous:?}")
                })
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 13 {
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
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(
                    result["payload"]["requested_item_uuid"],
                    json!(copy.unwrap())
                );
                equipped = true;
            }
            if let Some(entries) =
                result["payload"]["battle_update"]["events_delta"]["events"].as_array()
            {
                for event in entries {
                    if event["event"]["type"] == "HpChanged" {
                        events.insert(event["seq"].as_u64().unwrap(), event.clone());
                    }
                }
            }
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(w) = observed.and_then(|o| o.latest_ready_wait()) {
                assert_eq!(w.contributors.len(), 1);
                let c = &w.contributors[0];
                assert_eq!(c.equipment.definition_uuid, definition);
                assert_eq!(c.equipment.persistent_copy_uuid, copy);
                assert_eq!(c.per_second_percent, coefficient);
                assert!(c.waited_ms > 0);
                assert_eq!(c.until_ms, w.time_ms + 15000);
                assert_eq!(
                    c.percent,
                    (c.waited_ms * u64::from(coefficient) / 1000).min(500) as u32
                );
                assert_eq!(w.percent, c.percent as i32);
                let origins = trial.latest_ready_wait_origins().unwrap();
                assert!(origins[0]
                    .as_ref()
                    .unwrap()
                    .definition
                    .ends_with("DefenseProgram"));
                observations.insert(w.time_ms, json!({"witness":w,"origins":origins}));
            }
        }
        assert!(
            equipped && !observations.is_empty(),
            "{coefficient}: {observations:?}"
        );
        let w = &observations.values().next().unwrap()["witness"];
        let hits: Vec<_> = events
            .values()
            .filter(|event| {
                event["time_ms"].as_u64().unwrap() > w["time_ms"].as_u64().unwrap()
                    && event["event"]["source_instance_id"] == w["actor"]
                    && event["event"]["delta"].as_i64().is_some_and(|v| v < 0)
            })
            .cloned()
            .collect();
        assert!(
            !hits.is_empty(),
            "actual wearer must deal HP damage after cast: {events:?}; {w:?}"
        );
        expected.push(json!({"coefficient":coefficient,"observations":observations.values().collect::<Vec<_>>(),"hits":hits}));
    }
    assert_eq!(
        expected[0]["observations"][0]["witness"]["time_ms"],
        expected[1]["observations"][0]["witness"]["time_ms"]
    );
    assert!(
        expected[1]["observations"][0]["witness"]["percent"]
            .as_i64()
            .unwrap()
            > expected[0]["observations"][0]["witness"]["percent"]
                .as_i64()
                .unwrap()
    );
    assert_eq!(
        expected[0]["hits"][0]["time_ms"],
        expected[1]["hits"][0]["time_ms"]
    );
    assert_eq!(
        expected[0]["hits"][0]["event"]["target_instance_id"],
        expected[1]["hits"][0]["event"]["target_instance_id"]
    );
    assert!(
        expected[1]["hits"][0]["event"]["delta"].as_i64().unwrap()
            < expected[0]["hits"][0]["event"]["delta"].as_i64().unwrap(),
        "{expected:?}"
    );
    if let Some(directory) = std::env::var_os("READY_WAIT_COMPOSITION_FIXTURE_DIR") {
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
