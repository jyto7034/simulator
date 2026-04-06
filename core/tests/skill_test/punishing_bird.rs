use game_core::{ecs::resources::Position, game::enums::Side};

use super::{
    attack_starts_caused_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, step_ids, target_unit_ids,
};

#[test]
// 목적:
// Punishing Bird의 Rapid Peck이 opening 1회와 flurry 2회로 구성되고,
// 반복 step이 같은 적에게 누적 공격을 생성하는지 검증한다.
fn punishing_bird_rapid_peck_emits_one_opening_attack_and_two_flurry_attacks() {
    let mut legend = skill_dummy_board_legend();
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . C . . . . .
        . D! . . . . .
    "#;

    let result = run_abnormality_scenario(
        "o-02-56_punishing_bird",
        scenario_from_board(board, &legend),
    );
    let enemy = result
        .unit_instance_at(Position::new(1, 2), Some(Side::Opponent))
        .unwrap();

    let steps = result.first_cast_steps("punishing_bird_rapid_peck");
    assert_eq!(step_ids(&steps), vec!["peck_opening", "peck_flurry"]);

    let opening_attacks = attack_starts_caused_by(result.timeline(), steps[0].seq);
    assert_eq!(opening_attacks.len(), 1);
    assert_eq!(target_unit_ids(&opening_attacks), vec![enemy]);

    let flurry_attacks = attack_starts_caused_by(result.timeline(), steps[1].seq);
    assert_eq!(flurry_attacks.len(), 2);
    assert_eq!(target_unit_ids(&flurry_attacks), vec![enemy, enemy]);
}
