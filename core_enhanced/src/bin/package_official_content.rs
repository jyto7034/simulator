use game_core::game::data::content_pack::{package_official_with_skill_files, InputLimits};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    const USAGE: &str = "usage: package_official_content SOURCE_ROOT NEW_PACK_DIRECTORY [--skill-file ROOT_RELATIVE_PATH]...";
    if arguments.len() < 2 || (arguments.len() - 2) % 2 != 0 {
        return Err(USAGE.into());
    }
    let mut skill_files = Vec::new();
    for pair in arguments[2..].chunks_exact(2) {
        if pair[0] != "--skill-file" {
            return Err(USAGE.into());
        }
        skill_files.push(PathBuf::from(&pair[1]));
    }
    let receipt = package_official_with_skill_files(
        &PathBuf::from(&arguments[0]),
        &PathBuf::from(&arguments[1]),
        &skill_files,
        &InputLimits::default(),
    )?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
