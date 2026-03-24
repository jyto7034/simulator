mod common;

#[test]
fn load_game_data_from_ron_reads_step_based_skill_schema() {
    let game_data = common::load_game_data_from_ron();

    let skill = game_data
        .skill_data
        .get_by_id("scorched_explosion")
        .expect("scorched_explosion skill should exist in RON data");

    assert_eq!(skill.id, "scorched_explosion");
    assert_eq!(skill.steps.len(), 1);
    assert_eq!(skill.steps[0].id, "explode");
    assert_eq!(skill.steps[0].range_tiles, 2);

    let abno = game_data
        .abnormality_data
        .get_by_id("f-01-02")
        .expect("Scorched Girl abnormality should exist in RON data");
    assert_eq!(abno.skill_id.as_deref(), Some("scorched_explosion"));

    let white_night = game_data
        .skill_data
        .get_by_id("white_night_pale_benediction")
        .expect("WhiteNight skill should exist in RON data");
    assert_eq!(white_night.steps.len(), 2);
    assert_eq!(white_night.steps[0].id, "ally_blessing");
    assert_eq!(white_night.steps[1].id, "enemy_judgement");

    let mountain = game_data
        .abnormality_data
        .get_by_id("t-01-75_mountain")
        .expect("Mountain of Smiling Bodies abnormality should exist in RON data");
    assert_eq!(mountain.skill_id.as_deref(), Some("mountain_mass_consumption"));

    let one_sin = game_data
        .abnormality_data
        .get_by_id("o-03-03_one_sin")
        .expect("One Sin abnormality should exist in RON data");
    assert_eq!(one_sin.skill_id.as_deref(), Some("one_sin_penitence"));
}
