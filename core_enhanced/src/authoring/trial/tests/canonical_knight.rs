use super::*;
use crate::authoring::composition::canonical_knight_tests as canonical;
use crate::game::data::content_pack::{package_official_with_skill_files, ContentSelection};

#[test]
fn canonical_knight_published_boss_grant_relation_and_normal_release() {
    canonical_knight_world(false);
}

#[test]
fn canonical_knight_published_bloom_awakening_selection_and_area_release() {
    canonical_knight_world(true);
}

fn canonical_knight_world(bloom: bool) {
    let (fixture, mut document) = super::canonical_boss::boss_fixture(
        bloom,
        "suppress_knight_of_despair",
        "o-01-73_knight_of_despair",
        canonical::catalog(),
        canonical::relics(),
    );
    let sources = [
        "data/skills/knight-base.skill.yaml".to_string(),
        "data/skills/knight-bloom.skill.yaml".to_string(),
    ];
    for (bloom, source) in [false, true].into_iter().zip(&sources) {
        fixture.write(&format!("official/{source}"), canonical::original(bloom));
    }
    let error = package_official_with_skill_files(
        &fixture.root().join("official"),
        &fixture.root().join("unresolved"),
        &sources
            .iter()
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>(),
        &Default::default(),
    )
    .unwrap_err();
    assert!(format!("{error:?}").contains("Unresolved"));
    assert!(!fixture.root().join("unresolved").exists());
    for (bloom, source) in [false, true].into_iter().zip(&sources) {
        let mut completed = canonical::resolved(bloom);
        completed["skills"][0]["admission"]["capacity"] = json!(5);
        canonical::program(&mut completed)["admission"]["capacity"] = json!(5);
        fixture.write(
            &format!("official/{source}"),
            serde_json::to_string_pretty(&completed).unwrap(),
        );
    }
    let receipt = package_official_with_skill_files(
        &fixture.root().join("official"),
        &fixture.root().join("published"),
        &sources
            .iter()
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>(),
        &Default::default(),
    )
    .unwrap();
    let selection = ContentSelection {
        packs: vec!["published".into()],
        winners: vec![],
    };
    let cancel = CancellationToken::default();
    let content =
        crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
            .unwrap();
    assert_eq!(content.token().identity(), &receipt.content_identity);
    let (ground, scout_cell, scout) =
        super::canonical_boss::prepare_deployments(&fixture, &selection, &mut document);
    let prep = document["preparation_steps"].as_array().unwrap().len();
    // The public owner snapshot binds the same acquired fragment copy and caster.
    let mut select = authority_step("select_skill_relation", 24);
    select["command"]["employee_uuid"] = Value::Null;
    select["command"]["entry"] = json!("blessing");
    select["command"]["target"] = Value::Null;
    select["bindings"].as_array_mut().unwrap().extend([
        json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),
        json!({"from":{"step":24,"pointer":"/payload/battle_update/checkpoint/relation_actions/owners/0/caster"},"to":"/command/target","value_type":"string"}),
    ]);
    document["steps"].as_array_mut().unwrap().push(select);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":4000,"after_seq":0}}));
    let mut detach = authority_step("select_skill_relation", 26);
    detach["command"] = document["steps"][25 - prep]["command"].clone();
    detach["bindings"].as_array_mut().unwrap().extend(
        document["steps"][25 - prep]["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["to"] == "/command/employee_uuid" || b["to"] == "/command/target")
            .cloned(),
    );
    document["steps"].as_array_mut().unwrap().push(detach);
    let mut cast = authority_step("activate_skill", 27);
    cast["command"]["employee_uuid"] = Value::Null;
    cast["command"]["skill_id"] = json!("fragment_knight_blessing_base");
    cast["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    document["steps"].as_array_mut().unwrap().push(cast);
    for delta in [1, 500, 500, 500, 500, 500, 500] {
        document["steps"].as_array_mut().unwrap().push(
            json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":delta,"after_seq":0}}),
        );
    }
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
    // Three deployments cost 30 TP. The authored policy starts at 20 and
    // restores 1/s, so the third is legal at 10s. Keep the total 68s window.
    document["steps"][11 - prep]["clock"]["delta_ms"] = json!(10000);
    document["steps"][12 - prep]["clock"]["delta_ms"] = json!(58000);
    shift(&mut document, 12, 1);
    let mut scout_deploy = authority_step("deploy_unit", 11);
    scout_deploy["command"]["employee_uuid"] = Value::Null;
    scout_deploy["command"]["position"] = Value::Null;
    scout_deploy["command"]["facing"] = json!("up");
    scout_deploy["bindings"].as_array_mut().unwrap().extend([
        json!({"from":{"step":6,"pointer":format!("/payload/battle_update/checkpoint/deployment/unit_deploy_costs/{scout}/employee_uuid")},"to":"/command/employee_uuid","value_type":"string"}),
        json!({"from":{"step":5,"pointer":format!("/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{scout_cell}")},"to":"/command/position","value_type":"object"}),
    ]);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .insert(12 - prep, scout_deploy);
    if bloom {
        // Current indexes already include the scout deployment. Awakening
        // unlocks a form; explicit selection is a distinct public operation.
        shift(&mut document, 20, 1);
        let copy_binding = json!({"from":{"step":14,"pointer":"/payload/skill_fragment_diffs/0/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"});
        document["steps"][18 - prep] = json!({"kind":"command","command":{"type":"awaken_skill_fragment","copy_uuid":null,"form_id":"bloom"},"bindings":[copy_binding.clone()]});
        document["steps"][19 - prep] = json!({"kind":"command","command":{"type":"select_skill_fragment_form","copy_uuid":null,"form_id":"bloom"},"bindings":[copy_binding]});
        document["steps"].as_array_mut().unwrap().insert(
            20 - prep,
            json!({"kind":"command","command":{"type":"complete_node"}}),
        );
        document["steps"][21 - prep]["bindings"][0]["from"]["step"] = json!(20);
        let mut position = document["steps"][25 - prep]["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["to"] == "/command/position")
            .unwrap()
            .clone();
        position["to"] = json!("/command/target/position");
        let cast = &mut document["steps"][30 - prep];
        cast["command"]["skill_id"] = json!("fragment_knight_blessing_bloom");
        let mut begin = cast.clone();
        begin["command"]["type"] = json!("begin_skill_targeting");
        cast["command"]["target"] = json!({"type":"Tile","position":null});
        cast["bindings"].as_array_mut().unwrap().push(position);
        shift(&mut document, 30, 1);
        for binding in document["steps"][30 - prep]["bindings"]
            .as_array_mut()
            .unwrap()
        {
            if binding["to"].as_str().unwrap().starts_with("/context/") {
                binding["from"]["step"] = json!(30);
            }
        }
        document["steps"]
            .as_array_mut()
            .unwrap()
            .insert(30 - prep, begin);
    }
    let index = |original| {
        original
            + usize::from(original >= 12)
            + usize::from(bloom && original >= 19)
            + usize::from(bloom && original >= 28)
    };
    let mut trial =
        TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
            .unwrap();
    while !trial.is_complete() {
        let pending = trial.next_step_index();
        if let Err(error) = trial.advance(&cancel).map(|_| ()) {
            panic!(
                "knight step {pending}: {error:?}; previous={:?}",
                trial
                    .step(pending.saturating_sub(1))
                    .and_then(|s| s.public_result())
            );
        }
        if pending == index(12) {
            let result = trial.step(index(12)).unwrap().public_result().unwrap();
            let units: Vec<_> = result["payload"]["battle_update"]["checkpoint"]["units"].as_array().unwrap().iter().map(|u| json!({"id":u["unit_instance_id"],"owner":u["owner"],"position":u["position"],"stats":u["stats"],"source":u["unit_source"]})).collect();
            assert_eq!(
                result["payload"]["finished"], true,
                "opening must finish: {units:?}"
            );
        }
    }
    let result = |original| {
        trial
            .step(index(original))
            .unwrap()
            .public_result()
            .unwrap()
    };
    let grant = &result(13)["payload"]["skill_fragment_diffs"][0];
    assert_eq!(grant["fragment_id"], "fragment_knight_of_despair_trace");
    assert_eq!(result(14)["disposition"], "accepted");
    let owner = |index| {
        &result(index)["payload"]["battle_update"]["checkpoint"]["relation_actions"]["owners"][0]
    };
    assert_eq!(owner(24)["source_copy"], grant["copy_uuid"]);
    assert_eq!(owner(24)["stock"], 0);
    assert_eq!(owner(26)["stock"], 4);
    assert_eq!(owner(27)["stock"], 4);
    assert!(owner(27)["target"].is_null());
    assert_eq!(result(28)["disposition"], "accepted");
    if bloom {
        let awakened = result(17);
        assert_eq!(awakened["disposition"], "accepted");
        assert_eq!(awakened["payload"]["copy"]["copy_uuid"], grant["copy_uuid"]);
        assert_eq!(awakened["payload"]["enkephalin_spent"], 1);
        assert_eq!(awakened["payload"]["copy"]["selected_form"], "base");
        assert!(awakened["payload"]["copy"]["unlocked_forms"]
            .as_array()
            .unwrap()
            .contains(&json!("bloom")));
        let selected = result(18);
        assert_eq!(selected["disposition"], "accepted");
        assert_eq!(selected["payload"]["copy"]["selected_form"], "bloom");
        assert_eq!(
            selected["payload"]["copy"]["equipped_to"],
            result(9)["payload"]["employee_uuid"]
        );
    }
    let final_result = trial
        .step(trial.next_step_index() - 1)
        .unwrap()
        .public_result()
        .unwrap();
    let final_owner =
        &final_result["payload"]["battle_update"]["checkpoint"]["relation_actions"]["owners"][0];
    assert!(
        final_owner["stock"].as_u64().unwrap() < 4,
        "must release a real enhanced attack: {}",
        final_owner
    );
    assert!(final_owner["active"].is_null(), "{}", final_owner);
    let observed = trial.observation().unwrap();
    let hp = observed
        .active()
        .unwrap()
        .latest_nonzero_sourced_hp()
        .unwrap();
    assert_eq!(
        hp.authored_source().unwrap().source_file(),
        if bloom {
            "published/data/skills/knight-bloom.skill.yaml"
        } else {
            "published/data/skills/knight-base.skill.yaml"
        }
    );
    assert_eq!(
        serde_json::to_value(hp).unwrap()["fragment"]["copy_uuid"],
        grant["copy_uuid"]
    );
    if bloom {
        assert_eq!(final_owner["stock"], 0);
        assert_eq!(
            serde_json::to_value(hp).unwrap()["fragment"]["form_id"],
            "bloom"
        );
        let events = final_result["payload"]["battle_update"]["events_delta"]["events"]
            .as_array()
            .unwrap();
        let hits: Vec<_> = events
            .iter()
            .filter(|v| {
                v["event"]["type"] == "HpChanged"
                    && v["event"]["source_instance_id"] == final_owner["caster"]
                    && v["event"]["damage_source"] == "Ability"
            })
            .collect();
        assert_eq!(
            hits.len(),
            5,
            "one large sword and four small swords: {hits:?}"
        );
    }
    if let Some(root) = std::env::var_os("KNIGHT_SOURCE_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root).join(if bloom { "bloom" } else { "base" });
        std::fs::create_dir_all(&root).unwrap();
        copy_fixture(fixture.root(), &root.join("content"));
        std::fs::write(
            root.join("scenario.json"),
            serde_json::to_string_pretty(&document).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("expected.json"),
            serde_json::to_string_pretty(
                &json!({"grant":grant,"owner":final_owner,"latest_hp":hp}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
