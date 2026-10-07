use std::process::Command;

fn assert_input_error(args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_author_content"))
        .args(args)
        .output()
        .expect("author_content should execute");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");
    assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
    assert!(
        output.stdout.is_empty(),
        "invalid input must not emit receipts"
    );
    let diagnostic: serde_json::Value =
        serde_json::from_str(&stderr).expect("one structured diagnostic, not a panic");
    assert_eq!(diagnostic["kind"], "request", "{args:?}: {stderr}");
    assert_eq!(diagnostic["diagnostic"]["code"], "authoring_cli_input");
    assert_eq!(diagnostic["diagnostic"]["field_path"], "cli");
}

#[test]
fn missing_required_positionals_are_cli_errors_for_every_operation() {
    for operation in ["simulate", "explain", "validate", "describe"] {
        let positionals = [
            operation,
            "unused-root",
            "unused-selection",
            "unused-request",
        ];
        let required = if matches!(operation, "simulate" | "explain") {
            4
        } else {
            3
        };
        for count in 0..required {
            assert_input_error(&positionals[..count]);
        }
    }
}

#[test]
fn simulate_rejects_malformed_or_duplicate_options() {
    let prefix = [
        "simulate",
        "unused-root",
        "unused-selection",
        "unused-request",
    ];
    for tail in [
        vec!["--unknown"],
        vec!["unexpected-positional"],
        vec!["--trace-output"],
        vec!["--amount-observation", "--trace-output"],
        vec!["--amount-observation", "--amount-observation"],
        vec!["--trace-output", "unused-trace", "--unknown"],
        vec![
            "--trace-output",
            "first-trace",
            "--trace-output",
            "second-trace",
        ],
    ] {
        let args: Vec<_> = prefix.iter().copied().chain(tail).collect();
        assert_input_error(&args);
    }
}

#[test]
fn runtime_options_are_rejected_by_non_simulation_operations() {
    for operation in ["validate", "describe", "explain"] {
        let mut prefix = vec![operation, "unused-root", "unused-selection"];
        if operation == "explain" {
            prefix.push("unused-request");
        }
        for tail in [
            vec!["--amount-observation"],
            vec!["--trace-output", "unused-trace"],
        ] {
            let args: Vec<_> = prefix.iter().copied().chain(tail).collect();
            assert_input_error(&args);
        }
    }
}

#[test]
fn command_and_argument_bounds_fail_before_reading_files() {
    assert_input_error(&["unknown", "unused-root", "unused-selection"]);
    let long_argument = "x".repeat(4097);
    assert_input_error(&[
        "simulate",
        &long_argument,
        "unused-selection",
        "unused-request",
    ]);
    let mut excessive = vec![
        "simulate",
        "unused-root",
        "unused-selection",
        "unused-request",
    ];
    excessive.extend(["--amount-observation"; 32]);
    assert_input_error(&excessive);
}
