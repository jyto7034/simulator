//! Shared input adapter for standalone public-API probes after the R02 cutover.
//! Each probe still owns its original positive/negative API assertions.
use game_core::game::data::{
    content_pack::{load, ContentSelection, InputLimits},
    ValidatedGameData,
};
use std::path::PathBuf;

pub fn load_selected() -> ValidatedGameData {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    assert!(
        arguments.len() >= 2,
        "usage: API_PROBE TRUSTED_ROOT PACK_RELATIVE_DIRECTORY..."
    );
    load(
        &PathBuf::from(&arguments[0]),
        &ContentSelection {
            packs: arguments[1..].iter().map(PathBuf::from).collect(),
            winners: vec![],
        },
        &InputLimits::default(),
    )
    .expect("explicit external probe selection must publish")
}
