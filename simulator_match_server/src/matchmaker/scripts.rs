use std::path::Path;
use std::fs;
use std::sync::OnceLock;

static ATOMIC_MATCH_SCRIPT: OnceLock<String> = OnceLock::new();
static ATOMIC_LOADING_COMPLETE_SCRIPT: OnceLock<String> = OnceLock::new();
static ATOMIC_CANCEL_SESSION_SCRIPT: OnceLock<String> = OnceLock::new();
static CLEANUP_STALE_SESSION_SCRIPT: OnceLock<String> = OnceLock::new();

fn load_script(filename: &str) -> String {
    let script_path = Path::new("scripts").join(filename);
    fs::read_to_string(&script_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to load script {}: {}", script_path.display(), e);
            String::new()
        })
}

pub(super) fn get_atomic_match_script() -> &'static str {
    ATOMIC_MATCH_SCRIPT.get_or_init(|| load_script("ATOMIC_MATCH_SCRIPT.lua"))
}

pub(super) fn get_atomic_loading_complete_script() -> &'static str {
    ATOMIC_LOADING_COMPLETE_SCRIPT.get_or_init(|| load_script("ATOMIC_LOADING_COMPLETE_SCRIPT.lua"))
}

pub(super) fn get_atomic_cancel_session_script() -> &'static str {
    ATOMIC_CANCEL_SESSION_SCRIPT.get_or_init(|| load_script("ATOMIC_CANCEL_SESSION_SCRIPT.lua"))
}

pub(super) fn get_cleanup_stale_session_script() -> &'static str {
    CLEANUP_STALE_SESSION_SCRIPT.get_or_init(|| load_script("CLEANUP_STALE_SESSION_SCRIPT.lua"))
}
