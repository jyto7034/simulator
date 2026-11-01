use super::*;

#[test]
fn composition_hp_guard_world_actual_armor_reward_equip_block_and_origin() {
    let (fixture, _, mut document) = state_world_fixture();
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    assert!(equipment.get_by_id("snow_queen_armor").is_some());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"snow_queen_armor\"),",
            1,
        ),
    );
    // Only the second encounter uses this isolated ordinary enemy. The opening
    // reward combat and the actual armor's stats are unchanged.
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut striker = enemies
        .profiles
        .iter()
        .find(|p| p.id == "corroded_rusher")
        .unwrap()
        .clone();
    striker.id = "hp_guard_striker".into();
    striker.uuid = uuid::Uuid::from_u128(0x9182);
    striker.name = "Protection trial striker".into();
    striker.max_health = 1000;
    // The actual armor gives220HP/18DEF. Tier-scaled80 attack leaves a
    // surviving wound below30%;100 skips from68HP (>66) straight to death.
    striker.attack = 80;
    striker.defense = 0;
    striker.magic_resist = 0;
    striker.basic_attack.interval_ms = 1600;
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
            profile_id: "hp_guard_striker".into(),
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
    // Observe expiry separately from the next enemy hit200ms later.
    for _ in 0..160 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/hp_guard.skill.yaml")
        .to_string();
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    let mut short = None;
    for duration in [3000, 1500] {
        let current = source.replace(
            "value: 3000, unit: Milliseconds",
            &format!("value: {duration}, unit: Milliseconds"),
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
        let mut started = None;
        let mut expired = false;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("hp guard World step{pending}: {e:?}"))
                .index();
            if index == 14 {
                let equip = trial.session().step(14).unwrap().public_result().unwrap();
                assert_eq!(equip["disposition"], "accepted");
                let grant = trial.session().step(13).unwrap().public_result().unwrap();
                assert_eq!(
                    grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["definition_id"],
                    "snow_queen_armor"
                );
                assert_eq!(
                    equip["payload"]["requested_item_uuid"],
                    grant["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
                );
            }
            if index < 25 {
                continue;
            }
            let observation = trial.session().observation().unwrap();
            if observation.active().is_none() && observation.last_terminal().is_some() {
                break;
            }
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            let Some(latest) = observed.and_then(|o| o.latest_sourced_sustained()) else {
                continue;
            };
            let crate::authoring::ApplicationChange::Sustained {
                ended,
                expires_at_ms,
                target_state: Some(properties),
                ..
            } = latest.change()
            else {
                continue;
            };
            if !properties.damage_invulnerable {
                continue;
            }
            if started.is_none() && ended.is_none() {
                assert!(properties.basic_attack_locked && properties.active_skill_locked);
                assert_eq!(properties.received_heal_percent, 50);
                assert_eq!(expires_at_ms - latest.time_ms(), duration);
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                let cp = &result["payload"]["battle_update"]["checkpoint"];
                let target = latest.target_unit().to_string();
                let unit = cp["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|u| u["unit_instance_id"] == target)
                    .unwrap();
                let hp = unit["stats"]["current_health"].as_u64().unwrap();
                let max = unit["stats"]["max_health"].as_u64().unwrap();
                assert!(hp > 0 && hp * 100 <= max * 30, "{unit}");
                assert!(
                    !unit["basic_action"]["blocking"]
                        .as_array()
                        .unwrap()
                        .is_empty(),
                    "armor must retain real blocking: {unit}"
                );
                let authored = latest.authored_source().unwrap();
                let origin = trial.origin_for_observed(authored).unwrap();
                assert!(origin.definition.ends_with("StateProgram"));
                assert!(latest
                    .equipment_source()
                    .unwrap()
                    .persistent_copy_uuid
                    .is_some());
                assert!(latest.source().is_none());
                let value = json!({"duration":duration,"time_ms":latest.time_ms(),"expires_at_ms":expires_at_ms,"target_unit":target,"target_state":properties,
                    "equipment":latest.equipment_source(),"origin":origin,"hp":hp,"max_hp":max,"blocking":unit["basic_action"]["blocking"],"step":index});
                expected.push(value);
                started = Some(expires_at_ms);
                if short.is_none() {
                    let mut doc = document.clone();
                    doc["steps"]
                        .as_array_mut()
                        .unwrap()
                        .truncate(index + 1 - preparation);
                    short = Some(doc);
                }
            }
            if started.is_some()
                && ended == Some(crate::game::ability::sustained::SustainedEndReason::Expired)
            {
                expired = true;
                break;
            }
        }
        assert!(
            started.is_some() && expired,
            "actual armor must start and expire: {expected:?}; final={:?}",
            trial
                .session()
                .step(trial.session().next_step_index().saturating_sub(1))
                .and_then(|s| s.public_result())
                .map(|v| v["payload"]["battle_update"]["checkpoint"].clone())
        );
    }
    assert_eq!(expected[0]["time_ms"], expected[1]["time_ms"]);
    assert_eq!(expected[0]["hp"], expected[1]["hp"]);
    if let Some(directory) = std::env::var_os("HP_GUARD_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":short.unwrap()})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
