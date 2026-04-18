use game_core::{
    ecs::resources::Position,
    game::{
        battle::timeline::{AttackKind, TimelineEvent},
        enums::Side,
    },
};

use super::{
    attack_kinds, attack_starts_caused_by, damage_hp_changes_caused_by, passive_dummy_patch,
    run_abnormality_scenario, scenario_from_board, skill_dummy_board_legend, step_ids,
    target_unit_ids, BoardEntry, PlacedUnitKind,
};

#[test]
// 목적:
// Little Red의 marked_shot가 선행 적중한 뒤,
// hunt_finish가 같은 prey에게 추가 공격 2회를 생성하는지 확인한다.
fn little_red_follows_marked_shot_with_two_finish_attacks_on_the_same_prey() {
    let mut legend = skill_dummy_board_legend();
    legend
        .units
        .insert('E', BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . . . . . . .
        . C D! E! . . .
    "#;

    let result =
        run_abnormality_scenario("f-01-57_little_red", scenario_from_board(board, &legend));
    let enemy = result
        .unit_instance_at(Position::new(2, 2), Some(Side::Opponent))
        .unwrap();

    let cast = result.first_cast_of("little_red_hunt_the_prey");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == enemy
    ));

    let steps = result.first_cast_steps("little_red_hunt_the_prey");
    assert_eq!(step_ids(&steps), vec!["marked_shot", "hunt_finish"]);
    assert_eq!(
        target_unit_ids(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[0].seq
        )),
        vec![enemy]
    );
    let finish_attacks = attack_starts_caused_by(result.timeline(), steps[1].seq);
    assert_eq!(target_unit_ids(&finish_attacks), vec![enemy, enemy]);
    assert_eq!(
        attack_kinds(&finish_attacks),
        vec![Some(AttackKind::Triggered), Some(AttackKind::Triggered)]
    );
}
