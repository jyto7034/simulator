mod common;

use std::collections::BTreeSet;

use game_core::game::ability::DeliveryDef;

fn set_of(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn live_skill_catalog_matches_current_delivery_audit() {
    let game_data = common::load_game_data_from_ron();

    let actual_spatial_skills: BTreeSet<String> = game_data
        .skill_data
        .skills
        .iter()
        .filter(|skill| {
            skill
                .steps
                .iter()
                .any(|step| !matches!(step.delivery, DeliveryDef::Instant))
        })
        .map(|skill| skill.id.clone())
        .collect();

    let expected_spatial_skills = set_of(&[
        "plague_mass_heal",
        "fragment_universe_nova",
        "fairy_festival_blessing",
        "one_sin_penitence",
        "big_bird_dark_lamp",
        "queen_of_hatred_magical_beam",
        "little_red_hunt_the_prey",
        "mountain_mass_consumption",
        "melting_love_slime_infection",
        "white_night_pale_benediction",
    ]);

    assert_eq!(
        actual_spatial_skills, expected_spatial_skills,
        "live skill delivery composition changed; re-audit which skills should stay intentional Instant vs spatial delivery"
    );

    let actual_intentional_instant_skills: BTreeSet<String> = game_data
        .skill_data
        .skills
        .iter()
        .filter(|skill| {
            skill
                .steps
                .iter()
                .all(|step| matches!(step.delivery, DeliveryDef::Instant))
        })
        .map(|skill| skill.id.clone())
        .collect();

    let expected_intentional_instant_skills = set_of(&[
        "scorched_explosion",
        "red_shoes_berserk",
        "spider_bud_poison_stack",
        "unknown_distortion_strike",
        "punishing_bird_rapid_peck",
        "judgement_bird_scales",
        "nothing_there_goodbye",
        "paradise_lost_judgement",
        "fourth_match_ember",
        "penitence_guard",
        "resonance_pendant_opening_focus",
        "red_ribbon_haste",
    ]);

    assert_eq!(
        actual_intentional_instant_skills, expected_intentional_instant_skills,
        "live Instant-only skill set changed; re-audit whether any newly added skill should migrate to Projectile/Area"
    );
}

#[test]
fn live_skill_catalog_is_fully_covered_by_behavior_test_manifest() {
    let game_data = common::load_game_data_from_ron();
    let actual_live_skill_ids: BTreeSet<String> = game_data
        .skill_data
        .skills
        .iter()
        .map(|skill| skill.id.clone())
        .collect();

    let abnormality_skill_coverage = set_of(&[
        "scorched_explosion",
        "plague_mass_heal",
        "red_shoes_berserk",
        "fragment_universe_nova",
        "spider_bud_poison_stack",
        "fairy_festival_blessing",
        "unknown_distortion_strike",
        "one_sin_penitence",
        "punishing_bird_rapid_peck",
        "big_bird_dark_lamp",
        "judgement_bird_scales",
        "queen_of_hatred_magical_beam",
        "little_red_hunt_the_prey",
        "mountain_mass_consumption",
        "melting_love_slime_infection",
        "nothing_there_goodbye",
        "white_night_pale_benediction",
    ]);
    let item_and_artifact_skill_coverage = set_of(&[
        "paradise_lost_judgement",
        "fourth_match_ember",
        "penitence_guard",
        "resonance_pendant_opening_focus",
        "red_ribbon_haste",
    ]);

    let covered_skill_ids: BTreeSet<String> = abnormality_skill_coverage
        .union(&item_and_artifact_skill_coverage)
        .cloned()
        .collect();

    assert_eq!(
        actual_live_skill_ids, covered_skill_ids,
        "a live skill was added without updating the scenario/item behavior-test coverage manifest"
    );
}
