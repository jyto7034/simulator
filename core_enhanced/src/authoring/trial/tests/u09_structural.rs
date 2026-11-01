//! U09 normal external catalogs, public commands, consumed HP and fresh Continue.
use super::*;
use crate::application::{ApplicationCheckpoint, ApplicationCommand, ApplicationDocument};
use crate::authoring::{TraceLimits, TraceOptions};
use crate::game::data::{
    relic_data::RelicDatabase, reward_data::RewardDatabase,
    skill_fragment_data::SkillFragmentDatabase,
};
use crate::game::map::{FacilityTemplateDatabase, NodeAssignmentProfileDatabase};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const TEST_NAME: &str = "authoring::trial::tests::u09_structural::rv003_external_structural_grants_queries_consumption_and_fresh_continue";
const CHILD: &str = "U09_PUBLIC_RESTORE_CHILD";
macro_rules! fixture_text { ($name:literal) => { include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/goals/core_relic_structural_activation/execution/single_goal/test_parts/runtime_validation001/fixtures/", $name)) }; }

fn fixture() -> (ExternalFixture, ValidatedGameData, Value) {
    public_amount_trial_fixture(|fixture| {
        let path = "official/data/enemies/corroded_employees.ron";
        fixture.write(
            path,
            fixture
                .read(path)
                .replacen("max_health: 170,", "max_health: 500,", 1),
        );
        macro_rules! merge {
            ($ty:ty, $field:ident, $path:literal, $file:literal) => {{
                let mut original: $ty = ron::de::from_str(&fixture.read($path)).unwrap();
                let extra: $ty = ron::de::from_str(fixture_text!($file)).unwrap();
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
        merge!(
            SkillFragmentDatabase,
            fragments,
            "official/data/skill_fragments/base.ron",
            "fragments.ron"
        );
        merge!(
            RewardDatabase,
            rewards,
            "official/data/events/rewards/base.ron",
            "rewards.ron"
        );
        let extra = fixture_text!("skills.ron")
            .trim()
            .strip_prefix("SkillDatabase(skills: [")
            .unwrap()
            .strip_suffix("])")
            .unwrap();
        let path = "official/data/skills/base.ron";
        fixture.write(
            path,
            fixture
                .read(path)
                .replacen("skills: [", &format!("skills: [{extra},"), 1),
        );
        let path = "official/data/map/node_assignment_profiles.ron";
        let mut assignments: NodeAssignmentProfileDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let intro = assignments
            .profiles
            .iter_mut()
            .find(|p| p.id == "standard_floor_1")
            .unwrap();
        intro
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
            .push("u09_public_supply".into());
        intro.fixed_slot_assignments.push(
            ron::de::from_str(
                r#"(slot_id:"u09_public_save",kind_id:"save_point",payload:SavePoint)"#,
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
            .find(|t| t.id.as_str() == "standard_intro_a")
            .unwrap();
        intro.slots.push(ron::de::from_str(r#"(id:"u09_public_save",role:Ordinary,allowed_categories:[SavePoint],tags:["save_point_candidate"])"#).unwrap());
        intro.connections.push(ron::de::from_str(r#"(from_slot_id:"intake_corridor",to_slot_id:"u09_public_save",direction:Bidirectional)"#).unwrap());
        fixture.write(path, ron::ser::to_string(&templates).unwrap());
    })
}
fn shift(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "step" && value.as_u64().is_some_and(|n| n >= 15) {
                    *value = json!(value.as_u64().unwrap() + 6);
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
        _ => {}
    }
}
fn scenario(token: &ValidatedGameData, mut doc: Value) -> Value {
    let baseline = simulate(
        fixture_content(token),
        checked(&doc),
        TrialLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let grant = &baseline.step(13).unwrap().public_result().unwrap()["payload"];
    let index = grant["skill_fragment_diffs"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["fragment_id"] == "u09_public_fragment")
        .unwrap();
    let save_node = grant["completion"]["payload"]["map"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["slot_id"] == "u09_public_save")
        .unwrap()["id"]
        .clone();
    for step in doc["steps"].as_array_mut().unwrap() {
        if step["command"]["type"] == "equip_skill_fragment" {
            step["bindings"][0]["from"]["pointer"] =
                json!(format!("/payload/skill_fragment_diffs/{index}/copy_uuid"));
        }
        if step["command"]["skill_id"] == "fragment_freischutz_black_round" {
            step["command"]["skill_id"] = json!("u09_public_strike");
        }
        if step["clock"]["delta_ms"] == 23000 {
            step["clock"]["delta_ms"] = json!(8000);
        }
    }
    doc["steps"].as_array_mut().unwrap().last_mut().unwrap()["clock"]["delta_ms"] = json!(1);
    shift(&mut doc);
    doc["steps"].as_array_mut().unwrap().splice(6..6, [
        json!({"kind":"command","command":{"type":"request_relic_state"}}),
        json!({"kind":"command","command":{"type":"equip_relic","copy_uuid":null,"employee_uuid":null,"slot":2},"bindings":[{"from":{"step":15,"pointer":"/payload/copies/0/copy_uuid"},"to":"/command/copy_uuid","value_type":"string"},{"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]}),
        json!({"kind":"command","command":{"type":"request_relic_effective_plan","employee_uuid":null},"bindings":[{"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}]}),
        json!({"kind":"command","command":{"type":"select_map_node","node_id":save_node}}),
        json!({"kind":"command","command":{"type":"confirm_enter_node"}}),
        json!({"kind":"command","command":{"type":"complete_node"}}),
    ]);
    doc
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
    fn visit(root: &Path, dir: &Path, rows: &mut Vec<Value>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &entry.path(), rows);
            } else {
                let b = fs::read(entry.path()).unwrap();
                rows.push(json!({"path":entry.path().strip_prefix(root).unwrap(),"bytes":b.len(),"sha256":format!("{:x}",Sha256::digest(&b))}));
            }
        }
    }
    let mut rows = vec![];
    visit(root, root, &mut rows);
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    json!(rows)
}
fn command(app: &mut CoreApplication, player: Uuid, value: Value) -> Value {
    let cmd = ApplicationCommand::from_document(ApplicationDocument::new(value.clone())).unwrap();
    app.execute(player, cmd, Default::default())
        .unwrap_or_else(|e| panic!("{value}: {e:?}"))
        .payload()
        .as_value()
        .clone()
}
fn assert_plan(payload: &Value) {
    let plan = payload;
    assert_eq!(
        plan["execution_status"], "runtime_supported_prospective_plan",
        "{payload}"
    );
    let effects = plan["effects"].as_array().unwrap();
    assert_eq!(effects.len(), 2);
    let mut amounts: Vec<_> = effects
        .iter()
        .flat_map(|e| e["properties"].as_array().unwrap())
        .filter(|p| p["property"] == "DamageAmount")
        .map(|p| p["final_value"]["Hp"].as_i64().unwrap())
        .collect();
    amounts.sort();
    assert_eq!(amounts, vec![30, 75]);
    assert_eq!(
        effects
            .iter()
            .filter(|e| e["key"]["kind"] == "original")
            .count(),
        1
    );
    assert_eq!(
        effects
            .iter()
            .filter(|e| e["key"]["kind"] == "generated")
            .count(),
        1
    );
}
fn restore_child(dir: &Path) {
    let saved: Value = serde_json::from_slice(&fs::read(dir.join("save.json")).unwrap()).unwrap();
    assert_ne!(saved["pid"], json!(std::process::id()));
    let selection = serde_json::from_value(saved["selection"].clone()).unwrap();
    let token = crate::game::data::content_pack::load(
        &dir.join("content"),
        &selection,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(token.identity()).unwrap(),
        saved["generation"]
    );
    let player = serde_json::from_value(saved["player"].clone()).unwrap();
    let checkpoint = ApplicationCheckpoint::new(
        serde_json::to_vec(&saved["checkpoint"]).unwrap(),
        saved["loads"].as_u64().unwrap() as u8,
        saved["peak"].as_u64().unwrap(),
    );
    let mut app = CoreApplication::new(token, 900);
    app.install_authoritative_player_profile(saved["profile"].clone().into()).unwrap();
    app.continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap();
    assert_eq!(
        command(&mut app, player, json!({"type":"request_relic_state"})),
        saved["relics"]
    );
    let plan = command(
        &mut app,
        player,
        json!({"type":"request_relic_effective_plan","employee_uuid":saved["employee"]}),
    );
    assert_plan(&plan);
    assert_eq!(plan, saved["plan"]);
    copy_tree(&dir.join("content"), &dir.join("changed_content"));
    let path = dir.join("changed_content/official/data/skills/base.ron");
    let bytes = fs::read_to_string(&path).unwrap();
    assert!(bytes.contains("amount: 23"));
    fs::write(path, bytes.replacen("amount: 23", "amount: 24", 1)).unwrap();
    let changed = crate::game::data::content_pack::load(
        &dir.join("changed_content"),
        &selection,
        &Default::default(),
    )
    .unwrap();
    assert_ne!(
        serde_json::to_value(changed.identity()).unwrap(),
        saved["generation"]
    );
    let mut rejected = CoreApplication::new(changed, 900);
    rejected.install_authoritative_player_profile(saved["profile"].clone().into()).unwrap();
    let before = rejected.certified_input_fingerprint().unwrap();
    assert!(rejected
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .is_err());
    assert_eq!(before, rejected.certified_input_fingerprint().unwrap());
    assert!(rejected.export_active_run_checkpoint().unwrap().is_none());
    println!(
        "U09_FRESH_CONTINUE same_content_plan30_75=true changed_generation_atomic_reject=true"
    );
}
#[test]
fn rv003_external_structural_grants_queries_consumption_and_fresh_continue() {
    if let Some(dir) = std::env::var_os(CHILD) {
        restore_child(Path::new(&dir));
        return;
    }
    // Reuse authored input bytes after fixture-only oracle corrections; never
    // use observed outcomes as expected data or skip normal public execution.
    let (_owned, content_root, selection, token, doc) =
        if let Some(input) = std::env::var_os("U09_PUBLIC_INPUT") {
            let input = PathBuf::from(input);
            let selection: crate::game::data::content_pack::ContentSelection =
                serde_json::from_slice(&fs::read(input.join("selection.json")).unwrap()).unwrap();
            let expected: Value =
                serde_json::from_slice(&fs::read(input.join("content_manifest.json")).unwrap())
                    .unwrap();
            assert_eq!(manifest(&input.join("content")), expected);
            let token = crate::game::data::content_pack::load(
                &input.join("content"),
                &selection,
                &Default::default(),
            )
            .unwrap();
            let doc: Value =
                serde_json::from_slice(&fs::read(input.join("scenario.json")).unwrap()).unwrap();
            (None, input.join("content"), selection, token, doc)
        } else {
            let (fixture, token, doc) = fixture();
            let doc = scenario(&token, doc);
            let root = fixture.root().to_path_buf();
            let selection = fixture.selection();
            (Some(fixture), root, selection, token, doc)
        };
    let export = std::env::var_os("U09_PUBLIC_CLI_EXPORT").map(PathBuf::from);
    let dir = export
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("u09-public-{}", Uuid::new_v4())));
    fs::create_dir(&dir).unwrap();
    copy_tree(&content_root, &dir.join("content"));
    write_json(&dir.join("selection.json"), &selection);
    write_json(&dir.join("scenario.json"), &doc);
    write_json(&dir.join("generation.json"), token.identity());
    write_json(
        &dir.join("content_manifest.json"),
        &manifest(&dir.join("content")),
    );
    write_json(
        &dir.join("markers.json"),
        &json!({"grant":15,"plan":17,"save":20,"ready":30,"activate":32,"resolved":33}),
    );
    let player: Uuid = serde_json::from_value(doc["initial"]["player_id"].clone()).unwrap();
    let mut outcomes = vec![];
    for mode in 0..4 {
        let cancel = CancellationToken::default();
        let trace = match mode {
            0 | 3 => TraceOptions::default(),
            1 => TraceOptions {
                limits: Some(TraceLimits::default()),
            },
            _ => TraceOptions {
                limits: Some(TraceLimits {
                    max_rows: 1,
                    max_bytes: 1,
                    max_row_bytes: 1,
                    max_attempt_work: 1,
                }),
            },
        };
        let mut session = TrialSession::prepare(
            fixture_content(&token),
            checked(&doc),
            TrialLimits {
                amount_observation: mode != 3,
                trace,
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
        while !session.is_complete() {
            let index = session.next_step_index();
            session
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("step {index} mode {mode}: {e:?}"));
            if index == 17 {
                assert_plan(&session.step(index).unwrap().public_result().unwrap()["payload"]);
            }
            if index == 20 && mode == 0 {
                let saved = session
                    .state
                    .application
                    .export_active_run_checkpoint()
                    .unwrap()
                    .unwrap();
                let employee = session.step(9).unwrap().public_result().unwrap()["payload"]
                    ["employee_uuid"]
                    .clone();
                let relics = command(
                    &mut session.state.application,
                    player,
                    json!({"type":"request_relic_state"}),
                );
                let plan = command(
                    &mut session.state.application,
                    player,
                    json!({"type":"request_relic_effective_plan","employee_uuid":employee}),
                );
                let payload: String =
                    String::from_utf8(saved.core_payload_json().to_vec()).unwrap();
                for forbidden in ["execution_templates", "relic_effective", "effective_owner"] {
                    assert!(!payload.contains(forbidden));
                }
                write_json(
                    &dir.join("save.json"),
                    &json!({"profile":session.state.application.snapshot(ApplicationSnapshotKind::PlayerProfile).unwrap().read_model().clone(),"pid":std::process::id(),"player":player,"employee":employee,"selection":selection,"generation":token.identity(),"checkpoint":serde_json::from_str::<Value>(&payload).unwrap(),"loads":saved.loads_used(),"peak":saved.run_peak_certified_power(),"relics":relics,"plan":plan}),
                );
                let result = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", TEST_NAME, "--nocapture"])
                    .env(CHILD, &dir)
                    .output()
                    .unwrap();
                write_new(&dir.join("child.stdout"), &result.stdout);
                write_new(&dir.join("child.stderr"), &result.stderr);
                assert!(
                    result.status.success(),
                    "child: {} {}",
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
        let before = &session.step(30).unwrap().public_result().unwrap()["payload"]
            ["battle_update"]["checkpoint"];
        let after = &session.step(33).unwrap().public_result().unwrap()["payload"]["battle_update"]
            ["checkpoint"];
        let readiness = &before["deployment"]["deployed_units"][0]["skill_readiness"];
        assert_eq!(readiness["skill_id"], "u09_public_strike");
        assert_eq!(readiness["resonance_current"], 1);
        assert_eq!(readiness["eligibility"]["enabled"], true);
        let enemy = before["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["owner"] == "Opponent")
            .unwrap();
        let later = after["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["unit_instance_id"] == enemy["unit_instance_id"])
            .unwrap();
        assert_eq!(
            enemy["stats"]["current_health"].as_u64().unwrap()
                - later["stats"]["current_health"].as_u64().unwrap(),
            105
        );
        let facts = serde_json::to_value(
            session
                .state
                .application
                .battle_runtime_facts()
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        let latest = facts["actors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| &a["summary"]["caused"]["latest_damage"])
            .find(|h| h["requested_damage"] == 75)
            .expect("generated effect consumed 75");
        assert_eq!(
            latest["hp_before"].as_u64().unwrap() - latest["hp_after"].as_u64().unwrap(),
            75
        );
        if mode == 2 {
            let trace = session.trace().unwrap();
            assert!(
                trace.active().unwrap().gap().is_some(),
                "actually saturated trace records a gap"
            );
        }
        outcomes.push(
            json!({"checkpoint":after,"facts":facts,"accepted_work":session.accepted_work()}),
        );
        let mut raw = session.receipt_documents().collect::<Vec<_>>().join("\n");
        raw.push('\n');
        write_new(&dir.join(format!("trial_mode{mode}.jsonl")), raw.as_bytes());
    }
    for other in 1..4 {
        assert_eq!(outcomes[0]["checkpoint"], outcomes[other]["checkpoint"]);
        assert_eq!(outcomes[0]["facts"], outcomes[other]["facts"]);
    }
    // Amount replay capture has an existing mandatory retained/work charge.
    // Only trace mode changes in the first three runs; default is mode3.
    assert_eq!(outcomes[0]["accepted_work"], outcomes[1]["accepted_work"]);
    assert_eq!(outcomes[0]["accepted_work"], outcomes[2]["accepted_work"]);
    write_json(&dir.join("outcomes.json"), &outcomes);
    write_json(&dir.join("export_manifest.json"), &manifest(&dir));
    println!(
        "U09_PUBLIC_EXPORT {}",
        json!({"root":dir,"literal_oracle":{"original":30,"generated":75,"hp_delta":105},"default_observed_saturated_equal":true})
    );
    if export.is_none() {
        fs::remove_dir_all(dir).unwrap();
    }
}

mod families;

mod amount_cases;
