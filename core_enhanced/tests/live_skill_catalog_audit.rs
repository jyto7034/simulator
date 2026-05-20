mod common;

use std::collections::BTreeSet;

use game_core::game::ability::DeliveryDef;
use game_core::game::data::skill_fragment_data::{SkillFragmentEffectDef, SkillFragmentId};

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
        .map(|skill| skill.id.as_str().to_string())
        .collect();

    let expected_spatial_skills = set_of(&[
        "plague_mass_heal",
        "fragment_universe_nova",
        "fairy_festival_blessing",
        "all_around_helper_grinder_mk4",
        "one_sin_penitence",
        "fragment_one_sin_penitence",
        "fragment_scorched_spark",
        "big_bird_dark_lamp",
        "queen_of_hatred_magical_beam",
        "little_red_hunt_the_prey",
        "mountain_mass_consumption",
        "melting_love_slime_infection",
        "white_night_pale_benediction",
        "singing_machine_rhythmic_grinder",
        "freischutz_magic_bullet",
        "fragment_freischutz_black_round",
        "funeral_butterfly_eulogy_volley",
        "laetitia_surprise_gift",
        "snow_queen_crystal_prison",
        "rudolta_sleigh_charge",
        "dreaming_current_coral_surge",
        "alriune_flower_burial",
        "naked_nest_skin_burst",
        "blue_star_gravity_chant",
        "silent_orchestra_finale",
        "yin_black_pulse",
        "yang_white_resonance",
        "burrowing_heaven_gaze_punishment",
        "parasite_tree_root_blessing",
        "porcubbus_spore_bloom",
        "fragment_spider_bud_red_eyes",
        "fragment_funeral_butterfly_eulogy",
        "fragment_helper_grinder_trace",
        "fragment_alriune_faint_aroma",
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
        .map(|skill| skill.id.as_str().to_string())
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
        "fragment_red_shoes_impulse",
        "meat_lantern_lure_trap",
        "censored_mind_break",
        "scarecrow_wisdom_reap",
        "woodsman_heart_rip",
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
        .map(|skill| skill.id.as_str().to_string())
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
        "singing_machine_rhythmic_grinder",
        "all_around_helper_grinder_mk4",
        "freischutz_magic_bullet",
        "funeral_butterfly_eulogy_volley",
        "laetitia_surprise_gift",
        "snow_queen_crystal_prison",
        "rudolta_sleigh_charge",
        "meat_lantern_lure_trap",
        "dreaming_current_coral_surge",
        "alriune_flower_burial",
        "naked_nest_skin_burst",
        "blue_star_gravity_chant",
        "silent_orchestra_finale",
        "censored_mind_break",
        "yin_black_pulse",
        "yang_white_resonance",
        "burrowing_heaven_gaze_punishment",
        "parasite_tree_root_blessing",
        "porcubbus_spore_bloom",
        "scarecrow_wisdom_reap",
        "woodsman_heart_rip",
    ]);
    let item_and_artifact_skill_coverage = set_of(&[
        "paradise_lost_judgement",
        "fourth_match_ember",
        "penitence_guard",
        "resonance_pendant_opening_focus",
        "red_ribbon_haste",
    ]);
    let skill_fragment_skill_coverage = set_of(&[
        "fragment_one_sin_penitence",
        "fragment_scorched_spark",
        "fragment_red_shoes_impulse",
        "fragment_freischutz_black_round",
        "fragment_spider_bud_red_eyes",
        "fragment_funeral_butterfly_eulogy",
        "fragment_helper_grinder_trace",
        "fragment_alriune_faint_aroma",
    ]);

    let covered_skill_ids: BTreeSet<String> = abnormality_skill_coverage
        .union(&item_and_artifact_skill_coverage)
        .cloned()
        .collect::<BTreeSet<_>>()
        .union(&skill_fragment_skill_coverage)
        .cloned()
        .collect();

    assert_eq!(
        actual_live_skill_ids, covered_skill_ids,
        "a live skill was added without updating the scenario/item behavior-test coverage manifest"
    );
}

#[test]
fn a_rank_original_and_fragment_skills_are_independent_live_contracts() {
    let game_data = common::load_game_data_from_ron();
    let pairs = [
        (
            "o-03-03_one_sin",
            "one_sin_penitence",
            "fragment_one_sin_penitence",
            "fragment_one_sin_penitence",
        ),
        (
            "f-01-02",
            "scorched_explosion",
            "fragment_scorched_spark",
            "fragment_scorched_spark",
        ),
        (
            "o-01-45",
            "spider_bud_poison_stack",
            "fragment_spider_bud_red_eyes",
            "fragment_spider_bud_red_eyes",
        ),
        (
            "t-09-09",
            "red_shoes_berserk",
            "fragment_red_shoes_impulse",
            "fragment_red_shoes_impulse",
        ),
        (
            "t-02-43_freischutz",
            "freischutz_magic_bullet",
            "fragment_freischutz_black_round",
            "fragment_freischutz_black_round",
        ),
        (
            "t-02-99_funeral_butterfly",
            "funeral_butterfly_eulogy_volley",
            "fragment_funeral_butterfly_eulogy",
            "fragment_funeral_butterfly_eulogy",
        ),
        (
            "t-05-41_all_around_helper",
            "all_around_helper_grinder_mk4",
            "fragment_helper_grinder_trace",
            "fragment_helper_grinder_trace",
        ),
        (
            "t-04-53_alriune",
            "alriune_flower_burial",
            "fragment_alriune_faint_aroma",
            "fragment_alriune_faint_aroma",
        ),
    ];

    for (abnormality_id, original_skill_id, fragment_id, fragment_skill_id) in pairs {
        let abnormality = game_data
            .abnormality_data
            .get_by_id(abnormality_id)
            .unwrap_or_else(|| panic!("missing A-rank abnormality '{abnormality_id}'"));
        assert_eq!(
            abnormality.skill_id.as_ref().map(|id| id.as_str()),
            Some(original_skill_id),
            "A-rank abnormality '{abnormality_id}' must point at its original skill"
        );

        let original_skill = game_data
            .skill_data
            .get_by_id(original_skill_id)
            .unwrap_or_else(|| panic!("missing original skill '{original_skill_id}'"));
        let fragment_skill = game_data
            .skill_data
            .get_by_id(fragment_skill_id)
            .unwrap_or_else(|| panic!("missing fragment skill '{fragment_skill_id}'"));

        assert_ne!(
            original_skill.id.as_str(),
            fragment_skill.id.as_str(),
            "original and fragment skills must never share the same SkillDef id"
        );
        assert_ne!(
            format!("{:?}", original_skill.steps),
            format!("{:?}", fragment_skill.steps),
            "original skill '{original_skill_id}' and fragment skill '{fragment_skill_id}' must differ in their step contracts"
        );

        let fragment = game_data
            .skill_fragment_data
            .get_by_id(&SkillFragmentId::from(fragment_id))
            .unwrap_or_else(|| panic!("missing skill fragment '{fragment_id}'"));
        match &fragment.effect {
            SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id, ..
            } => assert_eq!(
                imitation_skill_id.as_str(),
                fragment_skill_id,
                "fragment '{fragment_id}' must point at its independent imitation skill"
            ),
            SkillFragmentEffectDef::BasicAttackModifier { .. } => {
                panic!("A-rank fragment '{fragment_id}' must be an active skill")
            }
        }
    }
}
