//! Fixture-only direct CoreApplication replay; no TrialSession observation owner.
use game_core::{
    application::{ApplicationCommand, ApplicationDocument, ApplicationExecutionContext,
        ApplicationSnapshotKind, CoreApplication},
    authoring::{self, CancellationToken},
    game::data::content_pack::ContentSelection,
};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uuid::Uuid;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 5, "ROOT SELECTION SCENARIO EXPECTED_TRIAL_REPORT OUTPUT");
    let selection: ContentSelection = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let scenario: Value = serde_json::from_slice(&fs::read(&args[2]).unwrap()).unwrap();
    let expected: Value = serde_json::from_slice(&fs::read(&args[3]).unwrap()).unwrap();
    let content = authoring::validate(Path::new(&args[0]), &selection, &Default::default(),
        &CancellationToken::default()).unwrap();
    let player: Uuid = serde_json::from_value(scenario["initial"]["player_id"].clone()).unwrap();
    let mut app = CoreApplication::new(content.token().clone(), scenario["initial"]["run_seed"].as_u64().unwrap());
    assert!(app.battle_runtime_facts().unwrap().is_none());
    let mut outputs: Vec<Value> = Vec::new();
    let mut observations: Vec<Value> = Vec::new();
    for source in scenario["preparation_steps"].as_array().unwrap().iter()
        .chain(scenario["steps"].as_array().unwrap()) {
        let mut step = source.clone();
        for binding in source["bindings"].as_array().into_iter().flatten() {
            let value = outputs[binding["from"]["step"].as_u64().unwrap() as usize]
                .pointer(binding["from"]["pointer"].as_str().unwrap()).unwrap().clone();
            *step.pointer_mut(binding["to"].as_str().unwrap()).unwrap() = value;
        }
        let output = match step["kind"].as_str().unwrap() {
            "command" => {
                let context = ApplicationExecutionContext {
                    expected_battle_authority: step["context"]["expected_battle_authority"].as_object()
                        .map(|_| serde_json::from_value(step["context"]["expected_battle_authority"].clone()).unwrap()),
                    ..Default::default()
                };
                let command = ApplicationCommand::from_document(ApplicationDocument::new(step["command"].clone())).unwrap();
                serde_json::to_value(app.execute(player, command, context).unwrap()).unwrap()
            }
            "snapshot" => {
                let kind: ApplicationSnapshotKind = serde_json::from_value(step["snapshot"].clone()).unwrap();
                serde_json::to_value(app.snapshot(kind).unwrap()).unwrap()
            }
            "clock" => {
                let clock=&step["clock"];
                match clock["mode"].as_str().unwrap() {
                    "elapsed" => serde_json::to_value(app.advance_active_battle_by(clock["delta_ms"].as_u64().unwrap(),
                        clock["after_seq"].as_u64().unwrap_or(0)).unwrap()).unwrap(),
                    "host_tick" => serde_json::to_value(app.advance_active_battle_for_host_tick(clock["delta_ms"].as_u64().unwrap(),
                        clock["after_seq"].as_u64().unwrap_or(0)).unwrap()).unwrap(),
                    other => panic!("unsupported fixture clock {other}"),
                }
            }
            "assert" => {
                assert_eq!(step["assertion"]["test"], "type");
                assert_eq!(step["assertion"]["value_type"], "string");
                assert!(outputs[step["from"]["step"].as_u64().unwrap() as usize]
                    .pointer(step["from"]["pointer"].as_str().unwrap()).unwrap().is_string());
                Value::Null
            }
            other => panic!("unsupported fixture kind {other}"),
        };
        outputs.push(output);
        if let Some(facts)=app.battle_runtime_facts().unwrap() {
            observations.push(serde_json::to_value(facts).unwrap());
        }
    }
    assert_eq!(outputs.len(), 13);
    assert!(app.battle_runtime_facts().unwrap().is_none(), "result consumption releases the current world fact owner");
    let terminal=observations.last().unwrap();
    assert_eq!(terminal, &expected["first_terminal"]["runtime_facts"]);
    assert!(terminal["summary"]["damage_hp_lost"].as_u64().unwrap()>0);
    fs::write(&args[4],serde_json::to_vec_pretty(&json!({"passed":true,"steps":outputs.len(),
        "execution":"ordinary CoreApplication; no TrialSession/authoring observation owner",
        "terminal":terminal,"current_owner_after_result":null})).unwrap()).unwrap();
    println!("{}",json!({"passed":true,"steps":outputs.len(),"native_trial_terminal_equal":true}));
}
