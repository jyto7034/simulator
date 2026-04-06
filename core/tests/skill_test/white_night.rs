use std::collections::HashSet;

use game_core::ecs::resources::Position;
use game_core::game::enums::Side;

use super::{
    hp_changes_caused_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, stat_changes_caused_by, step_ids, target_unit_ids, BoardEntry,
    PlacedUnitKind, RuntimeStartPatch, UnitPatch,
};

#[test]
// 목적:
// White Night의 3단계 스킬이 아군 치유 -> 아군 강화 -> 적 심판 순서로 진행되고,
// 각 단계의 대상 집합이 의도대로 분리되는지 확인한다.
fn white_night_heals_and_buffs_allies_before_judging_enemies() {
    let mut legend = skill_dummy_board_legend();
    legend
        .units
        .insert('A', BoardEntry::Player(PlacedUnitKind::SkillDummy));
    legend
        .units
        .insert('E', BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    legend
        .units
        .insert('F', BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    legend.patches.insert(
        '@',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(280),
            },
            ..Default::default()
        },
    );
    legend
        .patches
        .insert('!', passive_dummy_patch(300, Some(220)));
    legend.patches.insert('#', passive_dummy_patch(900, None));

    let board = r#"
        . . . . . . .
        . S! C@ A! . . .
        . D# E# F# . . .
    "#;

    let result =
        run_abnormality_scenario("o-01-45_white_night", scenario_from_board(board, &legend));
    let ally_targets: HashSet<_> = [
        Position::new(1, 1),
        Position::new(2, 1),
        Position::new(3, 1),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Player))
            .unwrap()
    })
    .collect();
    let enemy_targets: HashSet<_> = [
        Position::new(1, 2),
        Position::new(2, 2),
        Position::new(3, 2),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Opponent))
            .unwrap()
    })
    .collect();

    let steps = result.first_cast_steps("white_night_pale_benediction");
    assert_eq!(
        step_ids(&steps),
        vec!["ally_salvation", "ally_blessing", "enemy_judgement"]
    );

    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[0].seq))
            .into_iter()
            .collect::<HashSet<_>>(),
        ally_targets
    );
    assert_eq!(
        target_unit_ids(&stat_changes_caused_by(result.timeline(), steps[1].seq))
            .into_iter()
            .collect::<HashSet<_>>(),
        ally_targets
    );
    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[2].seq))
            .into_iter()
            .collect::<HashSet<_>>(),
        enemy_targets
    );
}
