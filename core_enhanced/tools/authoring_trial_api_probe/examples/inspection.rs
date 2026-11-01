//! Positive read-only SkillPreview after public reward/equip/battle/focus commands.
use game_core::{application::{ApplicationExecutionContext, ApplicationSnapshotKind},
    authoring::{self, CancellationToken, TraceLimits, TraceOptions, TrialLimits, TrialPreview,
        TrialScenario, TrialSession}, game::{ability::SkillId,
        battle::event_log::SkillCastTarget, data::content_pack::ContentSelection}};
use serde_json::{json, Value};
use std::{fs, path::Path};

fn receipts(session: &TrialSession) -> String {
    session.receipt_documents().map(|s| format!("{s}\n")).collect()
}
fn capture(session: &TrialSession, cancel: &CancellationToken) -> Value {
    json!({"receipts":receipts(session),"work":session.accepted_work(),"bytes":session.receipt_bytes(),
        "index":session.next_step_index(),"identity":session.content_identity(),
        "snapshot":session.snapshot(ApplicationSnapshotKind::Run,&Default::default(),cancel).unwrap().document(),
        "reconnect":session.reconnect(0,&Default::default(),cancel).unwrap().document(),
        "observation":session.observation().unwrap()})
}
fn write(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn binding(step: usize, pointer: &str, to: &str, kind: &str) -> Value {
    json!({"from":{"step":step,"pointer":pointer},"to":to,"value_type":kind})
}
fn authority(step: usize) -> Vec<Value> {
    vec![binding(step,"/payload/battle_update/battle_uuid","/context/expected_battle_authority/battle_uuid","string"),
        binding(step,"/payload/battle_update/battle_state_revision","/context/expected_battle_authority/battle_state_revision","number")]
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(args.len(),4);
    let root = Path::new(&args[0]);
    let selection: ContentSelection = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let raw = fs::read(&args[2]).unwrap();
    let out = Path::new(&args[3]);
    fs::create_dir(out).unwrap();
    let cancel = CancellationToken::default();
    let load = || authoring::validate(root,&selection,&Default::default(),&cancel).unwrap();
    let parse = |raw: &[u8]| TrialScenario::from_json_slice(raw,&Default::default()).unwrap();
    let mut discovery = TrialSession::prepare(load(),parse(&raw),Default::default(),&cancel).unwrap();
    while !discovery.is_complete() {
        if let Err(failure) = discovery.advance(&cancel) {
            fs::write(out.join("discovery_receipts.ndjson"),receipts(&discovery)).unwrap();
            fs::write(out.join("discovery_failure.json"),authoring::render_trial_failure(&failure,&Default::default(),&cancel).unwrap().document()).unwrap();
            panic!("discovery rejected: {failure:?}");
        }
    }
    fs::write(out.join("discovery_receipts.ndjson"),receipts(&discovery)).unwrap();
    assert_eq!(discovery.next_step_index(),20);
    let current = discovery.step(19).unwrap().public_result().unwrap();
    let units = current["payload"]["battle_update"]["checkpoint"]["units"].as_array().unwrap();
    let target_index = units.iter().position(|unit| unit["owner"] == "Opponent").unwrap();
    let target: SkillCastTarget = SkillCastTarget::Unit { unit_instance_id:
        serde_json::from_value(units[target_index]["unit_instance_id"].clone()).unwrap() };
    let employee = serde_json::from_value(discovery.step(9).unwrap().public_result().unwrap()["payload"]["employee_uuid"].clone()).unwrap();
    let readiness = &current["payload"]["battle_update"]["checkpoint"]["deployment"]["deployed_units"][0]["skill_readiness"];
    write(&out.join("READINESS.json"),readiness);
    assert_eq!(readiness["skill_id"],"fragment_freischutz_black_round");
    assert_eq!(readiness["resonance_current"],readiness["resonance_max"]);
    let skill = SkillId::new("fragment_freischutz_black_round");
    let mut document: Value = serde_json::from_slice(&raw).unwrap();
    let steps = document["steps"].as_array_mut().unwrap();
    let mut begin_bindings = authority(19);
    begin_bindings.push(binding(9,"/payload/employee_uuid","/command/employee_uuid","string"));
    steps.push(json!({"kind":"command","command":{"type":"begin_skill_targeting","employee_uuid":null,"skill_id":skill},
        "context":{"expected_battle_authority":{"battle_uuid":null,"battle_state_revision":null}},"bindings":begin_bindings}));
    let mut activate_bindings = authority(20);
    activate_bindings.push(binding(9,"/payload/employee_uuid","/command/employee_uuid","string"));
    activate_bindings.push(binding(19,&format!("/payload/battle_update/checkpoint/units/{target_index}/unit_instance_id"),
        "/command/target/unit_instance_id","string"));
    steps.push(json!({"kind":"command","command":{"type":"activate_skill","employee_uuid":null,"skill_id":skill,
        "target":{"type":"Unit","unit_instance_id":null}},"context":{"expected_battle_authority":{"battle_uuid":null,"battle_state_revision":null}},
        "bindings":activate_bindings}));
    steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":700,"after_seq":0}}));
    let final_raw = serde_json::to_vec(&document).unwrap();
    fs::write(out.join("scenario.json"),&final_raw).unwrap();
    let mut baseline = None;
    let mut summaries = Vec::new();
    for (name, inspect, limits) in [("baseline",false,None),("off",true,None),
        ("on",true,Some(TraceLimits::default())),
        ("gap",true,Some(TraceLimits{max_bytes:1,max_row_bytes:1,..Default::default()}))] {
        let settings = TrialLimits {trace:TraceOptions{limits},..Default::default()};
        let mut session = TrialSession::prepare(load(),parse(&final_raw),settings,&cancel).unwrap();
        let mut previews = Vec::new();
        while !session.is_complete() {
            if let Err(failure) = session.advance(&cancel) {
                fs::write(out.join(format!("{name}_receipts.ndjson")),receipts(&session)).unwrap();
                fs::write(out.join(format!("{name}_failure.json")),authoring::render_trial_failure(&failure,&Default::default(),&cancel).unwrap().document()).unwrap();
                panic!("{name}: {failure:?}");
            }
            if inspect && session.next_step_index()==21 {
                let before = capture(&session,&cancel);
                let revision = session.step(20).unwrap().public_result().unwrap()["payload"]["battle_update"]["battle_state_revision"].clone();
                for _ in 0..3 {
                    let query = TrialPreview::PreviewSkillTarget {employee_uuid:employee,skill_id:skill.clone(),target};
                    let result = session.preview(query,ApplicationExecutionContext::default(),&Default::default(),&cancel).unwrap();
                    let result: Value = serde_json::from_str(result.document()).unwrap();
                    write(&out.join(format!("{name}_preview_attempt.json")),&result);
                    assert_eq!(result["kind"],"skill_target_preview");
                    assert_eq!(result["disposition"],"read_only");
                    assert_eq!(result["payload"]["eligibility"]["enabled"],true);
                    assert_eq!(result["payload"]["battle_state_revision"],revision);
                    if let Some(previous) = previews.first() {assert_eq!(&result,previous);}
                    previews.push(result);
                    assert_eq!(capture(&session,&cancel),before);
                }
                write(&out.join(format!("{name}_before_after.json")),&before);
            }
        }
        let receipt = receipts(&session);
        fs::write(out.join(format!("{name}_receipts.ndjson")),&receipt).unwrap();
        write(&out.join(format!("{name}_previews.json")),&json!(previews));
        assert_eq!(session.next_step_index(),23);
        let observation = session.observation().unwrap();
        assert!(observation.active().is_some() && observation.last_terminal().is_some());
        let hit = serde_json::to_value(observation.active().unwrap().latest_nonzero_sourced_hp().unwrap()).unwrap();
        assert_eq!(hit["source"]["skill_id"],skill.as_str());
        assert!(hit["change"]["before"].as_u64().unwrap()>hit["change"]["after"].as_u64().unwrap());
        if let Some(baseline) = &baseline {assert_eq!(&receipt,baseline);} else {baseline=Some(receipt);}
        let trace = session.trace().unwrap();
        if let Some(view) = trace.active() {
            fs::write(out.join(format!("{name}_trace.ndjson")),view.ndjson()).unwrap();
            write(&out.join(format!("{name}_trace.json")),&json!({"key":view.key(),"rows":view.retained_rows(),"gap":view.gap()}));
        }
        summaries.push(json!({"mode":name,"positive_previews":if inspect{3}else{0},"accepted_steps":23,
            "active_battle":true,"prior_terminal":true,"manual_skill_hp":hit,"accepted_work":session.accepted_work()}));
    }
    write(&out.join("RESULT.json"),&json!({"passed":true,"api_replays":4,"discovery_replays":1,"positive_previews":9,"summaries":summaries}));
    println!("{}",json!({"passed":true,"api_replays":4,"positive_previews":9,"steps":23}));
}
