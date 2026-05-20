use game_core::{
    game::resources::Position,
    game::{battle::timeline::TimelineEvent, enums::Side},
};

use super::{
    damage_hp_changes_caused_by, hp_deltas, passive_dummy_patch, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry,
    PlacedUnitKind,
};

#[test]
// 목적:
// 가까운 적 하나와 더 먼 적이 함께 있을 때,
// Scorched Explosion이 가장 가까운 적만 단일 대상으로 피해를 주는지 검증한다.
fn scorched_girl_targets_the_nearest_enemy_and_only_damages_that_target() {
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

    let result = run_abnormality_scenario("f-01-02", scenario_from_board(board, &legend));
    let near_enemy = result
        .unit_instance_at(Position::new(1, 2), Some(Side::Opponent))
        .unwrap();
    let far_enemy = result
        .unit_instance_at(Position::new(4, 3), Some(Side::Opponent))
        .unwrap();

    let cast = result.first_cast_of("scorched_explosion");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == near_enemy
    ));

    let steps = result.first_cast_steps("scorched_explosion");
    assert_eq!(step_ids(&steps), vec!["explode"]);

    let hp_changes = damage_hp_changes_caused_by(result.timeline(), steps[0].seq);
    assert_eq!(hp_changes.len(), 1);
    assert_eq!(target_unit_ids(&hp_changes), vec![near_enemy]);
    assert_eq!(hp_deltas(&hp_changes), vec![-50]);
    assert!(
        !target_unit_ids(&hp_changes).contains(&far_enemy),
        "far enemy should not be hit by Scorched Explosion"
    );
}
