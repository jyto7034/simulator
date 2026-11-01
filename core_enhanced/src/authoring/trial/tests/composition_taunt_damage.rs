use super::*;

#[test]
fn composition_taunt_damage_world_rewards_equip_both_and_live_coefficients() {
    const TAKEN: &str = "taunted-enemy-damage-reduction";
    const BASIC: &str = "taunted-enemy-basic-attack-boost";
    let (fixture, mut document) = duration_modifiers::modifier_reward_world_fixture(TAKEN, false);
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut extra = relics
        .relics
        .iter()
        .find(|r| r.id == TAKEN)
        .unwrap()
        .clone();
    extra.id = BASIC.into();
    extra.uuid = uuid::Uuid::from_u128(0xd271);
    relics.relics.push(extra);
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: crate::game::data::reward_data::RewardDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let reward = rewards
        .rewards
        .iter_mut()
        .find(|r| r.id == "duration_supply")
        .unwrap();
    reward
        .effects
        .push(ron::de::from_str(&format!("GrantRelic(relic_id:\"{BASIC}\")")).unwrap());
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    // A second real equip command; all later authority references follow the insertion.
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= 17) {
                        *v = json!(v.as_u64().unwrap() + 1);
                    } else {
                        shift(v);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    shift(v);
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().insert(8,json!({"kind":"command","command":{"type":"equip_relic","copy_uuid":null,"employee_uuid":null,"slot":1},"bindings":[
        {"from":{"step":15,"pointer":"/payload/copies/1/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]}));
    fixture.author_trial_facility();
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/taunt_damage.skill.yaml")
            .replace("authored-taunt-damage", "fragment_freischutz_black_round")
            .replace("capacity: 100", "capacity: 25")
            .replace("duration_ms: 1501", "duration_ms: 2001");
    let mut expected = Vec::new();
    for taken in [20, 40] {
        let current = source.replace("Count: 20", &format!("Count: {taken}"));
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
        let mut observations = Vec::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("taunt damage World step {pending}: {e:?}"))
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
            if index == 17 {
                let payload = &result["payload"];
                assert_eq!(result["disposition"], "accepted");
                let copies = payload["copies"].as_array().unwrap();
                assert_eq!(copies.len(), 2);
                for name in [TAKEN, BASIC] {
                    assert!(copies
                        .iter()
                        .any(|c| c["definition_id"] == name && c["variant"] == "Normal"));
                }
                let loadout = &payload["loadouts"][0];
                assert_eq!(loadout["employee_uuid"], owner.clone().unwrap());
                for copy in copies {
                    assert!(loadout["slots"]
                        .as_array()
                        .unwrap()
                        .contains(&copy["copy_uuid"]));
                }
            }
            if index < 30 {
                continue;
            }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            for effect in cp["sustained_effects"]["active"].as_array().unwrap() {
                let Some(taunt) = effect["control"].get("ApproachTaunt") else {
                    continue;
                };
                assert_eq!(effect["is_channel"], false);
                assert_eq!(effect["expires_at_ms"], 25002);
                assert_eq!(taunt["profile"]["damage_reduction_percent"], taken);
                assert_eq!(taunt["profile"]["basic_damage_bonus_percent"], 30);
                let target = cp["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|u| u["unit_instance_id"] == effect["target"])
                    .unwrap();
                assert_eq!(target["owner"], "Opponent");
                active.push(json!({"step":index,"effect":effect}));
            }
            observations.push(json!({"step":index,"units":cp["units"],"latest":trial.session().observation().unwrap().active().unwrap().latest()}));
        }
        assert!(!active.is_empty());
        // The next real weapon release is at 24,701ms, while this control is
        // active. Physical attack 12 against defense 0 becomes floor(12*1.3)=15.
        let release = observations.iter().find(|o| o["step"] == 35).unwrap();
        let hit = &release["latest"];
        assert_eq!(hit["time_ms"], 24701);
        assert_eq!(hit["source_unit"], active[0]["effect"]["caster"]);
        assert_eq!(hit["target_unit"], active[0]["effect"]["target"]);
        assert_eq!(hit["change"]["reason"], "BasicAttack");
        assert_eq!(
            hit["change"]["before"].as_u64().unwrap() - hit["change"]["after"].as_u64().unwrap(),
            15
        );
        let attacker = release["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["unit_instance_id"] == hit["source_unit"])
            .unwrap();
        let target = release["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["unit_instance_id"] == hit["target_unit"])
            .unwrap();
        assert_eq!(attacker["stats"]["attack"], 12);
        assert_eq!(target["stats"]["defense"], 0);
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
        for value in 0..3 {
            assert!(
                origin.inputs.contains(&format!("values[{value}].value")),
                "{origin:?}"
            );
        }
        expected.push(json!({"taken":taken,"active":active,"owner":owner,"latest":latest,"observations":observations}));
    }
    if let Some(directory) = std::env::var_os("TAUNT_DAMAGE_COMPOSITION_FIXTURE_DIR") {
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
