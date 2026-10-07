//! Real invalid content producers through the public authoring API.
use game_core::{authoring::{self, CancellationToken, TrialScenario, TrialSession},
    game::data::content_pack::ContentSelection};
use serde_json::{json, Value};
use std::{fs, path::Path};

fn receipts(session: &TrialSession) -> Vec<String> {
    session.receipt_documents().map(str::to_owned).collect()
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(args.len(), 4);
    let root = Path::new(&args[0]);
    let cases: Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let input = fs::read(&args[2]).unwrap();
    let output = Path::new(&args[3]);
    fs::create_dir(output).unwrap();
    let cancel = CancellationToken::default();
    let selection = ContentSelection { packs: vec!["official".into()], winners: vec![] };
    let load = || authoring::validate(&root.join("valid"), &selection, &Default::default(), &cancel).unwrap();
    let simulate = || authoring::simulate(load(), TrialScenario::from_json_slice(&input,
        &Default::default()).unwrap(), Default::default(), &cancel).unwrap();
    let a = simulate();
    let before = receipts(&a);
    let ready = a.ready_receipt().document().to_owned();
    let index = a.next_step_index();
    let lifecycle = format!("{:?}", a.lifecycle());
    let validation = authoring::validation_receipt(&load(), &Default::default(), &cancel).unwrap();
    let mut reports = Vec::new();
    for case in cases.as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let selected: ContentSelection = serde_json::from_value(case["selection"].clone()).unwrap();
        let mut previous = None;
        for _ in 0..2 {
            let error = authoring::validate(&root.join(name), &selected, &Default::default(), &cancel)
                .err().expect("invalid generation must not publish");
            let rendered = authoring::render_error(&error, &Default::default(), &cancel).unwrap();
            let doc = rendered.document().to_owned();
            if let Some(previous) = previous { assert_eq!(doc, previous); }
            previous = Some(doc);
            assert_eq!(receipts(&a), before);
            assert_eq!(a.ready_receipt().document(), ready);
            assert_eq!(a.next_step_index(), index);
            assert_eq!(format!("{:?}", a.lifecycle()), lifecycle);
        }
        fs::write(output.join(format!("{name}.json")), previous.unwrap()).unwrap();
        assert_eq!(authoring::validation_receipt(&load(), &Default::default(), &cancel)
            .unwrap().document(), validation.document());
        assert_eq!(receipts(&simulate()), before);
        reports.push(json!({"name":name,"rejected_attempts":2,"a_preserved":true,"valid_replay_recovered":true}));
    }
    let mut document: Value = serde_json::from_slice(&input).unwrap();
    let command = &mut document["preparation_steps"][6];
    assert_eq!(command["kind"], "command");
    command["bindings"] = json!([]);
    command["context"] = json!({"expected_battle_authority":{
        "battle_uuid":"00000000-0000-0000-0000-000000000000","battle_state_revision":0}});
    let invalid_input = serde_json::to_vec(&document).unwrap();
    fs::write(output.join("invalid_authority_scenario.json"), &invalid_input).unwrap();
    let mut previous = None;
    for _ in 0..2 {
        let failure = authoring::simulate(load(), TrialScenario::from_json_slice(&invalid_input,
            &Default::default()).unwrap(), Default::default(), &cancel).unwrap_err();
        assert!(failure.accepted().is_none());
        let rendered = authoring::render_trial_failure(failure.failure(), &Default::default(), &cancel).unwrap();
        let diagnostic: Value = serde_json::from_str(rendered.document()).unwrap();
        assert_eq!(failure.failure().position(), authoring::TrialPosition::Step { index: 6 });
        assert_eq!(diagnostic["error"]["diagnostic"]["code"], "stale_battle_state_authority");
        if let Some(previous) = previous { assert_eq!(rendered.document(), previous); }
        previous = Some(rendered.document().to_owned());
        assert_eq!(receipts(&a), before);
        assert_eq!(a.ready_receipt().document(), ready);
        assert_eq!(a.next_step_index(), index);
    }
    fs::write(output.join("invalid_authority.json"), previous.unwrap()).unwrap();
    assert_eq!(receipts(&simulate()), before);
    fs::write(output.join("RESULT.json"), serde_json::to_vec_pretty(&json!({"passed":true,
        "cases":reports,"a_steps":index,"a_receipts":before.len()})).unwrap()).unwrap();
    println!("{}", json!({"passed":true,"cases":reports.len(),"rejected_attempts":reports.len()*2}));
}
