use super::canonical_boss::{clock_step, insert, push};
use super::*;
use crate::authoring::composition::canonical_input_plan_tests as canonical;
use crate::game::data::content_pack::{package_official_with_skill_files, ContentSelection};

#[test]
fn canonical_input_plan_published_boss_reward_and_volley() {
    let (fixture, mut document) = super::canonical_boss::boss_fixture(
        false,
        "suppress_little_red",
        "f-01-57_little_red",
        canonical::catalog(),
        canonical::relics(),
    );
    // Preserve encounter class, waves, enemy identities and actual reward source.
    // Lower only escort stats for bounded acceptance preparation.
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for enemy in &mut enemies.profiles {
        if enemy.basic_attack_range_preset.is_some() {
            enemy.basic_attack.defense_tile_range = None;
        }
        if ["corroded_marksman", "corroded_medic", "corroded_rusher"].contains(&enemy.id.as_str()) {
            enemy.max_health = 70;
            enemy.attack = 0;
            enemy.defense = 0;
            enemy.magic_resist = 0;
        }
    }
    let mut observation_enemy = enemies
        .profiles
        .iter()
        .find(|e| e.id == "corroded_rusher")
        .unwrap()
        .clone();
    observation_enemy.id = "input_plan_observation_rusher".into();
    observation_enemy.uuid = uuid::Uuid::from_u128(0x9196);
    observation_enemy.max_health = 3000;
    observation_enemy.attack = 1;
    enemies.profiles.push(observation_enemy);
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
            profile_id: "input_plan_observation_rusher".into(),
            tier: crate::game::enums::Tier::I,
            count: 2,
        },
    ]);
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    let sources = vec![std::path::PathBuf::from(
        "data/skills/little-red.skill.yaml",
    )];
    fixture.write(
        "official/data/skills/little-red.skill.yaml",
        canonical::ORIGINAL,
    );
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
    let mut source = canonical::resolved();
    source["skills"][0]["admission"]["capacity"] = json!(5);
    canonical::program(&mut source)["admission"]["capacity"] = json!(5);
    fixture.write(
        "official/data/skills/little-red.skill.yaml",
        serde_json::to_string_pretty(&source).unwrap(),
    );
    // Explicit bounded World acceptance values, separate from the draft.
    let p = canonical::program(&mut source);
    p["normal"]["capacity"] = json!(1);
    p["enhanced"]["capacity"] = json!(1);
    p["cohort"]["interval_ms"] = json!(14000);
    fixture.write(
        "official/data/skills/little-red.skill.yaml",
        serde_json::to_string_pretty(&source).unwrap(),
    );
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
    document["steps"][10 - prep]["command"]["facing"] = json!("left");
    let mut prepare = authority_step("activate_skill", 25);
    prepare["command"]["skill_id"] = json!("fragment_little_red_volley");
    prepare["command"]["employee_uuid"] = Value::Null;
    prepare["command"]["target"] = Value::Null;
    prepare["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut prepare_index = push(&mut document, prepare);
    let mut commit = authority_step("commit_input_plan", prepare_index);
    commit["command"]["employee_uuid"] = Value::Null;
    commit["command"]["battle_uuid"] = Value::Null;
    commit["command"]["revision"] = Value::Null;
    commit["command"]["input"] = json!({"preparation":null,"request":116,"layout":{"Fresh":[{"kind":"Normal","tile":null}]},"aim":{"Precise":[{"x":1,"y":0}]}});
    for (pointer, to, kind) in [
        ("/payload/battle_uuid", "/command/battle_uuid", "string"),
        (
            "/payload/battle_update/battle_state_revision",
            "/command/revision",
            "number",
        ),
        (
            "/payload/battle_update/checkpoint/input_plans/preparation/id",
            "/command/input/preparation",
            "number",
        ),
    ] {
        commit["bindings"].as_array_mut().unwrap().push(
            json!({"from":{"step":prepare_index,"pointer":pointer},"to":to,"value_type":kind}),
        );
    }
    commit["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut position = document["steps"][24 - prep]["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["to"] == "/command/position")
        .unwrap()
        .clone();
    position["to"] = json!("/command/input/layout/Fresh/0/tile");
    commit["bindings"].as_array_mut().unwrap().push(position);
    let mut committed = push(&mut document, commit);
    let mut settled = clock_step(&mut document, 3001);
    // The boss/escort route finishes at 59100ms in this fixed acceptance map.
    // Advance using the same finite one-second requests a host can issue. A
    // single 58-second request exceeds the existing per-operation work budget.
    document["steps"][13 - prep]["clock"]["delta_ms"] = json!(1000);
    let preparation_offset = 49;
    for _ in 0..preparation_offset {
        insert(
            &mut document,
            14,
            json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":1000,"after_seq":0}}),
        );
    }
    prepare_index += preparation_offset;
    committed += preparation_offset;
    settled += preparation_offset;
    // Preserve a reproduction before execution, including failed preparation.
    if let Some(root) = std::env::var_os("INPUT_PLAN_WORLD_DIAGNOSTIC_DIR") {
        let root = std::path::PathBuf::from(root);
        assert!(!root.exists());
        std::fs::create_dir_all(&root).unwrap();
        copy_fixture(fixture.root(), &root.join("content"));
        std::fs::write(
            root.join("scenario.json"),
            serde_json::to_vec_pretty(&document).unwrap(),
        )
        .unwrap();
    }
    let cancel = CancellationToken::default();
    let content =
        crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
            .unwrap();
    assert_eq!(content.token().identity(), &receipt.content_identity);
    let mut trial =
        TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
            .unwrap();
    while !trial.is_complete() {
        let pending = trial.next_step_index();
        trial
            .advance(&cancel)
            .map(|_| ())
            .unwrap_or_else(|e| panic!("input-plan step {pending}: {e:?}"));
    }
    let result = |i| trial.step(i).unwrap().public_result().unwrap();
    let grant = &result(14 + preparation_offset)["payload"]["skill_fragment_diffs"][0];
    assert_eq!(grant["fragment_id"], "fragment_little_red_trace");
    assert_eq!(result(15 + preparation_offset)["disposition"], "accepted");
    let checkpoint = |i| &result(i)["payload"]["battle_update"]["checkpoint"];
    let prepared = &checkpoint(prepare_index)["input_plans"]["preparation"];
    assert_eq!(prepared["copy"], grant["copy_uuid"]);
    assert_eq!(prepared["normal"], 1);
    assert_eq!(prepared["enhanced"], 0);
    let shots = &checkpoint(committed)["input_plans"];
    assert!(shots["preparation"].is_null());
    assert_eq!(shots["receipts"][0]["copy"], grant["copy_uuid"]);
    assert_eq!(shots["receipts"][0]["normal"], 1);
    assert_eq!(shots["owners"][0]["normal"], 0);
    assert_eq!(shots["active_projectiles"].as_array().unwrap().len(), 1);
    let cohort = &checkpoint(prepare_index)["shared_cohorts"]["units"][0];
    assert_eq!(cohort["fragment_recipient"], grant["copy_uuid"]);
    let ended = &checkpoint(settled)["input_plans"];
    let hits = ended["settled_projectiles"][0]["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["actual_hp_loss"], 100);
    assert_ne!(hits[0]["target"], cohort["unit"]);
    assert_eq!(hits[1]["actual_hp_loss"], 150);
    assert_eq!(hits[1]["target"], cohort["unit"]);
    assert!(ended["active_projectiles"].as_array().unwrap().is_empty());
    assert_eq!(ended["settled_projectiles"].as_array().unwrap().len(), 1);
    let hp = serde_json::to_value(
        trial
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .latest_nonzero_sourced_hp(),
    )
    .unwrap();
    eprintln!(
        "input-plan published World {}",
        json!({"grant":grant,"prepared":prepared,"shots":shots,"ended":ended,"hp":hp})
    );
    assert_eq!(hp["fragment"]["copy_uuid"], grant["copy_uuid"], "{hp}");
    assert_eq!(hp["target_unit"], cohort["unit"]);
    assert_eq!(hp["change"]["before"], 300);
    assert_eq!(hp["change"]["after"], 150);
    assert_eq!(
        hp["source"]["authored"]["source_file"],
        "published/data/skills/little-red.skill.yaml"
    );
    if let Some(root) = std::env::var_os("INPUT_PLAN_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root);
        assert!(!root.exists());
        std::fs::create_dir_all(&root).unwrap();
        copy_fixture(fixture.root(), &root.join("content"));
        std::fs::write(
            root.join("scenario.json"),
            serde_json::to_vec_pretty(&document).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("expected.json"),
            serde_json::to_vec_pretty(
                &json!({"grant":grant,"prepared":prepared,"shots":shots,"ended":ended,"hp":hp}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
