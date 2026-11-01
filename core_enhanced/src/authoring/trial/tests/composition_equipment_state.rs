use super::*;

fn equipped_state_world_fixture() -> (ExternalFixture, Value) {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.id == "crimson_goggles")
        .unwrap()
        .clone();
    accessory.id = "state_fixture_teardrop".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe410);
    accessory.name = "검은 눈물 피격 중첩 연결 시험용".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"state_fixture_teardrop\"),", 1));
    fixture.author_trial_facility();
    // Preserve the opening victory and actual copy grant. Insert equip before
    // the next encounter; remap every later authority reference together.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    fn shift(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "step" && value.as_u64().is_some_and(|n| n >= 14) {
                        *value = json!(value.as_u64().unwrap() + 1);
                    } else {
                        shift(value);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    shift(value);
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().insert(5, json!({
        "kind":"command", "command":{"type":"equip_item","item_uuid":null,"target_unit":null},
        "bindings":[
            {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}
        ]
    }));
    document["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"clock",
        "clock":{"mode":"elapsed","delta_ms":20000,"after_seq":0}}));
    (fixture, document)
}

#[test]
fn composition_equipment_state_world_reward_equip_and_hostile_damage() {
    let (fixture, document) = equipped_state_world_fixture();
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_state.skill.yaml")
            .replace("equipment: r11_weapon", "equipment: state_fixture_teardrop");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for cap in [21, 42] {
        let current = source.replace(
            "attack_speed_points: 21",
            &format!("attack_speed_points: {cap}"),
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
                .unwrap_or_else(|e| panic!("equipment state World step {pending}: {e:?}"))
                .index();
            if index == 14 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                assert_eq!(result["disposition"], "accepted");
                let granted = trial.session().step(13).unwrap().public_result().unwrap();
                let item = &granted["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "state_fixture_teardrop");
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
            }
        }
        let observation = trial.session().observation().unwrap();
        let terminal = observation
            .last_terminal()
            .expect("actual equipped attacks must finish encounter");
        assert!(observation.active().is_none());
        let latest = terminal
            .mandatory()
            .latest_sourced_sustained()
            .expect("actual hostile damage must trigger the accessory");
        let authored = latest.authored_source().unwrap();
        assert_observed_origin_integrity(&trial, authored);
        let origin = trial.origin_for_observed(authored).unwrap();
        assert!(origin.definition.ends_with("StateProgram"));
        assert!(authored
            .field_path()
            .starts_with("equipment_states[0].program.handlers[0].body"));
        assert!(latest
            .equipment_source()
            .unwrap()
            .persistent_copy_uuid
            .is_some());
        assert!(
            latest.source().is_none(),
            "equipment listener does not invent a cast"
        );
        expected.push(json!({"cap":cap,"actual_time_ms":terminal.actual_time_ms(),
            "verdict":terminal.verdict(),"latest":latest,"origin":origin}));
    }
    eprintln!("equipment state World: {expected:?}");
    assert!(
        expected[1]["actual_time_ms"].as_u64().unwrap()
            < expected[0]["actual_time_ms"].as_u64().unwrap(),
        "stronger maximum stack must shorten real attack cycles: {expected:?}"
    );
    if let Some(directory) = std::env::var_os("EQUIPMENT_STATE_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(
            directory.join("trial.json"),
            serde_json::to_vec_pretty(&json!({
                "root":directory.join("content"),"selection":fixture.selection(),"scenario":document
            }))
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

#[test]
fn composition_predicates_world_shared_condition_edit_changes_real_equipped_outcome() {
    let (fixture, document) = equipped_state_world_fixture();
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/predicate_state.skill.yaml")
            .replace("equipment: r11_weapon", "equipment: state_fixture_teardrop");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for minimum in [0, 1] {
        let current = source.replacen("value: 0", &format!("value: {minimum}"), 1);
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
            trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("predicate World step {pending}: {e:?}"));
        }
        let observation = trial.session().observation().unwrap();
        let terminal = observation
            .last_terminal()
            .expect("actual equipped attacks finish the encounter");
        let latest = terminal
            .mandatory()
            .latest_sourced_sustained()
            .expect("hostile damage triggers the equipped state");
        let origin = trial
            .origin_for_observed(latest.authored_source().unwrap())
            .unwrap();
        assert!(latest
            .equipment_source()
            .unwrap()
            .persistent_copy_uuid
            .is_some());
        assert!(origin.definition.ends_with("StateProgram"));
        expected.push(json!({"minimum":minimum,"actual_time_ms":terminal.actual_time_ms(),"latest":latest,"origin":origin}));
    }
    assert!(expected[0]["actual_time_ms"].as_u64().unwrap() < expected[1]["actual_time_ms"].as_u64().unwrap(),
        "requiring more than one retained stack resets each single stack and removes the higher speed tiers: {expected:?}");
    if let Some(directory) = std::env::var_os("PREDICATE_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(
            directory.join("trial.json"),
            serde_json::to_vec_pretty(&json!({
                "root":directory.join("content"),"selection":fixture.selection(),"scenario":document
            }))
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

#[test]
fn composition_dot_recovery_world_reward_armor_equip_and_actual_dot_heal() {
    use crate::game::{
        battle::buffs::{BuffDatabase, BuffKind, BuffMetadata, BuffReapplyPolicy},
        data::equipment_data::EquipmentType,
        stats::{Effect, TriggerEffectTarget, TriggerType, TriggeredEffect},
    };
    let (fixture, mut document) = equipped_state_world_fixture();
    let steps = document["steps"].as_array_mut().unwrap();
    steps.pop();
    steps.pop();
    steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":10000,"after_seq":0}}));
    // The inherited fixture intentionally uses attack 1 for harmless reactions.
    // Give this isolated enemy real wounds exceeding either recovery amount.
    let enemy_path = "official/data/enemies/corroded_employees.ron";
    let enemy_source = fixture.read(enemy_path);
    let rusher = enemy_source.find("id: \"corroded_rusher\"").unwrap();
    assert!(enemy_source[rusher..].contains("attack: 1,"));
    fixture.write(
        enemy_path,
        format!(
            "{}{}",
            &enemy_source[..rusher],
            enemy_source[rusher..].replacen("attack: 1,", "attack: 21,", 1)
        ),
    );
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let armor = equipment
        .items
        .iter_mut()
        .find(|i| i.id == "state_fixture_teardrop")
        .unwrap();
    armor.id = "dot_recovery_fixture_armor".into();
    armor.name = "자기 지속 피해 고유 대상 비례 회복 방어구 시험용".into();
    armor.equipment_type = EquipmentType::Armor;
    for weapon in equipment
        .items
        .iter_mut()
        .filter(|i| i.equipment_type == EquipmentType::Weapon)
    {
        // Test-only weapon proc supplies a real owner-attributed DOT. Armor has no affliction proc.
        weapon
            .triggered_effects
            .entry(TriggerType::OnAttack)
            .or_default()
            .push(TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                Effect::ApplyBuff {
                    buff_id: "dot_recovery_fixture".into(),
                    duration_ms: 5000,
                },
            ));
    }
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture
            .read(path)
            .replace("state_fixture_teardrop", "dot_recovery_fixture_armor"),
    );
    let path = "official/data/buffs/base.ron";
    let mut buffs: BuffDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    buffs.buffs.push(BuffMetadata {
        name: "dot_recovery_fixture".into(),
        kind: BuffKind::PeriodicDamage {
            damage_per_tick: 1,
            damage_type: crate::game::battle::damage::DamageType::True,
        },
        tick_interval_ms: 1000,
        max_stacks: 1,
        reapply_policy: BuffReapplyPolicy::RefreshDuration,
    });
    fixture.write(path, ron::ser::to_string(&buffs).unwrap());
    let source = include_str!(
        "../../../../tests/fixtures/skill_composition/equipment_dot_recovery.skill.yaml"
    )
    .replace("r11_weapon", "dot_recovery_fixture_armor");
    let cancel = CancellationToken::default();
    let mut expected = vec![];
    for cap in [15, 5] {
        let current = source.replace(
            "value: 15, unit: Magnitude",
            &format!("value: {cap}, unit: Magnitude"),
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
            let index = trial.session().next_step_index();
            trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("DOT recovery World step {index}: {e:?}"));
        }
        let equip = trial.session().step(14).unwrap().public_result().unwrap();
        assert_eq!(equip["disposition"], "accepted");
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let item = &grant["payload"]["inventory_diff"]["added"][0]["Equipment"];
        assert_eq!(item["definition_id"], "dot_recovery_fixture_armor");
        assert_eq!(equip["payload"]["requested_item_uuid"], item["uuid"]);
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let latest = observed
            .latest_nonzero_sourced_hp()
            .expect("armor must cause a real authored heal");
        let authored = latest.authored_source().unwrap();
        assert_eq!(authored.definition_id(), "dot_recovery_fixture_armor");
        assert!(
            latest.source().is_none(),
            "equipment clock does not invent a cast"
        );
        assert_eq!(
            latest
                .equipment_source()
                .unwrap()
                .persistent_copy_uuid
                .unwrap()
                .to_string(),
            item["uuid"].as_str().unwrap()
        );
        assert_eq!(latest.change().hp_delta(), if cap == 15 { 10 } else { 5 });
        let origin = trial.origin_for_observed(authored).unwrap();
        expected.push(json!({"cap":cap,"hp_delta":latest.change().hp_delta(),"latest":latest,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("DOT_RECOVERY_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn composition_dot_kill_world_reward_accessory_equip_and_periodic_kill() {
    use crate::game::{
        battle::buffs::{BuffDatabase, BuffKind, BuffMetadata, BuffReapplyPolicy},
        data::equipment_data::EquipmentType,
        stats::{Effect, TriggerEffectTarget, TriggerType, TriggeredEffect},
    };
    let (fixture, mut document) = equipped_state_world_fixture();
    let steps = document["steps"].as_array_mut().unwrap();
    steps.pop();
    steps.pop();
    steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":10000,"after_seq":0}}));
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let accessory = equipment
        .items
        .iter_mut()
        .find(|i| i.id == "state_fixture_teardrop")
        .unwrap();
    accessory.id = "dot_kill_fixture_accessory".into();
    accessory.name = "자기 지속 피해 대상 처치 공명 액세서리 시험용".into();
    assert_eq!(accessory.equipment_type, EquipmentType::Accessory);
    for weapon in equipment
        .items
        .iter_mut()
        .filter(|i| i.equipment_type == EquipmentType::Weapon)
    {
        weapon
            .triggered_effects
            .entry(TriggerType::OnAttack)
            .or_default()
            .push(TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                Effect::ApplyBuff {
                    buff_id: "dot_kill_fixture".into(),
                    duration_ms: 5000,
                },
            ));
    }
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture
            .read(path)
            .replace("state_fixture_teardrop", "dot_kill_fixture_accessory"),
    );
    let path = "official/data/buffs/base.ron";
    let mut buffs: BuffDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    buffs.buffs.push(BuffMetadata {
        name: "dot_kill_fixture".into(),
        kind: BuffKind::PeriodicDamage {
            damage_per_tick: 500,
            damage_type: crate::game::battle::damage::DamageType::True,
        },
        tick_interval_ms: 1000,
        max_stacks: 1,
        reapply_policy: BuffReapplyPolicy::RefreshDuration,
    });
    fixture.write(path, ron::ser::to_string(&buffs).unwrap());
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_dot_kill.skill.yaml")
            .replace("r11_weapon", "dot_kill_fixture_accessory");
    let cancel = CancellationToken::default();
    let mut expected = vec![];
    for reward in [7, 3] {
        let current = source.replace(
            "value: 7, unit: Magnitude",
            &format!("value: {reward}, unit: Magnitude"),
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
            let index = trial.session().next_step_index();
            trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("DOT kill World step {index}: {e:?}"));
        }
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let item = &grant["payload"]["inventory_diff"]["added"][0]["Equipment"];
        let equip = trial.session().step(14).unwrap().public_result().unwrap();
        assert_eq!(equip["disposition"], "accepted");
        assert_eq!(equip["payload"]["requested_item_uuid"], item["uuid"]);
        assert_eq!(item["definition_id"], "dot_kill_fixture_accessory");
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let latest = observed
            .latest_sourced_resonance()
            .expect("actual authored resonance reward");
        let authored = latest.authored_source().unwrap();
        assert_eq!(authored.definition_id(), "dot_kill_fixture_accessory");
        assert!(
            latest.source().is_none(),
            "equipment reward must not invent a cast"
        );
        assert_eq!(
            latest
                .equipment_source()
                .unwrap()
                .persistent_copy_uuid
                .unwrap()
                .to_string(),
            item["uuid"].as_str().unwrap()
        );
        let change = serde_json::to_value(latest.change()).unwrap();
        assert_eq!(change["channel"], "resonance");
        assert_eq!(
            change["after"].as_u64().unwrap() - change["before"].as_u64().unwrap(),
            reward
        );
        let origin = trial.origin_for_observed(authored).unwrap();
        expected.push(json!({"reward":reward,"latest":latest,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("DOT_KILL_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}

#[path = "composition_plain_shield.rs"]
mod plain_shield;
