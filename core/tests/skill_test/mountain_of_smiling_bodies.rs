use std::collections::HashSet;

use game_core::{ecs::resources::Position, game::enums::Side};

use super::{
    hp_changes_caused_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, stat_changes_caused_by, step_ids, target_unit_ids, BoardEntry,
    PlacedUnitKind, RuntimeStartPatch, UnitPatch,
};

#[test]
// 목적:
// Mountain of Smiling Bodies가 주변 적을 광역으로 타격한 뒤,
// 성장 버프와 자가 회복을 순서대로 자신에게 적용하는지 검증한다.
fn mountain_of_smiling_bodies_bursts_the_cluster_then_grows_and_heals_itself() {
    let mut legend = skill_dummy_board_legend();
    for symbol in ['E', 'F'] {
        legend
            .units
            .insert(symbol, BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    }
    legend.patches.insert(
        '@',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(350),
            },
            ..Default::default()
        },
    );
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . . . . . . .
        . . C@ F! . . .
        . . D! E! . . .
    "#;

    let result = run_abnormality_scenario("t-01-75_mountain", scenario_from_board(board, &legend));
    let cluster_targets: HashSet<_> = [
        Position::new(2, 3),
        Position::new(3, 3),
        Position::new(3, 2),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Opponent))
            .unwrap()
    })
    .collect();
    let caster = result.caster_instance_id();

    let steps = result.first_cast_steps("mountain_mass_consumption");
    assert_eq!(
        step_ids(&steps),
        vec!["consume_burst", "consume_growth", "consume_heal"]
    );

    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[0].seq))
            .into_iter()
            .collect::<HashSet<_>>(),
        cluster_targets
    );
    assert_eq!(
        target_unit_ids(&stat_changes_caused_by(result.timeline(), steps[1].seq)),
        vec![caster]
    );
    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[2].seq)),
        vec![caster]
    );
}
