use game_core::{
    game::resources::Position,
    game::{
        battle::timeline::{AttackKind, TimelineEvent},
        enums::Side,
    },
};

use super::{
    attack_starts_caused_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, step_ids, target_unit_ids,
};

#[test]
// 목적:
// Red Shoes의 Berserk가 ExtraAttack(count: 2) 정의대로
// 트리거 공격 2회를 실제로 예약하는지 확인한다.
fn red_shoes_berserk_schedules_two_triggered_follow_up_attacks() {
    let mut legend = skill_dummy_board_legend();
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . C . . . . .
        . D! . . . . .
    "#;

    let result = run_abnormality_scenario("t-09-09", scenario_from_board(board, &legend));
    let enemy = result
        .unit_instance_at(Position::new(1, 2), Some(Side::Opponent))
        .unwrap();

    let cast = result.first_cast_of("red_shoes_berserk");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == enemy
    ));

    let steps = result.first_cast_steps("red_shoes_berserk");
    assert_eq!(step_ids(&steps), vec!["berserk"]);

    let attacks = attack_starts_caused_by(result.timeline(), steps[0].seq);
    assert_eq!(attacks.len(), 2);
    assert_eq!(target_unit_ids(&attacks), vec![enemy, enemy]);
    assert!(attacks.iter().all(|entry| {
        matches!(
            entry.event,
            TimelineEvent::AttackStart {
                kind: Some(AttackKind::Triggered),
                ..
            }
        )
    }));
}
