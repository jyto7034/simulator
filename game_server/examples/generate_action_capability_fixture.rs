use game_core::{
    application::{ApplicationSnapshotKind, CoreApplication},
    game::data::GameDataBuilder,
};
use serde_json::json;

fn main() {
    let core = CoreApplication::new(GameDataBuilder::empty().build_arc(), 7);
    let state = core
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("not-started capability fixture must serialize");
    let fixture = json!({
        "generator": "game_server/examples/generate_action_capability_fixture.rs",
        "state": state.read_model().as_value(),
    });

    if let Some(path) = std::env::args().nth(1) {
        let checked_in: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&path).expect("checked-in fixture must be readable"),
        )
        .expect("checked-in fixture must be valid JSON");
        assert_eq!(
            checked_in, fixture,
            "checked-in fixture differs from typed Rust serialization: {path}"
        );
        return;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&fixture).expect("fixture must serialize")
    );
}
