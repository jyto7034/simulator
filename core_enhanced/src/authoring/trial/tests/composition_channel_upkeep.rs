use super::*;

#[test]
fn composition_channel_upkeep_world_reward_corrupt_preview_equip_and_cost() {
    use crate::game::data::{
        relic_data::{RelicCorruptionVariant, RelicDatabase, RelicPenalty},
        reward_data::RewardDatabase,
    };
    let (fixture, mut document) =
        duration_modifiers::modifier_reward_world_fixture("upkeep-conversion", false);
    let path = "official/data/relics/base.ron";
    let mut relics: RelicDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    relics
        .relics
        .iter_mut()
        .find(|r| r.id == "upkeep-conversion")
        .unwrap()
        .corruption = Some(RelicCorruptionVariant {
        effects: vec![],
        rules: vec![],
        penalties: vec![RelicPenalty::AuthoredSupportUpkeep],
    });
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: RewardDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    rewards
        .rewards
        .iter_mut()
        .find(|r| r.id == "duration_supply")
        .unwrap()
        .effects
        .push(crate::game::reward::RewardEffect::GrantRelicCorruptionCatalyst { amount: 1 });
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= 16) {
                        *v = json!(v.as_u64().unwrap() + 2)
                    } else {
                        shift(v)
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    shift(v)
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().splice(7..7,["preview_relic_corruption","corrupt_relic"].map(|kind|json!({
        "kind":"command","command":{"type":kind,"copy_uuid":null},"bindings":[
            {"from":{"step":15,"pointer":"/payload/copies/0/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"}]})));
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    let activate_index = document["steps"]
        .as_array()
        .unwrap()
        .iter()
        .position(|s| s["command"]["type"] == "activate_skill")
        .unwrap();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(activate_index);
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/channel_upkeep.skill.yaml")
            .replace(
                "authored-channel-support",
                "fragment_freischutz_black_round",
            )
            .replace("single-target-conversion", "upkeep-conversion")
            .replace("capacity: 100", "capacity: 25");
    // The base pack must already satisfy its cross-catalog penalty contract
    // before the composition overlay replaces this same skill definition.
    let path = "official/data/skills/base.ron";
    let mut skills: crate::game::data::skill_data::SkillDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let native = crate::authoring::composition::compile_yaml(&source)
        .unwrap()
        .load_native()
        .unwrap();
    skills
        .skills
        .retain(|s| !native.skills.iter().any(|new| new.id == s.id));
    skills.skills.extend(native.skills);
    fixture.write(path, ron::ser::to_string(&skills).unwrap());
    let cancel = CancellationToken::default();
    let mut probe = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&document),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    probe.run_to_end(&cancel).unwrap();
    let last = preparation + activate_index - 1;
    let result = probe.session().step(last).unwrap().public_result().unwrap();
    let cp = &result["payload"]["battle_update"]["checkpoint"];
    let ally = probe.session().step(10).unwrap().public_result().unwrap()["payload"]
        ["employee_uuid"]
        .clone();
    let target = cp["units"]
        .as_array()
        .unwrap()
        .iter()
        .position(|u| u["unit_source"]["employee_uuid"] == ally)
        .expect("actual deployed recipient");
    let preview = probe.session().step(16).unwrap().public_result().unwrap();
    assert_eq!(
        preview["payload"]["support_conversions"][0]["conversion"]["replacement"]["RegularBasic"]
            ["health_upkeep"]["max_health_percent"],
        10,
        "{preview}"
    );
    assert_eq!(
        probe.session().step(17).unwrap().public_result().unwrap()["payload"]["copies"][0]
            ["variant"],
        "Corrupt"
    );
    let mut begin = authority_step("begin_skill_targeting", last);
    begin["command"]["employee_uuid"] = Value::Null;
    begin["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    begin["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut activate = authority_step("activate_skill", last + 1);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["command"]["target"] = json!({"type":"Unit","unit_instance_id":null});
    activate["bindings"].as_array_mut().unwrap().extend([
        json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),
        json!({"from":{"step":last,"pointer":format!("/payload/battle_update/checkpoint/units/{target}/unit_instance_id")},"to":"/command/target/unit_instance_id","value_type":"string"}),
    ]);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .extend([begin, activate]);
    for _ in 0..10 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    drop(probe);
    let mut expected = Vec::new();
    for percent in [10, 20] {
        let current = source.replace(
            "max_health_percent: 10",
            &format!("max_health_percent: {percent}"),
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
            let pending = trial.session().next_step_index();
            trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("upkeep World step {pending}: {e:?}"));
        }
        let observation = trial.session().observation().unwrap();
        let active = observation.active().expect("short channel remains active");
        let cost = active
            .latest_nonzero_sourced_hp()
            .expect("actual HP payment");
        let authored = cost.authored_source().unwrap();
        let origin = trial.origin_for_observed(authored).unwrap();
        assert!(origin.inputs.iter().any(|p| p == "values[0].value"));
        let crate::authoring::ApplicationChange::Hp {
            before,
            after,
            reason,
        } = cost.change()
        else {
            panic!("expected HP cost")
        };
        assert_eq!(
            reason,
            crate::game::battle::event_log::HpChangeReason::HealthCost
        );
        assert_eq!(cost.source_unit(), Some(cost.target_unit()));
        assert_eq!(before - after, if percent == 10 { 18 } else { 36 });
        expected.push(json!({"percent":percent,"time_ms":cost.time_ms(),"source_unit":cost.source_unit(),"target_unit":cost.target_unit(),"before":before,"after":after,"reason":reason,"origin":origin}));
    }
    eprintln!("channel upkeep World: {expected:?}");
    if let Some(directory) = std::env::var_os("CHANNEL_UPKEEP_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
