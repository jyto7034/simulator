mod common;

use common::create_test_game_data;
use game_core::config::balance;
use game_core::ecs::resources::{Qliphoth, QliphothLevel};
use game_core::game::enums::{OrdealType, PhaseEventType, PhaseType};
use game_core::game::events::GeneratorContext;
use game_core::game::managers::event_manager::EventManager;
use game_core::game::managers::ordeal_scheduler::OrdealScheduler;
use rand::{Rng, SeedableRng};

/// QliphothLevel별로 대표 amount를 설정하는 헬퍼
fn qliphoth_for_level(target: QliphothLevel) -> Qliphoth {
    let thresholds = balance::qliphoth_thresholds();
    let amount = match target {
        QliphothLevel::Stable => thresholds.stable_max,
        QliphothLevel::Caution => thresholds.caution_max,
        QliphothLevel::Critical => thresholds.critical_max,
        QliphothLevel::Meltdown => thresholds.meltdown,
    };

    let mut q = Qliphoth::new();
    q.set_amount(amount);
    assert_eq!(q.level(), target, "Qliphoth level should match target");
    q
}

/// Stable 단계에서는 Ordeal 스케줄을 그대로 따라야 한다.
#[test]
fn qliphoth_stable_follows_schedule() {
    let game_data = create_test_game_data();
    let world = bevy_ecs::world::World::new();

    let qliphoth = qliphoth_for_level(QliphothLevel::Stable);
    let ordeal = OrdealType::Dawn;
    let phase = PhaseType::I;

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let phase_event = EventManager::generate_event(qliphoth, ordeal, phase, &ctx);

    let scheduled = OrdealScheduler::get_phase_event_type(ordeal, phase)
        .expect("Schedule should exist for given ordeal/phase");

    assert_eq!(
        phase_event.event_type(),
        scheduled,
        "Stable Qliphoth should follow ordeal schedule"
    );
}

/// Critical 단계에서는 스케줄과 무관하게 항상 Suppression 이어야 한다.
#[test]
fn qliphoth_critical_forces_suppression() {
    let game_data = create_test_game_data();
    let world = bevy_ecs::world::World::new();

    let qliphoth = qliphoth_for_level(QliphothLevel::Critical);

    // Dawn Phase I 은 스케줄상 EventSelection 이다.
    let ordeal = OrdealType::Dawn;
    let phase = PhaseType::I;
    let scheduled = OrdealScheduler::get_phase_event_type(ordeal, phase)
        .expect("Schedule should exist for given ordeal/phase");
    assert_eq!(
        scheduled,
        PhaseEventType::EventSelection,
        "Precondition: Dawn Phase I should be EventSelection"
    );

    let ctx = GeneratorContext::new(&world, &game_data, 9999);
    let phase_event = EventManager::generate_event(qliphoth, ordeal, phase, &ctx);

    assert_eq!(
        phase_event.event_type(),
        PhaseEventType::Suppression,
        "Critical Qliphoth should force Suppression regardless of schedule"
    );
}

/// Meltdown 단계에서도 현재는 특수 이벤트 대신 Suppression 으로 고정된다.
#[test]
fn qliphoth_meltdown_forces_suppression_even_on_ordeal_phase() {
    let game_data = create_test_game_data();
    let world = bevy_ecs::world::World::new();

    let qliphoth = qliphoth_for_level(QliphothLevel::Meltdown);

    // Dawn Phase VI 은 스케줄상 Ordeal 이다.
    let ordeal = OrdealType::Dawn;
    let phase = PhaseType::VI;
    let scheduled = OrdealScheduler::get_phase_event_type(ordeal, phase)
        .expect("Schedule should exist for given ordeal/phase");
    assert_eq!(
        scheduled,
        PhaseEventType::Ordeal,
        "Precondition: Dawn Phase VI should be Ordeal"
    );

    let ctx = GeneratorContext::new(&world, &game_data, 4242);
    let phase_event = EventManager::generate_event(qliphoth, ordeal, phase, &ctx);

    assert_eq!(
        phase_event.event_type(),
        PhaseEventType::Suppression,
        "Meltdown Qliphoth should force Suppression instead of scheduled Ordeal"
    );
}

/// Caution 단계에서는 확률적으로 Suppression 또는 스케줄 이벤트가 나와야 한다.
/// 여기서는 동일한 RNG 규칙을 사용해 기대 타입을 계산한 뒤, 실제 결과와 비교한다.
#[test]
fn qliphoth_caution_uses_probabilistic_suppression() {
    let game_data = create_test_game_data();
    let world = bevy_ecs::world::World::new();

    let qliphoth = qliphoth_for_level(QliphothLevel::Caution);

    let ordeal = OrdealType::Dawn;
    let phase = PhaseType::I;
    let scheduled = OrdealScheduler::get_phase_event_type(ordeal, phase)
        .expect("Schedule should exist for given ordeal/phase");

    let seed = 12345_u64;
    let ctx = GeneratorContext::new(&world, &game_data, seed);

    // EventManager 내부에서 사용하는 것과 동일한 RNG 규칙으로 roll 계산
    let suppress_chance = balance::qliphoth_suppress_chance();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let roll = rng.gen_range(0..100);

    let expected_type = if roll < suppress_chance.caution {
        PhaseEventType::Suppression
    } else {
        scheduled
    };

    let phase_event = EventManager::generate_event(qliphoth, ordeal, phase, &ctx);

    assert_eq!(
        phase_event.event_type(),
        expected_type,
        "Caution Qliphoth should either follow schedule or switch to Suppression based on RNG"
    );
}
