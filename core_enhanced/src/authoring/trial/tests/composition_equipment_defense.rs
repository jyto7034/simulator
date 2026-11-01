use super::*;

#[test]
fn composition_equipment_defense_world_reward_equip_and_hostile_skill() {
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.equipment_type == crate::game::data::equipment_data::EquipmentType::Armor)
        .unwrap()
        .clone();
    accessory.id = "barrier_fixture_shroud".into();
    accessory.uuid = uuid::Uuid::from_u128(0xe370);
    accessory.name = "양막 스킬 차단 연결 시험용".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),",
        "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"barrier_fixture_shroud\"),", 1));
    let path = "official/data/skills/base.ron";
    let mut skills =
        crate::game::data::skill_data::SkillDatabase::from_ron(path, &fixture.read(path)).unwrap();
    let hostile = crate::game::data::skill_data::SkillDatabase::from_ron("barrier-enemy.ron", r#"
      (skills:[(id:"barrier-fixture-enemy", name:"Barrier enemy fixture", kind:Targeted,
        focus_time_ms:0, use_lifetime:Action,
        cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true),
        steps:[(id:"hit",delay_ms:0,target:CastTarget,delivery:Instant,
          effects:[(id:"hit",op:Damage(amount:1,damage_type:True))])])])"#).unwrap();
    skills.skills.extend(hostile.skills);
    fixture.write(path, ron::ser::to_string(&skills).unwrap());
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.skill_id = Some("barrier-fixture-enemy".into());
    // A basic attack release gains ten resonance and schedules a real auto cast.
    enemy.resonance.start = 90;
    enemy.resonance.max = 100;
    // Normal profiles require a preset. Typed loading has expanded it already;
    // serialize its authored reference, without the derived duplicate pattern.
    for profile in &mut enemies.profiles {
        if profile.basic_attack_range_preset.is_some() {
            profile.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
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
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_defense.skill.yaml")
            .replace("equipment: r11_weapon", "equipment: barrier_fixture_shroud");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for recharge in [20000u64, 10000] {
        let current = source.replace("recharge_ms: 20000", &format!("recharge_ms: {recharge}"));
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
        let mut granted_copy = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("barrier World step {pending}: {e:?}"))
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
                assert_eq!(item["definition_id"], "barrier_fixture_shroud");
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
                granted_copy = Some(item["uuid"].clone());
            }
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let barrier = observed
            .latest_skill_barrier()
            .expect("actual hostile skill must reach equipped player");
        assert_eq!(json!(barrier.equipment.persistent_copy_uuid),
            granted_copy.expect("actual reward must be equipped before this effect"));
        assert_eq!(barrier.ready_at_ms - barrier.time_ms, recharge);
        assert_eq!(
            barrier.incoming.cast.as_ref().unwrap().skill_id.as_str(),
            "barrier-fixture-enemy"
        );
        let origin = trial.latest_skill_barrier_origin().unwrap();
        assert!(origin.definition.ends_with("DefenseProgram"), "{origin:?}");
        expected.push(json!({"recharge":recharge,"barrier":barrier,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("EQUIPMENT_DEFENSE_COMPOSITION_FIXTURE_DIR") {
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
