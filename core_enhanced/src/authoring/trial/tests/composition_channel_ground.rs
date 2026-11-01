use super::*;

#[test]
fn composition_channel_ground_world_reward_equipment_membership_and_expiry() {
    let (fixture, mut document) =
        duration_modifiers::modifier_reward_world_fixture("long-breath", false);
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut range = relics.relics[0].clone();
    range.id = "widened-horizon".into();
    range.uuid = uuid::Uuid::from_u128(0xd241);
    relics.relics.push(range);
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    // Cover the isolated encounter with an explicit test mask. This is not the
    // unresolved release geometry of N08, and no extra unit is created.
    let source=include_str!("../../../../tests/fixtures/skill_composition/channel_ground.skill.yaml")
        .replace("authored-channel-ground","fragment_freischutz_black_round")
        .replace("capacity: 100","capacity: 25")
        .replace("rows: [XXX, X@X, XXX]","rows: [XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXX@XXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX]");
    for _ in 0..2 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    let mut previous_trial: Option<CompositionTrial> = None;
    for percent in [25u64, 50] {
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
        let mut owner = None;
        let mut declared = None;
        let mut checkpoints = Vec::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("ground World step {pending}: {e:?}"))
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
                assert_eq!(
                    payload["loadouts"][0]["employee_uuid"],
                    owner.clone().unwrap()
                );
                assert_eq!(
                    payload["loadouts"][0]["slots"][2],
                    payload["copies"][0]["copy_uuid"]
                );
            }
            if index < 29 {
                continue;
            }
            let update = &result["payload"]["battle_update"];
            for entry in update["events_delta"]["events"].as_array().unwrap() {
                let event = &entry["event"];
                if event["type"] == "SkillAreaDeclared"
                    && event["skill_id"] == "fragment_freischutz_black_round"
                {
                    assert_eq!(
                        event["duration_ms"],
                        json!((2001 * (100 + percent)).div_ceil(100))
                    );
                    assert_eq!(event["start_time_ms"], 23001);
                    declared = Some(event.clone());
                }
            }
            let cp = &update["checkpoint"];
            checkpoints
                .push(json!({"step":index,"time_ms":cp["battle_time_ms"],"units":cp["units"]}));
        }
        let area = trial
            .session()
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .latest_area()
            .unwrap()
            .clone();
        assert_eq!(
            area.change,
            crate::authoring::AreaChange::Ended {
                reason: crate::game::ability::sustained::SustainedEndReason::Expired
            }
        );
        assert_eq!(area.started_at_ms, 23001);
        assert_eq!(area.time_ms, 23001 + (2001 * (100 + percent)).div_ceil(100));
        assert!(area.before.unwrap().contributions >= 3 && area.after.is_none());
        if let Some(previous) = &previous_trial {
            let (file, _, _) = previous.latest_area_origin().unwrap();
            assert_eq!(file, previous.identity().artifact_path);
            assert_ne!(file, trial.identity().artifact_path);
            assert_ne!(
                previous.identity().source_digest,
                trial.identity().source_digest
            );
        }
        let (file, path, origin) = trial.latest_area_origin().unwrap();
        assert_eq!(file, trial.identity().artifact_path);
        assert_eq!(path, "skills[0].steps[0].delivery");
        assert!(
            origin.definition.starts_with("definitions[0].body")
                && origin.definition.ends_with(".delivery"),
            "{origin:?}"
        );
        assert!(origin.inputs.iter().any(|p| p == "values[0].value"));
        assert!(origin.inputs.iter().any(|p| p == "values[1].value"));
        let declared = declared.expect("real World area declaration");
        let expiry = 23001 + (2001 * (100 + percent)).div_ceil(100);
        let during = checkpoints
            .iter()
            .find(|c| {
                c["time_ms"]
                    .as_u64()
                    .is_some_and(|t| t >= 24001 && t < expiry)
            })
            .expect("checkpoint after dwell");
        assert!(
            during["units"]
                .as_array()
                .unwrap()
                .iter()
                .any(|u| u["owner"] == "Opponent" && u["control"]["hard_cc_active"] == true),
            "World dwell never arrived: {during}"
        );
        let after = checkpoints
            .iter()
            .find(|c| c["time_ms"].as_u64().is_some_and(|t| t >= expiry))
            .expect("checkpoint after expiry");
        assert!(
            after["units"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|u| u["owner"] == "Opponent")
                .all(|u| u["control"]["hard_cc_active"] == false),
            "World area did not release stun: {after}"
        );
        previous_trial = Some(trial);
        expected.push(json!({"percent":percent,"declared":declared,"expiry":expiry,"checkpoints":checkpoints,"latest_area":area,"origin":origin}));
    }
    if let Some(path) = std::env::var_os("CHANNEL_GROUND_COMPOSITION_FIXTURE_DIR") {
        let path = std::path::PathBuf::from(path);
        assert!(!path.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &path.join("content"));
        std::fs::write(path.join("source.skill.yaml"), source).unwrap();
        std::fs::write(path.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":path.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            path.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}


#[test]
fn composition_channel_modifiers_world_reward_equip_and_progress_arrival() {
    let (fixture, document) = duration_modifiers::modifier_reward_world_fixture("tightening-shackles", false);
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    for (i, id) in ["long-breath", "widened-horizon", "unbroken-trance"].into_iter().enumerate() {
        let mut relic = relics.relics[0].clone();
        relic.id = id.into();
        relic.uuid = uuid::Uuid::from_u128(0xd250 + i as u128);
        relics.relics.push(relic);
    }
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    let source = include_str!("../../../../tests/fixtures/skill_composition/channel_modifiers.skill.yaml")
        .replace("authored-channel-ground", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("rows: [XXX, X@X, XXX]", "rows: [XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXX@XXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX]");
    // The catalogue checks all legal coequipment before a World exists. A
    // conflicting authored plan must fail there, not after giving a reward.
    let relic_source = fixture.read("official/data/relics/base.ron");
    for shared_arrival_rule in [false, true] {
        let mut invalid = crate::authoring::composition::parse_yaml(&source).unwrap();
        let mut second = invalid["definitions"][0]["body"]["Action"]["modifiers"][2].clone();
        second["id"] = json!("conflicting-progress");
        if shared_arrival_rule {
            let statuses = &mut invalid["values"][0]["value"]["Delivery"]["TileArea"]["area"]["ground_status"]["statuses"];
            let mut status = statuses[0].clone();
            status["definition"] = json!("another-progress");
            statuses.as_array_mut().unwrap().push(status);
            second["change"]["AreaRampRate"]["status"] = json!(3);
        }
        invalid["definitions"][0]["body"]["Action"]["modifiers"].as_array_mut().unwrap().push(second);
        let error = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &invalid.to_string(), checked(&document), None, TrialLimits::default(), &CancellationToken::default()).err().expect("conflicting writers reject catalogue publication");
        assert!(format!("{error:?}").contains("Multiple ramp-rate writers"), "{error:?}");
        assert_eq!(fixture.read("official/data/relics/base.ron"), relic_source);
    }
    let mut expected = Vec::new();
    for numerator in [3, 4] {
        let current = source.replace("Count: 3", &format!("Count: {numerator}"));
        let cancel = CancellationToken::default();
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current, checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        let mut owner = None;
        let mut checkpoints = Vec::new();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("progress World step {pending}: {e:?}")).index();
            let result = trial.session().step(index).unwrap().public_result().unwrap();
            if index == 9 { owner = Some(result["payload"]["employee_uuid"].clone()); }
            if index == 16 {
                assert_eq!(result["disposition"], "accepted");
                let payload = &result["payload"];
                assert_eq!(payload["copies"][0]["definition_id"], "tightening-shackles");
                assert_eq!(payload["loadouts"][0]["employee_uuid"], owner.clone().unwrap());
                assert_eq!(payload["loadouts"][0]["slots"][2], payload["copies"][0]["copy_uuid"]);
            }
            if index < 29 { continue; }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            let time = cp["battle_time_ms"].as_u64().unwrap();
            let opponents: Vec<_> = cp["units"].as_array().unwrap().iter().filter(|u| u["owner"] == "Opponent").collect();
            assert!(!opponents.is_empty());
            let arrived = time >= 23001 + (2000u64).div_ceil(numerator) && time < 25002;
            assert!(opponents.iter().all(|u| u["control"]["hard_cc_active"] == arrived), "ratio {numerator}/2 @ {time}: {opponents:?}");
            checkpoints.push(json!({"step":index,"time_ms":time,"units":cp["units"]}));
        }
        let area = trial.session().observation().unwrap().active().unwrap().latest_area().unwrap().clone();
        assert_eq!(area.started_at_ms, 23001);
        assert_eq!(area.expires_at_ms, 25002); // Unequipped long-breath must not apply.
        assert_eq!(area.time_ms, 25002);
        assert_eq!(area.change, crate::authoring::AreaChange::Ended { reason: crate::game::ability::sustained::SustainedEndReason::Expired });
        let (file, field, origin) = trial.latest_area_origin().unwrap();
        assert_eq!(file, trial.identity().artifact_path);
        assert_eq!(field, "skills[0].steps[0].delivery");
        for input in ["values[3].value", "values[4].value", "skills[0].modifiers[0]"] {
            assert!(origin.inputs.iter().any(|p| p == input));
        }
        expected.push(json!({"numerator":numerator,"denominator":2,"arrival":23001+2000u64.div_ceil(numerator),"latest_area":area,"checkpoints":checkpoints,"origin":origin}));
    }
    if let Some(path) = std::env::var_os("CHANNEL_MODIFIERS_COMPOSITION_FIXTURE_DIR") {
        let path = std::path::PathBuf::from(path);
        assert!(!path.exists(), "preserve earlier evidence");
        copy_fixture(fixture.root(), &path.join("content"));
        std::fs::write(path.join("source.skill.yaml"), source).unwrap();
        std::fs::write(path.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":path.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(path.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[test]
fn composition_control_vulnerability_world_reward_equip_and_independent_lifetime() {
    let (fixture, mut document) = duration_modifiers::modifier_reward_world_fixture("broken-balance", false);
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    for (i,id) in ["long-breath","widened-horizon","unbroken-trance","tightening-shackles"].into_iter().enumerate() {
        let mut r = relics.relics[0].clone();
        r.id = id.into(); r.uuid = uuid::Uuid::from_u128(0xd260+i as u128);
        relics.relics.push(r);
    }
    fixture.write(path,ron::ser::to_string(&relics).unwrap());
    let source = include_str!("../../../../tests/fixtures/skill_composition/control_vulnerability.skill.yaml")
        .replace("authored-channel-ground","fragment_freischutz_black_round")
        .replace("capacity: 100","capacity: 25")
        .replace("rows: [XXX, X@X, XXX]","rows: [XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXX@XXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX, XXXXXXXXXXXXXXX]");
    for _ in 0..14 { document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}})); }
    let relic_source = fixture.read(path);
    // This complete six-relic publication is slightly larger than the embedded
    // authoring budget. Preserve its refusal, then explicitly select a finite
    // host profile. The source/scenario cannot choose or raise this setting.
    use crate::runtime_safety::{RuntimeSafetySettings, EMBEDDED_RUNTIME_SAFETY_RON};
    let settings_text = std::str::from_utf8(EMBEDDED_RUNTIME_SAFETY_RON).unwrap()
        .replace("authoring_execution: (\n        max_work: 10000000",
                 "authoring_execution: (\n        max_work: 20000000");
    let settings = std::sync::Arc::new(RuntimeSafetySettings::from_ron(
        "explicit composition World test host", settings_text.as_bytes()).unwrap());
    assert_eq!(settings.authoring_execution().max_work, 20_000_000);
    assert_eq!(settings.native_runtime().max_work, 10_000_000);
    assert_ne!(settings.effective_sha256(), RuntimeSafetySettings::embedded().effective_sha256());
    let tiny_text = settings_text.replace("max_work: 20000000", "max_work: 1");
    let tiny = std::sync::Arc::new(RuntimeSafetySettings::from_ron(
        "explicit refusing host", tiny_text.as_bytes()).unwrap());
    let refused = CompositionTrial::prepare_with_runtime_safety(fixture.root(), &fixture.selection(), &source,
        checked(&document), None, TrialLimits::default(), &CancellationToken::default(), tiny)
        .err().expect("explicit admission limit must govern captured base");
    assert!(matches!(refused, crate::authoring::composition::CompositionTrialFailure::Content(
        AuthoringError::Execution { diagnostic: crate::authoring::ExecutionFailure {
            code: "authoring_work_limit", phase: crate::authoring::ExecutionPhase::Acquisition,
            limit: Some(1), ..
        }})), "{refused:?}");
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    let refused = CompositionTrial::prepare_with_runtime_safety(fixture.root(), &fixture.selection(), &source,
        checked(&document), None, TrialLimits::default(), &cancelled, settings.clone())
        .err().expect("explicit host selection cannot bypass cancellation");
    assert!(matches!(refused, crate::authoring::composition::CompositionTrialFailure::Content(
        AuthoringError::Execution { diagnostic: crate::authoring::ExecutionFailure {
            code: "authoring_cancelled", ..
        }})), "{refused:?}");
    let refusal = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &source,
        checked(&document), None, TrialLimits::default(), &CancellationToken::default())
        .err().expect("the original default-budget refusal remains visible");
    assert!(matches!(refusal, crate::authoring::composition::CompositionTrialFailure::Content(
        AuthoringError::Execution { diagnostic: crate::authoring::ExecutionFailure {
            code: "authoring_work_limit", phase: crate::authoring::ExecutionPhase::GeneratedSetup,
            limit: Some(10_000_000), ..
        }})), "{refusal:?}");
    assert_eq!(fixture.read(path), relic_source);
    let mut invalid = crate::authoring::composition::parse_yaml(&source).unwrap();
    let mut second = invalid["definitions"][0]["body"]["Action"]["modifiers"][3].clone();
    second["id"] = json!("conflicting-weakness");
    invalid["definitions"][0]["body"]["Action"]["modifiers"].as_array_mut().unwrap().push(second);
    let error = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &invalid.to_string(), checked(&document), None, TrialLimits::default(), &CancellationToken::default()).err().expect("duplicate weakness writers reject before World publication");
    assert!(format!("{error:?}").contains("Multiple ground vulnerability writers"), "{error:?}");
    assert_eq!(fixture.read(path),relic_source);
    let mut invalid = crate::authoring::composition::parse_yaml(include_str!("../../../../tests/fixtures/skill_composition/freeze_vulnerability.skill.yaml")).unwrap();
    invalid["skills"][0]["id"] = json!("fragment_freischutz_black_round");
    invalid["skills"][0]["admission"]["capacity"] = json!(25);
    let mut second = invalid["definitions"][0]["body"]["Action"]["modifiers"][0].clone();
    second["id"] = json!("conflicting-direct-weakness");
    invalid["definitions"][0]["body"]["Action"]["modifiers"].as_array_mut().unwrap().push(second);
    let error = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &invalid.to_string(), checked(&document), None, TrialLimits::default(), &CancellationToken::default()).err().expect("duplicate direct writers reject before World publication");
    assert!(format!("{error:?}").contains("Multiple vulnerability writes"), "{error:?}");
    assert_eq!(fixture.read(path),relic_source);
    let mut expected = Vec::new();
    for percent in [15,30] {
        let current = source.replace("Count: 15", &format!("Count: {percent}"));
        let cancel = CancellationToken::default();
        let mut trial = CompositionTrial::prepare_with_runtime_safety(fixture.root(), &fixture.selection(), &current, checked(&document), None, TrialLimits::default(), &cancel, settings.clone()).unwrap();
        assert_eq!(trial.session().state.content.token().database().runtime_safety().effective_sha256(), settings.effective_sha256());
        assert_eq!(trial.session().state.limits.execution.max_work, 10_000_000);
        let mut copy = None;
        let mut owner = None;
        let mut active = Vec::new();
        let mut observed_after_area = false;
        let mut observed_after_weakness = false;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("weakness World step {pending}: {e:?}")).index();
            let result = trial.session().step(index).unwrap().public_result().unwrap();
            if index == 9 { owner = Some(result["payload"]["employee_uuid"].clone()); }
            if index == 16 {
                assert_eq!(result["disposition"],"accepted");
                let payload = &result["payload"];
                assert_eq!(payload["copies"][0]["definition_id"],"broken-balance");
                copy = Some(payload["copies"][0]["copy_uuid"].clone());
                assert_eq!(payload["loadouts"][0]["employee_uuid"],owner.clone().unwrap());
                assert_eq!(payload["loadouts"][0]["slots"][2],copy.clone().unwrap());
            }
            if index < 29 { continue; }
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            let time = cp["battle_time_ms"].as_u64().unwrap();
            let effects: Vec<_> = cp["sustained_effects"]["active"].as_array().unwrap().iter().filter(|r| r["hit_status"]["modifier_source"]["definition"] == "broken-balance").collect();
            if time < 24001 || time >= 29001 { assert!(effects.is_empty(), "wrong application time {time}: {effects:?}"); }
            else {
                assert!(!effects.is_empty(), "missing weakness at {time}");
                for r in &effects {
                    assert_eq!(r["expires_at_ms"],29001);
                    assert_eq!(r["hit_status"]["kind"],json!({"DamageTaken":{"percent":percent}}));
                    assert_eq!(r["hit_status"]["modifier_source"]["copy_uuid"],copy.clone().unwrap());
                }
                let obs = trial.session().observation().unwrap().active().unwrap();
                let observed = obs.latest_sourced_sustained().unwrap();
                assert_eq!(observed.time_ms(), 24001);
                assert!(matches!(observed.change(), crate::authoring::ApplicationChange::Sustained { ended: None, expires_at_ms: 29001, .. }));
                assert!(trial.latest_sustained_area_origin().is_some(), "actual World producer: {:?}", observed.area_source());
                if time >= 25002 { observed_after_area = true; }
                active.push(json!({"step":index,"time_ms":time,"effects":effects}));
            }
            if time >= 25002 { assert!(cp["units"].as_array().unwrap().iter().filter(|u|u["owner"]=="Opponent").all(|u|u["control"]["hard_cc_active"]==false)); }
            if time >= 29001 { observed_after_weakness = true; }
        }
        assert!(observed_after_area && observed_after_weakness);
        let observation = trial.session().observation().unwrap().active().unwrap();
        let effect = observation.latest_sourced_sustained().unwrap();
        assert_eq!(effect.time_ms(), 29001);
        assert!(matches!(effect.change(), crate::authoring::ApplicationChange::Sustained { ended: Some(crate::game::ability::sustained::SustainedEndReason::Expired), expires_at_ms: 29001, .. }));
        assert!(effect.authored_source().is_none(), "Delivery producer must not invent an EffectSite");
        let producer = effect.area_source().unwrap();
        assert_eq!(producer.status_index, 1);
        // This World loadout has one active fragment provider. Expect the
        // captured identity independently of the formatter/source lookup.
        assert_eq!(producer.definition, "broken-balance:action-0-n1:broken-balance");
        assert_eq!(producer.modifier_source.as_ref().unwrap().rule, "action-0-n1:broken-balance");
        assert_eq!(producer.modifier_source.as_ref().unwrap().definition, "broken-balance");
        assert_eq!(json!(producer.modifier_source.as_ref().unwrap().copy_uuid), copy.clone().unwrap());
        assert_area_origin_claims(&trial, producer);
        let (file, field, finite_origin) = trial.latest_sustained_area_origin().unwrap();
        assert_eq!(file, trial.identity().artifact_path);
        assert_eq!(field, "skills[0].steps[0].delivery");
        for i in [5,6] { assert!(finite_origin.inputs.contains(&format!("values[{i}].value"))); }
        let area = observation.latest_area().unwrap().clone();
        assert_eq!(producer.area_id, area.area_id);
        assert_eq!(area.time_ms,25002);
        let (_,_,origin) = trial.latest_area_origin().unwrap();
        for i in [5,6] { assert!(origin.inputs.contains(&format!("values[{i}].value"))); }
        expected.push(json!({"percent":percent,"copy":copy,"active":active,"latest_area":area,"origin":origin,"observation":trial.session().observation().unwrap().active().unwrap()}));
    }
    if let Some(path) = std::env::var_os("CONTROL_VULNERABILITY_COMPOSITION_FIXTURE_DIR") {
        let path = std::path::PathBuf::from(path);
        assert!(!path.exists(),"preserve earlier evidence");
        copy_fixture(fixture.root(),&path.join("content"));
        std::fs::write(path.join("source.skill.yaml"),source).unwrap();
        std::fs::write(path.join("runtime-safety.ron"), settings_text).unwrap();
        std::fs::write(path.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":path.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(path.join("expected.json"),serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

// Query-validation fixtures do not claim that the synthetic occurrence ran.
// The real occurrence/copy test above remains the execution evidence.
fn assert_area_origin_claims(trial: &CompositionTrial, producer: &crate::authoring::view::AreaEffectSource) {
    // This tests static claim validation, not runtime occurrence authority.
    // The execution caller supplies a real observed producer; the isolated
    // query caller supplies a declared producer without claiming it executed.
    assert!(trial.observed_area_origin(producer).is_some());
    let mut invalid = Vec::new();
    let mut mixed = producer.clone();
    mixed.status_index = 0; // Existing slow, not the stun vulnerability.
    invalid.push(("other_status", mixed));
    let mut mixed = producer.clone();
    mixed.status_index = usize::MAX;
    invalid.push(("absent_status", mixed));
    let mut mixed = producer.clone();
    mixed.definition = "unrelated-vulnerability".into();
    invalid.push(("other_definition", mixed));
    let mut mixed = producer.clone();
    mixed.modifier_source.as_mut().unwrap().definition = "long-breath".into();
    invalid.push(("other_modifier_definition", mixed));
    let mut mixed = producer.clone();
    mixed.modifier_source.as_mut().unwrap().rule = "lifetime".into();
    invalid.push(("other_modifier_rule", mixed));
    let mut mixed = producer.clone();
    mixed.modifier_source = None;
    invalid.push(("missing_modifier", mixed));
    let mut mixed = producer.clone();
    mixed.generation = None;
    invalid.push(("missing_generation", mixed));
    let mut mixed = producer.clone();
    mixed.step_id = "absent-step".into();
    invalid.push(("absent_step", mixed));
    for rule in [
        "n1:broken-balance", // No captured provider qualifier.
        "action-01-n1:broken-balance", // Noncanonical provider spelling.
        "action-+0-n1:broken-balance",
        "action--1-n1:broken-balance",
        "action-999999999999999999999999999999999999-n1:broken-balance",
        "action-0-n2:broken-balance", // Another action.
        "action-0-n1:missing-rule",
        "action-0-",
        "other-0-n1:broken-balance",
    ] {
        let mut mixed = producer.clone();
        mixed.definition = format!("broken-balance:{rule}");
        mixed.modifier_source.as_mut().unwrap().rule = rule.into();
        invalid.push((rule, mixed));
    }
    let accepted: Vec<_> = invalid.iter()
        .filter(|(_, claim)| trial.observed_area_origin(claim).is_some())
        .map(|(label, _)| *label).collect();
    assert!(accepted.is_empty(), "area origin accepted mixed claims: {accepted:?}");
    assert!(trial.observed_area_origin(producer).is_some());

}

#[test]
fn integration_area_origin_rejects_mixed_registered_producer_claims() {
    let (fixture, _, document) = state_world_fixture();
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    let template: crate::game::data::relic_data::RelicDatabase = ron::de::from_str(r#"(relics:[(id:"area-verifier-template",uuid:"00000000-0000-0000-0000-00000000daff",name:"Area verifier selector",description:"Test selector",rarity:"fixture",allow_duplicates:true,effects:[(id:"unrelated_damage",selector:(all_tags:["unrelated_fixture_damage"]),percent:1)],dismantle_equipment_dust:1)])"#).unwrap();
    for (i, id) in ["long-breath", "widened-horizon", "tightening-shackles", "unbroken-trance", "broken-balance"].into_iter().enumerate() {
        let mut relic = template.relics[0].clone();
        relic.id = id.into();
        relic.uuid = uuid::Uuid::from_u128(0xda00 + i as u128);
        relics.relics.push(relic);
    }
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    let source = include_str!("../../../../tests/fixtures/skill_composition/control_vulnerability.skill.yaml");
    let artifact = crate::authoring::composition::compile_yaml(source).unwrap();
    let native = artifact.load_native().unwrap();
    let skill = &native.skills[0];
    let rule = skill.action_policy.as_ref().unwrap().modifiers.iter()
        .find(|r| r.relic_definition == "broken-balance").unwrap();
    let trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), source,
        checked(&document), None, TrialLimits::default(), &CancellationToken::default()).unwrap();
    let producer = crate::authoring::view::AreaEffectSource {
        generation: Some(std::sync::Arc::new(trial.identity().content.clone())),
        captured_cast: None,
        area_id: uuid::Uuid::from_u128(1),
        skill_id: "authored-channel-ground".into(),
        step_id: skill.steps[0].id.clone(),
        cast_sequence: 1,
        status_index: 1,
        // Independent expectation: the fixture's first provider and the first
        // expanded Action after its Call. Do not manufacture this with the
        // production rule formatter or the lookup being tested.
        definition: "broken-balance:action-0-n1:broken-balance".into(),
        modifier_source: Some(crate::game::ability::sustained::HitModifierSource {
            copy_uuid: uuid::Uuid::from_u128(2), definition: "broken-balance".into(),
            rule: "action-0-n1:broken-balance".into(),
        }),
    };
    assert_eq!(rule.id, "n1:broken-balance", "fixture's independently enumerated Call/Action address");
    let mut naked = producer.clone();
    naked.definition = "broken-balance:n1:broken-balance".into();
    naked.modifier_source.as_mut().unwrap().rule = "n1:broken-balance".into();
    let real_rule = trial.observed_area_origin(&producer).is_some();
    let absent_provider = trial.observed_area_origin(&naked).is_some();
    eprintln!("area rule verifier: runtime={real_rule}, missing_provider={absent_provider}");
    assert!(real_rule && !absent_provider, "query must accept the actual contribution and reject an unqualified authored rule");
    let (_, field, origin) = trial.observed_area_origin(&producer).unwrap();
    assert_eq!(field, "skills[0].steps[0].delivery");
    assert_eq!(origin.definition, "definitions[0].body.delivery");
    assert!(origin.inputs.contains(&"values[5].value".to_owned()));
    assert!(origin.inputs.contains(&"values[6].value".to_owned()));
    assert_area_origin_claims(&trial, &producer);
    assert_hit_counter_response_claims(&trial, &producer);
    let mut imported = crate::authoring::composition::Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert!(trial.session().composition_area_effect_origin(&imported, &producer).is_none());
    imported.verify_yaml_source(source).unwrap();
    assert!(trial.session().composition_area_effect_origin(&imported, &producer).is_some());
    let other = crate::authoring::composition::compile_yaml(&format!("{source}\n# other original bytes\n")).unwrap();
    assert!(trial.session().composition_area_effect_origin(&other, &producer).is_none());
    assert!(trial.session().composition_area_effect_origin(&artifact, &producer).is_some());

    // A native finite vulnerability has no modifier source. Its absence is
    // legitimate, unlike dropping the identity of a modifier-created effect.
    let mut direct = crate::authoring::composition::parse_yaml(source).unwrap();
    direct["definitions"][0]["body"]["Action"]["modifiers"] = json!([]);
    direct["skills"][0]["modifiers"] = json!([]);
    direct["values"][0]["value"]["Delivery"]["TileArea"]["area"]["ground_status"]["statuses"][1]["vulnerability"] =
        json!({"definition":"native-weakness","percent":15,"duration_ms":5000});
    let direct = direct.to_string();
    let direct_artifact = crate::authoring::composition::compile_yaml(&direct).unwrap();
    let native_trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &direct,
        checked(&document), None, TrialLimits::default(), &CancellationToken::default()).unwrap();
    let mut native_producer = producer.clone();
    native_producer.generation = Some(std::sync::Arc::new(native_trial.identity().content.clone()));
    native_producer.definition = "native-weakness".into();
    native_producer.modifier_source = None;
    let (_, field, origin) = native_trial.observed_area_origin(&native_producer).unwrap();
    assert_eq!(field, "skills[0].steps[0].delivery");
    assert_eq!(origin.definition, "definitions[0].body.delivery");
    assert!(origin.inputs.contains(&"values[0].value".to_owned()));
    let mut invalid = native_producer.clone();
    invalid.modifier_source = producer.modifier_source.clone();
    assert!(native_trial.observed_area_origin(&invalid).is_none());
    let mut invalid = native_producer.clone();
    invalid.definition = "residence-stun".into();
    assert!(native_trial.observed_area_origin(&invalid).is_none());
    let mut invalid = native_producer.clone();
    invalid.status_index = 0;
    assert!(native_trial.observed_area_origin(&invalid).is_none());
    assert!(native_trial.observed_area_origin(&producer).is_none());
    assert!(trial.observed_area_origin(&native_producer).is_none());
    assert!(native_trial.session().composition_area_effect_origin(&direct_artifact, &native_producer).is_some());
}


// This exercises the actual response projection with a registered area and a
// deliberately inconsistent observation. It does not claim a synthetic hit ran.
fn assert_hit_counter_response_claims(trial: &CompositionTrial, producer: &crate::authoring::view::AreaEffectSource) {
    use crate::game::{
        ability::{authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource}, sustained::{HitCounterOutcome, ThresholdControlKind}},
        battle::{core::authoring_observation::{ApplicationWitness, MandatoryObservation}, ids::UnitInstanceId},
        world::authoring_observation::TrialObservationView,
    };
    let source = EffectSource {
        source_file: trial.identity().artifact_path.clone(),
        site: EffectSite {
            action: ActionSite { definition_id: "authored-channel-ground".into(), action_id: "not-registered".into() },
            effect_id: "not-registered".into(),
        },
        field_path: "skills[0].steps[0].effects[999]".into(),
        origin_definition_id: "authored-channel-ground".into(),
        origin: EffectOrigin::Original,
    };
    assert!(trial.origin_for(&source).is_none());
    let area_origin = trial.observed_area_origin(producer).expect("registered area lookup reaches a valid alternative");
    assert_eq!(area_origin.1, "skills[0].steps[0].delivery");
    assert_eq!(area_origin.2.definition, "definitions[0].body.delivery");
    let outcome = HitCounterOutcome {
        before: 4, added: 1, after: 0, threshold: 5,
        control: ThresholdControlKind::Stun, triggered: true, control_applied: true,
        counter_expires_at_ms: None, control_expires_at_ms: Some(1123),
    };
    // Retry absence after the malformed claim. The valid fallback cannot be
    // used to launder a present-but-invalid authored source.
    for has_claim in [false, true, false] {
        let mut observed = MandatoryObservation::default();
        observed.latest_sourced_hit_counter = Some(ApplicationWitness {
                ordinal: 1, time_ms: 123, source_unit: Some(UnitInstanceId::from(uuid::Uuid::from_u128(1))),
                target_unit: UnitInstanceId::from(uuid::Uuid::from_u128(2)), fragment: None,
                change: crate::authoring::ApplicationChange::HitCounter { outcome },
                source: None, authored: has_claim.then(|| std::sync::Arc::new(source.clone())),
                area_source: Some(std::sync::Arc::new(producer.clone())), equipment: None,
            });
        let view = crate::authoring::view::ObservationView(TrialObservationView {
            run_resources: None, active_typed_resources: None, active_runtime_facts: None,
            active: Some(&observed), last_terminal: None,
        });
        let calls = std::cell::Cell::new(0);
        let response = crate::authoring::composition::hit_counter_response(
            view.active().unwrap().latest_sourced_hit_counter().unwrap(), false,
            Some(area_origin.clone()), |claim| {
                calls.set(calls.get() + 1);
                trial.origin_for_observed(claim).cloned()
            },
        ).unwrap();
        let wire = serde_json::to_value(response).unwrap();
        assert_eq!(calls.get(), usize::from(has_claim));
        assert_eq!(wire["time_ms"], 123);
        assert_eq!(wire["before"], 4);
        assert_eq!(wire["control_expires_at_ms"], 1123);
        if has_claim {
            assert!(wire["origin"].is_null(), "invalid authored source must not inherit the valid area origin: {wire}");
            assert_eq!(wire["field_path"], "skills[0].steps[0].effects[999]");
        } else {
            assert_eq!(wire["origin"]["definition"], "definitions[0].body.delivery");
            assert_eq!(wire["field_path"], "skills[0].steps[0].delivery");
        }
    }
}
