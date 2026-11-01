use super::*;
use crate::authoring::composition::canonical_hatred_tests as canonical;
use crate::game::data::content_pack::{package_official_with_skill_files, ContentSelection};

#[test]
fn canonical_hatred_published_boss_grant_beam_manual_and_automatic() {
    published_world(false);
}
#[test]
fn canonical_hatred_published_bloom_awakening_beam_manual_and_automatic() {
    published_world(true);
}

use super::canonical_boss::{clock_step, insert, push};
fn cast_steps(document: &mut Value, skill: &str, ready: usize, deploy: usize) -> usize {
    let mut begin = authority_step("begin_skill_targeting", ready);
    begin["command"]["skill_id"] = json!(skill);
    begin["command"]["employee_uuid"] = Value::Null;
    begin["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut cast = begin.clone();
    let begin_index = push(document, begin);
    cast["command"]["type"] = json!("activate_skill");
    cast["command"]["target"] = json!({"type":"Tile","position":null});
    for b in cast["bindings"].as_array_mut().unwrap() {
        if b["to"].as_str().unwrap().starts_with("/context/") {
            b["from"]["step"] = json!(begin_index);
        }
    }
    let prep = document["preparation_steps"].as_array().unwrap().len();
    let mut position = document["steps"][deploy - prep]["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["to"] == "/command/position")
        .unwrap()
        .clone();
    position["to"] = json!("/command/target/position");
    cast["bindings"].as_array_mut().unwrap().push(position);
    push(document, cast)
}
fn published_world(bloom: bool) {
    let (fixture, mut document) = super::canonical_boss::boss_fixture(
        bloom,
        "suppress_queen_of_hatred",
        "o-01-04_queen_of_hatred",
        canonical::catalog(),
        canonical::relics(),
    );
    // Keep the later observation encounter alive for the unchanged 15s gain.
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let original = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    original.max_health = 70;
    let mut observation_enemy = original.clone();
    observation_enemy.id = "hatred_observation_rusher".into();
    observation_enemy.uuid = uuid::Uuid::from_u128(0x9194);
    observation_enemy.max_health = 30000;
    enemies.profiles.push(observation_enemy);
    for profile in &mut enemies.profiles {
        if profile.basic_attack_range_preset.is_some() {
            profile.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let path = "official/data/pve/encounters.ron";
    let mut encounters: crate::game::data::pve_data::PveEncounterDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let later = encounters
        .encounters
        .iter_mut()
        .find(|e| e.id == "normal_breach_response")
        .unwrap();
    assert_eq!(later.waves.len(), 1);
    later.waves[0].source = crate::game::data::pve_data::PveWaveSource::Manual(vec![
        crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
            profile_id: "hatred_observation_rusher".into(),
            tier: crate::game::enums::Tier::I,
            count: 2,
        },
    ]);
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    let sources: Vec<std::path::PathBuf> = [
        "data/skills/hatred-base.skill.yaml",
        "data/skills/hatred-bloom.skill.yaml",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    for (form, path) in [false, true].into_iter().zip(&sources) {
        fixture.write(
            &format!("official/{}", path.display()),
            canonical::original(form),
        );
    }
    let rejected = fixture.root().join("unresolved");
    let error = package_official_with_skill_files(
        &fixture.root().join("official"),
        &rejected,
        &sources,
        &Default::default(),
    )
    .unwrap_err();
    assert!(format!("{error:?}").contains("Unresolved"));
    assert!(!rejected.exists());
    for (form, path) in [false, true].into_iter().zip(&sources) {
        let mut source = canonical::resolved(form);
        for skill in source["skills"].as_array_mut().unwrap() {
            skill["admission"]["capacity"] = json!(5);
        }
        for value in source["values"].as_array_mut().unwrap() {
            for entry in value["value"]["Effects"][0]["op"]["CounterChannel"]["program"]["entries"]
                .as_array_mut()
                .unwrap()
            {
                entry["admission"]["capacity"] = json!(5);
            }
        }
        fixture.write(
            &format!("official/{}", path.display()),
            serde_json::to_string_pretty(&source).unwrap(),
        );
    }
    let receipt = package_official_with_skill_files(
        &fixture.root().join("official"),
        &fixture.root().join("published"),
        &sources,
        &Default::default(),
    )
    .unwrap();
    let selection = ContentSelection {
        packs: vec!["published".into()],
        winners: vec![],
    };
    let (ground, scout_cell, scout) =
        super::canonical_boss::prepare_deployments(&fixture, &selection, &mut document);
    let prep = document["preparation_steps"].as_array().unwrap().len();
    document["steps"][11 - prep]["clock"]["delta_ms"] = json!(10000);
    document["steps"][12 - prep]["clock"]["delta_ms"] = json!(58000);
    let mut deploy = authority_step("deploy_unit", 11);
    deploy["command"]["employee_uuid"] = Value::Null;
    deploy["command"]["position"] = Value::Null;
    deploy["command"]["facing"] = json!("up");
    deploy["bindings"].as_array_mut().unwrap().extend([
        json!({"from":{"step":6,"pointer":format!("/payload/battle_update/checkpoint/deployment/unit_deploy_costs/{scout}/employee_uuid")},"to":"/command/employee_uuid","value_type":"string"}),
        json!({"from":{"step":5,"pointer":format!("/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{scout_cell}")},"to":"/command/position","value_type":"object"})]);
    insert(&mut document, 12, deploy);
    if bloom {
        let binding = json!({"from":{"step":14,"pointer":"/payload/skill_fragment_diffs/0/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"});
        document["steps"][18 - prep] = json!({"kind":"command","command":{"type":"awaken_skill_fragment","copy_uuid":null,"form_id":"bloom"},"bindings":[binding.clone()]});
        document["steps"][19 - prep] = json!({"kind":"command","command":{"type":"select_skill_fragment_form","copy_uuid":null,"form_id":"bloom"},"bindings":[binding]});
        insert(
            &mut document,
            20,
            json!({"kind":"command","command":{"type":"complete_node"}}),
        );
        document["steps"][21 - prep]["bindings"][0]["from"]["step"] = json!(20);
    }
    let offset = usize::from(bloom);
    let root = if bloom {
        "fragment_hatred_love_bloom"
    } else {
        "fragment_hatred_love_base"
    };
    let manual = format!("{root}__entry_{}", if bloom { 0 } else { 1 });
    let beam_cast = cast_steps(&mut document, root, 25 + offset, 24 + offset);
    let beam_done = clock_step(&mut document, 2001);
    let manual_ready = clock_step(&mut document, 15000);
    let manual_cast = cast_steps(&mut document, &manual, manual_ready, 24 + offset);
    let manual_done = clock_step(&mut document, if bloom { 5001 } else { 2001 });
    clock_step(&mut document, 45000);
    let auto_done = clock_step(&mut document, 50000);
    let cancel = CancellationToken::default();
    let content =
        crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
            .unwrap();
    assert_eq!(content.token().identity(), &receipt.content_identity);
    let mut trial =
        TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
            .unwrap();
    let mut observations = vec![];
    while !trial.is_complete() {
        let pending = trial.next_step_index();
        trial.advance(&cancel).map(|_| ()).unwrap_or_else(|e| {
            panic!(
                "hatred bloom={bloom} step {pending}: {e:?}; previous={:?}",
                trial
                    .step(pending.saturating_sub(1))
                    .and_then(|s| s.public_result())
            )
        });
        if pending == 13 {
            assert_eq!(
                trial.step(13).unwrap().public_result().unwrap()["payload"]["finished"],
                true,
                "{}",
                trial.step(13).unwrap().public_result().unwrap()["payload"]["battle_update"]
                    ["checkpoint"]
            );
        }
        if pending >= beam_cast {
            if let Some(hp) = trial
                .observation()
                .unwrap()
                .active()
                .and_then(|o| o.latest_nonzero_sourced_hp())
            {
                observations.push(json!({"step":pending,"hp":hp}));
            }
        }
    }
    let result = |i| trial.step(i).unwrap().public_result().unwrap();
    let owner =
        |i| &result(i)["payload"]["battle_update"]["checkpoint"]["counter_channels"]["owners"][0];
    let grant = &result(14)["payload"]["skill_fragment_diffs"][0];
    assert_eq!(grant["fragment_id"], "fragment_queen_of_hatred_trace");
    assert_eq!(result(15)["disposition"], "accepted");
    assert_eq!(owner(25 + offset)["source_copy"], grant["copy_uuid"]);
    for i in [beam_cast, manual_cast] {
        assert_eq!(result(i)["disposition"], "accepted");
    }
    assert!(owner(beam_done)["active"].is_null());
    assert!(
        owner(beam_done)["count"].as_u64().unwrap() >= 2,
        "{}",
        owner(beam_done)
    );
    assert!((3..=5).contains(&owner(manual_ready)["count"].as_u64().unwrap()));
    assert_eq!(owner(manual_done)["count"], 0);
    assert!(owner(manual_done)["active"].is_null());
    assert_eq!(owner(auto_done)["count"], 0, "{}", owner(auto_done));
    assert!(owner(auto_done)["active"].is_null(), "{}", owner(auto_done));
    let latest = trial
        .observation()
        .unwrap()
        .active()
        .unwrap()
        .latest_nonzero_sourced_hp()
        .unwrap();
    let latest = serde_json::to_value(latest).unwrap();
    assert_eq!(latest["fragment"]["copy_uuid"], grant["copy_uuid"]);
    assert_eq!(latest["source"]["skill_id"], format!("{root}__entry_2"));
    assert_eq!(
        latest["source"]["authored"]["source_file"],
        format!(
            "published/data/skills/hatred-{}.skill.yaml",
            if bloom { "bloom" } else { "base" }
        )
    );
    assert!(
        latest["change"]["after"].as_i64().unwrap() < latest["change"]["before"].as_i64().unwrap()
    );
    // HP change reason describes its application path; damage_source carries
    // the gameplay damage class. Match the actual automatic hit, not that enum.
    assert!(
        result(auto_done)["payload"]["battle_update"]["events_delta"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["time_ms"] == latest["time_ms"]
                && row["event"]["type"] == "HpChanged"
                && row["event"]["source_instance_id"] == latest["source_unit"]
                && row["event"]["target_instance_id"] == latest["target_unit"]
                && row["event"]["damage_source"] == "Ability"
                && row["event"]["hp_after"] == latest["change"]["after"])
    );
    for (step, skill) in [(beam_done, root), (manual_done, manual.as_str())] {
        let hp = &observations.iter().find(|row| row["step"] == step).unwrap()["hp"];
        assert_eq!(hp["source"]["skill_id"], skill);
        assert_eq!(hp["fragment"]["copy_uuid"], grant["copy_uuid"]);
        assert_eq!(
            hp["source"]["authored"]["source_file"],
            latest["source"]["authored"]["source_file"]
        );
    }

    assert_eq!(
        latest["fragment"]["form_id"],
        if bloom { "bloom" } else { "base" }
    );
    if bloom {
        assert_eq!(
            result(18)["payload"]["copy"]["copy_uuid"],
            grant["copy_uuid"]
        );
        assert_eq!(result(18)["payload"]["enkephalin_spent"], 1);
        assert_eq!(result(18)["payload"]["copy"]["selected_form"], "base");
        assert_eq!(result(19)["payload"]["copy"]["selected_form"], "bloom");
    }
    eprintln!(
        "hatred World bloom={bloom} observations={}",
        json!(observations)
    );
    if let Some(root) = std::env::var_os("HATRED_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root).join(if bloom { "bloom" } else { "base" });
        assert!(!root.exists());
        std::fs::create_dir_all(&root).unwrap();
        copy_fixture(fixture.root(), &root.join("content"));
        std::fs::write(
            root.join("scenario.json"),
            serde_json::to_vec_pretty(&document).unwrap(),
        )
        .unwrap();
        std::fs::write(root.join("expected.json"),serde_json::to_vec_pretty(&json!({"grant":grant,"observations":observations,"owner":owner(auto_done),"latest":latest})).unwrap()).unwrap();
    }
}
