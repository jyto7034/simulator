use std::collections::HashSet;

use game_core::{
    ecs::resources::Position,
    game::{
        enums::Side,
        stats::{StatId, StatModifierKind},
    },
};

use super::{
    healing_hp_changes_caused_by, hp_deltas, passive_dummy_patch, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, stat_changes_caused_by, stat_modifier_summaries,
    step_ids, target_unit_ids, RuntimeStartPatch, UnitPatch,
};

#[test]
// 목적:
// Fairy Festival의 blessing이 범위 내 아군 전원에게 heal과 attack buff를 적용하고,
// 적은 같은 보드에 있어도 대상에 포함되지 않는지 검증한다.
fn fairy_festival_blessing_heals_and_buffs_all_allies_in_range_but_not_enemies() {
    let mut legend = skill_dummy_board_legend();
    legend.patches.insert(
        '@',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(150),
                resonance_current: Some(100),
                pending_cast: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    legend
        .patches
        .insert('!', passive_dummy_patch(400, Some(10)));
    legend.patches.insert('#', passive_dummy_patch(800, None));

    let board = r#"
        . . . . . . .
        . . S! C@ S! . .
        . . . . . . .
        . . . D# . . .
    "#;

    let result = run_abnormality_scenario("f-01-37", scenario_from_board(board, &legend));
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
    let enemy = result
        .unit_instance_at(Position::new(3, 3), Some(Side::Opponent))
        .unwrap();

    let steps = result.first_cast_steps("fairy_festival_blessing");
    assert_eq!(step_ids(&steps), vec!["fairy_bless"]);

    let heals = healing_hp_changes_caused_by(result.timeline(), steps[0].seq);
    let healed_targets: HashSet<_> = target_unit_ids(&heals).into_iter().collect();
    assert_eq!(heals.len(), 3);
    assert_eq!(healed_targets, ally_targets);
    assert_eq!(hp_deltas(&heals), vec![20, 20, 20]);

    let buffs = stat_changes_caused_by(result.timeline(), steps[0].seq);
    let buffed_targets: HashSet<_> = target_unit_ids(&buffs).into_iter().collect();
    assert_eq!(buffs.len(), 3);
    assert_eq!(buffed_targets, ally_targets);
    assert_eq!(
        stat_modifier_summaries(&buffs),
        vec![(StatId::Attack, StatModifierKind::Percent, 15); 3]
    );
    assert!(
        !buffed_targets.contains(&enemy),
        "enemy should not receive Fairy Festival's buff"
    );
}
