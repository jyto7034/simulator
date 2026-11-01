use super::*;

#[test]
fn composition_plain_shield_world_catalog_reward_equip_block_and_basic_attack() {
    let (fixture, mut document) = equipped_state_world_fixture();
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let weapon = equipment
        .items
        .iter()
        .find(|w| w.id == "placeholder_melting_love_weapon")
        .unwrap();
    let profile = serde_json::to_value(weapon.weapon_profile.as_ref().unwrap()).unwrap();
    assert_eq!(profile["range_role"], "Melee");
    assert_eq!(profile["weapon_archetype"], "Shield");
    assert_eq!(profile["deployment_affinity"], "ground_only");
    assert_eq!(profile["block_capacity"], 1);
    assert_eq!(profile["damage_type"], "Physical");
    assert_eq!(profile["delivery"], "Instant");
    assert_eq!(profile["air_capable"], false);
    assert!(profile["platform_rest"].is_null());
    assert!(weapon.triggered_effects.is_empty() && weapon.ability_activations.is_empty());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture
            .read(path)
            .replace("state_fixture_teardrop", "placeholder_melting_love_weapon"),
    );
    fixture.author_trial_facility();
    // Keep the real reward, equip, route placement and first second of combat.
    // Observe ordinary play in bounded one-second steps, without injecting state.
    document["steps"].as_array_mut().unwrap().pop();
    for _ in 0..12 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock",
            "clock":{"mode":"elapsed","delta_ms":1000,"after_seq":0}}));
    }
    let cancel = CancellationToken::default();
    // Ordinary equipment has no executable composition module. Load the actual
    // catalog directly instead of inventing a skill merely to enter this trial.
    let content = crate::authoring::validate(
        fixture.root(),
        &fixture.selection(),
        &crate::authoring::AuthoringLimits::default(),
        &cancel,
    )
    .unwrap();
    let mut trial = crate::authoring::TrialSession::prepare(
        content,
        checked(&document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let mut owner = Value::Null;
    let mut blocked = Value::Null;
    let mut damage = Value::Null;
    let mut equipped = false;
    fn hp_from(value: &Value, owner: &Value) -> Option<Value> {
        match value {
            Value::Object(map) => {
                if map.get("type").is_some_and(|v| v == "HpChanged")
                    && map.get("source_instance_id") == Some(owner)
                    && map
                        .get("delta")
                        .and_then(Value::as_i64)
                        .is_some_and(|d| d < 0)
                    && map.get("damage_source").is_some_and(|v| v == "BasicAttack")
                {
                    return Some(value.clone());
                }
                map.values().find_map(|v| hp_from(v, owner))
            }
            Value::Array(values) => values.iter().find_map(|v| hp_from(v, owner)),
            _ => None,
        }
    }
    while !trial.is_complete() {
        let pending = trial.next_step_index();
        trial
            .advance(&cancel)
            .unwrap_or_else(|e| panic!("plain shield World step {pending}: {e:?}"));
        let Some(result) = trial.step(pending).unwrap().public_result() else {
            continue;
        };
        if pending == 14 {
            assert_eq!(result["disposition"], "accepted");
            let granted = trial.step(13).unwrap().public_result().unwrap();
            let item = &granted["payload"]["inventory_diff"]["added"][0]["Equipment"];
            assert_eq!(item["definition_id"], "placeholder_melting_love_weapon");
            assert_eq!(result["payload"]["requested_item_uuid"], item["uuid"]);
            equipped = true;
        }
        if pending < 24 {
            continue;
        }
        if let Some(units) = result["payload"]["battle_update"]["checkpoint"]["units"].as_array() {
            if let Some(player) = units
                .iter()
                .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
            {
                owner = player["unit_instance_id"].clone();
                assert_eq!(player["tactical_state"]["base_block_capacity"], 1);
                assert_eq!(player["tactical_state"]["effective_block_capacity"], 1);
                assert_eq!(player["tactical_state"]["tile_kind"], "ground");
                if let Some(enemy) = player["basic_action"]["blocking"]
                    .as_array()
                    .and_then(|v| v.first())
                {
                    // A committed windup has no new-action preview. Check the
                    // priority when selectable and the actual damage below.
                    if player["basic_action"]["preview_available"] == true {
                        assert_eq!(player["basic_action"]["selected_target_id"], *enemy);
                    }
                    let target = units
                        .iter()
                        .find(|u| u["unit_instance_id"] == *enemy)
                        .unwrap();
                    assert_eq!(target["basic_action"]["blocked_by"], owner);
                    blocked = json!({"owner":player,"enemy":target});
                }
            }
        }
        if !owner.is_null() {
            if let Some(hit) = hp_from(&result, &owner) {
                assert_eq!(hit["damage_type"], "Physical");
                damage = hit;
            }
        }
    }
    assert!(equipped);
    assert!(
        !blocked.is_null(),
        "catalog shield must establish a real block and select that enemy"
    );
    assert!(
        !damage.is_null(),
        "catalog shield must deal ordinary physical basic-attack damage"
    );
    assert_eq!(
        damage["target_instance_id"],
        blocked["enemy"]["unit_instance_id"]
    );
    if let Some(path) = std::env::var_os("PLAIN_SHIELD_WORLD_EVIDENCE") {
        let path = std::path::Path::new(&path);
        assert!(!path.exists(), "do not overwrite prior evidence");
        std::fs::write(
            path,
            serde_json::to_vec_pretty(
                &json!({"profile":profile,"blocked":blocked,"damage":damage}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
