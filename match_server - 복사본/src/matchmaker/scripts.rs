// Embed Lua scripts at compile time to avoid runtime FS/working-dir issues.
const ATOMIC_MATCH_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/ATOMIC_MATCH_SCRIPT.lua"
));
const ATOMIC_LOADING_COMPLETE_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/ATOMIC_LOADING_COMPLETE_SCRIPT.lua"
));
const ATOMIC_CANCEL_SESSION_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/ATOMIC_CANCEL_SESSION_SCRIPT.lua"
));
const CLEANUP_STALE_SESSION_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/CLEANUP_STALE_SESSION_SCRIPT.lua"
));

pub(super) fn get_atomic_match_script() -> &'static str { ATOMIC_MATCH_SCRIPT }
pub(super) fn get_atomic_loading_complete_script() -> &'static str { ATOMIC_LOADING_COMPLETE_SCRIPT }
pub(super) fn get_atomic_cancel_session_script() -> &'static str { ATOMIC_CANCEL_SESSION_SCRIPT }
pub(super) fn get_cleanup_stale_session_script() -> &'static str { CLEANUP_STALE_SESSION_SCRIPT }
