use super::*;
use crate::authoring::composition::canonical_area_tests as canonical;
use crate::game::data::content_pack::{package_official_with_skill_files, ContentSelection};

#[test]
fn canonical_area_woodsman_real_encounter_grant_equip_and_published_three_strikes() {
    published_world("woodsman");
}

#[test]
fn canonical_area_monk_real_encounter_grant_equip_and_published_burst_recovery() {
    published_world("clouded-monk");
}

fn published_world(family: &str) {
    let (fixture, _, mut document) = state_world_fixture();
    let metadata = canonical::catalog(family);
    let fragment_id = metadata.id.as_str().to_owned();
    let skill_id = canonical::resolved(family)["skills"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (encounter, abnormality) = if family == "woodsman" {
        (
            "suppress_warm_hearted_woodsman",
            "f-05-32_warm_hearted_woodsman",
        )
    } else {
        ("suppress_clouded_monk", "d-01-110_clouded_monk")
    };
    let path = "official/data/skill_fragments/base.ron";
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let current = fragments
        .fragments
        .iter_mut()
        .find(|f| f.id == metadata.id)
        .unwrap();
    assert_eq!(current.uuid, metadata.uuid);
    *current = metadata;
    fixture.write(path, ron::ser::to_string(&fragments).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: crate::game::data::reward_data::RewardDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for reward in &mut rewards.rewards {
        reward.effects.retain(|effect| {
            !matches!(effect,
            crate::game::reward::RewardEffect::GrantSkillFragment { fragment_id }
                if fragment_id.as_str() == "authoring_preview_fragment")
        });
    }
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    let path = "official/data/map/facility_templates.ron";
    let mut maps: crate::game::map::FacilityTemplateDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let opening = maps.layouts[0]
        .activities
        .iter_mut()
        .find(|a| a.id == "trial_intake_corridor")
        .unwrap();
    opening.kind_id = crate::game::map::MapNodeKindId::new("combat_elite");
    opening.payload = crate::game::map::MapNodePayload::Encounter {
        encounter_id: Some(encounter.into()),
    };
    fixture.write(path, ron::ser::to_string(&maps).unwrap());
    let path = "official/data/abnormalities/base.ron";
    let mut abnormalities: crate::game::data::abnormality_data::AbnormalityDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = abnormalities
        .items
        .iter_mut()
        .find(|a| a.id == abnormality)
        .unwrap();
    // Bound the acceptance setup without rewriting encounter identity, waves,
    // terminal policies, or the authoritative encounter-origin reward source.
    enemy.max_health = 70;
    enemy.attack = 0;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for relic in canonical::relics().relics {
        assert!(relics.get_by_id(&relic.id).is_none());
        relics.relics.push(relic);
    }
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    fixture.write(
        "official/data/bundle.ron",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../game_resources/data/bundle.ron"
        )),
    );

    let prep = document["preparation_steps"].as_array().unwrap().len();
    if family == "woodsman" {
        // This encounter approaches the opening employee from (3, 6), to the
        // right of its (2, 6) cell. The generic fixture faces left and cannot
        // attack the primary abnormality after it reaches the adjacent cell.
        document["steps"][9 - prep]["command"]["facing"] = json!("right");
    }
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - prep);
    let mut activate = authority_step("activate_skill", 24);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!(skill_id);
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    document["steps"].as_array_mut().unwrap().push(activate);
    for duration in [1, 100, 100, 100] {
        document["steps"].as_array_mut().unwrap().push(
            json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":duration,"after_seq":0}}),
        );
    }
    let source_path = "data/skills/canonical-area.skill.yaml";
    fixture.write(
        &format!("official/{source_path}"),
        canonical::original(family),
    );
    let rejected = fixture.root().join("unresolved");
    let error = package_official_with_skill_files(
        &fixture.root().join("official"),
        &rejected,
        &[source_path.into()],
        &Default::default(),
    )
    .unwrap_err();
    assert!(format!("{error:?}").contains("Unresolved"));
    assert!(!rejected.exists());

    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    let mut digests = Vec::new();
    for edit in [false, true] {
        let mut source = canonical::resolved(family);
        if edit {
            // Change one authored scalar only. No definition/copy identity or
            // runtime code changes between the two published generations.
            source["values"][0]["value"]["Amount"]["root"]["Ratio"]["numerator"]["Mul"]["scalar"]
                ["Const"]["value"] = json!(if family == "woodsman" { 150 } else { 200 });
        }
        let source = serde_json::to_string_pretty(&source).unwrap();
        let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
        let name = if edit {
            "published-edited"
        } else {
            "published-original"
        };
        fixture.write(&format!("official/{source_path}"), &source);
        let receipt = package_official_with_skill_files(
            &fixture.root().join("official"),
            &fixture.root().join(name),
            &[source_path.into()],
            &Default::default(),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(fixture.root().join(name).join(source_path)).unwrap(),
            source
        );
        let selection = ContentSelection {
            packs: vec![name.into()],
            winners: vec![],
        };
        let content =
            crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
                .unwrap();
        assert_eq!(content.token().identity(), &receipt.content_identity);
        digests.push(receipt.content_identity);
        let mut trial =
            TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
                .unwrap();
        while trial.next_step_index() < 13 {
            trial.advance(&cancel).unwrap();
        }
        assert_eq!(
            trial.step(12).unwrap().public_result().unwrap()["payload"]["finished"],
            true,
            "the actual encounter must terminate before claiming its reward"
        );
        trial.run_to_end(&cancel).unwrap_or_else(|e| {
            panic!(
                "{family} actual World failed at step {}: {e:?}; previous={:?}",
                trial.next_step_index(),
                trial
                    .step(trial.next_step_index().saturating_sub(1))
                    .and_then(|s| s.public_result())
            )
        });
        let grant =
            &trial.step(13).unwrap().public_result().unwrap()["payload"]["skill_fragment_diffs"][0];
        assert_eq!(grant["fragment_id"], fragment_id);
        assert_eq!(
            trial.step(14).unwrap().public_result().unwrap()["disposition"],
            "accepted"
        );
        let before = &trial.step(24).unwrap().public_result().unwrap()["payload"]["battle_update"]
            ["checkpoint"];
        let player = before["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
            .unwrap();
        let ready = &before["deployment"]["deployed_units"][0]["skill_readiness"];
        assert_eq!(ready["skill_id"], skill_id);
        assert_eq!(
            trial.step(25).unwrap().public_result().unwrap()["disposition"],
            "accepted"
        );
        let mut damages = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for index in 26..30 {
            let reply = trial.step(index).unwrap().public_result().unwrap();
            // after_seq=0 returns cumulative events, not just this clock's
            // impacts. Count each real event once, retaining acceptance order.
            for row in reply["payload"]["battle_update"]["events_delta"]["events"]
                .as_array()
                .unwrap()
            {
                let event = &row["event"];
                if event["type"] == "HpChanged"
                    && event["damage_source"] == "Ability"
                    && event["source_instance_id"] == player["unit_instance_id"]
                    && row["time_ms"].as_u64().unwrap()
                        >= before["battle_time_ms"].as_u64().unwrap()
                    && seen.insert(row["seq"].as_u64().unwrap())
                {
                    damages.push(event["raw_damage"].as_u64().unwrap());
                }
            }
        }
        assert_eq!(
            damages.len(),
            if family == "woodsman" { 3 } else { 1 },
            "{family} damage={damages:?}"
        );
        assert_eq!(
            damages[0],
            match (family, edit) {
                ("woodsman", false) => 12,
                ("woodsman", true) => 18,
                (_, false) => 18,
                (_, true) => 24,
            }
        );
        if family == "woodsman" {
            assert_eq!(damages[1], 12);
            assert!(damages[2] >= 24);
        }
        let observed = trial.observation().unwrap().active().unwrap();
        let latest = observed.latest_nonzero_sourced_hp().unwrap();
        let origin = latest.authored_source().unwrap();
        assert_eq!(origin.source_file(), format!("{name}/{source_path}"));
        assert_eq!(origin.definition_id(), skill_id);
        let fragment = serde_json::to_value(latest).unwrap()["fragment"].clone();
        assert_eq!(fragment["copy_uuid"], grant["copy_uuid"]);
        assert_eq!(fragment["definition_id"], fragment_id);
        let trace = trial.composition_origin(&artifact, origin.source()).unwrap();
        // Same published filename/site and identical gameplay data, but a
        // different original byte version must not explain this observation.
        let other_original = crate::authoring::composition::compile_yaml(&format!("{source}\n")).unwrap();
        assert_ne!(other_original.source_digest(), artifact.source_digest());
        assert!(trial.composition_origin(&other_original, origin.source()).is_none());
        let mut mixed = origin.source().clone();
        mixed.field_path.push_str(".unknown_native_field");
        assert!(trial.composition_origin(&artifact, &mixed).is_none());
        assert!(
            trace
                .definition
                .starts_with("definitions[0].body.effects[0]")
                || trace
                    .definition
                    .starts_with("definitions[1].body.effects[0]")
        );
        let mut recovery = None;
        if family == "clouded-monk" {
            let final_cp = &trial.step(29).unwrap().public_result().unwrap()["payload"]
                ["battle_update"]["checkpoint"];
            let after = final_cp["units"]
                .as_array()
                .unwrap()
                .iter()
                .find(|u| u["unit_instance_id"] == player["unit_instance_id"])
                .unwrap();
            let hp_before = player["stats"]["current_health"].as_u64().unwrap();
            let hp_after = after["stats"]["current_health"].as_u64().unwrap();
            assert!(
                hp_before < player["stats"]["max_health"].as_u64().unwrap(),
                "recovery test must have real missing HP"
            );
            assert!(
                hp_after > hp_before,
                "actual on-hit recovery must change HP"
            );
            // This healing belongs to the delivery reaction, not to a forged
            // Damage EffectSite. The actual heal fact owns its cast/copy source.
            let facts = trial.observation().unwrap().active_runtime_facts().unwrap();
            let heals: Vec<_> = facts
                .actors()
                .filter_map(|(_, row)| row.caused.latest_nonzero_heal.as_ref())
                .filter(|heal| {
                    heal.source
                        .cast
                        .as_ref()
                        .is_some_and(|cast| cast.skill_id.as_str() == skill_id)
                })
                .collect();
            assert_eq!(heals.len(), 1);
            let heal = heals[0];
            assert_eq!(
                json!(heal.source.fragment.as_ref().unwrap().copy_uuid),
                grant["copy_uuid"]
            );
            assert_eq!(
                heal.source
                    .fragment
                    .as_ref()
                    .unwrap()
                    .definition_id
                    .as_str(),
                fragment_id
            );
            assert_eq!(
                heal.requested_delta,
                (player["stats"]["max_health"].as_u64().unwrap() / 5) as i64
            );
            assert_eq!(
                heal.actual_delta(),
                heal.requested_delta.min(
                    (player["stats"]["max_health"].as_u64().unwrap() - u64::from(heal.hp_before))
                        as i64
                )
            );
            assert_eq!(json!(heal.target), player["unit_instance_id"]);
            recovery = Some(serde_json::to_value(heal).unwrap());
        }
        expected.push(json!({"edit":edit,"source_digest":artifact.source_digest(),"selection":selection,"damages":damages,"grant":grant,"latest":latest,"recovery":recovery,"ready":ready}));
    }
    assert_ne!(digests[0], digests[1]);
    if let Some(destination) = std::env::var_os("CANONICAL_AREA_STRIKES_FIXTURE_DIR") {
        let destination = std::path::PathBuf::from(destination).join(family);
        assert!(!destination.exists(), "preserve prior evidence");
        copy_fixture(fixture.root(), &destination.join("content"));
        std::fs::write(
            destination.join("scenario.json"),
            serde_json::to_vec_pretty(&document).unwrap(),
        )
        .unwrap();
        std::fs::write(
            destination.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
        std::fs::write(
            destination.join("original.skill.yaml"),
            canonical::original(family),
        )
        .unwrap();
        std::fs::write(
            destination.join("original.catalog.json"),
            canonical::catalog_original(family),
        )
        .unwrap();
        std::fs::write(
            destination.join("original-relics.catalog.json"),
            canonical::RELICS_ORIGINAL,
        )
        .unwrap();
    }
    eprintln!("{family} canonical World {}", json!(expected));
}
