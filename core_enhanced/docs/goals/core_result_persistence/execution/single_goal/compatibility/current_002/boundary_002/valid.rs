use game_core::application::*;
fn check(p: PreparedNonBattleApplication<'_>) { let _ = p.result(); let _ = p.checkpoint(); let _ = p.enters_battle(); let _ = p.commit(); }
fn main() {}
