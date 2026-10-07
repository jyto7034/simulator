use game_core::game::data::content_pack::{package_official, InputLimits};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.len() != 2 {
        return Err("usage: package_official_content SOURCE_ROOT NEW_PACK_DIRECTORY".into());
    }
    let receipt = package_official(
        &PathBuf::from(&arguments[0]),
        &PathBuf::from(&arguments[1]),
        &InputLimits::default(),
    )?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
