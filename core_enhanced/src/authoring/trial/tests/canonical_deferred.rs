use super::*;
use crate::authoring::composition::canonical_deferred_tests as canonical;
use crate::game::data::content_pack::{package_official_with_skill_files, ContentSelection};

#[test]
fn canonical_queen_published_encounter_staff_fragment_and_two_reactive_heals() {
    published_world("queen-bee");
}
#[test]
fn canonical_rudolta_published_encounter_fragment_three_gifts_and_regular_attack() {
    published_world("rudolta");
}

fn insert_step(document: &mut Value, index: usize, step: Value) {
    fn shift(v: &mut Value, index: usize) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= index as u64) {
                        *v = json!(v.as_u64().unwrap() + 1);
                    } else {
                        shift(v, index);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    shift(v, index);
                }
            }
            _ => (),
        }
    }
    shift(document, index);
    let prep = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .insert(index - prep, step);
}
fn published_world(family: &str) {
    let (fixture, _, mut document) = state_world_fixture();
    let metadata = canonical::catalog(family);
    let fragment_id = metadata.id.as_str().to_owned();
    let skill_id = canonical::resolved(family)["skills"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (encounter, abnormality) = if family == "rudolta" {
        ("suppress_rudolta", "f-02-49_rudolta")
    } else {
        ("suppress_queen_bee", "t-04-50_queen_bee")
    };
    let path = "official/data/skill_fragments/base.ron";
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let current = fragments
        .fragments
        .iter_mut()
        .find(|f| f.id == metadata.id)
        .unwrap();
    assert_eq!(current.uuid, metadata.uuid);
    *current = metadata;
    fixture.write(path, ron::ser::to_string(&fragments).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: crate::game::data::reward_data::RewardDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for reward in &mut rewards.rewards {
        reward.effects.retain(|effect| {
            !matches!(effect,
            crate::game::reward::RewardEffect::GrantSkillFragment { fragment_id }
                if fragment_id.as_str() == "authoring_preview_fragment")
        });
    }
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    let path = "official/data/map/facility_templates.ron";
    let mut maps: crate::game::map::FacilityTemplateDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let opening = maps.layouts[0]
        .activities
        .iter_mut()
        .find(|a| a.id == "trial_intake_corridor")
        .unwrap();
    opening.kind_id = crate::game::map::MapNodeKindId::new("combat_elite");
    opening.payload = crate::game::map::MapNodePayload::Encounter {
        encounter_id: Some(encounter.into()),
    };
    fixture.write(path, ron::ser::to_string(&maps).unwrap());
    let path = "official/data/abnormalities/base.ron";
    let mut abnormalities: crate::game::data::abnormality_data::AbnormalityDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = abnormalities
        .items
        .iter_mut()
        .find(|a| a.id == abnormality)
        .unwrap();
    // Bound the acceptance setup without rewriting encounter identity, waves,
    // terminal policies, or the authoritative encounter-origin reward source.
    enemy.max_health = 70;
    enemy.attack = 0;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
    fixture.write(
        "official/data/bundle.ron",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../game_resources/data/bundle.ron"
        )),
    );

    let prep = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - prep);
    let queen = family == "queen-bee";
    // The two real encounters use different layouts. Face the defended cell
    // and keep the shield beside the route rather than blocking outside the
    // sword's range. The original 68s window left a living enemy at the goal;
    // 98s is a bounded observation window, using two legal clock advances.
    document["steps"][9 - prep]["command"]["facing"] = json!(if queen { "left" } else { "right" });
    document["steps"][11 - prep]["clock"]["delta_ms"] = json!(38000);
    if !queen {
        for binding in document["steps"][10 - prep]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if binding["to"] == "/command/position" {
                binding["from"]["pointer"] =
                    json!("/payload/battle_setup_snapshot/deployment_zones/0/cells/6");
            }
        }
    }
    let ready_index;
    let activate_index;
    if queen {
        insert_step(
            &mut document,
            14,
            json!({"kind":"command","command":{"type":"equip_item","item_uuid":null,"target_unit":null},"bindings":[
                {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
                {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}
            ]}),
        );
        let ground = document["steps"][24 - prep]["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["to"] == "/command/position")
            .unwrap()["from"]["pointer"]
            .clone();
        let facing = document["steps"][24 - prep]["command"]["facing"].clone();
        // Actual queen staff is platform-only. Keep the recipient on the
        // interception cell, outside the staff's ordinary healing footprint.
        for binding in document["steps"][24 - prep]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if binding["to"] == "/command/position" {
                binding["from"]["pointer"] =
                    json!("/payload/battle_setup_snapshot/deployment_zones/1/cells/3");
            }
        }
        let mut recipient = authority_step("deploy_unit", 24);
        recipient["command"]["employee_uuid"] = Value::Null;
        recipient["command"]["position"] = Value::Null;
        recipient["command"]["facing"] = facing;
        recipient["bindings"].as_array_mut().unwrap().extend([
            json!({"from":{"step":6,"pointer":"/payload/battle_update/checkpoint/deployment/unit_deploy_costs/0/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),
            json!({"from":{"step":21,"pointer":ground},"to":"/command/position","value_type":"object"}),
        ]);
        insert_step(&mut document, 25, recipient);
        ready_index = 26;
        let mut begin = authority_step("begin_skill_targeting", ready_index);
        begin["command"]["employee_uuid"] = Value::Null;
        begin["command"]["skill_id"] = json!(skill_id);
        begin["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
        document["steps"].as_array_mut().unwrap().push(begin);
        activate_index = 28;
    } else {
        ready_index = 24;
        activate_index = 25;
    }
    let mut activate = authority_step("activate_skill", activate_index - 1);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!(skill_id);
    activate["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    if queen {
        activate["command"]["target"] = json!({"type":"Unit","unit_instance_id":null});
        activate["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":25,"pointer":"/payload/unit_instance_id"},"to":"/command/target/unit_instance_id","value_type":"string"}));
    }
    document["steps"].as_array_mut().unwrap().push(activate);
    let durations = if queen {
        vec![300; 20]
    } else {
        vec![1, 100, 100, 100, 100, 1000]
    };
    for duration in durations {
        document["steps"].as_array_mut().unwrap().push(
            json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":duration,"after_seq":0}}),
        );
    }
    let source_path = "data/skills/canonical-deferred.skill.yaml";
    fixture.write(
        &format!("official/{source_path}"),
        canonical::original(family),
    );
    let rejected = fixture.root().join("unresolved");
    let error = package_official_with_skill_files(
        &fixture.root().join("official"),
        &rejected,
        &[source_path.into()],
        &Default::default(),
    )
    .unwrap_err();
    assert!(format!("{error:?}").contains("Unresolved"));
    assert!(!rejected.exists());
    let cancel = CancellationToken::default();
    let mut expected = vec![];
    for edit in [false, true] {
        let mut source = canonical::resolved(family);
        if edit {
            canonical::edit_power(&mut source, family);
        }
        let source = serde_json::to_string_pretty(&source).unwrap();
        let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
        fixture.write(&format!("official/{source_path}"), &source);
        let name = if edit {
            "published-edited"
        } else {
            "published-original"
        };
        let receipt = package_official_with_skill_files(
            &fixture.root().join("official"),
            &fixture.root().join(name),
            &[source_path.into()],
            &Default::default(),
        )
        .unwrap();
        let selection = ContentSelection {
            packs: vec![name.into()],
            winners: vec![],
        };
        let content =
            crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
                .unwrap();
        assert_eq!(content.token().identity(), &receipt.content_identity);
        let mut trial =
            TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
                .unwrap();
        let mut heals = vec![];
        let mut seen_heal = None;
        while !trial.is_complete() {
            let pending = trial.next_step_index();
            if let Err(error) = trial.advance(&cancel).map(|_| ()) {
                panic!(
                    "{family} step {pending}: {error:?}; previous={:?}",
                    trial
                        .step(pending.saturating_sub(1))
                        .and_then(|s| s.public_result())
                );
            }
            if pending == 12 {
                assert_eq!(
                    trial.step(12).unwrap().public_result().unwrap()["payload"]["finished"],
                    true,
                    "{family} opening checkpoint: {}",
                    trial.step(12).unwrap().public_result().unwrap()["payload"]["battle_update"]
                        ["checkpoint"]
                );
            }
            if pending == 13 && queen {
                assert_eq!(
                    trial.step(13).unwrap().public_result().unwrap()["payload"]["inventory_diff"]
                        ["added"][0]["Equipment"]["definition_id"],
                    "placeholder_queen_bee_weapon"
                );
            }
            if pending <= activate_index {
                continue;
            }
            if let Some(hp) = trial
                .observation()
                .unwrap()
                .active()
                .and_then(|a| a.latest_nonzero_sourced_hp())
            {
                if hp
                    .authored_source()
                    .is_some_and(|s| s.definition_id() == skill_id)
                    && hp.change().hp_delta() > 0
                    && seen_heal != Some(hp.ordinal())
                {
                    seen_heal = Some(hp.ordinal());
                    heals.push(serde_json::to_value(hp).unwrap());
                }
            }
        }
        let grant = trial.step(13).unwrap().public_result().unwrap()["payload"]
            ["skill_fragment_diffs"][0]
            .clone();
        assert_eq!(grant["fragment_id"], fragment_id);
        assert_eq!(
            trial
                .step(if queen { 15 } else { 14 })
                .unwrap()
                .public_result()
                .unwrap()["disposition"],
            "accepted"
        );
        assert_eq!(
            trial.step(activate_index).unwrap().public_result().unwrap()["disposition"],
            "accepted"
        );
        let before = &trial.step(ready_index).unwrap().public_result().unwrap()["payload"]
            ["battle_update"]["checkpoint"];
        let actor_uuid =
            &trial.step(9).unwrap().public_result().unwrap()["payload"]["employee_uuid"];
        let actor = before["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["unit_source"]["employee_uuid"] == *actor_uuid)
            .unwrap();
        let final_index = trial.next_step_index() - 1;
        let final_cp = &trial.step(final_index).unwrap().public_result().unwrap()["payload"]
            ["battle_update"]["checkpoint"];
        let mut damages = vec![];
        let mut basics = 0;
        for row in trial.step(final_index).unwrap().public_result().unwrap()["payload"]
            ["battle_update"]["events_delta"]["events"]
            .as_array()
            .unwrap()
        {
            let e = &row["event"];
            if row["time_ms"].as_u64().unwrap() < before["battle_time_ms"].as_u64().unwrap()
                || e["type"] != "HpChanged"
                || e["source_instance_id"] != actor["unit_instance_id"]
            {
                continue;
            }
            if e["damage_source"] == "Ability" {
                damages.push(e["raw_damage"].as_u64().unwrap());
            }
            if e["damage_source"] == "BasicAttack" {
                basics += 1;
            }
        }
        let latest = trial
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .latest_nonzero_sourced_hp()
            .unwrap();
        assert_eq!(
            latest.authored_source().unwrap().source_file(),
            format!("{name}/{source_path}")
        );
        assert_eq!(latest.authored_source().unwrap().definition_id(), skill_id);
        assert_eq!(
            serde_json::to_value(latest).unwrap()["fragment"]["copy_uuid"],
            grant["copy_uuid"]
        );
        let mut heal_requested = None;
        if queen {
            assert_eq!(
                heals.len(),
                2,
                "two real hostile actions must consume exactly two bees: {heals:?}"
            );
            let target =
                &trial.step(25).unwrap().public_result().unwrap()["payload"]["unit_instance_id"];
            assert!(heals.iter().all(|h| h["target_unit"] == *target));
            assert!(final_cp["sustained_effects"]["active"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| s["state"]["definition"] != "queen-bee-royal-hive"));
            let facts = trial.observation().unwrap().active_runtime_facts().unwrap();
            let heal = facts
                .actors()
                .filter_map(|(_, row)| row.caused.latest_nonzero_heal.as_ref())
                .find(|h| {
                    h.source
                        .cast
                        .as_ref()
                        .is_some_and(|cast| cast.skill_id.as_str() == skill_id)
                })
                .unwrap();
            let recipient = before["units"]
                .as_array()
                .unwrap()
                .iter()
                .find(|u| u["unit_instance_id"] == *target)
                .unwrap();
            assert_eq!(
                heal.requested_delta,
                (recipient["stats"]["max_health"].as_u64().unwrap() * if edit { 10 } else { 20 }
                    / 100) as i64
            );
            heal_requested = Some(heal.requested_delta);
        } else {
            assert_eq!(damages, vec![if edit { 24 } else { 18 }; 3]);
            assert!(
                basics > 0,
                "regular attacks must continue alongside/after the schedule"
            );
        }
        expected.push(json!({"edit":edit,"selection":selection,"source_digest":artifact.source_digest(),"grant":grant,"damages":damages,"basic_hits":basics,"heals":heals,"heal_requested":heal_requested,"latest":latest,"ready_index":ready_index,"activate_index":activate_index,"final_index":final_index}));
    }
    if let Some(destination) = std::env::var_os("CANONICAL_DEFERRED_FIXTURE_DIR") {
        let destination = std::path::PathBuf::from(destination).join(family);
        assert!(!destination.exists());
        copy_fixture(fixture.root(), &destination.join("content"));
        for (name, value) in [
            ("scenario.json", &document),
            ("expected.json", &json!(expected)),
        ] {
            std::fs::write(
                destination.join(name),
                serde_json::to_vec_pretty(value).unwrap(),
            )
            .unwrap();
        }
        std::fs::write(
            destination.join("original.skill.yaml"),
            canonical::original(family),
        )
        .unwrap();
        std::fs::write(
            destination.join("original.catalog.json"),
            canonical::catalog_original(family),
        )
        .unwrap();
    }
    eprintln!("{family} published World {}", json!(expected));
}
