use std::collections::HashSet;

use game_core::ecs::resources::Position;
use game_core::game::enums::Side;

use super::{
    buffs_applied_by, hp_changes_caused_by, passive_dummy_patch, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry,
    PlacedUnitKind,
};

#[test]
// 목적:
// Melting Love의 slime_orb가 가장 약한 적 하나를 먼저 맞추고,
// slime_spread가 그 주변 클러스터 전체에 피해와 poison을 퍼뜨리는지 확인한다.
fn melting_love_orb_marks_the_lowest_health_enemy_and_spread_hits_the_whole_cluster() {
    let mut legend = skill_dummy_board_legend();
    for symbol in ['E', 'F'] {
        legend
            .units
            .insert(symbol, BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    }
    legend
        .patches
        .insert('!', passive_dummy_patch(2_000, Some(1_200)));
    legend
        .patches
        .insert('#', passive_dummy_patch(2_000, Some(1_800)));

    let board = r#"
        . . . . . . .
        . . . . . . .
        . C D! . . . .
        . . E# F# . . .
    "#;

    let result =
        run_abnormality_scenario("d-03-109_melting_love", scenario_from_board(board, &legend));
    let primary_target = result
        .unit_instance_at(Position::new(2, 2), Some(Side::Opponent))
        .unwrap();
    let spread_targets: HashSet<_> = [
        Position::new(2, 2),
        Position::new(2, 3),
        Position::new(3, 3),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Opponent))
            .unwrap()
    })
    .collect();

    let steps = result.first_cast_steps("melting_love_slime_infection");
    assert_eq!(step_ids(&steps), vec!["slime_orb", "slime_spread"]);

    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[0].seq)),
        vec![primary_target]
    );
    assert_eq!(
        target_unit_ids(&buffs_applied_by(result.timeline(), steps[0].seq)),
        vec![primary_target]
    );
    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[1].seq))
            .into_iter()
            .collect::<HashSet<_>>(),
        spread_targets
    );
    assert_eq!(
        target_unit_ids(&buffs_applied_by(result.timeline(), steps[1].seq))
            .into_iter()
            .collect::<HashSet<_>>(),
        spread_targets
    );
}
