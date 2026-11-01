use std::sync::Arc;

use game_core::game::data::{
    GameDataBase, GameDataBaseParts, GameDataBuilder, ValidatedGameData,
};
use game_core::game::events::combat::CombatExecutor;

fn require_validated_input<T: Into<ValidatedGameData>>() {}

fn main() {
    let _ = GameDataBuilder::empty();
    let _: Option<GameDataBaseParts> = None;
    require_validated_input::<Arc<GameDataBase>>();
    let _ = CombatExecutor::build_battle_with_combat_preview;
}
