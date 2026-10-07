use game_core::application::*;
fn check() { let _ = serde_json::from_str::<PreparedNonBattleApplication<'static>>("{}"); }
fn main() {}
