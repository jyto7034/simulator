use game_core::authoring::{
    composition::{run_composition, CompositionRunRequest, MAX_RUN_REQUEST_BYTES},
    CancellationToken,
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
        .take((MAX_RUN_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_RUN_REQUEST_BYTES {
        return Err("run request byte limit exceeded".into());
    }
    let request: CompositionRunRequest =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let response = run_composition(request, &CancellationToken::default())?;
    let bytes = serde_json::to_vec(&response).map_err(|e| e.to_string())?;
    std::io::stdout()
        .lock()
        .write_all(&bytes)
        .map_err(|e| e.to_string())
}
