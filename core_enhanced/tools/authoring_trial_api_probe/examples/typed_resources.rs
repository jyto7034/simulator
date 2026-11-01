//! Ordinary default-release CoreApplication resource replay; no fixture APIs.
use game_core::{
    application::{
        ApplicationCommand, ApplicationDocument, ApplicationError, ApplicationExecutionContext,
        ApplicationSnapshotKind, CoreApplication,
    },
    authoring::{self, CancellationToken},
    game::data::content_pack::ContentSelection,
};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uuid::Uuid;

fn observe(app: &CoreApplication) -> Value {
    json!({
        "run": app.run_resources(), "battle": app.battle_typed_resources(),
        "facts": app.battle_runtime_facts().unwrap(),
        "snapshot": app.snapshot(ApplicationSnapshotKind::Run).unwrap(),
    })
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 4, "ROOT SELECTION SCENARIO NEW_OUTPUT");
    let output = Path::new(&args[3]);
    assert!(!output.exists());
    let selection: ContentSelection = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let scenario: Value = serde_json::from_slice(&fs::read(&args[2]).unwrap()).unwrap();
    let content = authoring::validate(
        Path::new(&args[0]),
        &selection,
        &Default::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let identity = serde_json::to_value(content.token().identity()).unwrap();
    let player: Uuid = serde_json::from_value(scenario["initial"]["player_id"].clone()).unwrap();
    let mut app = CoreApplication::new(
        content.token().clone(),
        scenario["initial"]["run_seed"].as_u64().unwrap(),
    );
    let mut outputs: Vec<Value> = Vec::new();
    let mut observations = Vec::new();
    let mut terminal = Value::Null;
    let mut failure = Value::Null;
    for source in scenario["preparation_steps"]
        .as_array()
        .unwrap()
        .iter()
        .chain(scenario["steps"].as_array().unwrap())
    {
        let mut step = source.clone();
        for binding in source["bindings"].as_array().into_iter().flatten() {
            let value = outputs[binding["from"]["step"].as_u64().unwrap() as usize]
                .pointer(binding["from"]["pointer"].as_str().unwrap())
                .unwrap()
                .clone();
            *step.pointer_mut(binding["to"].as_str().unwrap()).unwrap() = value;
        }
        let before = observe(&app);
        let result = (|| -> Result<Value, ApplicationError> {
            Ok(match step["kind"].as_str().unwrap() {
                "command" => {
                    let context = ApplicationExecutionContext {
                        expected_battle_authority: step["context"]["expected_battle_authority"]
                            .as_object()
                            .map(|_| {
                                serde_json::from_value(
                                    step["context"]["expected_battle_authority"].clone(),
                                )
                                .unwrap()
                            }),
                        ..Default::default()
                    };
                    let command = ApplicationCommand::from_document(ApplicationDocument::new(
                        step["command"].clone(),
                    ))?;
                    serde_json::to_value(app.execute(player, command, context)?).unwrap()
                }
                "snapshot" => serde_json::to_value(
                    app.snapshot(serde_json::from_value(step["snapshot"].clone()).unwrap())?,
                )
                .unwrap(),
                "clock" => {
                    let clock = &step["clock"];
                    let delta = clock["delta_ms"].as_u64().unwrap();
                    let after = clock["after_seq"].as_u64().unwrap_or(0);
                    match clock["mode"].as_str().unwrap() {
                        "elapsed" => {
                            serde_json::to_value(app.advance_active_battle_by(delta, after)?)
                                .unwrap()
                        }
                        "host_tick" => serde_json::to_value(
                            app.advance_active_battle_for_host_tick(delta, after)?,
                        )
                        .unwrap(),
                        mode => panic!("unsupported clock {mode}"),
                    }
                }
                "assert" => {
                    assert_eq!(step["assertion"]["test"], "type");
                    assert_eq!(step["assertion"]["value_type"], "string");
                    assert!(outputs[step["from"]["step"].as_u64().unwrap() as usize]
                        .pointer(step["from"]["pointer"].as_str().unwrap())
                        .unwrap()
                        .is_string());
                    Value::Null
                }
                kind => panic!("unsupported step {kind}"),
            })
        })();
        let after = observe(&app);
        if !after["facts"].is_null() {
            terminal = after["facts"].clone();
        }
        match result {
            Ok(value) => {
                outputs.push(value);
                observations.push(after);
            }
            Err(error) => {
                failure = json!({"step": outputs.len(), "code": error.code(),
                    "error": format!("{error:?}"), "before": before, "after": after});
                break;
            }
        }
    }
    let result = json!({"identity": identity, "steps": outputs.len(), "outputs": outputs,
        "observations": observations, "terminal_facts": terminal, "failure": failure,
        "final": observe(&app), "execution": "ordinary default-release CoreApplication"});
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!(
        "{}",
        json!({"steps": result["steps"], "failure": result["failure"]["code"]})
    );
}
