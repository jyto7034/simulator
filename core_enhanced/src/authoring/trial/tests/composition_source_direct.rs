use super::*;

#[test]
fn composition_source_direct_world_equipped_weapon_stacks_and_damage() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"clock",
        "clock":{"mode":"elapsed","delta_ms":10000,"after_seq":0}}));
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/source_direct.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: standard_training_sword",
            );
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [3, 6] {
        let current = source.replace(
            "per_stack_percent: 3",
            &format!("per_stack_percent: {percent}"),
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
            let step = trial.session().next_step_index();
            trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("source direct World step {step}: {e:?}"));
        }
        let result = trial.session().step(25).unwrap().public_result().unwrap();
        let cp = &result["payload"]["battle_update"]["checkpoint"];
        let effect = cp["sustained_effects"]["active"]
            .as_array()
            .expect("active encounter checkpoint")
            .iter()
            .find(|v| v["hit_status"]["definition"] == "insight")
            .expect("actual regular weapon hit must stack");
        let stack = &effect["hit_status"]["kind"]["SourceDirectDamageTaken"];
        assert_eq!(stack["per_stack_percent"], percent);
        assert!(stack["stacks"].as_u64().is_some_and(|n| n > 0 && n <= 5));
        let target = cp["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["unit_instance_id"] == effect["target"])
            .unwrap();
        let observation = trial.session().observation().unwrap();
        let latest = observation
            .active()
            .unwrap()
            .latest_sourced_sustained()
            .unwrap();
        let authored = latest.authored_source().unwrap();
        let origin = trial.origin_for_observed(authored).unwrap();
        assert!(origin.definition.contains("AttackProgram"));
        assert!(authored.field_path().ends_with("program.on_hit_status"));
        assert!(latest
            .equipment_source()
            .unwrap()
            .persistent_copy_uuid
            .is_some());
        assert!(
            latest.source().is_none(),
            "weapon hit must not invent a skill cast"
        );
        expected.push(
            json!({"percent":percent,"effect":effect,"hp":target["stats"]["current_health"],
            "latest":latest,"origin":origin}),
        );
    }
    eprintln!("source direct World: {expected:?}");
    assert!(expected[1]["hp"].as_u64().unwrap() < expected[0]["hp"].as_u64().unwrap());
    if let Some(directory) = std::env::var_os("SOURCE_DIRECT_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(
            directory.join("trial.json"),
            serde_json::to_vec_pretty(&json!({
            "root":directory.join("content"),"selection":fixture.selection(),"scenario":document}))
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
