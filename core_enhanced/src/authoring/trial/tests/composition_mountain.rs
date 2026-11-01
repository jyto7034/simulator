use super::*;

const RELICS: [&str; 5] = [
    "widened-horizon",
    "long-breath",
    "taunted-enemy-damage-reduction",
    "taunted-enemy-basic-attack-boost",
    "double-harvest",
];

#[test]
fn composition_mountain_world_legal_loadouts_taunt_and_weapon_growth() {
    let (fixture, document) = duration_modifiers::modifier_reward_world_fixture(RELICS[0], false);
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for (index, name) in RELICS.iter().enumerate().skip(1) {
        let mut relic = relics.relics.last().unwrap().clone();
        relic.id = (*name).into();
        relic.uuid = uuid::Uuid::from_u128(0xd290 + index as u128);
        relics.relics.push(relic);
    }
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: crate::game::data::reward_data::RewardDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let reward = rewards
        .rewards
        .iter_mut()
        .find(|r| r.id == "duration_supply")
        .unwrap();
    for name in &RELICS[1..] {
        reward
            .effects
            .push(ron::de::from_str(&format!("GrantRelic(relic_id:\"{name}\")")).unwrap());
    }
    reward.effects.push(
        ron::de::from_str(r#"GrantEquipment(equipment_id:"mountain_fixture_weapon")"#).unwrap(),
    );
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    // Register the real common weapon action through the content loader. Keep
    // fixture equipment identity so the ordinary reward/equip path is unchanged.
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut weapon = equipment
        .items
        .iter()
        .find(|w| w.id == "standard_training_sword")
        .unwrap()
        .clone();
    weapon.id = "mountain_fixture_weapon".into();
    weapon.uuid = uuid::Uuid::from_u128(0xe290);
    weapon.name = "웃음 연결 검사 무기".into();
    let profile = weapon.weapon_profile.as_mut().unwrap();
    profile.weapon_archetype = crate::game::data::equipment_data::WeaponArchetype::Shield;
    profile.block_capacity = 1;
    profile.action = crate::game::data::abnormality_data::BasicAttackAction::DamageArea {
        footprint: crate::game::battle::tile_range::TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
        },
        secondary_percent: 40,
    };
    equipment.items.push(weapon);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    fixture.author_trial_facility();
    let source = include_str!("../../../../tests/fixtures/skill_composition/mountain.skill.yaml")
        .replace("authored-mountain", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace(
            "equipment: r11_weapon",
            "equipment: mountain_fixture_weapon",
        );
    // Inventory DTOs use generated UUID order, not reward declaration order.
    // This encounter never activates the replacement skill: discover addresses
    // through the real reward/query sequence with the same authoring overlay.
    let mut order_scenario = document.clone();
    order_scenario["steps"].as_array_mut().unwrap().truncate(7);
    let cancel = CancellationToken::default();
    // The six-relic publication exceeds the embedded offline authoring budget.
    // Keep that refusal observable and select the same finite host profile as
    // the control-vulnerability World fixture. Native execution stays at 10m;
    // neither content nor a trial request can relax the host setting.
    let refused = CompositionTrial::prepare(
        fixture.root(), &fixture.selection(), &source, checked(&order_scenario),
        None, TrialLimits::default(), &cancel,
    ).err().expect("preserve the original default publication refusal");
    assert!(matches!(refused, crate::authoring::composition::CompositionTrialFailure::Content(
        AuthoringError::Execution { diagnostic: crate::authoring::ExecutionFailure {
            code: "authoring_work_limit", phase: crate::authoring::ExecutionPhase::GeneratedSetup,
            limit: Some(10_000_000), ..
        }})), "{refused:?}");
    use crate::runtime_safety::{RuntimeSafetySettings, EMBEDDED_RUNTIME_SAFETY_RON};
    let settings_text = std::str::from_utf8(EMBEDDED_RUNTIME_SAFETY_RON).unwrap()
        .replace("authoring_execution: (\n        max_work: 10000000",
                 "authoring_execution: (\n        max_work: 20000000");
    let settings = std::sync::Arc::new(RuntimeSafetySettings::from_ron(
        "explicit six-relic World test host", settings_text.as_bytes()).unwrap());
    assert_eq!(settings.authoring_execution().max_work, 20_000_000);
    assert_eq!(settings.native_runtime().max_work, 10_000_000);
    let mut order_trial = CompositionTrial::prepare_with_runtime_safety(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&order_scenario),
        None,
        TrialLimits::default(),
        &cancel,
        settings.clone(),
    )
    .unwrap();
    let initial: Value = serde_json::from_str(order_trial.session().receipt_documents().next().unwrap()).unwrap();
    let validation = &initial["payload"]["validation_usage"];
    let publication_work = validation["work"].as_u64().expect("publication work receipt");
    assert!((10_000_000..=20_000_000).contains(&publication_work));
    assert!((1..=20_000_000).contains(&order_trial.base_validation_usage().work));
    eprintln!("mountain explicit offline publication: base={:?}, validation={}",
        order_trial.base_validation_usage(), validation);
    order_trial.run_to_end(&cancel).unwrap();
    let order = order_trial
        .session()
        .step(15)
        .unwrap()
        .public_result()
        .unwrap()["payload"]["copies"]
        .as_array()
        .unwrap()
        .clone();
    let copy_indices: Vec<_> = RELICS
        .iter()
        .map(|name| {
            order
                .iter()
                .position(|c| c["definition_id"] == *name)
                .unwrap()
        })
        .collect();
    drop(order_trial);
    let mut expected = Vec::new();
    for (mode, equipped) in [("range-duration", [0, 1, 4]), ("damage", [2, 3, 4])] {
        let mut scenario = document.clone();
        // Preserve every original binding; three new real equip commands precede
        // the next encounter and shift its authority references by three.
        fn shift(value: &mut Value) {
            match value {
                Value::Object(map) => {
                    for (key, value) in map {
                        if key == "step" && value.as_u64().is_some_and(|n| n >= 17) {
                            *value = json!(value.as_u64().unwrap() + 3);
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
        shift(&mut scenario);
        let equip = |relic: usize, slot: usize| {
            let index = copy_indices[relic];
            json!({"kind":"command","command":{"type":"equip_relic","copy_uuid":null,"employee_uuid":null,"slot":slot},"bindings":[
            {"from":{"step":15,"pointer":format!("/payload/copies/{index}/copy_uuid")},"to":"/command/copy_uuid","value_type":"string"},
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]})
        };
        scenario["steps"][7] = equip(equipped[0], 2);
        scenario["steps"]
            .as_array_mut()
            .unwrap()
            .splice(8..8, [equip(equipped[1], 1), equip(equipped[2], 0), json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
                {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
                {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}]})]);
        scenario["steps"].as_array_mut().unwrap().push(
            json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":60000,"after_seq":0}}),
        );
        let cancel = CancellationToken::default();
        let mut trial = CompositionTrial::prepare_with_runtime_safety(
            fixture.root(),
            &fixture.selection(),
            &source,
            checked(&scenario),
            None,
            TrialLimits::default(),
            &cancel,
            settings.clone(),
        )
        .unwrap();
        let mut prior = None;
        let mut active = Vec::new();
        let mut owner = None;
        let mut copy = None;
        let mut observations = Vec::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("mountain {mode} step {pending}: {e:?}"))
                .index();
            let result = trial
                .session()
                .step(index)
                .unwrap()
                .public_result()
                .unwrap();
            if index == 9 {
                owner = Some(result["payload"]["employee_uuid"].clone());
            }
            if index == 18 {
                let payload = &result["payload"];
                assert_eq!(payload["copies"].as_array().unwrap().len(), 5);
                let loadout = &payload["loadouts"][0];
                assert_eq!(loadout["employee_uuid"], owner.clone().unwrap());
                assert_eq!(loadout["slots"].as_array().unwrap().len(), 3);
                for selected in equipped {
                    let selected_copy = &payload["copies"][copy_indices[selected]];
                    assert!(loadout["slots"]
                        .as_array()
                        .unwrap()
                        .contains(&selected_copy["copy_uuid"]));
                    assert_eq!(selected_copy["definition_id"], RELICS[selected]);
                }
            }
            if index == 19 {
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(result["payload"]["target_unit"], owner.clone().unwrap());
                let granted = trial.session().step(13).unwrap().public_result().unwrap();
                let item = &granted["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(item["definition_id"], "mountain_fixture_weapon");
                assert_eq!(item["weapon_profile"]["weapon_archetype"], "Shield");
                assert_eq!(
                    item["weapon_profile"]["action"]["DamageArea"]["secondary_percent"],
                    40
                );
                copy = Some(uuid::Uuid::parse_str(item["uuid"].as_str().unwrap()).unwrap());
                assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
            }
            let observation = trial.session().observation().unwrap();
            if index == 29 {
                prior = observation.run_resources().cloned();
            }
            if index < 32 {
                continue;
            }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            if let Some(effects) = cp["sustained_effects"]["active"].as_array() {
                for effect in effects {
                    let Some(taunt) = effect["control"].get("ApproachTaunt") else {
                        continue;
                    };
                    assert_eq!(
                        effect["expires_at_ms"],
                        if mode == "range-duration" {
                            25503
                        } else {
                            25002
                        }
                    );
                    assert_eq!(
                        taunt["profile"]["damage_reduction_percent"]
                            .as_u64()
                            .unwrap_or(0),
                        if mode == "damage" { 20 } else { 0 }
                    );
                    assert_eq!(
                        taunt["profile"]["basic_damage_bonus_percent"]
                            .as_u64()
                            .unwrap_or(0),
                        if mode == "damage" { 30 } else { 0 }
                    );
                    assert_eq!(taunt["profile"]["attack_range_units"], 1000000);
                    active.push(json!({"step":index,"effect":effect}));
                }
            }
            if let Some(latest) = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
                .and_then(|o| o.latest())
            {
                observations.push(json!({"step":index,"latest":latest}));
            }
        }
        assert!(
            !active.is_empty(),
            "{mode}: must execute the authored taunt"
        );
        let observation = trial.session().observation().unwrap();
        assert!(
            observation.last_terminal().is_some() && observation.active().is_none(),
            "{mode}: actual weapon must finish encounter"
        );
        let run = observation.run_resources().unwrap();
        let before = prior.unwrap();
        let entries: Vec<_> = run
            .entries
            .iter()
            .filter(|e| e.resource.as_str() == "weapon_mass")
            .collect();
        let changed: Vec<_> = entries
            .iter()
            .filter_map(|entry| {
                let previous = before
                    .entries
                    .iter()
                    .find(|e| e.resource == entry.resource && e.owner == entry.owner)
                    .map_or(0, |e| e.balance);
                (entry.balance != previous).then_some((entry, previous))
            })
            .collect();
        assert_eq!(
            changed.len(),
            1,
            "only the actual killing weapon copy grows: {run:?}"
        );
        assert_eq!(
            changed[0].0.balance,
            changed[0].1 + 4,
            "ordinary primary kill adds exactly twice the authored 2"
        );
        assert!(matches!(
            changed[0].0.owner,
            crate::game::resource::ResourceOwner::EquipmentCopy(_)
        ));
        assert_eq!(
            changed[0].0.owner,
            crate::game::resource::ResourceOwner::EquipmentCopy(copy.unwrap())
        );
        expected.push(json!({"mode":mode,"equipped":equipped,"owner":owner,"observed_copy":copy,"active":active,"before":before,"run":run,"observations":observations}));
        if let Some(directory) = std::env::var_os("MOUNTAIN_COMPOSITION_FIXTURE_DIR") {
            let directory = std::path::PathBuf::from(directory).join(mode);
            assert!(!directory.exists(), "preserve earlier evidence");
            copy_fixture(fixture.root(), &directory.join("content"));
            std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
            std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":scenario})).unwrap()).unwrap();
            std::fs::write(
                directory.join("expected.json"),
                serde_json::to_vec_pretty(expected.last().unwrap()).unwrap(),
            )
            .unwrap();
        }
    }
}
