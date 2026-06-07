use game_core::{
    game::resources::Position,
    game::{battle::timeline::TimelineEvent, enums::Side},
};

use super::{
    damage_hp_changes_caused_by, healing_hp_changes_caused_by, hp_deltas, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry,
    RuntimeStartPatch, StaticUnitPatch, UnitPatch,
};

#[test]
// 목적:
// One Sin이 가장 체력이 낮은 적을 먼저 겨냥하고,
// 첫 피해가 들어간 뒤에만 자가 회복 step이 이어지는지 확인한다.
fn one_sin_targets_the_lowest_health_enemy_and_heals_self_after_landing_judgement() {
    let mut legend = skill_dummy_board_legend();
    legend
        .units
        .insert('E', BoardEntry::Opponent(super::PlacedUnitKind::SkillDummy));
    legend.patches.insert(
        '@',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(100),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    legend.patches.insert(
        '!',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(1_200),
                ..Default::default()
            },
            static_patch: StaticUnitPatch {
                movement: Some(game_core::game::data::abnormality_data::MovementDef {
                    speed_units_per_ms: 0,
                    radius_units: 350_000,
                }),
                ..Default::default()
            },
        },
    );
    legend.patches.insert(
        '#',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(1_800),
                ..Default::default()
            },
            static_patch: StaticUnitPatch {
                movement: Some(game_core::game::data::abnormality_data::MovementDef {
                    speed_units_per_ms: 0,
                    radius_units: 350_000,
                }),
                ..Default::default()
            },
        },
    );

    let board = r#"
        . . . . . . .
        . C@ D! E# . . .
    "#;

    let result = run_abnormality_scenario("o-03-03_one_sin", scenario_from_board(board, &legend));
    let low_hp_enemy = result
        .unit_instance_at(Position::new(2, 1), Some(Side::Opponent))
        .unwrap();
    let caster = result.caster_instance_id();

    let cast = result.first_cast_of("one_sin_penitence");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == low_hp_enemy
    ));

    let steps = result.first_cast_steps("one_sin_penitence");
    assert_eq!(
        step_ids(&steps),
        vec!["penitence_judgement", "penitence_absolution"]
    );

    assert_eq!(
        target_unit_ids(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[0].seq
        )),
        vec![low_hp_enemy]
    );
    assert_eq!(
        hp_deltas(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[0].seq
        )),
        vec![-38]
    );
    assert_eq!(
        target_unit_ids(&healing_hp_changes_caused_by(
            result.timeline(),
            steps[1].seq
        )),
        vec![caster]
    );
    assert_eq!(
        hp_deltas(&healing_hp_changes_caused_by(
            result.timeline(),
            steps[1].seq
        )),
        vec![18]
    );
}
