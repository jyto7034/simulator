use std::collections::HashSet;

use game_core::{game::enums::Side, game::resources::Position};

use super::{
    damage_hp_changes_caused_by, hp_deltas, passive_dummy_patch, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry,
    PlacedUnitKind,
};

#[test]
// 목적:
// Fragment Nova가 전방 tile range 안의 적 클러스터에는 광역 피해를 주고,
// 범위 밖의 먼 적은 맞추지 않는지 확인한다.
fn fragment_of_the_universe_nova_hits_the_forward_tile_cluster_but_not_distant_targets() {
    let mut legend = skill_dummy_board_legend();
    for symbol in ['E', 'F', 'X'] {
        legend
            .units
            .insert(symbol, BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    }
    legend.patches.insert('!', passive_dummy_patch(100, None));

    let board = r#"
        . . . . . . .
        . . . . . . .
        . . C D! E! . .
        . . . . F! X! .
        . . . . . . .
    "#;

    let result = run_abnormality_scenario("f-05-52", scenario_from_board(board, &legend));
    let cluster_targets: HashSet<_> = [
        Position::new(3, 2),
        Position::new(4, 2),
        Position::new(4, 3),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Opponent))
            .unwrap()
    })
    .collect();
    let distant_enemy = result
        .unit_instance_at(Position::new(5, 3), Some(Side::Opponent))
        .unwrap();

    let steps = result.first_cast_steps("fragment_universe_nova");
    assert_eq!(step_ids(&steps), vec!["nova"]);

    let hp_changes = damage_hp_changes_caused_by(result.timeline(), steps[0].seq);
    let damaged_targets: HashSet<_> = target_unit_ids(&hp_changes).into_iter().collect();
    assert_eq!(hp_changes.len(), 3);
    assert_eq!(damaged_targets, cluster_targets);
    assert_eq!(hp_deltas(&hp_changes), vec![-35, -35, -35]);
    assert!(
        !damaged_targets.contains(&distant_enemy),
        "distant enemy should stay outside Fragment Nova"
    );
}
