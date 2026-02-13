mod common;

#[test]
fn common_helpers_build_minimal_game_data() {
    let _ = common::create_test_game_data();
    let _ = common::empty_game_data();
}
