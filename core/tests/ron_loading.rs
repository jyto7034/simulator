mod common;

use game_core::game::ability::{
    DeliveryDef, SkillAreaAnchorSource, SkillAreaShapeDef, SkillTarget,
};

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
    assert_eq!(white_night.steps.len(), 3);
    assert_eq!(white_night.steps[0].id, "ally_salvation");
    assert_eq!(white_night.steps[1].id, "ally_blessing");
    assert_eq!(white_night.steps[2].id, "enemy_judgement");

    let mountain = game_data
        .abnormality_data
        .get_by_id("t-01-75_mountain")
        .expect("Mountain of Smiling Bodies abnormality should exist in RON data");
    assert_eq!(
        mountain.skill_id.as_deref(),
        Some("mountain_mass_consumption")
    );

    let one_sin = game_data
        .abnormality_data
        .get_by_id("o-03-03_one_sin")
        .expect("One Sin abnormality should exist in RON data");
    assert_eq!(one_sin.skill_id.as_deref(), Some("one_sin_penitence"));

    let one_sin_skill = game_data
        .skill_data
        .get_by_id("one_sin_penitence")
        .expect("One Sin skill should exist in RON data");
    assert_eq!(one_sin_skill.steps.len(), 2);
    assert_eq!(one_sin_skill.steps[0].id, "penitence_judgement");
    assert_eq!(one_sin_skill.steps[1].id, "penitence_absolution");
    assert_eq!(
        one_sin_skill.steps[0]
            .presentation
            .projectile_vfx_id
            .as_deref(),
        Some("one_sin_penitence_judgement")
    );
    assert_eq!(
        one_sin_skill.steps[1].presentation.impact_vfx_id.as_deref(),
        Some("one_sin_penitence_absolution")
    );
    assert_eq!(
        one_sin_skill.steps[0].presentation.target_anchor.as_deref(),
        Some("Head")
    );

    let queen = game_data
        .skill_data
        .get_by_id("queen_of_hatred_magical_beam")
        .expect("Queen skill should exist in RON data");
    assert!(matches!(
        queen.steps[0].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Line {
                    length_units: 5_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));

    let melting_love = game_data
        .skill_data
        .get_by_id("melting_love_slime_infection")
        .expect("Melting Love skill should exist in RON data");
    assert!(matches!(
        melting_love.steps[1].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 2_000_000,
                    height_units: 2_000_000,
                },
                anchor: SkillAreaAnchorSource::ImpactContext,
                ..
            }
        }
    ));

    let white_night = game_data
        .skill_data
        .get_by_id("white_night_pale_benediction")
        .expect("WhiteNight skill should exist in RON data");
    assert!(matches!(
        white_night.steps[0].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 6_000_000,
                    height_units: 6_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: true,
                ..
            }
        }
    ));
    assert!(matches!(
        white_night.steps[2].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 6_000_000,
                    height_units: 6_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: false,
                ..
            }
        }
    ));

    assert_eq!(
        white_night.steps[0].presentation.impact_vfx_id.as_deref(),
        Some("white_night_pale_benediction_salvation")
    );
    assert_eq!(
        white_night.steps[1].presentation.cast_state.as_deref(),
        Some("Cast")
    );
    assert_eq!(
        white_night.steps[2].presentation.target_anchor.as_deref(),
        Some("Head")
    );

    let plague = game_data
        .skill_data
        .get_by_id("plague_mass_heal")
        .expect("Plague skill should exist in RON data");
    assert!(matches!(
        plague.steps[0].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 6_000_000,
                    height_units: 6_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: true,
                ..
            }
        }
    ));

    let fragment = game_data
        .skill_data
        .get_by_id("fragment_universe_nova")
        .expect("Fragment skill should exist in RON data");
    assert!(matches!(
        fragment.steps[0].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 4_000_000,
                    height_units: 4_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));

    let fairy = game_data
        .skill_data
        .get_by_id("fairy_festival_blessing")
        .expect("Fairy skill should exist in RON data");
    assert!(matches!(
        fairy.steps[0].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 6_000_000,
                    height_units: 6_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: true,
                ..
            }
        }
    ));

    let dark_lamp = game_data
        .skill_data
        .get_by_id("big_bird_dark_lamp")
        .expect("Dark Lamp skill should exist in RON data");
    assert!(matches!(
        dark_lamp.steps[0].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 2_000_000,
                    height_units: 2_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));
    assert!(matches!(
        dark_lamp.steps[1].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 2_000_000,
                    height_units: 2_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));

    let mountain_skill = game_data
        .skill_data
        .get_by_id("mountain_mass_consumption")
        .expect("Mountain skill should exist in RON data");
    assert!(matches!(
        mountain_skill.steps[0].delivery,
        DeliveryDef::Area {
            area: game_core::game::ability::SkillAreaDeliveryDef {
                shape: SkillAreaShapeDef::Box {
                    width_units: 2_000_000,
                    height_units: 2_000_000,
                },
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));
}

#[test]
fn live_skill_area_targets_use_area_delivery() {
    let game_data = common::load_game_data_from_ron();

    for skill in game_data.skill_data.skills.iter() {
        for step in &skill.steps {
            let expects_spatial_area = matches!(
                step.target,
                SkillTarget::Allies { .. } | SkillTarget::Enemies { .. }
            );
            if expects_spatial_area {
                assert!(
                    matches!(step.delivery, DeliveryDef::Area { .. }),
                    "skill `{}` step `{}` targets an area but still uses {:?}",
                    skill.id,
                    step.id,
                    step.delivery
                );
            }
        }
    }
}

#[test]
fn projectile_vfx_steps_use_projectile_delivery() {
    let game_data = common::load_game_data_from_ron();

    for skill in game_data.skill_data.skills.iter() {
        for step in &skill.steps {
            if step.presentation.projectile_vfx_id.is_some() {
                assert!(
                    matches!(step.delivery, DeliveryDef::Projectile { .. }),
                    "skill `{}` step `{}` declares projectile_vfx_id but uses {:?}",
                    skill.id,
                    step.id,
                    step.delivery
                );
            }
        }
    }
}

#[test]
fn remaining_instant_steps_are_intentionally_non_spatial() {
    let game_data = common::load_game_data_from_ron();

    for skill in game_data.skill_data.skills.iter() {
        for step in &skill.steps {
            if matches!(step.delivery, DeliveryDef::Instant) {
                assert!(
                    !matches!(
                        step.target,
                        SkillTarget::Allies { .. } | SkillTarget::Enemies { .. }
                    ),
                    "skill `{}` step `{}` still uses Instant for area target {:?}",
                    skill.id,
                    step.id,
                    step.target
                );
                assert!(
                    step.presentation.projectile_vfx_id.is_none(),
                    "skill `{}` step `{}` still uses Instant despite projectile_vfx_id",
                    skill.id,
                    step.id
                );
            }
        }
    }
}
