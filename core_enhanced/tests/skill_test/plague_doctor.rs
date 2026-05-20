use std::collections::HashSet;

use game_core::game::enums::Side;
use game_core::game::resources::Position;

use super::{
    healing_hp_changes_caused_by, hp_deltas, passive_dummy_patch, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, step_ids, target_unit_ids, RuntimeStartPatch,
    UnitPatch,
};

#[test]
// 목적:
// Plague Doctor의 mass_heal이 범위 내 아군만 회복하고,
// 적 유닛은 치유 대상에서 제외되는지 확인한다.
fn plague_doctor_mass_heal_restores_allies_in_radius_and_ignores_enemies() {
    let mut legend = skill_dummy_board_legend();
    legend.patches.insert(
        '@',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(180),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    legend
        .patches
        .insert('!', passive_dummy_patch(300, Some(200)));
    legend.patches.insert('#', passive_dummy_patch(800, None));

    let board = r#"
        . . . . . . .
        . S! C@ S! . . .
        . . . D# . . .
    "#;

    let result = run_abnormality_scenario("o-02-56", scenario_from_board(board, &legend));
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
    let enemy = result
        .unit_instance_at(Position::new(3, 2), Some(Side::Opponent))
        .unwrap();

    let steps = result.first_cast_steps("plague_mass_heal");
    assert_eq!(step_ids(&steps), vec!["mass_heal"]);

    let heals = healing_hp_changes_caused_by(result.timeline(), steps[0].seq);
    let healed_targets: HashSet<_> = target_unit_ids(&heals).into_iter().collect();
    assert_eq!(heals.len(), 3);
    assert_eq!(healed_targets, ally_targets);
    assert_eq!(hp_deltas(&heals), vec![40, 40, 40]);
    assert!(
        !healed_targets.contains(&enemy),
        "enemy should not receive Plague Doctor's heal"
    );
}
