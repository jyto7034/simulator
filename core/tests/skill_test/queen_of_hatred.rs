use std::collections::HashSet;

use game_core::{ecs::resources::Position, game::enums::Side};

use super::{
    hp_changes_caused_by, hp_deltas, passive_dummy_patch, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry,
    PlacedUnitKind,
};

#[test]
// 목적:
// Queen of Hatred의 Magical Beam이 일직선 적만 타격하며,
// trace 1회와 overdrive 2회 반복이 모두 반영되는지 검증한다.
fn queen_of_hatred_beam_hits_the_line_and_repeats_overdrive_twice() {
    let mut legend = skill_dummy_board_legend();
    for symbol in ['E', 'F', 'X'] {
        legend
            .units
            .insert(symbol, BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    }
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . . . . . . .
        . . . . . . .
        . C D! E! F! . .
        . . . . X! . .
    "#;

    let result = run_abnormality_scenario(
        "o-01-04_queen_of_hatred",
        scenario_from_board(board, &legend),
    );
    let line_targets: HashSet<_> = [
        Position::new(2, 3),
        Position::new(3, 3),
        Position::new(4, 3),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Opponent))
            .unwrap()
    })
    .collect();
    let off_line_enemy = result
        .unit_instance_at(Position::new(4, 4), Some(Side::Opponent))
        .unwrap();

    let steps = result.first_cast_steps("queen_of_hatred_magical_beam");
    assert_eq!(step_ids(&steps), vec!["beam_trace", "beam_overdrive"]);

    let trace_hits = hp_changes_caused_by(result.timeline(), steps[0].seq);
    assert_eq!(trace_hits.len(), 3);
    assert_eq!(
        target_unit_ids(&trace_hits)
            .into_iter()
            .collect::<HashSet<_>>(),
        line_targets
    );
    assert_eq!(hp_deltas(&trace_hits), vec![-32, -32, -32]);

    let overdrive_hits = hp_changes_caused_by(result.timeline(), steps[1].seq);
    assert_eq!(overdrive_hits.len(), 6);
    assert!(
        target_unit_ids(&overdrive_hits)
            .into_iter()
            .all(|target| line_targets.contains(&target)),
        "Overdrive should only hit enemies on the beam line"
    );
    assert_eq!(hp_deltas(&overdrive_hits), vec![-28; 6]);
    assert!(
        !target_unit_ids(&overdrive_hits).contains(&off_line_enemy),
        "off-line enemy should not be hit by Magical Beam"
    );
}
