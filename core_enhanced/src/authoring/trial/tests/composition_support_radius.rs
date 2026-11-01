use super::*;

#[test]
fn composition_support_radius_world_reward_equip_and_actual_circular_membership() {
    let (fixture, document) = duration_modifiers::modifier_reward_world_fixture("resonator", true);
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/support_radius.skill.yaml")
            .replace(
                "authored-channel-support",
                "fragment_freischutz_black_round",
            )
            .replace("capacity: 100", "capacity: 25")
            .replace("duration_ms: 201", "duration_ms: 1501")
            .replace("radius_units: 1600000", "radius_units: 800000");
    let mut expected = Vec::new();
    for percent in [25, 50] {
        let current = source.replace("Count: 25", &format!("Count: {percent}"));
        let cancel = CancellationToken::default();
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
        let mut active = Vec::new();
        let mut owner = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("radius World step {pending}: {e:?}"))
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
            if index == 16 {
                assert_eq!(result["disposition"], "accepted");
                let p = &result["payload"];
                assert_eq!(p["copies"][0]["definition_id"], "resonator");
                assert_eq!(p["copies"][0]["variant"], "Normal");
                assert_eq!(p["loadouts"][0]["employee_uuid"], owner.clone().unwrap());
                assert_eq!(p["loadouts"][0]["slots"][2], p["copies"][0]["copy_uuid"]);
            }
            if index < 29 {
                continue;
            }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            if let Some(effect) = cp["sustained_effects"]["active"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["is_channel"] == true)
            {
                assert_eq!(effect["expires_at_ms"], 24502);
                let members = effect["members"].as_array().unwrap();
                assert_eq!(members.len(), 1);
                let units = cp["units"].as_array().unwrap();
                let caster = units
                    .iter()
                    .find(|u| u["unit_instance_id"] == effect["caster"])
                    .unwrap();
                let member = units
                    .iter()
                    .find(|u| u["unit_instance_id"] == members[0])
                    .unwrap();
                let dx = caster["position"]["x"].as_i64().unwrap()
                    - member["position"]["x"].as_i64().unwrap();
                let dy = caster["position"]["y"].as_i64().unwrap()
                    - member["position"]["y"].as_i64().unwrap();
                // One full tile is outside the authored 0.8 radius but within
                // both equipped radii (1.0 at 25%, 1.2 at 50%).
                assert_eq!(dx * dx + dy * dy, 1);
                active.push(json!({"step":index,"effect":effect}));
            }
        }
        assert!(!active.is_empty());
        let latest = trial
            .session()
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .latest_sourced_sustained()
            .unwrap();
        assert!(matches!(
            latest.change(),
            crate::authoring::ApplicationChange::Sustained {
                ended: Some(crate::game::ability::sustained::SustainedEndReason::Expired),
                ..
            }
        ));
        let origin = trial
            .origin_for_observed(latest.authored_source().unwrap())
            .unwrap();
        assert!(origin.inputs.iter().any(|p| p == "values[1].value"));
        assert!(origin
            .inputs
            .iter()
            .any(|p| p.ends_with("change.SupportRadius")));
        expected.push(json!({"percent":percent,"active":active,"owner":owner,"latest":latest}));
    }
    if let Some(directory) = std::env::var_os("SUPPORT_RADIUS_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier fixture evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
