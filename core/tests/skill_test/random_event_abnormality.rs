use game_core::{
    ecs::resources::Position,
    game::{battle::timeline::TimelineEvent, enums::Side},
};

use super::{
    hp_changes_caused_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry, PlacedUnitKind,
};

#[test]
// 목적:
// 랜덤 이벤트 전용 환상체도 일반 전투 경로를 타며,
// Distortion Strike가 가장 가까운 적 하나만 정확히 타격하는지 확인한다.
fn random_event_abnormality_targets_the_nearest_enemy_for_distortion_strike() {
    let mut legend = skill_dummy_board_legend();
    legend
        .units
        .insert('E', BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . C . . . . .
        . D! . . . . .
        . . . . E! . .
    "#;

    let result = run_abnormality_scenario(
        "random_event_abnormality_1",
        scenario_from_board(board, &legend),
    );
    let near_enemy = result
        .unit_instance_at(Position::new(1, 2), Some(Side::Opponent))
        .unwrap();
    let far_enemy = result
        .unit_instance_at(Position::new(4, 3), Some(Side::Opponent))
        .unwrap();

    let cast = result.first_cast_of("unknown_distortion_strike");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == near_enemy
    ));

    let steps = result.first_cast_steps("unknown_distortion_strike");
    assert_eq!(step_ids(&steps), vec!["strike"]);

    let hp_changes = hp_changes_caused_by(result.timeline(), steps[0].seq);
    assert_eq!(hp_changes.len(), 1);
    assert_eq!(target_unit_ids(&hp_changes), vec![near_enemy]);
    assert!(
        !target_unit_ids(&hp_changes).contains(&far_enemy),
        "far enemy should not be hit by Distortion Strike"
    );
}
