use super::canonical_boss::{clock_step, insert, push};
use super::*;
use crate::authoring::composition::canonical_pair_tests as canonical;
use crate::game::data::content_pack::{package_official_with_skill_files, ContentSelection};

#[test]
fn canonical_pair_published_yin_reward_installs_heals_and_expires() {
    published_world(false);
}
#[test]
fn canonical_pair_published_yang_reward_installs_heals_and_expires() {
    published_world(true);
}

fn published_world(yang: bool) {
    let family = if yang { "yang" } else { "yin" };
    let abnormality = if yang { "o-05-102_yang" } else { "o-01-64_yin" };
    let (fixture, mut document) = super::canonical_boss::boss_fixture(
        false,
        &format!("suppress_{family}"),
        abnormality,
        canonical::catalog(yang),
        canonical::relics(),
    );
    // Publish both existing reward identities against the same skill definition.
    // They remain different inventory copies/origins; no grant alias is added.
    let path = "official/data/skill_fragments/base.ron";
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut other = canonical::catalog(!yang);
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = &mut other.effect
    else {
        panic!()
    };
    forms.get_mut("base").unwrap().resonance_max = 5;
    let previous = fragments
        .fragments
        .iter_mut()
        .find(|f| f.id == other.id)
        .unwrap();
    *previous = other;
    fixture.write(path, ron::ser::to_string(&fragments).unwrap());

    // Preserve encounter class, waves, enemy identities and actual reward source.
    // Lower only escort stats for bounded acceptance preparation.
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for enemy in &mut enemies.profiles {
        if enemy.basic_attack_range_preset.is_some() {
            enemy.basic_attack.defense_tile_range = None;
        }
        if ["corroded_marksman", "corroded_medic"].contains(&enemy.id.as_str()) {
            enemy.max_health = 70;
            enemy.attack = 0;
            enemy.defense = 0;
            enemy.magic_resist = 0;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let sources = vec![std::path::PathBuf::from("data/skills/yin-yang.skill.yaml")];
    fixture.write(
        "official/data/skills/yin-yang.skill.yaml",
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
        "official/data/skills/yin-yang.skill.yaml",
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
    let mut begin = authority_step("begin_skill_targeting", 25);
    begin["command"]["skill_id"] = json!("fragment_yin_yang_cycle");
    begin["command"]["employee_uuid"] = Value::Null;
    begin["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut cast = begin.clone();
    let begin_index = push(&mut document, begin);
    cast["command"]["type"] = json!("activate_skill");
    cast["command"]["target"] = json!({"type":"TilePair","first":null,"second":null});
    for b in cast["bindings"].as_array_mut().unwrap() {
        if b["to"].as_str().unwrap().starts_with("/context/") {
            b["from"]["step"] = json!(begin_index);
        }
    }
    let position = document["steps"][24 - prep]["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["to"] == "/command/position")
        .unwrap()
        .clone();
    for field in ["first", "second"] {
        let mut binding = position.clone();
        binding["to"] = json!(format!("/command/target/{field}"));
        cast["bindings"].as_array_mut().unwrap().push(binding);
    }
    let cast_index = push(&mut document, cast);
    let installed = clock_step(&mut document, 1);
    let healed = clock_step(&mut document, 1000);
    let expired = clock_step(&mut document, 2000);
    let cancel = CancellationToken::default();
    let content =
        crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
            .unwrap();
    assert_eq!(content.token().identity(), &receipt.content_identity);
    let mut trial =
        TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
            .unwrap();
    let mut hp = Value::Null;
    while !trial.is_complete() {
        let pending = trial.next_step_index();
        trial.advance(&cancel).map(|_| ()).unwrap_or_else(|e| {
            panic!(
                "{family} step {pending}: {e:?}; previous={:?}",
                trial
                    .step(pending.saturating_sub(1))
                    .and_then(|s| s.public_result())
            )
        });
        if pending == healed {
            hp = serde_json::to_value(
                trial
                    .observation()
                    .unwrap()
                    .active()
                    .unwrap()
                    .latest_nonzero_sourced_hp(),
            )
            .unwrap();
        }
    }
    let result = |i| trial.step(i).unwrap().public_result().unwrap();
    let grant = &result(14)["payload"]["skill_fragment_diffs"][0];
    assert_eq!(grant["fragment_id"], format!("fragment_{family}_trace"));
    assert_eq!(result(15)["disposition"], "accepted");
    assert_eq!(result(cast_index)["disposition"], "accepted");
    let areas =
        &result(installed)["payload"]["battle_update"]["checkpoint"]["paired_areas"]["areas"];
    assert_eq!(areas.as_array().unwrap().len(), 1);
    assert_eq!(areas[0]["copy"], grant["copy_uuid"]);
    assert_eq!(areas[0]["first_center"], areas[0]["second_center"]);
    assert!(
        result(expired)["payload"]["battle_update"]["checkpoint"]["paired_areas"]["areas"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(hp["fragment"]["copy_uuid"], grant["copy_uuid"], "{hp}");
    assert_eq!(hp["fragment"]["definition_id"], grant["fragment_id"]);
    assert_eq!(hp["source"]["skill_id"], "fragment_yin_yang_cycle");
    assert_eq!(
        hp["source"]["authored"]["source_file"],
        "published/data/skills/yin-yang.skill.yaml"
    );
    assert!(hp["change"]["after"].as_i64().unwrap() > hp["change"]["before"].as_i64().unwrap());
    eprintln!(
        "paired World {family}: {}",
        json!({"grant":grant,"areas":areas,"hp":hp})
    );
    if let Some(root) = std::env::var_os("PAIRED_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root).join(family);
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
            serde_json::to_vec_pretty(&json!({"grant":grant,"areas":areas,"hp":hp})).unwrap(),
        )
        .unwrap();
    }
}
