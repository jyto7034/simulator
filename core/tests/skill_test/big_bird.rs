use std::collections::HashSet;

use game_core::{
    ecs::resources::Position,
    game::{
        battle::buffs::BuffId,
        battle::timeline::TimelineEvent,
        data::abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
        enums::Side,
    },
};

use super::{
    buff_ids, buffs_applied_by, damage_hp_changes_caused_by, hp_deltas, passive_dummy_patch,
    run_abnormality_scenario, scenario_from_board, skill_dummy_board_legend, step_ids,
    target_unit_ids, BoardEntry, PlacedUnitKind, StaticUnitPatch, UnitPatch,
};

#[test]
// 목적:
// Big Bird의 Dark Lamp가 "시전자 중심"이 아니라 "선택된 적 중심"으로 anchor를 잡고,
// 그 적 주변 클러스터에만 silence/burst를 적용하는지 확인한다.
fn big_bird_silences_and_bursts_only_enemies_within_chebyshev_radius_one() {
    let mut legend = skill_dummy_board_legend();
    for symbol in ['F', 'G', 'H', 'X'] {
        legend
            .units
            .insert(symbol, BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    }
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . . . . . . .
        . . C F! G! . .
        . . . . H! X! .
    "#;

    let result = run_abnormality_scenario("o-02-40_big_bird", scenario_from_board(board, &legend));
    let anchored_cluster_targets: HashSet<_> = [
        Position::new(3, 2),
        Position::new(4, 2),
        Position::new(4, 3),
    ]
    .into_iter()
    .map(|position| {
        result
            .unit_instance_at(position, Some(Side::Opponent))
            .unwrap()
    })
    .collect();
    let distant_enemy = result
        .unit_instance_at(Position::new(5, 3), Some(Side::Opponent))
        .unwrap();

    let steps = result.first_cast_steps("big_bird_dark_lamp");
    assert_eq!(step_ids(&steps), vec!["lamp_gaze", "lamp_burst"]);

    let silence_targets: HashSet<_> =
        target_unit_ids(&buffs_applied_by(result.timeline(), steps[0].seq))
            .into_iter()
            .collect();
    assert_eq!(silence_targets, anchored_cluster_targets);
    assert_eq!(
        buff_ids(&buffs_applied_by(result.timeline(), steps[0].seq)),
        vec![BuffId::from_name("silence"); 3]
    );

    let burst_targets: HashSet<_> = target_unit_ids(&damage_hp_changes_caused_by(
        result.timeline(),
        steps[1].seq,
    ))
    .into_iter()
    .collect();
    assert_eq!(burst_targets, anchored_cluster_targets);
    assert_eq!(
        hp_deltas(&damage_hp_changes_caused_by(
            result.timeline(),
            steps[1].seq
        )),
        vec![-70, -70, -70]
    );
    assert!(
        !burst_targets.contains(&distant_enemy),
        "distant enemy should stay outside the anchored Dark Lamp burst"
    );
}

#[test]
// 목적:
// Big Bird가 부여한 silence가 실제로 적의 autocast 시작을 막고,
// silence 만료 뒤에야 스킬 시전이 재개되는지 확인한다.
fn big_bird_silence_defers_enemy_autocast_until_buff_expires() {
    let mut legend = skill_dummy_board_legend();
    legend.units.insert(
        'O',
        BoardEntry::Opponent(PlacedUnitKind::Abnormality("o-03-03_one_sin")),
    );
    legend.patches.insert(
        '!',
        UnitPatch {
            static_patch: StaticUnitPatch {
                max_health: Some(2_000),
                attack: Some(1),
                movement: Some(MovementDef {
                    speed_units_per_ms: 0,
                }),
                basic_attack: Some(BasicAttackDef {
                    range_tiles: 1,
                    interval_ms: 1,
                    windup_ms: 0,
                    delivery: game_core::game::ability::DeliveryDef::Instant,
                }),
                resonance: Some(ResonanceDef {
                    start: 90,
                    max: 100,
                    gain_lock_ms: 0,
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    legend.patches.insert(
        '#',
        UnitPatch {
            static_patch: StaticUnitPatch {
                max_health: Some(2_000),
                attack: Some(1),
                movement: Some(MovementDef {
                    speed_units_per_ms: 0,
                }),
                basic_attack: Some(BasicAttackDef {
                    range_tiles: 1,
                    interval_ms: 500,
                    windup_ms: 0,
                    delivery: game_core::game::ability::DeliveryDef::Instant,
                }),
                resonance: Some(ResonanceDef {
                    start: 90,
                    max: 100,
                    gain_lock_ms: 0,
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let board = r#"
        . . . . .
        . . . . .
        . . C! O# .
    "#;

    let result = run_abnormality_scenario("o-02-40_big_bird", scenario_from_board(board, &legend));
    let silenced_enemy = result
        .unit_instance_at(Position::new(3, 2), Some(Side::Opponent))
        .unwrap();

    let silence_applied = result
        .timeline()
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::BuffApplied {
                    target_instance_id,
                    buff_id,
                    ..
                } if *target_instance_id == silenced_enemy && *buff_id == BuffId::from_name("silence")
            )
        })
        .expect("Big Bird should apply silence to the adjacent enemy caster");
    let silence_expired = result
        .timeline()
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::BuffExpired {
                    target_instance_id,
                    buff_id,
                    ..
                } if *target_instance_id == silenced_enemy && *buff_id == BuffId::from_name("silence")
            )
        })
        .expect("silence should eventually expire");

    let enemy_autocast_start = result
        .timeline()
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AutoCastStart {
                    caster_instance_id,
                    skill_id,
                    ..
                } if *caster_instance_id == silenced_enemy && skill_id.as_deref() == Some("one_sin_penitence")
            )
        })
        .expect("silenced enemy should eventually start its autocast after silence ends");
    let enemy_cast = result
        .timeline()
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityCast {
                    caster_instance_id,
                    skill_id,
                    ..
                } if *caster_instance_id == silenced_enemy && skill_id == "one_sin_penitence"
            )
        })
        .expect("silenced enemy should eventually cast after silence expires");

    assert!(
        silence_applied.time_ms < silence_expired.time_ms,
        "silence must be applied before it can expire"
    );
    assert!(
        enemy_autocast_start.time_ms > silence_expired.time_ms,
        "enemy autocast should not start while silence is active"
    );
    assert!(
        enemy_cast.time_ms >= enemy_autocast_start.time_ms,
        "ability cast should happen after the deferred autocast start"
    );
}
