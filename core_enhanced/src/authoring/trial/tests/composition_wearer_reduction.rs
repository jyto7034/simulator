use super::*;

#[test]
fn wearer_reduction_world_actual_armor_reward_copy_and_current_hp_curve() {
    let (fixture, _, mut document) = state_world_fixture();
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    assert!(equipment
        .get_by_id("placeholder_schadenfreude_armor")
        .is_some());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"placeholder_schadenfreude_armor\"),",
            1,
        ),
    );
    // Isolate the second encounter while preserving the prerequisite reward battle.
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut striker = enemies
        .profiles
        .iter()
        .find(|p| p.id == "corroded_rusher")
        .unwrap()
        .clone();
    striker.id = "wearer_reduction_striker".into();
    striker.uuid = uuid::Uuid::from_u128(0x9191);
    striker.name = "Current HP reduction trial striker".into();
    striker.max_health = 5000;
    striker.attack = 30;
    striker.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    enemies.range_presets.push(
        serde_json::from_value(json!({
            "id": "wearer_reduction_range",
            "range": {"include_anchor_tile":true,"rows":[
                "XXXXXXXXX","XXXXXXXXX","XXXXXXXXX","XXXXXXXXX","XXXX@XXXX",
                "XXXXXXXXX","XXXXXXXXX","XXXXXXXXX","XXXXXXXXX"]}
        }))
        .unwrap(),
    );
    striker.basic_attack_range_preset = Some("wearer_reduction_range".into());
    striker.basic_attack.defense_tile_range = None;
    striker.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
    striker.basic_attack.ranged_reposition_ms = 0;
    striker.defense = 0;
    striker.magic_resist = 0;
    striker.basic_attack.interval_ms = 1000;
    enemies.profiles.push(striker);
    // Deserialization resolves range presets; avoid serializing both authored authorities.
    for profile in &mut enemies.profiles {
        if profile.basic_attack_range_preset.is_some() {
            profile.basic_attack.defense_tile_range = None;
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
            profile_id: "wearer_reduction_striker".into(),
            tier: crate::game::enums::Tier::I,
            count: 1,
        },
    ]);
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    fixture.author_trial_facility();
    // Put the wearer in range at enemy spawn, while the defended objective is
    // still outside it. Ordinary target selection then exercises this armor.
    let path = "official/data/map/battlefield_templates.ron";
    let mut templates: Vec<crate::game::combat_preview::BattlefieldTemplateDefinition> =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let template = templates
        .iter_mut()
        .find(|t| t.id == "corridor_hook_control_01")
        .unwrap();
    let mut authored = serde_json::to_value(&*template).unwrap();
    assert_eq!(authored["rows"][3], "  .....  ");
    authored["rows"][3] = json!("  ...P.  ");
    *template = serde_json::from_value(authored).unwrap();
    fixture.write(path, ron::ser::to_string(&templates).unwrap());
    let deploy = document["steps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|step| step["command"]["type"] == "deploy_unit")
        .unwrap();
    deploy["command"]["position"] = json!({"x":5,"y":3});
    deploy["bindings"]
        .as_array_mut()
        .unwrap()
        .retain(|b| b["to"] != "/command/position");
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
    document["steps"][25 - preparation]["clock"]["delta_ms"] = json!(1);
    // Observe repeated hostile basics on the actual equipped armor wearer.
    for _ in 0..110 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/wearer_reduction.skill.yaml")
            .to_string();
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [50, 35] {
        let current = source.replace("value: 50,", &format!("value: {percent},"));
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
        let mut events = std::collections::BTreeMap::new();
        let mut wearer = None;
        let mut last_checkpoint = Value::Null;
        while !trial.session().is_complete() {
            let index = trial.advance(&cancel).unwrap().index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if [14, 24].contains(&index) {
                assert_eq!(result["disposition"], "accepted");
            }
            if index < 25 {
                continue;
            }
            let update = &result["payload"]["battle_update"];
            if let Some(units) = update["checkpoint"]["units"].as_array() {
                last_checkpoint = update["checkpoint"].clone();
                wearer = units
                    .iter()
                    .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
                    .map(|u| u["unit_instance_id"].clone())
                    .or(wearer);
            }
            if let Some(entries) = update["events_delta"]["events"].as_array() {
                for event in entries {
                    if event["event"]["type"] == "HpChanged" {
                        events.insert(event["seq"].as_u64().unwrap(), event.clone());
                    }
                }
            }
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .expect("armor wearer remains alive through ordinary hits");
        let w = observed
            .latest_wearer_reduction()
            .unwrap_or_else(|| panic!("actual current HP reduction witness; checkpoint={last_checkpoint}; hp_events={events:?}"));
        let wearer = wearer.unwrap();
        let wearer_state = last_checkpoint["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|unit| unit["unit_instance_id"] == wearer)
            .unwrap();
        assert_eq!(wearer_state["stats"]["max_health"], 200);
        assert_eq!(wearer_state["stats"]["defense"], 20);
        assert_eq!(wearer_state["stats"]["magic_resist"], 0);
        assert_eq!(serde_json::to_value(w.target).unwrap(), wearer);
        assert!(w.incoming.action.is_some());
        assert!(w.incoming.actor.is_some());
        assert_eq!(w.percent, percent as u32);
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        assert_eq!(
            grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["definition_id"],
            "placeholder_schadenfreude_armor"
        );
        assert_eq!(
            serde_json::to_value(w.equipment.persistent_copy_uuid).unwrap(),
            grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
        );
        assert_eq!(
            w.equipment.definition_uuid,
            equipment
                .get_by_id("placeholder_schadenfreude_armor")
                .unwrap()
                .uuid
        );
        let origin = trial.latest_wearer_reduction_origin().unwrap();
        assert!(origin.definition.ends_with("DefenseProgram"), "{origin:?}");
        assert!(!origin.calls.is_empty());
        assert_eq!(w.source_file, trial.identity().artifact_path);
        assert!(w.field_path.ends_with("WearerReduction.percent"));
        let damage: Vec<_> = events
            .values()
            .filter(|e| {
                e["event"]["target_instance_id"] == wearer
                    && e["event"]["delta"].as_i64().is_some_and(|d| d < 0)
            })
            .cloned()
            .collect();
        assert_eq!(damage.len(), 6, "{damage:?}");
        // The independent expected sequence follows the authored HP ratio/cap
        // and public pre-hit HP. True damage ignores the armor's Defense20.
        let mut hp = 200u64;
        for e in &damage {
            assert_eq!(e["event"]["hp_before"], hp);
            assert_eq!(e["event"]["damage_source"], "BasicAttack");
            assert_eq!(e["event"]["damage_type"], "True");
            assert_eq!(e["event"]["raw_damage"], 27);
            let rate = ((200 - hp) * 100 / 200).min(percent);
            let loss = 27 * (100 - rate) / 100;
            assert_eq!(-e["event"]["delta"].as_i64().unwrap(), loss as i64);
            hp -= loss;
            assert_eq!(e["event"]["hp_after"], hp);
        }
        let last = damage.last().unwrap();
        assert_eq!(last["time_ms"].as_u64(), Some(w.time_ms));
        assert_eq!(
            last["event"]["hp_before"].as_u64(),
            Some(u64::from(w.current_hp))
        );
        assert_eq!(w.max_hp, 200);
        let loss = -last["event"]["delta"].as_i64().unwrap();
        expected.push(json!({"percent":percent,"witness":w,"origin":origin,"damage":damage,"last_hp_loss":loss}));
    }
    assert!(
        expected[0]["last_hp_loss"].as_u64() < expected[1]["last_hp_loss"].as_u64(),
        "same-source cap edit must alter actual HP loss: {expected:?}"
    );
    if let Some(directory) = std::env::var_os("WEARER_REDUCTION_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(
            directory.join("trial.json"),
            serde_json::to_vec_pretty(&json!({
            "root":directory.join("content"),"selection":fixture.selection(),"scenario":document}))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
