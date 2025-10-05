const ENQUEUE_PLAYER_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/ENQUEUE_PLAYER.lua"
));
const DEQUEUE_PLAYER_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/DEQUEUE_PLAYER.lua"
));
const TRY_MATCH_POP_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/TRY_MATCH_POP.lua"
));

pub fn enqueue_player_script() -> &'static str {
    ENQUEUE_PLAYER_SCRIPT
}

pub fn dequeue_player_script() -> &'static str {
    DEQUEUE_PLAYER_SCRIPT
}

pub fn try_match_pop_script() -> &'static str {
    TRY_MATCH_POP_SCRIPT
}
