use game_core::{ecs::resources::Position, game::enums::Side};

use super::{
    hp_changes_caused_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, stat_changes_caused_by, step_ids, target_unit_ids, RuntimeStartPatch,
    UnitPatch,
};

#[test]
// 목적:
// Nothing There의 Goodbye가 첫 타격 후 자가 공격력 상승을 얻고,
// 마지막 finish 단계에서 같은 적에게 2회 추가 피해를 주는지 검증한다.
fn nothing_there_goodbye_tears_open_then_adapts_and_finishes_twice() {
    let mut legend = skill_dummy_board_legend();
    legend.patches.insert(
        '@',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(430),
            },
            ..Default::default()
        },
    );
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . C@ . . . . .
        . D! . . . . .
    "#;

    let result =
        run_abnormality_scenario("o-06-20_nothing_there", scenario_from_board(board, &legend));
    let enemy = result
        .unit_instance_at(Position::new(1, 2), Some(Side::Opponent))
        .unwrap();
    let caster = result.caster_instance_id();

    let steps = result.first_cast_steps("nothing_there_goodbye");
    assert_eq!(
        step_ids(&steps),
        vec!["tear_open", "mimic_adaptation", "goodbye_finish"]
    );
    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[0].seq)),
        vec![enemy]
    );
    assert_eq!(
        target_unit_ids(&stat_changes_caused_by(result.timeline(), steps[1].seq)),
        vec![caster]
    );
    assert_eq!(
        target_unit_ids(&hp_changes_caused_by(result.timeline(), steps[2].seq)),
        vec![enemy, enemy]
    );
}
