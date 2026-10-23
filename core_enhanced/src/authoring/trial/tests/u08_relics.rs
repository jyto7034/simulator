//! New external IDs enter through normal rewards, commands and real battles.
//! Exported inputs outlive ExternalFixture so the release CLI can consume them.
use super::*;
use crate::application::{ApplicationCheckpoint, ApplicationCommand, ApplicationDocument};
use crate::authoring::{AmountBattle, AmountQuery, TraceLimits, TraceOptions};
use crate::game::{
    data::{
        event_data::EventDatabase, relic_data::RelicDatabase, reward_data::RewardDatabase,
        skill_fragment_data::SkillFragmentDatabase,
    },
    map::{FacilityTemplateDatabase, NodeAssignmentProfileDatabase},
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const TEST_NAME: &str = "authoring::trial::tests::u08_relics::u08_external_ron_normal_grants_savepoint_event_replay_and_fresh_process";
const CHILD: &str = "U08_PUBLIC_RESTORE_CHILD";

fn fixture() -> (ExternalFixture, ValidatedGameData, Value) {
    public_amount_trial_fixture(|fixture| {
        // The inherited preview target has only170HP. A durable external
        // authored target distinguishes amplified210 from base100 in real HP.
        let enemies_path = "official/data/enemies/corroded_employees.ron";
        let enemies = fixture.read(enemies_path);
        assert_eq!(enemies.matches("max_health: 170,").count(), 1);
        fixture.write(
            enemies_path,
            enemies.replacen("max_health: 170,", "max_health: 500,", 1),
        );
        macro_rules! merge {
            ($ty:ty, $field:ident, $path:literal, $file:literal) => {{
                let mut original: $ty = ron::de::from_str(&fixture.read($path)).unwrap();
                let extra: $ty = ron::de::from_str(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/u08/trial001/",
                    $file
                )))
                .unwrap();
                original.$field.extend(extra.$field);
                fixture.write($path, ron::ser::to_string(&original).unwrap());
            }};
        }
        merge!(
            RelicDatabase,
            relics,
            "official/data/relics/base.ron",
            "relics.ron"
        );
        // Raw amount constants use i128, which this RON serializer cannot
        // emit. Preserve the authored catalog bytes and add the new entry.
        let extra_skills = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/u08/trial001/skills.ron"
        ));
        let entries = extra_skills
            .trim()
            .strip_prefix("(skills: [")
            .unwrap()
            .strip_suffix("])")
            .unwrap();
        let path = "official/data/skills/base.ron";
        let original = fixture.read(path);
        assert_eq!(original.matches("skills: [").count(), 1);
        fixture.write(
            path,
            original.replacen("skills: [", &format!("skills: [{entries}"), 1),
        );
        merge!(
            SkillFragmentDatabase,
            fragments,
            "official/data/skill_fragments/base.ron",
            "fragments.ron"
        );
        merge!(
            EventDatabase,
            events,
            "official/data/events/story/base.ron",
            "events.ron"
        );
        merge!(
            RewardDatabase,
            rewards,
            "official/data/events/rewards/base.ron",
            "rewards.ron"
        );
        // Only this owned external copy changes. The normal first victory
        // resolves an authored reward ID; no admin grant or state injection.
        let path = "official/data/map/node_assignment_profiles.ron";
        let mut assignments: NodeAssignmentProfileDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let intro = assignments
            .profiles
            .iter_mut()
            .find(|profile| profile.id == "standard_floor_1")
            .unwrap();
        let intake = intro
            .fixed_slot_assignments
            .iter_mut()
            .find(|slot| slot.slot_id.as_str() == "intake_corridor")
            .unwrap();
        intake
            .authored_combat
            .as_mut()
            .unwrap()
            .outcome_policies
            .iter_mut()
            .find(|outcome| outcome.outcome_id == "victory")
            .unwrap()
            .reward_ids
            .push("u08_trial_supply".into());
        let event = intro
            .fixed_slot_assignments
            .iter_mut()
            .find(|slot| slot.slot_id.as_str() == "records_annex")
            .unwrap();
        event.payload = crate::game::map::MapNodePayload::Event {
            event_id: Some(crate::game::data::event_data::EventId::new(
                "u08_trial_corruption",
            )),
        };
        intro.fixed_slot_assignments.push(
            ron::de::from_str(
                r#"(slot_id:"u08_trial_save",kind_id:"save_point",payload:SavePoint)"#,
            )
            .unwrap(),
        );
        fixture.write(path, ron::ser::to_string(&assignments).unwrap());
        let path = "official/data/map/facility_templates.ron";
        let mut templates: FacilityTemplateDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let intro = templates
            .templates
            .iter_mut()
            .find(|template| template.id.as_str() == "standard_intro_a")
            .unwrap();
        intro.slots.push(ron::de::from_str(r#"(id:"u08_trial_save",role:Ordinary,allowed_categories:[SavePoint],tags:["save_point_candidate"])"#).unwrap());
        for (from, to) in [
            ("intake_corridor", "u08_trial_save"),
            ("u08_trial_save", "records_annex"),
            ("records_annex", "supply_annex"),
        ] {
            intro.connections.push(
                ron::de::from_str(&format!(
                    r#"(from_slot_id:"{from}",to_slot_id:"{to}",direction:Bidirectional)"#
                ))
                .unwrap(),
            );
        }
        fixture.write(path, ron::ser::to_string(&templates).unwrap());
    })
}

fn saved_external_relic_fixture() -> (ExternalFixture, ValidatedGameData, Uuid, CoreApplication) {
    let (fixture, token, initial) = fixture();
    let (document, markers, _) = scenario(&token, initial);
    let player: Uuid = serde_json::from_value(document["initial"]["player_id"].clone()).unwrap();
    let cancellation = CancellationToken::default();
    let mut session = TrialSession::prepare(
        fixture_content(&token),
        checked(&document),
        TrialLimits::default(),
        &cancellation,
    )
    .unwrap();
    while session.next_step_index() <= markers["save_complete"] {
        session.advance(&cancellation).unwrap();
    }
    let relics = command(
        &mut session.state.application,
        player,
        json!({"type":"request_relic_state"}),
    );
    assert_eq!(relics["copies"][0]["definition_id"], "u08_trial_blend");
    assert_eq!(relics["copies"][0]["variant"], "Normal");
    assert_eq!(
        relics["loadouts"][0]["slots"][2],
        relics["copies"][0]["copy_uuid"]
    );
    assert!(session
        .state
        .application
        .export_active_run_checkpoint()
        .unwrap()
        .is_some());
    (fixture, token, player, session.state.application)
}

fn external_public_roots(app: &CoreApplication) -> Value {
    let snapshots: Vec<_> = ApplicationSnapshotKind::ALL
        .iter()
        .map(|kind| app.snapshot(*kind).unwrap().read_model().as_value().clone())
        .collect();
    let saved = app
        .export_active_run_checkpoint()
        .unwrap()
        .map(|checkpoint| {
            json!({"bytes":checkpoint.core_payload_json(), "loads":checkpoint.loads_used(),
            "peak":checkpoint.run_peak_certified_power()})
        });
    json!({"snapshots":snapshots, "save":saved,
        "fingerprint":app.certified_input_fingerprint().unwrap(), "identity":app.content_identity()})
}

#[test]
fn u08_external_relic_invalid_catalogs_preserve_published_token_and_nonempty_run() {
    use crate::game::data::{content_pack, live_ron::LiveRonPhase};
    use content_pack::{CatalogKind, DefinitionKind};
    struct Case {
        label: &'static str,
        path: &'static str,
        input: String,
        phase: LiveRonPhase,
        catalog: CatalogKind,
        definition: Option<DefinitionKind>,
        field: Option<String>,
        causes: Vec<&'static str>,
    }
    fn changed(source: &str, edit: impl FnOnce(&mut RelicDatabase)) -> String {
        let mut data: RelicDatabase = ron::de::from_str(source).unwrap();
        edit(&mut data);
        ron::ser::to_string(&data).unwrap()
    }
    let (fixture, token, _, app) = saved_external_relic_fixture();
    let before = external_public_roots(&app);
    let identity = token.identity().clone();
    let definitions = serde_json::to_vec(&token.relic_data).unwrap();
    let path = "official/data/relics/base.ron";
    let original = fixture.read(path);
    let reward_path = "official/data/events/rewards/base.ron";
    let reward_original = fixture.read(reward_path);
    let mut cases = Vec::new();
    for (label, from, to, cause, field) in [
        (
            "nonfinite_flat_nan",
            "flat:20",
            "flat:nan",
            "Expected integer",
            "relics[0].effects[0].flat",
        ),
        (
            "nonfinite_percent_infinity",
            "percent:20",
            "percent:inf",
            "Expected integer",
            "relics[0].effects[0].percent",
        ),
        (
            "unknown_schema",
            "effects:",
            "unsupported_recipe:true,effects:",
            "unsupported_recipe",
            "relics[0].unsupported_recipe",
        ),
        (
            "unsupported_conditional_recipe",
            "selector:(",
            "selector:(health_below_percent:50,",
            "health_below_percent",
            "relics[0].effects[0].selector.health_below_percent",
        ),
        (
            "unsupported_penalty_variant",
            "DefenseFlat(",
            "HealOnKill(",
            "HealOnKill",
            "relics[0].corruption.penalties[0]",
        ),
    ] {
        let input = original.replacen(from, to, 1);
        assert_ne!(input, original, "{label}");
        cases.push(Case {
            label,
            path,
            input,
            phase: LiveRonPhase::Parse,
            catalog: CatalogKind::Relics,
            definition: None,
            field: Some(field.into()),
            causes: vec![cause],
        });
    }
    for (label, edit, causes) in [
        (
            "invalid_stable_effect_address",
            (|data: &mut RelicDatabase| data.relics[0].effects[0].id = "invalid/effect".into())
                as fn(&mut RelicDatabase),
            vec![
                "relics[0].effects",
                "[0].id",
                "invalid or duplicate effect identity",
            ],
        ),
        (
            "duplicate_effect_identity",
            |data: &mut RelicDatabase| {
                data.relics[0].effects[1].id = data.relics[0].effects[0].id.clone()
            },
            vec![
                "relics[0].effects",
                "[1].id",
                "invalid or duplicate effect identity",
            ],
        ),
        (
            "duplicate_definition_uuid",
            |data: &mut RelicDatabase| {
                let mut duplicate = data.relics[0].clone();
                duplicate.id.push_str("_duplicate");
                data.relics.push(duplicate);
            },
            vec!["relics[1].id", "duplicate definition identity"],
        ),
        (
            "missing_equipment",
            |data: &mut RelicDatabase| {
                data.relics[0].effects[0].selector.equipment_id = Some("missing_equipment".into())
            },
            vec![
                "relic 'u08_trial_blend' effect 'first'",
                "unknown equipment 'missing_equipment'",
            ],
        ),
        (
            "missing_fragment",
            |data: &mut RelicDatabase| {
                data.relics[0].effects[0].selector.fragment_id = Some("missing_fragment".into())
            },
            vec![
                "relic 'u08_trial_blend' effect 'first'",
                "unknown fragment 'missing_fragment'",
            ],
        ),
        (
            "missing_fragment_form",
            |data: &mut RelicDatabase| {
                data.relics[0].effects[0].selector.fragment_id = Some("u08_trial_fragment".into());
                data.relics[0].effects[0].selector.form_id = Some("missing_form".into());
            },
            vec![
                "relic 'u08_trial_blend' effect 'first'",
                "unknown fragment form 'missing_form'",
            ],
        ),
        (
            "form_without_fragment",
            |data: &mut RelicDatabase| {
                data.relics[0].effects[0].selector.form_id = Some("base".into())
            },
            vec![
                "relics[0].effects",
                "[0].selector.form_id",
                "requires an exact fragment",
            ],
        ),
        (
            "corrupt_empty_effects",
            |data: &mut RelicDatabase| data.relics[0].corruption.as_mut().unwrap().effects.clear(),
            vec![
                "relics[0].corruption.effects",
                "at least one effect or rule is required",
            ],
        ),
        (
            "corrupt_nonnegative_penalty",
            |data: &mut RelicDatabase| {
                data.relics[0].corruption.as_mut().unwrap().penalties[0] =
                    crate::game::data::relic_data::RelicPenalty::DefenseFlat { amount: 0 }
            },
            vec![
                "relics[0].corruption.penalties",
                "authored negative flat penalties",
            ],
        ),
    ] {
        cases.push(Case {
            label,
            path,
            input: changed(&original, edit),
            phase: LiveRonPhase::FinalValidation,
            catalog: CatalogKind::Relics,
            definition: Some(DefinitionKind::Relic),
            field: Some("relics".into()),
            causes,
        });
    }
    let mut rewards: RewardDatabase = ron::de::from_str(&reward_original).unwrap();
    let reward_index = rewards
        .rewards
        .iter()
        .position(|row| row.id == "u08_trial_supply")
        .unwrap();
    let grant = &mut rewards.rewards[reward_index].effects[0];
    assert!(
        matches!(grant, crate::game::reward::RewardEffect::GrantRelic { relic_id } if relic_id == "u08_trial_blend")
    );
    *grant = crate::game::reward::RewardEffect::GrantRelic {
        relic_id: "missing_relic".into(),
    };
    cases.push(Case {
        label: "reward_unknown_relic",
        path: reward_path,
        input: ron::ser::to_string(&rewards).unwrap(),
        phase: LiveRonPhase::FinalValidation,
        catalog: CatalogKind::Rewards,
        definition: Some(DefinitionKind::Reward),
        field: Some(format!("rewards[{reward_index}].effects[0]")),
        causes: vec!["references unknown relic 'missing_relic'"],
    });
    let input = reward_original.replacen(
        "GrantRelic(relic_id:",
        "GrantRelic(variant:Corrupt,relic_id:",
        1,
    );
    assert_ne!(input, reward_original);
    cases.push(Case {
        label: "reward_authored_corrupt_copy",
        path: reward_path,
        input,
        phase: LiveRonPhase::Parse,
        catalog: CatalogKind::Rewards,
        definition: None,
        field: Some(format!(
            "rewards[{reward_index}].effects[0].GrantRelic.variant"
        )),
        causes: vec!["variant"],
    });

    for case in cases {
        fixture.write(path, &original);
        fixture.write(reward_path, &reward_original);
        fixture.write(case.path, &case.input);
        let plain = content_pack::load(
            fixture.root(),
            &fixture.selection(),
            &content_pack::InputLimits::default(),
        )
        .err()
        .unwrap_or_else(|| panic!("invalid case published: {}", case.label));
        let Err(AuthoringError::Content { diagnostic }) = crate::authoring::validate(
            fixture.root(),
            &fixture.selection(),
            &crate::authoring::AuthoringLimits::default(),
            &CancellationToken::default(),
        ) else {
            panic!("expected publication refusal: {}", case.label);
        };
        assert_eq!(diagnostic, plain, "{}", case.label);
        assert_eq!(
            diagnostic.phase, case.phase,
            "{}: {diagnostic:?}",
            case.label
        );
        assert_eq!(
            diagnostic.code,
            match case.phase {
                LiveRonPhase::Parse => "ron_parse",
                LiveRonPhase::FinalValidation => "content_validation",
                _ => unreachable!(),
            }
        );
        assert_eq!(diagnostic.source_file, case.path, "{}", case.label);
        assert_eq!(
            diagnostic.pack_id.as_deref(),
            Some("official"),
            "{}",
            case.label
        );
        assert_eq!(
            diagnostic.catalog_kind,
            Some(case.catalog),
            "{}",
            case.label
        );
        assert_eq!(
            diagnostic.definition_kind, case.definition,
            "{}",
            case.label
        );
        assert_eq!(diagnostic.field_path, case.field, "{}", case.label);
        for cause in case.causes {
            assert!(
                diagnostic.cause.contains(cause),
                "{} expected {cause}: {diagnostic:?}",
                case.label
            );
        }
        println!(
            "U08_EXTERNAL_REJECTION {} {}",
            case.label,
            serde_json::to_string(&diagnostic).unwrap()
        );
        assert_eq!(token.identity(), &identity);
        assert_eq!(serde_json::to_vec(&token.relic_data).unwrap(), definitions);
        assert_eq!(external_public_roots(&app), before);
    }
    fixture.write(path, original);
    fixture.write(reward_path, reward_original);
    let recovered = crate::authoring::validate(
        fixture.root(),
        &fixture.selection(),
        &crate::authoring::AuthoringLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(recovered.token().identity(), &identity);
    assert_eq!(external_public_roots(&app), before);
}

#[test]
fn u08_external_changed_or_missing_relic_generation_rejects_original_nonempty_save_atomically() {
    use crate::application::ApplicationErrorKind;
    let (fixture, token, player, app) = saved_external_relic_fixture();
    let before = external_public_roots(&app);
    let saved = app.export_active_run_checkpoint().unwrap().unwrap();
    let saved_bytes = saved.core_payload_json().to_vec();
    let identity = token.identity().clone();
    let definitions = serde_json::to_vec(&token.relic_data).unwrap();
    let path = "official/data/relics/base.ron";
    let original = fixture.read(path);
    let reward_path = "official/data/events/rewards/base.ron";
    let reward_original = fixture.read(reward_path);
    for remove in [false, true] {
        let mut relics: RelicDatabase = ron::de::from_str(&original).unwrap();
        let mut rewards: RewardDatabase = ron::de::from_str(&reward_original).unwrap();
        assert_eq!(relics.relics[0].id, "u08_trial_blend");
        if remove {
            relics.relics.clear();
            // Remove only the deleted definition's supply reference so this
            // is a valid new catalog, not another publication-failure case.
            let reward = rewards
                .rewards
                .iter_mut()
                .find(|row| row.id == "u08_trial_supply")
                .unwrap();
            let original_count = reward.effects.len();
            reward.effects.retain(|effect| !matches!(effect, crate::game::reward::RewardEffect::GrantRelic { relic_id } if relic_id == "u08_trial_blend"));
            assert_eq!(reward.effects.len(), original_count - 1);
        } else {
            relics.relics[0].effects[0].flat = 21;
        }
        fixture.write(path, ron::ser::to_string(&relics).unwrap());
        fixture.write(reward_path, ron::ser::to_string(&rewards).unwrap());
        let changed = crate::authoring::validate(
            fixture.root(),
            &fixture.selection(),
            &crate::authoring::AuthoringLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap();
        assert_ne!(changed.token().identity(), &identity);
        if remove {
            assert!(changed
                .token()
                .relic_data
                .get_by_id("u08_trial_blend")
                .is_none());
        } else {
            assert_eq!(
                changed
                    .token()
                    .relic_data
                    .get_by_id("u08_trial_blend")
                    .unwrap()
                    .effects[0]
                    .flat,
                21
            );
        }
        let mut destination = CoreApplication::new(changed.token().clone(), 0xB808);
        let profile = crate::game::encounter_compendium::PlayerProfileState {
            highest_floor_reached: 99,
            ..Default::default()
        };
        destination
            .install_authoritative_player_profile(serde_json::to_value(profile).unwrap().into())
            .unwrap();
        let destination_before = external_public_roots(&destination);
        for _ in 0..2 {
            let error = destination
                .continue_from_active_run_checkpoint(player, &saved)
                .unwrap_err();
            assert_eq!(error.kind(), ApplicationErrorKind::InvalidStaticData);
            assert_eq!(error.code(), "invalid_static_data");
            assert_eq!(
                error.details().as_value()["reason"],
                "run checkpoint content identity does not match the retained generation"
            );
            assert_eq!(external_public_roots(&destination), destination_before);
        }
        assert_eq!(saved.core_payload_json(), saved_bytes);
        assert_eq!(token.identity(), &identity);
        assert_eq!(serde_json::to_vec(&token.relic_data).unwrap(), definitions);
        assert_eq!(external_public_roots(&app), before);
        println!(
            "U08_ACTUAL_RELIC_GENERATION remove={remove} old={} new={}",
            identity.semantic_sha256,
            changed.token().identity().semantic_sha256
        );
    }
    fixture.write(path, original);
    fixture.write(reward_path, reward_original);
    let restored_content = crate::authoring::validate(
        fixture.root(),
        &fixture.selection(),
        &crate::authoring::AuthoringLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(restored_content.token().identity(), &identity);
    let mut restored = CoreApplication::new(restored_content.token().clone(), 0xC808);
    // Continue restores run state; the host supplies separately persisted
    // profile roots through their existing bootstrap ports.
    restored
        .install_authoritative_player_profile(
            app.snapshot(ApplicationSnapshotKind::PlayerProfile)
                .unwrap()
                .read_model()
                .clone(),
        )
        .unwrap();
    restored
        .merge_encounter_compendium_profile(
            app.snapshot(ApplicationSnapshotKind::EncounterCompendium)
                .unwrap()
                .read_model()
                .clone(),
        )
        .unwrap();
    restored
        .continue_from_active_run_checkpoint(player, &saved)
        .unwrap();
    assert_eq!(
        restored
            .export_active_run_checkpoint()
            .unwrap()
            .unwrap()
            .core_payload_json(),
        saved_bytes
    );
    assert_eq!(external_public_roots(&restored), before);
    assert_eq!(external_public_roots(&app), before);
}

fn write_new(path: &Path, bytes: &[u8]) {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap()
        .write_all(bytes)
        .unwrap();
}
fn write_json(path: &Path, value: &impl serde::Serialize) {
    write_new(path, &serde_json::to_vec_pretty(value).unwrap());
}
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            assert!(entry.file_type().unwrap().is_file());
            write_new(&target, &fs::read(entry.path()).unwrap());
        }
    }
}
fn manifest(root: &Path) -> Value {
    fn visit(root: &Path, directory: &Path, rows: &mut Vec<Value>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &entry.path(), rows);
            } else {
                let bytes = fs::read(entry.path()).unwrap();
                rows.push(json!({"path":entry.path().strip_prefix(root).unwrap().to_str().unwrap(), "bytes":bytes.len(), "sha256":format!("{:x}",Sha256::digest(&bytes))}));
            }
        }
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows);
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    json!(rows)
}
fn command(app: &mut CoreApplication, player: Uuid, input: Value) -> Value {
    let parsed =
        ApplicationCommand::from_document(ApplicationDocument::new(input.clone())).unwrap();
    app.execute(player, parsed, Default::default())
        .unwrap_or_else(|error| panic!("{input}: {error:?}"))
        .payload()
        .as_value()
        .clone()
}
fn history(checkpoint: &ApplicationCheckpoint, event_node: &str) -> Value {
    let value: Value = serde_json::from_slice(checkpoint.core_payload_json()).unwrap();
    value["continuation"]["current"]["event_sessions"][event_node]["relic_corruption_selections"]
        .clone()
}
fn expected_history(copy: &Value) -> Value {
    json!([{"source_scene_id":"first","copy_uuid":copy},{"source_scene_id":"second","copy_uuid":null}])
}
fn assert_plan(plan: &Value, expected: u32) {
    let target = plan["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|target| {
            target["kind"] == "skill_damage"
                && target["site"]["action"]["definition_id"] == "u08_trial_damage"
        })
        .unwrap();
    assert_eq!(
        target["evaluation"],
        json!({"status":"available","base_amount":100,"effective_amount":expected})
    );
}

fn restore_child(directory: &Path) {
    let saved: Value =
        serde_json::from_slice(&fs::read(directory.join("save.json")).unwrap()).unwrap();
    assert_ne!(saved["parent_pid"], json!(std::process::id()));
    let selection = serde_json::from_value(saved["selection"].clone()).unwrap();
    let token = crate::game::data::content_pack::load(
        &directory.join("content"),
        &selection,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(token.identity()).unwrap(),
        saved["generation"]
    );
    let player = serde_json::from_value(saved["player_id"].clone()).unwrap();
    let mut app = CoreApplication::new(token, 900);
    app.install_authoritative_player_profile(saved["profile"].clone().into()).unwrap();
    let checkpoint = ApplicationCheckpoint::new(
        serde_json::to_vec(&saved["checkpoint"]).unwrap(),
        saved["loads_used"].as_u64().unwrap() as u8,
        saved["peak"].as_u64().unwrap(),
    );
    app.continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap();
    let relics = command(&mut app, player, json!({"type":"request_relic_state"}));
    assert_eq!(relics, saved["relics"]);
    assert_eq!(
        serde_json::to_value(app.snapshot(ApplicationSnapshotKind::Inventory).unwrap()).unwrap(),
        saved["inventory"]
    );
    assert_eq!(
        serde_json::to_value(app.certified_input_fingerprint().unwrap()).unwrap(),
        saved["fingerprint"]
    );
    let plan = command(
        &mut app,
        player,
        json!({"type":"request_relic_numeric_plan","employee_uuid":saved["employee"]}),
    );
    assert_plan(&plan, 210);
    let exported = app.export_active_run_checkpoint().unwrap().unwrap();
    assert_eq!(exported.loads_used(), checkpoint.loads_used());
    assert_eq!(
        history(&exported, saved["event_node"].as_str().unwrap()),
        saved["history"]
    );
    println!(
        "U08_FRESH_PROCESS {}",
        json!({"pid":std::process::id(),"parent_pid":saved["parent_pid"],"generation":saved["generation"],"history":saved["history"]})
    );
}

fn shift_old_bindings(value: &mut Value, first: u64, count: u64) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "step" && value.as_u64().is_some_and(|index| index >= first) {
                    *value = json!(value.as_u64().unwrap() + count);
                } else {
                    shift_old_bindings(value, first, count);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                shift_old_bindings(value, first, count);
            }
        }
        _ => {}
    }
}
fn simple(input: Value) -> Value {
    json!({"kind":"command","command":input})
}
fn employee_query(kind: &str) -> Value {
    json!({"kind":"command","command":{"type":kind,"employee_uuid":null},"bindings":[{"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]})
}
fn add(
    steps: &mut Vec<Value>,
    markers: &mut BTreeMap<&'static str, usize>,
    name: &'static str,
    step: Value,
) {
    markers.insert(name, 15 + steps.len());
    steps.push(step);
}
fn scenario(
    token: &ValidatedGameData,
    mut document: Value,
) -> (Value, BTreeMap<&'static str, usize>, Value) {
    let cancel = CancellationToken::default();
    let baseline = simulate(
        fixture_content(token),
        checked(&document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let grant = &baseline.step(13).unwrap().public_result().unwrap()["payload"];
    let fragment_index = grant["skill_fragment_diffs"]
        .as_array()
        .unwrap()
        .iter()
        .position(|row| row["fragment_id"] == "u08_trial_fragment")
        .unwrap();
    let nodes = grant["completion"]["payload"]["map"]["nodes"]
        .as_array()
        .unwrap();
    let node =
        |slot: &str| nodes.iter().find(|node| node["slot_id"] == slot).unwrap()["id"].clone();
    let save_node = node("u08_trial_save");
    let event_node = node("records_annex");
    let old_steps = document["steps"].as_array_mut().unwrap();
    let equip = old_steps
        .iter_mut()
        .find(|step| step["command"]["type"] == "equip_skill_fragment")
        .unwrap();
    for binding in equip["bindings"].as_array_mut().unwrap() {
        if binding["to"] == "/command/copy_uuid" {
            binding["from"]["pointer"] = json!(format!(
                "/payload/skill_fragment_diffs/{fragment_index}/copy_uuid"
            ));
        }
    }
    for step in old_steps.iter_mut() {
        if step["command"]["skill_id"] == "fragment_freischutz_black_round" {
            step["command"]["skill_id"] = json!("u08_trial_damage");
        }
    }
    // This fragment fills at one resonance. The inherited 23-second wait
    // outlives this relic-enhanced battle (E006 terminal event: 10,750ms).
    let ready_clock = old_steps
        .iter_mut()
        .find(|step| step["clock"]["delta_ms"] == 23000)
        .unwrap();
    ready_clock["clock"]["delta_ms"] = json!(8000);
    old_steps.last_mut().unwrap()["clock"]["delta_ms"] = json!(1);
    let mut inserted = Vec::new();
    let mut markers = BTreeMap::new();
    add(
        &mut inserted,
        &mut markers,
        "granted",
        simple(json!({"type":"request_relic_state"})),
    );
    add(
        &mut inserted,
        &mut markers,
        "equipped",
        json!({"kind":"command","command":{"type":"equip_relic","copy_uuid":null,"employee_uuid":null,"slot":2},"bindings":[
        {"from":{"step":15,"pointer":"/payload/copies/0/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"},
        {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]}),
    );
    add(
        &mut inserted,
        &mut markers,
        "normal_plan",
        employee_query("request_relic_numeric_plan"),
    );
    add(
        &mut inserted,
        &mut markers,
        "save_select",
        simple(json!({"type":"select_map_node","node_id":save_node})),
    );
    add(
        &mut inserted,
        &mut markers,
        "save_enter",
        simple(json!({"type":"confirm_enter_node"})),
    );
    add(
        &mut inserted,
        &mut markers,
        "save_complete",
        simple(json!({"type":"complete_node"})),
    );
    for pass in 0..2 {
        inserted.push(simple(
            json!({"type":"select_map_node","node_id":event_node}),
        ));
        add(
            &mut inserted,
            &mut markers,
            if pass == 0 {
                "event_enter"
            } else {
                "event_reenter"
            },
            simple(json!({"type":"confirm_enter_node"})),
        );
        inserted.push(json!({"kind":"command","command":{"type":"choose_relic_corruption","node_id":event_node,"event_id":"u08_trial_corruption","current_scene_id":"first","copy_uuid":null},"bindings":[
            {"from":{"step":15,"pointer":"/payload/copies/0/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"}]}));
        add(
            &mut inserted,
            &mut markers,
            if pass == 0 {
                "first_history"
            } else {
                "second_history"
            },
            simple(
                json!({"type":"choose_relic_corruption","node_id":event_node,"event_id":"u08_trial_corruption","current_scene_id":"second","copy_uuid":null}),
            ),
        );
        inserted.push(simple(json!({"type":"advance_event_scene","node_id":event_node,"event_id":"u08_trial_corruption","current_scene_id":"end"})));
        add(
            &mut inserted,
            &mut markers,
            if pass == 0 {
                "first_corrupt_plan"
            } else {
                "corrupt_plan"
            },
            employee_query("request_relic_numeric_plan"),
        );
        if pass == 0 {
            add(
                &mut inserted,
                &mut markers,
                "load_savepoint",
                simple(json!({"type":"load_run_checkpoint"})),
            );
            add(
                &mut inserted,
                &mut markers,
                "restored_normal",
                simple(json!({"type":"request_relic_state"})),
            );
            add(
                &mut inserted,
                &mut markers,
                "restored_plan",
                employee_query("request_relic_numeric_plan"),
            );
        }
    }
    add(
        &mut inserted,
        &mut markers,
        "durable",
        simple(json!({"type":"request_relic_state"})),
    );
    markers.insert("battle_ready", 24 + inserted.len());
    shift_old_bindings(&mut document, 15, inserted.len() as u64);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .splice(6..6, inserted);
    (document, markers, event_node)
}

#[test]
fn u08_external_ron_normal_grants_savepoint_event_replay_and_fresh_process() {
    if let Some(directory) = std::env::var_os(CHILD) {
        restore_child(Path::new(&directory));
        return;
    }
    let (fixture, token, initial) = fixture();
    let (document, markers, event_node) = scenario(&token, initial);
    let export = std::env::var_os("U08_PUBLIC_CLI_EXPORT").map(PathBuf::from);
    let directory = export
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("u08-trial-{}", Uuid::new_v4())));
    fs::create_dir(&directory)
        .expect("fresh export directory required; existing outputs are never overwritten");
    copy_tree(fixture.root(), &directory.join("content"));
    write_json(&directory.join("selection.json"), &fixture.selection());
    write_json(&directory.join("scenario.json"), &document);
    write_json(&directory.join("generation.json"), token.identity());
    write_json(
        &directory.join("content_manifest.json"),
        &manifest(&directory.join("content")),
    );
    let player: Uuid = serde_json::from_value(document["initial"]["player_id"].clone()).unwrap();
    let mut query_documents = Vec::new();
    let mut damage_documents = Vec::new();
    let mut expected_copy = Value::Null;
    for traced in [false, true] {
        let cancel = CancellationToken::default();
        let mut session = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits {
                amount_observation: true,
                trace: if traced {
                    TraceOptions {
                        limits: Some(TraceLimits::default()),
                    }
                } else {
                    TraceOptions::default()
                },
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
        let mut normal_state = Value::Null;
        while !session.is_complete() {
            let index = session.next_step_index();
            if let Err(error) = session.advance(&cancel) {
                let preceding = index
                    .checked_sub(1)
                    .and_then(|prior| session.step(prior))
                    .and_then(|step| step.public_result());
                panic!("step={index} trace={traced}: {error:?}; preceding={preceding:?}");
            }
            let payload = session.step(index).unwrap().public_result().unwrap()["payload"].clone();
            if index == markers["battle_ready"] {
                assert_eq!(
                    payload["finished"], false,
                    "skill activation requires a live battle"
                );
                let readiness = &payload["battle_update"]["checkpoint"]["deployment"]
                    ["deployed_units"][0]["skill_readiness"];
                assert_eq!(readiness["skill_id"], "u08_trial_damage");
                assert_eq!(readiness["resonance_current"], 1);
                assert_eq!(readiness["resonance_max"], 1);
                assert_eq!(readiness["eligibility"]["enabled"], true);
                assert_eq!(readiness["target_available"], true);
            }
            if index == markers["granted"] {
                assert_eq!(payload["copies"].as_array().unwrap().len(), 1);
                assert_eq!(payload["copies"][0]["definition_id"], "u08_trial_blend");
                assert_eq!(payload["copies"][0]["variant"], "Normal");
                assert_eq!(payload["resources"]["relic_corruption_catalysts"], 3);
                if expected_copy.is_null() {
                    expected_copy = payload["copies"][0]["copy_uuid"].clone();
                }
                assert_eq!(payload["copies"][0]["copy_uuid"], expected_copy);
            }
            if [markers["normal_plan"], markers["restored_plan"]].contains(&index) {
                assert_plan(&payload, 180);
            }
            if [markers["first_corrupt_plan"], markers["corrupt_plan"]].contains(&index) {
                assert_plan(&payload, 210);
            }
            if index == markers["save_complete"] {
                normal_state = command(
                    &mut session.state.application,
                    player,
                    json!({"type":"request_relic_state"}),
                );
            }
            if [markers["event_enter"], markers["event_reenter"]].contains(&index) {
                assert!(payload["event"]
                    .get("relic_corruption_selections")
                    .is_none());
                assert_eq!(payload["event"]["interaction"]["catalyst_cost"], 0);
            }
            if [markers["first_history"], markers["second_history"]].contains(&index) {
                assert_eq!(
                    payload["event"]["relic_corruption_selections"],
                    expected_history(&expected_copy)
                );
            }
            if index == markers["restored_normal"] {
                assert_eq!(payload, normal_state);
                let saved = session
                    .state
                    .application
                    .export_active_run_checkpoint()
                    .unwrap()
                    .unwrap();
                assert!(history(&saved, event_node.as_str().unwrap()).is_null());
                assert_eq!(saved.loads_used(), 1);
            }
            if index == markers["durable"] {
                assert_eq!(payload["copies"][0]["variant"], "Corrupt");
                assert_eq!(payload["resources"], normal_state["resources"]);
                assert_eq!(payload["loadouts"], normal_state["loadouts"]);
                let saved = session
                    .state
                    .application
                    .export_active_run_checkpoint()
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    history(&saved, event_node.as_str().unwrap()),
                    expected_history(&expected_copy)
                );
                if !traced {
                    let employee = session.step(9).unwrap().public_result().unwrap()["payload"]
                        ["employee_uuid"]
                        .clone();
                    write_json(
                        &directory.join("save.json"),
                        &json!({"profile":session.state.application.snapshot(ApplicationSnapshotKind::PlayerProfile).unwrap().read_model().clone(),"parent_pid":std::process::id(),"player_id":player,"selection":fixture.selection(),"generation":token.identity(),"checkpoint":serde_json::from_slice::<Value>(saved.core_payload_json()).unwrap(),"loads_used":saved.loads_used(),"peak":saved.run_peak_certified_power(),"relics":payload,"inventory":session.state.application.snapshot(ApplicationSnapshotKind::Inventory).unwrap(),"fingerprint":session.state.application.certified_input_fingerprint().unwrap(),"employee":employee,"event_node":event_node,"history":expected_history(&expected_copy)}),
                    );
                    let output = std::process::Command::new(std::env::current_exe().unwrap())
                        .args(["--exact", TEST_NAME, "--nocapture"])
                        .env(CHILD, &directory)
                        .output()
                        .unwrap();
                    write_new(&directory.join("fresh_process.stdout"), &output.stdout);
                    write_new(&directory.join("fresh_process.stderr"), &output.stderr);
                    assert!(
                        output.status.success(),
                        "fresh process: {}\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    assert!(String::from_utf8_lossy(&output.stdout).contains("U08_FRESH_PROCESS"));
                }
            }
        }
        let query = AmountQuery::Latest {
            battle: if session.observation().unwrap().active().is_some() {
                AmountBattle::Active
            } else {
                AmountBattle::LastTerminal
            },
        };
        let before = (
            session.accepted_work(),
            session.next_step_index(),
            session
                .receipt_documents()
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        );
        let preview = session
            .preview_amount(query, &ExecutionLimits::default(), &cancel)
            .unwrap();
        let explanation = session
            .explain_amount(query, &ExecutionLimits::default(), &cancel)
            .unwrap();
        assert_eq!(
            preview.document(),
            session
                .preview_amount(query, &ExecutionLimits::default(), &cancel)
                .unwrap()
                .document()
        );
        assert_eq!(
            before,
            (
                session.accepted_work(),
                session.next_step_index(),
                session
                    .receipt_documents()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            )
        );
        let value: Value = serde_json::from_str(preview.document()).unwrap();
        assert_eq!(value["status"], "evaluated", "{value}");
        assert_eq!(value["site"]["action"]["definition_id"], "u08_trial_damage");
        assert_eq!(value["narrowed_base_amount"]["value"], 100);
        assert_eq!(value["effective_relic_amount"]["value"], 210);
        assert_eq!(
            value["relic_numeric"]["contributions"][0]["copy_uuid"],
            expected_copy
        );
        assert_eq!(
            value["relic_numeric"]["contributions"][0]["variant"],
            "Corrupt"
        );
        assert_eq!(value["source"]["fragment"]["skill_id"], "u08_trial_damage");
        let facts = serde_json::to_value(
            session
                .state
                .application
                .battle_runtime_facts()
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        let actual = facts["actors"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|actor| actor["summary"]["caused"]["latest_damage"].as_object())
            .find(|hit| {
                hit["source"]["relic_numeric"]["target"]["kind"] == "skill_damage"
                    && hit["requested_damage"] == 210
            })
            .expect("actual consumed damage must carry the new relic skill binding");
        assert_eq!(
            actual["source"]["relic_numeric"]["contributions"][0]["copy_uuid"],
            expected_copy
        );
        let hp_before = actual["hp_before"].as_u64().unwrap();
        let hp_after = actual["hp_after"].as_u64().unwrap();
        assert!(
            hp_before > 210 && hp_after < hp_before,
            "a non-clamped hit must distinguish amplified210 from base100: {actual:?}"
        );
        assert_eq!(hp_before - hp_after, 210);
        let source = &actual["source"];
        assert_eq!(source["authored"]["origin"], "Original");
        assert_eq!(
            source["authored"]["origin_definition_id"],
            "u08_trial_damage"
        );
        assert_eq!(
            source["authored"]["source_file"],
            "official/data/skills/base.ron"
        );
        assert_eq!(source["fragment"]["definition_id"], "u08_trial_fragment");
        assert_eq!(source["fragment"]["form_id"], "base");
        assert_eq!(source["fragment"]["skill_id"], "u08_trial_damage");
        let grants =
            &session.step(13).unwrap().public_result().unwrap()["payload"]["skill_fragment_diffs"];
        let granted_fragment = grants
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["fragment_id"] == "u08_trial_fragment")
            .unwrap();
        assert_eq!(
            source["fragment"]["copy_uuid"],
            granted_fragment["copy_uuid"]
        );
        assert_eq!(source["relic_numeric"]["fragment"], source["fragment"]);
        assert_eq!(
            source["relic_numeric"]["employee_uuid"],
            session.step(9).unwrap().public_result().unwrap()["payload"]["employee_uuid"]
        );
        assert_eq!(
            source["relic_numeric"]["generation"],
            serde_json::to_value(token.identity()).unwrap()
        );
        let segments = actual["segments"].as_array().unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0]["damage_type"], "True");
        assert_eq!(segments[0]["raw_damage"], 210);
        assert_eq!(segments[0]["calculated_damage"], 210);
        damage_documents.push(Value::Object(actual.clone()));
        query_documents.push((
            preview.document().to_owned(),
            explanation.document().to_owned(),
        ));
    }
    assert_eq!(
        query_documents[0], query_documents[1],
        "trace cannot change immutable numeric replay"
    );
    assert_eq!(damage_documents[0], damage_documents[1]);
    write_json(
        &directory.join("expected.json"),
        &json!({"test":TEST_NAME,"generation":token.identity(),"copy_uuid":expected_copy,"markers":markers,"event_node":event_node,"consumed_damage":damage_documents[0],"preview":serde_json::from_str::<Value>(&query_documents[0].0).unwrap(),"explain":serde_json::from_str::<Value>(&query_documents[0].1).unwrap()}),
    );
    let exported_manifest = manifest(&directory);
    write_json(&directory.join("export_manifest.json"), &exported_manifest);
    println!(
        "U08_PUBLIC_EXPORT {}",
        json!({"root":directory,"generation":token.identity(),"manifest_sha256":format!("{:x}",Sha256::digest(fs::read(directory.join("export_manifest.json")).unwrap()))})
    );
    if export.is_none() {
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn u08_external_ron_normal_reward_and_declined_corruption_consume_authored_damage_in_real_hp() {
    let (fixture, token, initial) = fixture();
    let identity = token.identity().clone();
    let definitions = serde_json::to_vec(&token.relic_data).unwrap();
    let inputs = manifest(fixture.root());
    let (mut document, markers, _) = scenario(&token, initial);
    // Keep the public reward, equip, savepoint and battle sequence. Declining
    // each Event offer is a normal command, so the granted copy stays Normal.
    // This local scenario does not alter the existing Corrupt CLI export.
    let mut declines = 0;
    let mut removed_copy_bindings = 0;
    for step in document["steps"].as_array_mut().unwrap() {
        if step["command"]["type"] == "choose_relic_corruption" {
            assert!(step["command"]["copy_uuid"].is_null());
            if let Some(bindings) = step.get_mut("bindings").and_then(Value::as_array_mut) {
                let before = bindings.len();
                bindings.retain(|binding| binding["to"] != "/command/copy_uuid");
                removed_copy_bindings += before - bindings.len();
            }
            declines += 1;
        }
    }
    assert_eq!((declines, removed_copy_bindings), (4, 2));
    let cancel = CancellationToken::default();
    let content = crate::authoring::validate(
        fixture.root(),
        &fixture.selection(),
        &crate::authoring::AuthoringLimits::default(),
        &cancel,
    )
    .unwrap();
    assert_eq!(content.token().identity(), &identity);
    let mut session = TrialSession::prepare(
        content,
        checked(&document),
        TrialLimits {
            amount_observation: true,
            ..Default::default()
        },
        &cancel,
    )
    .unwrap();
    let mut copy = Value::Null;
    let mut normal_state = Value::Null;
    while !session.is_complete() {
        let index = session.next_step_index();
        session
            .advance(&cancel)
            .unwrap_or_else(|error| panic!("normal step={index}: {error:?}"));
        let payload = session.step(index).unwrap().public_result().unwrap()["payload"].clone();
        if index == markers["granted"] {
            assert_eq!(payload["copies"].as_array().unwrap().len(), 1);
            assert_eq!(payload["copies"][0]["definition_id"], "u08_trial_blend");
            assert_eq!(payload["copies"][0]["variant"], "Normal");
            assert_eq!(payload["resources"]["relic_corruption_catalysts"], 3);
            copy = payload["copies"][0]["copy_uuid"].clone();
            assert!(copy.is_string());
        }
        if [
            markers["normal_plan"],
            markers["first_corrupt_plan"],
            markers["restored_plan"],
            markers["corrupt_plan"],
        ]
        .contains(&index)
        {
            assert_plan(&payload, 180);
        }
        if [markers["first_history"], markers["second_history"]].contains(&index) {
            assert_eq!(
                payload["event"]["relic_corruption_selections"],
                expected_history(&Value::Null)
            );
        }
        if [markers["restored_normal"], markers["durable"]].contains(&index) {
            assert_eq!(payload["copies"].as_array().unwrap().len(), 1);
            assert_eq!(payload["copies"][0]["copy_uuid"], copy);
            assert_eq!(payload["copies"][0]["variant"], "Normal");
            assert_eq!(payload["resources"]["relic_corruption_catalysts"], 3);
            assert_eq!(payload["loadouts"].as_array().unwrap().len(), 1);
            assert_eq!(payload["loadouts"][0]["slots"], json!([null, null, copy]));
            normal_state = payload.clone();
        }
        if index == markers["battle_ready"] {
            assert_eq!(payload["finished"], false);
            let readiness = &payload["battle_update"]["checkpoint"]["deployment"]["deployed_units"]
                [0]["skill_readiness"];
            assert_eq!(readiness["skill_id"], "u08_trial_damage");
            assert_eq!(readiness["eligibility"]["enabled"], true);
            assert_eq!(readiness["target_available"], true);
        }
    }
    let employee =
        session.step(9).unwrap().public_result().unwrap()["payload"]["employee_uuid"].clone();
    assert_eq!(normal_state["loadouts"][0]["employee_uuid"], employee);
    let grants =
        &session.step(13).unwrap().public_result().unwrap()["payload"]["skill_fragment_diffs"];
    let fragment = grants
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["fragment_id"] == "u08_trial_fragment")
        .unwrap()["copy_uuid"]
        .clone();
    let query = AmountQuery::Latest {
        battle: if session.observation().unwrap().active().is_some() {
            AmountBattle::Active
        } else {
            AmountBattle::LastTerminal
        },
    };
    let preview = session
        .preview_amount(query, &ExecutionLimits::default(), &cancel)
        .unwrap();
    let amount: Value = serde_json::from_str(preview.document()).unwrap();
    assert_eq!(amount["status"], "evaluated");
    assert_eq!(
        amount["site"]["action"]["definition_id"],
        "u08_trial_damage"
    );
    assert_eq!(amount["narrowed_base_amount"]["value"], 100);
    assert_eq!(amount["effective_relic_amount"]["value"], 180);
    let facts = serde_json::to_value(
        session
            .state
            .application
            .battle_runtime_facts()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let actual = facts["actors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|actor| actor["summary"]["caused"]["latest_damage"].as_object())
        .find(|hit| {
            hit["source"]["relic_numeric"]["target"]["kind"] == "skill_damage"
                && hit["source"]["authored"]["origin_definition_id"] == "u08_trial_damage"
        })
        .expect("the normally granted and equipped relic must reach real damage consumption");
    // Independent authored oracle: (100 + 20 + 0) * (100 + 20 + 30) / 100 = 180.
    // The unchanged external fixture has a 500 HP target; no runtime HP injection.
    let hp_before = actual["hp_before"].as_u64().unwrap();
    let hp_after = actual["hp_after"].as_u64().unwrap();
    assert!(hp_before > 180 && hp_after < hp_before);
    assert_eq!(hp_before - hp_after, 180);
    assert_eq!(actual["requested_damage"], 180);
    let source = &actual["source"];
    let binding = &source["relic_numeric"];
    assert_eq!(
        binding["contributions"],
        json!([
            {"copy_uuid":copy,"definition_id":"u08_trial_blend","variant":"Normal","effect_id":"first","flat":20,"percent":20},
            {"copy_uuid":copy,"definition_id":"u08_trial_blend","variant":"Normal","effect_id":"second","flat":0,"percent":30}
        ])
    );
    assert_eq!(binding["employee_uuid"], employee);
    assert_eq!(binding["actor"], source["actor"]);
    assert_eq!(
        binding["generation"],
        serde_json::to_value(&identity).unwrap()
    );
    assert_eq!(binding["damage_type"], "True");
    assert!(binding["equipment"].is_null());
    assert_eq!(source["fragment"]["copy_uuid"], fragment);
    assert_eq!(source["fragment"]["definition_id"], "u08_trial_fragment");
    assert_eq!(source["fragment"]["form_id"], "base");
    assert_eq!(source["fragment"]["skill_id"], "u08_trial_damage");
    assert_eq!(binding["fragment"], source["fragment"]);
    assert_eq!(source["authored"]["origin"], "Original");
    assert_eq!(
        source["authored"]["source_file"],
        "official/data/skills/base.ron"
    );
    assert_eq!(amount["relic_numeric"], *binding);
    let segments = actual["segments"].as_array().unwrap();
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0]["damage_type"], "True");
    assert_eq!(segments[0]["raw_damage"], 180);
    assert_eq!(segments[0]["calculated_damage"], 180);
    let player = serde_json::from_value(document["initial"]["player_id"].clone()).unwrap();
    assert_eq!(
        command(
            &mut session.state.application,
            player,
            json!({"type":"request_relic_state"})
        ),
        normal_state
    );
    assert_eq!(token.identity(), &identity);
    assert_eq!(serde_json::to_vec(&token.relic_data).unwrap(), definitions);
    assert_eq!(manifest(fixture.root()), inputs);
    println!(
        "U08_PUBLIC_NORMAL_DAMAGE {}",
        json!({"base":100,"expected":180,"hp_before":hp_before,"hp_after":hp_after,"damage":actual,"normal_state":normal_state,"generation":identity})
    );
}
