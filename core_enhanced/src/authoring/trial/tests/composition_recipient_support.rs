use super::*;
#[test]
fn recipient_support_world_actual_brooch_reward_block_staff_heal_and_enemy_control() {
    use crate::game::data::equipment_data::{EquipmentDatabase, WeaponCombatProfile};
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(true);
    let path = "official/data/equipments/base.ron";
    let mut equipment: EquipmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    let brooch_id = equipment
        .get_by_id("black_swan_memory_brooch")
        .unwrap()
        .uuid;
    let mut staff = equipment
        .get_by_id("standard_training_healing_staff")
        .unwrap()
        .clone();
    staff.id = "recipient_trial_staff".into();
    staff.uuid = uuid::Uuid::from_u128(0xe861);
    staff.triggered_effects.clear();
    staff.ability_activations.clear();
    let mut profile: WeaponCombatProfile = ron::de::from_str(include_str!(
        "../../../../tests/fixtures/skill_composition/yin-yang-healing.weapon.ron"
    ))
    .unwrap();
    profile.action = crate::game::data::abnormality_data::BasicAttackAction::HealLowestHealth {
        flat: 8,
        spell_power_percent: 0,
        max_targets: 1,
    };
    staff.weapon_profile = Some(profile);
    equipment.items.push(staff);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path,fixture.read(path).replacen("GrantEnkephalin(amount: 100),","GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"black_swan_memory_brooch\"), GrantEquipment(equipment_id: \"recipient_trial_staff\"),",1));
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut enemy = enemies
        .profiles
        .iter()
        .find(|p| p.id == "corroded_rusher")
        .unwrap()
        .clone();
    enemy.id = "recipient_trial_enemy".into();
    enemy.uuid = uuid::Uuid::from_u128(0xe862);
    enemy.max_health = 5000;
    enemy.attack = 15;
    enemy.basic_attack.interval_ms = 1000;
    enemy.skill_id = Some("recipient_trial_control".into());
    // Autocast becomes pending on the real basic-attack gain crossing to full.
    enemy.resonance.start = 90;
    enemy.resonance.max = 100;
    enemies.profiles.push(enemy);
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let path = "official/data/pve/encounters.ron";
    let mut encounters: crate::game::data::pve_data::PveEncounterDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let encounter = encounters
        .encounters
        .iter_mut()
        .find(|e| e.id == "normal_breach_response")
        .unwrap();
    encounter.waves.truncate(1);
    encounter.waves[0].source = crate::game::data::pve_data::PveWaveSource::Manual(vec![
        crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
            profile_id: "recipient_trial_enemy".into(),
            tier: crate::game::enums::Tier::I,
            count: 1,
        },
    ]);
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
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
                        *v = json!(v.as_u64().unwrap() + 2)
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
    for (index, employee) in [(0, 10), (1, 9)] {
        document["steps"].as_array_mut().unwrap().insert(5+index,json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[{"from":{"step":13,"pointer":format!("/payload/inventory_diff/added/{index}/Equipment/uuid")},"to":"/command/item_uuid","value_type":"string"},{"from":{"step":employee,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]}));
    }
    for (index, employee) in [(25, 10), (27, 9)] {
        for b in document["steps"][index - preparation]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if b["to"] == "/command/employee_uuid" {
                b["from"]["step"] = json!(employee);
            }
        }
    }
    document["steps"][26 - preparation]["clock"]["delta_ms"] = json!(1);
    let source=include_str!("../../../../tests/fixtures/skill_composition/recipient_support.skill.yaml").replace("skills: []",r#"skills:
  - id: recipient_trial_control
    name: 적의 유한 이동 감속 시험
    kind: Targeted
    cast_targeting:
      Explicit:
        selection: automatic_unit
        target: { EnemySingle: { rule: Nearest } }
        reach: pattern
        air_capable: true
        target_range: { include_anchor_tile: true, rows: [XXX, X@X, XXX] }
    prepare_ms: 0
    focus_permissions: { allows_move: true, allows_basic_attack: true }
    admission: null
    body:
      Action:
        id: enemy-control
        target: { Literal: { Target: CastTarget } }
        delivery: { Literal: { Delivery: Instant } }
        effects:
          - Append:
              effects:
                Literal:
                  Effects:
                    - id: finite-slow
                      op:
                        Sustained:
                          definition:
                            HitStatus:
                              duration_ms: 2000
                              profile: { definition: enemy-finite-slow, kind: { Slow: { percent: 30 } } }
        wait: EffectsSettled
"#);
    // Base pack validation precedes the composition overlay. Register the same
    // authored hostile skill so its profile reference is valid at both stages.
    let path = "official/data/skills/base.ron";
    let mut skills =
        crate::game::data::skill_data::SkillDatabase::from_ron(path, &fixture.read(path)).unwrap();
    skills.skills.extend(
        crate::authoring::composition::compile_yaml(&source)
            .unwrap()
            .load_native()
            .unwrap()
            .skills,
    );
    fixture.write(path, ron::ser::to_string(&skills).unwrap());
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
    let platform = zones.iter().position(|z| z["kind"] == "Platform").unwrap();
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
    for _ in 0..160 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    for (rate, reduction) in [(20, 50), (35, 25)] {
        let current = source
            .replace("value: 20,", &format!("value: {rate},"))
            .replace(
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
        let mut heals = std::collections::BTreeMap::new();
        let mut public_events = std::collections::BTreeMap::new();
        let mut last_checkpoint = Value::Null;
        let mut controls = Vec::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("recipient World step {pending}: {e:?}"))
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if [14, 15, 25, 27].contains(&index) {
                assert_eq!(result["disposition"], "accepted", "{result}");
            }
            let update = &result["payload"]["battle_update"];
            if update["checkpoint"].is_object() {
                last_checkpoint = update["checkpoint"].clone();
            }
            if let Some(events) = update["events_delta"]["events"].as_array() {
                for event in events {
                    public_events.insert(event["seq"].as_u64().unwrap(), event.clone());
                    if event["event"]["type"] == "HpChanged"
                        && event["event"]["delta"].as_i64().is_some_and(|d| d > 0)
                    {
                        heals.insert(event["seq"].as_u64().unwrap(), event.clone());
                    }
                }
            }
            if let Some(active) = update["checkpoint"]["sustained_effects"]["active"].as_array() {
                for r in active {
                    if r["hit_status"]["definition"] == "enemy-finite-slow" {
                        controls.push(r.clone());
                    }
                }
            }
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .expect("brooch wearer survives selected observation");
        let healing = observed
            .latest_received_heal()
            .expect("staff heals actually blocking wearer");
        let control = observed.latest_control_duration().unwrap_or_else(|| {
            if let Some(directory) = std::env::var_os("RECIPIENT_SUPPORT_COMPOSITION_FIXTURE_DIR") {
                let path =
                    std::path::PathBuf::from(directory).with_extension("control-failure.json");
                std::fs::write(
                    &path,
                    serde_json::to_vec_pretty(&json!({
                        "events": public_events, "checkpoint": last_checkpoint,
                        "healing": healing, "controls": controls,
                    }))
                    .unwrap(),
                )
                .unwrap();
                eprintln!("control failure evidence: {}", path.display());
            }
            panic!("enemy finite control attaches to brooch wearer")
        });
        assert_eq!(healing.blocked_hostiles, 1);
        assert_eq!(healing.percent, rate);
        assert_eq!(healing.before, 8);
        assert_eq!(healing.after, 8 * (100 + rate) / 100);
        assert_ne!(healing.actor, Some(healing.target));
        assert_eq!(control.target, healing.target);
        assert_eq!(control.original_ms, 2000);
        assert_eq!(control.effective_ms, 2000 * (100 - reduction as u64) / 100);
        assert_eq!(control.reduction_percent, reduction);
        let copy = trial.session().step(13).unwrap().public_result().unwrap()["payload"]
            ["inventory_diff"]["added"][0]["Equipment"]["uuid"]
            .clone();
        assert_eq!(
            serde_json::to_value(healing.equipment.persistent_copy_uuid).unwrap(),
            copy
        );
        assert_eq!(healing.equipment.definition_uuid, brooch_id);
        assert_eq!(
            control.equipment.persistent_copy_uuid,
            healing.equipment.persistent_copy_uuid
        );
        let public: Vec<_> = heals
            .values()
            .filter(|e| {
                e["event"]["target_instance_id"] == serde_json::to_value(healing.target).unwrap()
                    && e["event"]["delta"] == healing.after
            })
            .cloned()
            .collect();
        assert!(
            !public.is_empty(),
            "actual public heal must show coefficient result: {heals:?}"
        );
        assert!(
            !controls.is_empty(),
            "finite control must appear in public active states"
        );
        let heal_origin = trial.latest_received_heal_origin().unwrap();
        let control_origin = trial.latest_control_duration_origin().unwrap();
        assert!(
            heal_origin.definition.ends_with("SupportProgram")
                && control_origin.definition.ends_with("SupportProgram")
        );
        expected.push(json!({"rate":rate,"reduction":reduction,"healing":healing,"control":control,"heal_origin":heal_origin,"control_origin":control_origin,"public_heals":public,"public_controls":controls}));
    }
    assert_ne!(
        expected[0]["healing"]["after"],
        expected[1]["healing"]["after"]
    );
    assert_ne!(
        expected[0]["control"]["effective_ms"],
        expected[1]["control"]["effective_ms"]
    );
    if let Some(directory) = std::env::var_os("RECIPIENT_SUPPORT_COMPOSITION_FIXTURE_DIR") {
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
