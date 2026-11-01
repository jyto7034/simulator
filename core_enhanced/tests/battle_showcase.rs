//! Human-readable demo. Run with `cargo test -p game_core --test battle_showcase -- --nocapture`.
mod common;

// Reuse the existing real battle runner without importing the whole skill test suite.
mod skill_test {
    #[allow(unused_imports)]
    pub(crate) mod common;
}

use game_core::game::{
    battle::event_log::BattleLogEvent,
    enums::Side,
    resources::Position,
    stats::{StatId, StatModifierKind},
};
use skill_test::common::{
    damage_hp_changes_caused_by, hp_deltas, passive_dummy_patch, run_abnormality_scenario,
    scenario_from_board, skill_dummy_board_legend, stat_changes_caused_by, stat_modifier_summaries,
    step_ids, target_unit_ids, RuntimeStartPatch, UnitPatch,
};

#[test]
fn test_nothing_there_first_cast() {
    let mut legend = skill_dummy_board_legend();
    legend.patches.insert(
        '@',
        UnitPatch {
            runtime_start: RuntimeStartPatch {
                current_health: Some(430),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    legend.patches.insert('!', passive_dummy_patch(2_000, None));
    let board = ". . . . . . .\n. C@ D! . . . .";
    let run =
        run_abnormality_scenario("o-06-20_nothing_there", scenario_from_board(board, &legend));
    let enemy = run
        .unit_instance_at(Position::new(2, 1), Some(Side::Opponent))
        .unwrap();
    let cast = run.first_cast_of("nothing_there_goodbye");
    let steps = run.first_cast_steps("nothing_there_goodbye");
    assert_eq!(
        step_ids(&steps),
        ["tear_open", "mimic_adaptation", "goodbye_finish"]
    );

    // Assert actual results before announcing a successful demonstration.
    let first = damage_hp_changes_caused_by(run.event_log(), steps[0].seq);
    let buff = stat_changes_caused_by(run.event_log(), steps[1].seq);
    let finish = damage_hp_changes_caused_by(run.event_log(), steps[2].seq);
    assert_eq!(target_unit_ids(&first), [enemy]);
    assert_eq!(hp_deltas(&first), [-62]);
    assert_eq!(target_unit_ids(&buff), [run.caster_instance_id()]);
    assert_eq!(
        stat_modifier_summaries(&buff),
        [(StatId::Attack, StatModifierKind::Percent, 18)]
    );
    assert_eq!(target_unit_ids(&finish), [enemy, enemy]);
    assert_eq!(hp_deltas(&finish), [-58, -58]);

    let total_damage: i64 = first
        .iter()
        .chain(&finish)
        .map(|entry| match &entry.event {
            BattleLogEvent::HpChanged {
                hp_before,
                hp_after,
                ..
            } => i64::from(*hp_before) - i64::from(*hp_after),
            _ => unreachable!(),
        })
        .sum();

    println!("\n=== 전투 엔진 : 아무것도 없는 / Goodbye ===");
    println!("core 실행 · 기존 스킬 데이터 사용");
    println!("훈련용 배치: 시전자 C와 반격하지 않는 표적 D (초기 HP 2000)");
    println!("  . C D .\n");
    println!("첫 시전만 발췌 · 시간은 시전 시작 기준 · 수치는 실제 전투 로그");
    if let BattleLogEvent::HpChanged { hp_before, .. } = &first[0].event {
        println!("첫 타격 직전 표적 HP: {hp_before} (시전 이전 사건은 생략)");
    }
    println!("[+0.000s] Goodbye 시전");

    for (step, title) in steps
        .iter()
        .zip(["첫 타격", "자가 강화", "마무리 연속 타격"])
    {
        let elapsed = step.time_ms.checked_sub(cast.time_ms).unwrap();
        println!("[+{}.{:03}s] {title}", elapsed / 1000, elapsed % 1000);
        for entry in damage_hp_changes_caused_by(run.event_log(), step.seq) {
            if let BattleLogEvent::HpChanged {
                hp_before,
                hp_after,
                ..
            } = &entry.event
            {
                let elapsed = entry.time_ms.checked_sub(cast.time_ms).unwrap();
                println!(
                    "  [+{}.{:03}s] 표적 HP: {hp_before} → {hp_after} (실제 피해 {})",
                    elapsed / 1000,
                    elapsed % 1000,
                    hp_before - hp_after
                );
            }
        }
        for entry in stat_changes_caused_by(run.event_log(), step.seq) {
            if let BattleLogEvent::StatChanged {
                modifier,
                stats_before,
                stats_after,
                ..
            } = &entry.event
            {
                println!(
                    "  시전자 공격력 +{}%: {} → {}",
                    modifier.value, stats_before.attack, stats_after.attack
                );
            }
        }
    }
    println!(
        "\n검증 통과: 3단계 실행 · 실제 타격 {}회 · 스킬 피해 합계 {total_damage}",
        first.len() + finish.len()
    );
}
