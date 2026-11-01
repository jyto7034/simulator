//! One restricted-YAML editor request on stdin; versioned compile result on stdout.
use game_core::authoring::composition::{compile_request, CompileRequest, MAX_INPUT_BYTES};
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
        .take((MAX_INPUT_BYTES * 6 + 1025) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_INPUT_BYTES * 6 + 1024 {
        return Err("request byte limit exceeded".into());
    }
    let request: CompileRequest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let response = compile_request(request)?;
    let output = serde_json::to_vec(&response).map_err(|e| e.to_string())?;
    std::io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|e| e.to_string())
}
