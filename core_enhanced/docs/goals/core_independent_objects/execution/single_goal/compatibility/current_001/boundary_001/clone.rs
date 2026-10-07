use game_core::application::*;
fn need<T: Clone>() {} fn check() { need::<PreparedNonBattleApplication<'static>>(); }
fn main() {}
