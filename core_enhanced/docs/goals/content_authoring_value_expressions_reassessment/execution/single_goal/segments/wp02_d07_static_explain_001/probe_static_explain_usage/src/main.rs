use game_core::{
    authoring::{
        self, AuthoringError, AuthoringQueryResult, CancellationToken, ExecutionFailure,
        ExecutionLimits, ExplainQuery,
    },
    game::{ability::authoring::EffectSite, data::content_pack::ContentSelection},
};
use serde_json::{json, Value};
use std::path::Path;

fn rejected(result: Result<AuthoringQueryResult, AuthoringError>) -> ExecutionFailure {
    match result {
        Err(AuthoringError::Execution { diagnostic }) => diagnostic,
        Err(other) => panic!("unexpected rejection: {other}"),
        Ok(_) => panic!("limited static explanation unexpectedly succeeded"),
    }
}

fn main() {
    let root = std::env::args_os().nth(1).expect("trusted root");
    let selection = ContentSelection {
        packs: vec!["official".into()],
        winners: vec![],
    };
    let cancellation = CancellationToken::default();
    let content = authoring::validate(
        Path::new(&root),
        &selection,
        &Default::default(),
        &cancellation,
    )
    .unwrap();
    let site: EffectSite = serde_json::from_value(json!({
        "action": {
            "definition_id": "unknown_distortion_strike",
            "action_id": "strike"
        },
        "effect_id": "effect_01"
    }))
    .unwrap();
    let query = ExplainQuery::Effect { site };
    let first =
        authoring::explain(&content, &query, &ExecutionLimits::default(), &cancellation).unwrap();
    let document: Value = serde_json::from_str(first.document()).unwrap();

    let mut work = ExecutionLimits::default();
    work.max_work = first.usage().work - 1;
    let work_failure = rejected(authoring::explain(&content, &query, &work, &cancellation));
    let mut metadata = ExecutionLimits::default();
    metadata.max_metadata_bytes = first.usage().metadata_bytes_peak - 1;
    let metadata_failure = rejected(authoring::explain(
        &content,
        &query,
        &metadata,
        &cancellation,
    ));
    let mut result = ExecutionLimits::default();
    result.max_result_bytes = first.document().len() - 1;
    let result_failure = rejected(authoring::explain(&content, &query, &result, &cancellation));

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "query_usage": first.usage(),
            "document_bytes": first.document().len(),
            "semantic_fingerprint": document["value_expression"]["semantic_fingerprint"],
            "plan_cost": document["value_expression"]["cost"],
            "evaluation": document["value_expression"]["evaluation"],
            "one_below_failures": {
                "work": work_failure,
                "metadata": metadata_failure,
                "result": result_failure
            }
        }))
        .unwrap()
    );
}
