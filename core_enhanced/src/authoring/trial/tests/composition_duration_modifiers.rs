use super::*;

pub(super) fn modifier_reward_world_fixture(relic: &str, orthogonal: bool) -> (ExternalFixture, Value) {
    let (fixture, _, mut document, _, _) = channel_support_world_fixture_with_cell(orthogonal);
    use crate::game::data::{relic_data::RelicDatabase, reward_data::RewardDatabase};
    let path = "official/data/relics/base.ron";
    let mut relics: RelicDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    let mut duration_relic = relics.relics[0].clone();
    duration_relic.id = relic.into();
    duration_relic.uuid = uuid::Uuid::from_u128(0xd240);
    relics.relics.push(duration_relic);
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: RewardDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    rewards.rewards.push(ron::de::from_str(r#"(id:"duration_supply",uuid:"a524e100-0000-0000-0000-000000000301",name:"Duration fixture supply",description:"Normal victory reward",icon:"fixture",effects:[GrantRelic(relic_id:"long-breath")])"#.replace("long-breath", relic).as_str()).unwrap());
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    let path = "official/data/map/node_assignment_profiles.ron";
    let mut assignments: crate::game::map::NodeAssignmentProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    assignments
        .profiles
        .iter_mut()
        .find(|p| p.id == "standard_floor_1")
        .unwrap()
        .fixed_slot_assignments
        .iter_mut()
        .find(|s| s.slot_id.as_str() == "intake_corridor")
        .unwrap()
        .authored_combat
        .as_mut()
        .unwrap()
        .outcome_policies
        .iter_mut()
        .find(|p| p.outcome_id == "victory")
        .unwrap()
        .reward_ids
        .push("duration_supply".into());
    fixture.write(path, ron::ser::to_string(&assignments).unwrap());
    fixture.author_trial_facility();
    // Insert real inventory query/equip commands after the first victory, and
    // remap every later binding rather than borrowing an old authority value.
    fn shift(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "step" && value.as_u64().is_some_and(|n| n >= 15) {
                        *value = json!(value.as_u64().unwrap() + 2);
                    } else {
                        shift(value);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    shift(value);
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().splice(6..6,[
        json!({"kind":"command","command":{"type":"request_relic_state"}}),
        json!({"kind":"command","command":{"type":"equip_relic","copy_uuid":null,"employee_uuid":null,"slot":2},"bindings":[
            {"from":{"step":15,"pointer":"/payload/copies/0/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"},
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]}),
    ]);
    for _ in 0..2 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    (fixture, document)
}

#[test]
fn composition_duration_modifiers_world_reward_equip_and_actual_expiry() {
    let (fixture, document) = modifier_reward_world_fixture("long-breath", false);
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/duration_modifiers.skill.yaml")
            .replace(
                "authored-channel-support",
                "fragment_freischutz_black_round",
            )
            .replace("capacity: 100", "capacity: 25")
            .replace("duration_ms: 201", "duration_ms: 1501");
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
        let mut expires = None;
        let mut owner = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("duration World step {pending}: {e:?}"))
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
                .find(|r| r["is_channel"] == true)
            {
                assert_eq!(effect["members"].as_array().unwrap().len(), 1);
                let deadline = effect["expires_at_ms"].as_u64().unwrap();
                let duration = (1501u64 * (100 + percent)).div_ceil(100);
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
        expected.push(json!({"percent":percent,"expires_at_ms":expires,"active":active,"owner":owner,"latest":latest}));
    }
    if let Some(directory) = std::env::var_os("DURATION_MODIFIERS_COMPOSITION_FIXTURE_DIR") {
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
