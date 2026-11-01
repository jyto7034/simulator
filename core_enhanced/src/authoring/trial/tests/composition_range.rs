use super::*;

#[test]
fn composition_range_world_reward_equip_mask_membership_and_expiry() {
    let (fixture, document) =
        duration_modifiers::modifier_reward_world_fixture("widened-horizon", false);
    let source = include_str!("../../../../tests/fixtures/skill_composition/tile_range.skill.yaml")
        .replace("authored-tile-range", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25");
    let mut expected = Vec::new();
    for mode in ["unowned", "equipped", "edited"] {
        let current = if mode == "edited" {
            source.replace("rows: [XXX, X@X, XXX]", "rows: [.X., X@X, XXX]")
        } else {
            source.clone()
        };
        let mut scenario = document.clone();
        if mode == "unowned" {
            // Keep the inventory/reward and all indices; only omit actual equip.
            scenario["steps"][7] =
                json!({"kind":"command","command":{"type":"request_relic_state"}});
        }
        let cancel = CancellationToken::default();
        let mut trial = CompositionTrial::prepare(
            fixture.root(),
            &fixture.selection(),
            &current,
            checked(&scenario),
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
                .unwrap_or_else(|e| panic!("range World {mode} step {pending}: {e:?}"))
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
                assert_eq!(payload["copies"][0]["definition_id"], "widened-horizon");
                assert_eq!(payload["copies"][0]["variant"], "Normal");
                if mode != "unowned" {
                    assert_eq!(
                        payload["loadouts"][0]["employee_uuid"],
                        owner.clone().unwrap()
                    );
                    assert_eq!(
                        payload["loadouts"][0]["slots"][2],
                        payload["copies"][0]["copy_uuid"]
                    );
                }
            }
            if index < 29 {
                continue;
            }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            for effect in cp["sustained_effects"]["active"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["control"].get("ApproachTaunt").is_some())
            {
                assert_ne!(mode, "unowned");
                assert_eq!(effect["expires_at_ms"], 24502);
                assert_eq!(
                    effect["control"]["ApproachTaunt"]["profile"]["attack_range_units"],
                    1000000
                );
                let units = cp["units"].as_array().unwrap();
                let target = units
                    .iter()
                    .find(|u| u["unit_instance_id"] == effect["target"])
                    .unwrap();
                let caster = units
                    .iter()
                    .find(|u| u["unit_instance_id"] == effect["caster"])
                    .unwrap();
                assert_eq!(target["owner"], "Opponent");
                if active.is_empty() {
                    // Outside the base anchor-only mask, inside the authored 3x3.
                    assert_ne!(target["position"], caster["position"]);
                    let dx = target["position"]["x"].as_i64().unwrap()
                        - caster["position"]["x"].as_i64().unwrap();
                    let dy = target["position"]["y"].as_i64().unwrap()
                        - caster["position"]["y"].as_i64().unwrap();
                    assert!(dx.abs() <= 1 && dy.abs() <= 1);
                }
                active.push(json!({"step":index,"effect":effect,"target":target,"caster":caster}));
            }
        }
        if mode == "unowned" {
            assert!(active.is_empty());
        } else {
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
            assert!(
                origin.inputs.iter().any(|p| p == "values[1].value"),
                "{origin:?}"
            );
        }
        expected.push(json!({"mode":mode,"owner":owner,"active":active}));
    }
    if let Some(directory) = std::env::var_os("TILE_RANGE_COMPOSITION_FIXTURE_DIR") {
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
