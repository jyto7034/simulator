//! Headless, socket-free adapter: one authored module on stdin, a portable RON
//! artifact on stdout. Failure emits a structured diagnostic and no artifact.
use game_core::authoring::composition::{
    compile_json, Diagnostic, DiagnosticCode, SourceTrace, MAX_INPUT_BYTES,
};
use std::io::{Read, Write};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| boundary_error(DiagnosticCode::Parse, e.to_string()))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(boundary_error(
            DiagnosticCode::Limit,
            "module byte limit exceeded".into(),
        ));
    }
    let input = std::str::from_utf8(&bytes)
        .map_err(|e| boundary_error(DiagnosticCode::Parse, e.to_string()))?;
    let artifact = compile_json(input)
        .and_then(|artifact| artifact.to_ron())
        .map_err(|e| serde_json::to_string(&e).unwrap_or_else(|_| e.to_string()))?;
    std::io::stdout()
        .lock()
        .write_all(artifact.as_bytes())
        .map_err(|e| boundary_error(DiagnosticCode::Native, e.to_string()))
}

fn boundary_error(code: DiagnosticCode, reason: String) -> String {
    serde_json::to_string(&Diagnostic {
        code,
        document: String::new(),
        source_digest: String::new(),
        source: SourceTrace {
            definition: "$".into(),
            calls: vec![],
            inputs: vec![],
        },
        reason,
    })
    .expect("input diagnostic is serializable")
}
