use game_core::{
    game::resources::Position,
    game::{
        battle::{buffs::BuffId, timeline::TimelineEvent},
        enums::Side,
    },
};

use super::{
    buff_ids, buffs_applied_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry, PlacedUnitKind,
};

#[test]
// 목적:
// Spider Bud의 Poison Stack이 근처 적 여러 명 중 하나만 골라
// poison 버프를 단일 대상에게만 적용하는지 검증한다.
fn spider_bud_applies_poison_only_to_the_nearest_enemy() {
    let mut legend = skill_dummy_board_legend();
    legend
        .units
        .insert('E', BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . C . E! . . .
        . D! . . . . .
    "#;

    let result = run_abnormality_scenario("o-01-45", scenario_from_board(board, &legend));
    let near_enemy = result
        .unit_instance_at(Position::new(1, 2), Some(Side::Opponent))
        .unwrap();

    let cast = result.first_cast_of("spider_bud_poison_stack");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == near_enemy
    ));

    let steps = result.first_cast_steps("spider_bud_poison_stack");
    assert_eq!(step_ids(&steps), vec!["poison_apply"]);

    let buffs = buffs_applied_by(result.timeline(), steps[0].seq);
    assert_eq!(buffs.len(), 1);
    assert_eq!(target_unit_ids(&buffs), vec![near_enemy]);
    assert_eq!(buff_ids(&buffs), vec![BuffId::from_name("poison")]);
}
