use game_core::{
    ecs::resources::Position,
    game::{
        battle::{buffs::BuffId, timeline::TimelineEvent},
        enums::Side,
    },
};

use super::{
    buff_ids, buffs_applied_by, damage_hp_changes_caused_by, hp_deltas, passive_dummy_patch,
    run_abnormality_scenario, scenario_from_board, skill_dummy_board_legend, step_ids,
    target_unit_ids, BoardEntry, PlacedUnitKind,
};

#[test]
// 목적:
// Judgement Bird가 가장 약한 적에게 피해와 stun을 먼저 적용하고,
// 이후 final verdict가 같은 대상에게 이어지는지 검증한다.
fn judgement_bird_weighs_the_lowest_health_enemy_then_delivers_the_final_verdict() {
    let mut legend = skill_dummy_board_legend();
    legend
        .units
        .insert('E', BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    legend
        .patches
        .insert('!', passive_dummy_patch(2_000, Some(1_200)));
    legend
        .patches
        .insert('#', passive_dummy_patch(2_000, Some(1_800)));

    let board = r#"
        . . . . . . .
        . C . D! . . .
        . . . E# . . .
    "#;

    let result = run_abnormality_scenario(
        "o-02-62_judgement_bird",
        scenario_from_board(board, &legend),
    );
    let low_hp_enemy = result
        .unit_instance_at(Position::new(3, 1), Some(Side::Opponent))
        .unwrap();

    let cast = result.first_cast_of("judgement_bird_scales");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == low_hp_enemy
    ));

    let steps = result.first_cast_steps("judgement_bird_scales");
    assert_eq!(step_ids(&steps), vec!["weigh_sins", "final_verdict"]);
    assert_eq!(
        target_unit_ids(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[0].seq
        )),
        vec![low_hp_enemy]
    );
    assert_eq!(
        hp_deltas(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[0].seq
        )),
        vec![-20]
    );
    assert_eq!(
        target_unit_ids(&buffs_applied_by(result.timeline(), steps[0].seq)),
        vec![low_hp_enemy]
    );
    assert_eq!(
        buff_ids(&buffs_applied_by(result.timeline(), steps[0].seq)),
        vec![BuffId::from_name("stun")]
    );
    assert_eq!(
        target_unit_ids(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[1].seq
        )),
        vec![low_hp_enemy]
    );
    assert_eq!(
        hp_deltas(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[1].seq
        )),
        vec![-65]
    );
}
