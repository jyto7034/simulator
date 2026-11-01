use game_core::{application::CoreApplication, game::data::ValidatedGameData};

fn main() {
    let game_data = ValidatedGameData::load_live_embedded().expect("live validated data");
    let mut application = CoreApplication::new(game_data, 1);
    let _ = application.replace_active_battle_event_log_fixture(8_192);
}
