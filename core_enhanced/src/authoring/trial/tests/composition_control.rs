use super::*;

#[test]
fn composition_control_world_reward_equip_and_actual_expiry() {
    let (fixture, document) =
        duration_modifiers::modifier_reward_world_fixture("long-breath", false);
    let source = include_str!("../../../../tests/fixtures/skill_composition/control.skill.yaml")
        .replace("authored-control", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25");
    let mut expected = Vec::new();
    for duration_ms in [1501, 2001] {
        let current = source.replace("duration_ms: 1501", &format!("duration_ms: {duration_ms}"));
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
        let mut expires = None;
        let mut owner = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("control World step {pending}: {e:?}"))
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
                let payload = &result["payload"];
                assert_eq!(result["disposition"], "accepted");
                assert_eq!(payload["copies"][0]["definition_id"], "long-breath");
                assert_eq!(payload["copies"][0]["variant"], "Normal");
                assert_eq!(
                    payload["loadouts"][0]["employee_uuid"],
                    owner.clone().unwrap()
                );
                assert_eq!(
                    payload["loadouts"][0]["slots"][2],
                    payload["copies"][0]["copy_uuid"]
                );
            }
            if index == 15 {
                assert_eq!(
                    result["payload"]["copies"][0]["definition_id"],
                    "long-breath"
                );
            }
            if index < 29 {
                continue;
            }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            if let Some(effect) = cp["sustained_effects"]["active"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["control"].get("ApproachTaunt").is_some())
            {
                assert_eq!(effect["is_channel"], false);
                let target = cp["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|u| u["unit_instance_id"] == effect["target"])
                    .unwrap();
                assert_eq!(target["owner"], "Opponent");
                assert_eq!(
                    effect["control"]["ApproachTaunt"]["profile"]["speed_percent"],
                    100
                );
                assert_eq!(
                    effect["control"]["ApproachTaunt"]["profile"]["attack_range_units"],
                    1_000_000
                );
                let deadline = effect["expires_at_ms"].as_u64().unwrap();
                let duration = (duration_ms as u64 * 125).div_ceil(100);
                // The unchanged scenario starts the effect at 23,001ms.
                assert_eq!(deadline, 23001 + duration);
                assert!(expires.is_none_or(|old| old == deadline));
                expires = Some(deadline);
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
        assert!(origin.inputs.iter().any(|p| p == "values[0].value"));
        assert!(origin.inputs.iter().any(|p| p == "values[1].value"));
        expected.push(json!({"duration_ms":duration_ms,"expires_at_ms":expires,"active":active,"owner":owner,"latest":latest}));
    }
    if let Some(directory) = std::env::var_os("CONTROL_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier fixture evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
