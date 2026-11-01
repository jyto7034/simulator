fn weapon_stages_world_fixture() -> (ExternalFixture, Value, String) {
    use crate::game::ability::{
        marks::{MarkFilter, MarkRatio},
        weapon_marks::*,
    };
    let (fixture, _, mut document) = state_world_fixture();
    let path = "official/data/skill_fragments/base.ron";
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let fragment = fragments
        .fragments
        .iter_mut()
        .find(|f| f.id.as_str() == "authoring_preview_fragment")
        .unwrap();
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = &mut fragment.effect
    else {
        panic!()
    };
    let mut bloom = forms["base"].clone();
    bloom.bloom_recipe = Some(
        crate::game::data::skill_fragment_data::SkillFragmentRecipe {
            enkephalin: 1,
            materials: vec![],
        },
    );
    forms.insert("bloom".into(), bloom);
    fixture.write(path, ron::ser::to_string(&fragments).unwrap());
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut weapon = equipment
        .items
        .iter()
        .find(|i| i.equipment_type == crate::game::data::equipment_data::EquipmentType::Weapon)
        .unwrap()
        .clone();
    weapon.id = "weapon_stages_fixture_gun".into();
    weapon.uuid = uuid::Uuid::from_u128(0xe550);
    weapon.name = "죽을 나비의 장례 실제 영구 승급 사격 시험 무기".into();
    weapon.triggered_effects.clear();
    weapon.ability_activations.clear();
    weapon.weapon_profile = Some(
        ron::de::from_str(include_str!(
            "../../../../tests/fixtures/skill_composition/butterfly.weapon.ron"
        ))
        .unwrap(),
    );
    let ratio = |numerator| MarkRatio {
        numerator,
        denominator: 1,
    };
    weapon.weapon_profile.as_mut().unwrap().mark_weapon = Some(MarkWeaponDef {
        evolution: WeaponEvolutionDef {
            fragment: "authoring_preview_fragment".into(),
            form: "bloom".into(),
            base: "paired-pistols".into(),
            upgraded: "minigun".into(),
        },
        key: "funeral-butterflies".into(),
        filter: MarkFilter {
            air: true,
            boss: true,
            abnormality: true,
            summon: false,
            temporary: false,
        },
        base: WeaponMarkHit {
            chance_percent: ratio(100),
            amount: 2,
            multiplier: ratio(1),
        },
        upgraded: WeaponMarkHit {
            chance_percent: ratio(100),
            amount: 1,
            multiplier: ratio(1),
        },
        modifiers: vec![],
    });
    equipment.items.push(weapon);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(path, fixture.read(path).replacen("GrantEnkephalin(amount: 100),", "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"weapon_stages_fixture_gun\"),", 1));
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.max_health = 5000;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let path = "official/data/map/facility_templates.ron";
    let mut templates: crate::game::map::FacilityTemplateDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let intro = templates
        .templates
        .iter_mut()
        .find(|t| t.id.as_str() == "standard_intro_a")
        .unwrap();
    intro.slots.push(
        ron::de::from_str(
            r#"(id:"weapon_stage_maintenance",role:Ordinary,allowed_categories:[Maintenance])"#,
        )
        .unwrap(),
    );
    for (from, to) in [
        ("intake_corridor", "weapon_stage_maintenance"),
        ("weapon_stage_maintenance", "supply_annex"),
    ] {
        intro.connections.push(
            ron::de::from_str(&format!(
                r#"(from_slot_id:"{from}",to_slot_id:"{to}",direction:Bidirectional)"#
            ))
            .unwrap(),
        );
    }
    fixture.write(path, ron::ser::to_string(&templates).unwrap());
    let path = "official/data/map/node_assignment_profiles.ron";
    let mut profiles: crate::game::map::NodeAssignmentProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    profiles
        .profiles
        .iter_mut()
        .find(|p| p.id == "standard_floor_1")
        .unwrap()
        .fixed_slot_assignments
        .push(
            ron::de::from_str(
                r#"(slot_id:"weapon_stage_maintenance",kind_id:"maintenance",payload:Maintenance)"#,
            )
            .unwrap(),
        );
    fixture.write(path, ron::ser::to_string(&profiles).unwrap());
    fixture.author_trial_facility();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    fn shift(value: &mut Value, from: u64, amount: u64) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "step" && value.as_u64().is_some_and(|n| n >= from) {
                        *value = json!(value.as_u64().unwrap() + amount);
                    } else {
                        shift(value, from, amount);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    shift(value, from, amount);
                }
            }
            _ => (),
        }
    }
    shift(&mut document, 14, 1);
    document["steps"].as_array_mut().unwrap().insert(14-preparation,json!({
        "kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},
        "bindings":[
            {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}
        ]
    }));
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/weapon_stages.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: weapon_stages_fixture_gun",
            );
    let cancel = CancellationToken::default();
    let mut pre = document.clone();
    pre["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(16 - preparation);
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
    let reward = probe.session().step(13).unwrap().public_result().unwrap();
    let nodes = reward["payload"]["completion"]["payload"]["map"]["nodes"]
        .as_array()
        .unwrap();
    let node = |slot: &str| nodes.iter().find(|n| n["slot_id"] == slot).unwrap()["id"].clone();
    let maintenance = node("weapon_stage_maintenance");
    let supply = node("supply_annex");
    let copy = reward["payload"]["skill_fragment_diffs"][0]["copy_uuid"].clone();
    let weapon = reward["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"].clone();
    assert!(copy.is_string() && weapon.is_string());
    assert_eq!(
        reward["payload"]["inventory_diff"]["added"][0]["Equipment"]["weapon_stage"],
        "Base"
    );
    drop(probe);
    shift(&mut document, 16, 4);
    document["steps"].as_array_mut().unwrap().splice(16-preparation..16-preparation,[
        json!({"kind":"command","command":{"type":"select_map_node","node_id":maintenance}}),
        json!({"kind":"command","command":{"type":"confirm_enter_node"}}),
        json!({"kind":"command","command":{"type":"awaken_skill_fragment_with_weapon","copy_uuid":copy,"form_id":"bloom","weapon_copy_uuid":weapon,"materials":null}}),
        json!({"kind":"command","command":{"type":"complete_node"}}),
    ]);
    document["steps"][20 - preparation] =
        json!({"kind":"command","command":{"type":"select_map_node","node_id":supply}});
    let mut pre = document.clone();
    pre["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(28 - preparation);
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
    let upgraded = probe.session().step(18).unwrap().public_result().unwrap();
    assert_eq!(upgraded["disposition"], "accepted");
    assert_eq!(
        upgraded["payload"]["inventory_diff"]["updated"][0]["Equipment"]["weapon_stage"],
        "Upgraded"
    );
    assert_eq!(
        upgraded["payload"]["inventory_diff"]["updated"][0]["Equipment"]["uuid"],
        weapon
    );
    let entry = probe.session().step(25).unwrap().public_result().unwrap();
    let setup = &entry["payload"]["battle_setup_snapshot"];
    let zones = setup["deployment_zones"].as_array().unwrap();
    let platform = zones.iter().position(|z| z["kind"] == "Platform").unwrap();
    let route = setup["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "defense_main")
        .unwrap();
    let mut chosen = None;
    for (index, cell) in zones[platform]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        for target in route["cells"].as_array().unwrap().iter().rev() {
            let dx = target["x"].as_i64().unwrap() - cell["x"].as_i64().unwrap();
            let dy = target["y"].as_i64().unwrap() - cell["y"].as_i64().unwrap();
            let facing = if (1..=6).contains(&dx.abs()) && dy.abs() <= 1 {
                Some(if dx > 0 { "right" } else { "left" })
            } else if (1..=6).contains(&dy.abs()) && dx.abs() <= 1 {
                Some(if dy > 0 { "down" } else { "up" })
            } else {
                None
            };
            if let Some(facing) = facing {
                chosen = Some((index, facing));
                break;
            }
        }
        if chosen.is_some() {
            break;
        }
    }
    let (cell, facing) = chosen.expect("actual Platform cell must face a reachable route tile");
    let deploy = &mut document["steps"][28 - preparation];
    deploy["command"]["facing"] = json!(facing);
    for binding in deploy["bindings"].as_array_mut().unwrap() {
        if binding["to"] == "/command/position" {
            binding["from"]["pointer"] = json!(format!(
                "/payload/battle_setup_snapshot/deployment_zones/{platform}/cells/{cell}"
            ));
        }
    }
    drop(probe);
    document["steps"][29 - preparation]["clock"]["delta_ms"] = json!(100);
    for _ in 0..200 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    (fixture, document, source)
}

#[test]
fn composition_weapon_stages_world_preflight_actual_bloom_copy_deploy_and_hit() {
    let (fixture, mut document, source) = weapon_stages_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    let cancel = CancellationToken::default();
    let mut expected: Vec<Value> = Vec::new();
    for distance in [100000, 200000] {
        let current = source.replace(
            "push_distance_units: 100000",
            &format!("push_distance_units: {distance}"),
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
                .unwrap_or_else(|e| panic!("weapon stage World step{pending}: {e:?}"))
                .index();
            let observation = trial.session().observation().unwrap();
            if observation
                .active()
                .and_then(|o| o.latest_weapon_stage())
                .is_some_and(|w| w.from_data_units != w.to_data_units && w.actual_hp_loss > 0)
            {
                if distance == 100000 {
                    document["steps"]
                        .as_array_mut()
                        .unwrap()
                        .truncate(index + 1 - preparation);
                    break;
                }
            }
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let shot = observed
            .latest_weapon_stage()
            .expect("actual evolved gun must hit and displace an enemy");
        assert_eq!(
            shot.stage,
            crate::game::ability::weapon_marks::WeaponStage::Upgraded
        );
        assert_eq!(shot.interval_ms, 250);
        assert_eq!(shot.windup_ms, 50);
        assert_eq!(shot.push_distance_units, distance);
        assert_eq!(shot.actual_hp_loss, 12);
        assert_ne!(shot.from_data_units, shot.to_data_units);
        let dx = (shot.to_data_units.0 - shot.from_data_units.0) as f64;
        let dy = (shot.to_data_units.1 - shot.from_data_units.1) as f64;
        assert!((dx.hypot(dy) - f64::from(distance)).abs() <= 2.0);
        if let Some(prior) = expected.first() {
            assert_eq!(
                serde_json::to_value(shot.from_data_units).unwrap(),
                prior["shot"]["from_data_units"]
            );
        }
        let reward = trial.session().step(13).unwrap().public_result().unwrap();
        assert_eq!(
            serde_json::to_value(shot.equipment.persistent_copy_uuid).unwrap(),
            reward["payload"]["inventory_diff"]["added"][0]["Equipment"]["uuid"]
        );
        let origin = trial.latest_weapon_stage_origin().unwrap();
        assert!(origin.definition.contains("AttackProgram"));
        expected.push(json!({"distance":distance,"shot":shot,"origin":origin}));
    }
    if let Some(directory) = std::env::var_os("WEAPON_STAGES_COMPOSITION_FIXTURE_DIR") {
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
