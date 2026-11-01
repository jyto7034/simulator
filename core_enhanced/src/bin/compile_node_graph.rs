//! One compiler request on stdin, one JSON response on stdout. This command
//! does not write files, mutate game state, or reinterpret natural-language actions.
use std::io::{Read, Write};
fn main() {
    let result = run();
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin()
        .take(7_000_001)
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    if input.len() > 7_000_000 {
        return Err("compiler request byte limit exceeded".into());
    }
    let request = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    let response = game_core::authoring::node_graph::compile_request(request)?;
    serde_json::to_writer(std::io::stdout().lock(), &response).map_err(|e| e.to_string())?;
    std::io::stdout()
        .write_all(b"\n")
        .map_err(|e| e.to_string())
}
