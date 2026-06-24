use std::collections::HashSet;

use game_core::{
    game::resources::Position,
    game::{
        enums::Side,
        stats::{StatId, StatModifierKind},
    },
};

use super::{
    damage_hp_changes_caused_by, healing_hp_changes_caused_by, hp_deltas, passive_dummy_patch,
    run_abnormality_scenario, scenario_from_board, skill_dummy_board_legend,
    stat_changes_caused_by, stat_modifier_summaries, step_ids, target_unit_ids, BoardEntry,
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
                ..Default::default()
            },
            ..Default::default()
        },
    );
    legend
        .patches
        .insert('!', passive_dummy_patch(300, Some(220)));
    legend.patches.insert('#', passive_dummy_patch(900, None));

    let board = r#"
        . . . F# . . .
        . . C@ A! S! . .
        . . . D# E# . .
    "#;

    let result =
        run_abnormality_scenario("o-01-45_white_night", scenario_from_board(board, &legend));
    let ally_targets: HashSet<_> = [
        Position::new(2, 1),
        Position::new(3, 1),
        Position::new(4, 1),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Player))
            .unwrap()
    })
    .collect();
    let enemy_targets: HashSet<_> = [
        Position::new(3, 0),
        Position::new(3, 2),
        Position::new(4, 2),
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
        target_unit_ids(&healing_hp_changes_caused_by(
            result.event_log(),
            steps[0].seq
        ))
        .into_iter()
        .collect::<HashSet<_>>(),
        ally_targets
    );
    assert_eq!(
        hp_deltas(&healing_hp_changes_caused_by(
            result.event_log(),
            steps[0].seq
        )),
        vec![55, 55, 55]
    );
    assert_eq!(
        target_unit_ids(&stat_changes_caused_by(result.event_log(), steps[1].seq))
            .into_iter()
            .collect::<HashSet<_>>(),
        ally_targets
    );
    assert_eq!(
        stat_modifier_summaries(&stat_changes_caused_by(result.event_log(), steps[1].seq)),
        vec![(StatId::Attack, StatModifierKind::Percent, 20); 3]
    );
    assert_eq!(
        target_unit_ids(&damage_hp_changes_caused_by(
            result.event_log(),
            steps[2].seq
        ))
        .into_iter()
        .collect::<HashSet<_>>(),
        enemy_targets
    );
    assert_eq!(
        hp_deltas(&damage_hp_changes_caused_by(
            result.event_log(),
            steps[2].seq
        )),
        vec![-28, -28, -28]
    );
}
