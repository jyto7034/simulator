use game_core::ecs::resources::GameState;
use game_core::game::behavior::{BehaviorResult, GameError, PlayerBehavior};
use game_core::game::data::random_event_data::{RandomEventInnerMetadata, RandomEventMetadata};
use game_core::game::enums::{GameOption, PhaseEvent};
use game_core::game::events::event_selection::random::RandomEventType;
use game_core::game::world::GameCore;
use uuid::Uuid;

use crate::common::{create_test_game_data, create_test_game_data_with_random_event};

fn contains_behavior_variant(haystack: &[PlayerBehavior], needle: &PlayerBehavior) -> bool {
    // NOTE: enum의 "variant"만 비교(필드 값 무시)하며, ActionValidator 정책과 맞춘다.
    let needle = std::mem::discriminant(needle);
    haystack.iter().any(|b| std::mem::discriminant(b) == needle)
}

fn start_game_and_request_phase(game: &mut GameCore, player_id: Uuid) -> PhaseEvent {
    game.execute(player_id, PlayerBehavior::StartNewGame)
        .unwrap();
    match game
        .execute(player_id, PlayerBehavior::RequestPhaseData)
        .unwrap()
    {
        BehaviorResult::RequestPhaseData(event) => event,
        other => panic!("expected RequestPhaseData, got {other:?}"),
    }
}

fn select_random_from_phase_event(
    game: &mut GameCore,
    player_id: Uuid,
    phase_event: &PhaseEvent,
) -> Result<BehaviorResult, GameError> {
    let random_uuid = phase_event
        .options()
        .iter()
        .find_map(|opt| match opt {
            GameOption::Random { event } => Some(event.uuid),
            _ => None,
        })
        .expect("Random option should exist");

    game.execute(
        player_id,
        PlayerBehavior::SelectEvent {
            event_id: random_uuid,
        },
    )
}

#[test]
fn random_routes_to_shop_stage_immediately() {
    // Given: Random이 Shop으로 라우팅되도록 데이터를 구성한다.
    let base = create_test_game_data();
    let shop_uuid = base.shop_data.shops[0].uuid;

    let random_event = RandomEventMetadata {
        id: "random_shop".to_string(),
        name: "Random Shop".to_string(),
        uuid: Uuid::from_u128(0xD00D_0000_0000_0001),
        event_type: RandomEventType::Shop,
        risk_level: game_core::game::enums::RiskLevel::ZAYIN,
        description: "routes to shop".to_string(),
        image: "shop.png".to_string(),
        inner_metadata: RandomEventInnerMetadata::Shop(shop_uuid),
    };
    let game_data = create_test_game_data_with_random_event(random_event);
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    // When: Phase 이벤트에서 Random을 선택한다.
    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let result = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap();

    // Then: 즉시 상점 스테이지로 진입한다.
    assert!(matches!(result, BehaviorResult::EventSelected));
    assert!(matches!(game.get_state(), GameState::InShop { .. }));
    assert_eq!(
        game.get_phase_events_count(),
        0,
        "이벤트 1개를 선택했으므로 CurrentPhaseEvents는 비워져야 한다"
    );

    // Then: 상점 행동이 허용된다.
    let allowed = game.get_allowed_actions();
    assert!(contains_behavior_variant(
        &allowed,
        &PlayerBehavior::PurchaseItem {
            item_uuid: Uuid::nil()
        }
    ));
    assert!(contains_behavior_variant(
        &allowed,
        &PlayerBehavior::ExitShop
    ));
}

#[test]
fn random_routes_to_bonus_stage_immediately() {
    // Given: Random이 Bonus로 라우팅되도록 데이터를 구성한다.
    let base = create_test_game_data();
    let bonus_uuid = base.bonus_data.bonuses[0].uuid;

    let random_event = RandomEventMetadata {
        id: "random_bonus".to_string(),
        name: "Random Bonus".to_string(),
        uuid: Uuid::from_u128(0xD00D_0000_0000_0002),
        event_type: RandomEventType::Bonus,
        risk_level: game_core::game::enums::RiskLevel::ZAYIN,
        description: "routes to bonus".to_string(),
        image: "bonus.png".to_string(),
        inner_metadata: RandomEventInnerMetadata::Bonus(bonus_uuid),
    };
    let game_data = create_test_game_data_with_random_event(random_event);
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    // When: Phase 이벤트에서 Random을 선택한다.
    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let result = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap();

    // Then: 즉시 보너스 스테이지로 진입한다.
    assert!(matches!(result, BehaviorResult::EventSelected));
    assert!(matches!(game.get_state(), GameState::InBonus { .. }));
    assert_eq!(
        game.get_phase_events_count(),
        0,
        "이벤트 1개를 선택했으므로 CurrentPhaseEvents는 비워져야 한다"
    );

    // Then: 보너스 행동이 허용된다.
    let allowed = game.get_allowed_actions();
    assert!(contains_behavior_variant(
        &allowed,
        &PlayerBehavior::ClaimBonus
    ));
    assert!(contains_behavior_variant(
        &allowed,
        &PlayerBehavior::ExitBonus
    ));
}

#[test]
fn random_routes_to_pve_stage_with_three_suppression_candidates() {
    // Given: Random이 Suppress(PvE)로 라우팅되도록 데이터를 구성한다.
    let base = create_test_game_data();
    let abno_uuid = base.abnormality_data.items[0].uuid;

    let random_event = RandomEventMetadata {
        id: "random_pve".to_string(),
        name: "Random PvE".to_string(),
        uuid: Uuid::from_u128(0xD00D_0000_0000_0003),
        event_type: RandomEventType::Suppress,
        risk_level: game_core::game::enums::RiskLevel::HE,
        description: "routes to suppression selection".to_string(),
        image: "pve.png".to_string(),
        inner_metadata: RandomEventInnerMetadata::Suppress(abno_uuid),
    };
    let game_data = create_test_game_data_with_random_event(random_event);
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    // When: Phase 이벤트에서 Random을 선택한다.
    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let result = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap();

    // Then: "진압 후보 3개" PhaseEvent가 즉시 반환된다.
    let BehaviorResult::RequestPhaseData(PhaseEvent::Suppression { candidates }) = result else {
        panic!("expected RequestPhaseData(Suppression)");
    };

    // Then: 후보는 3개이며, 게임 상태는 SelectingEvent(선택 스테이지)이다.
    assert_eq!(candidates.len(), 3);
    assert!(matches!(game.get_state(), GameState::SelectingEvent));
    assert_eq!(game.get_phase_events_count(), 3);

    // Then: PvE 시작 행동(StartSuppression)이 허용된다.
    let allowed = game.get_allowed_actions();
    assert!(contains_behavior_variant(
        &allowed,
        &PlayerBehavior::StartSuppression {
            abnormality_id: String::new()
        }
    ));

    // When: 후보에 없는 abnormality_id로 StartSuppression을 시도한다.
    let err = game
        .execute(
            player_id,
            PlayerBehavior::StartSuppression {
                abnormality_id: "not_in_candidates".to_string(),
            },
        )
        .unwrap_err();
    // Then: 후보 검증에 의해 거부된다.
    assert!(matches!(err, GameError::InvalidAction));

    // When: (fallback이 아닌) 실제 후보의 abnormality_id로 StartSuppression을 시도한다.
    let real_candidate = candidates
        .iter()
        .find(|c| c.abnormality_id != "fallback")
        .expect("at least one non-fallback suppression candidate should exist");
    let result = game.execute(
        player_id,
        PlayerBehavior::StartSuppression {
            abnormality_id: real_candidate.abnormality_id.clone(),
        },
    );

    // Then: PvE 전투가 수행되고(승패와 무관하게) Phase가 진행된다.
    // NOTE: 현재 구현은 battle 결과를 UI로 반환하지 않고 다음 Phase로 진행한다.
    assert!(matches!(
        result.unwrap(),
        BehaviorResult::AdvancePhase { .. }
    ));
    assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));
}

#[test]
fn random_resolution_failure_falls_back_instead_of_erroring() {
    // Given: Random이 존재하지 않는 Bonus UUID를 가리키도록 만든다.
    let base = create_test_game_data();
    let fallback_bonus_uuid = base.bonus_data.bonuses[0].uuid;

    let random_event = RandomEventMetadata {
        id: "random_broken".to_string(),
        name: "Random Broken".to_string(),
        uuid: Uuid::from_u128(0xD00D_0000_0000_0004),
        event_type: RandomEventType::Bonus,
        risk_level: game_core::game::enums::RiskLevel::ZAYIN,
        description: "broken inner metadata".to_string(),
        image: "broken.png".to_string(),
        inner_metadata: RandomEventInnerMetadata::Bonus(Uuid::from_u128(999_999)),
    };
    let game_data = create_test_game_data_with_random_event(random_event);
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    // When: Phase 이벤트에서 Random을 선택한다.
    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let result = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap();

    // Then: 에러 대신 fallback 스테이지로 라우팅된다(기본: Bonus).
    assert!(matches!(result, BehaviorResult::EventSelected));
    match game.get_state() {
        GameState::InBonus { bonus_uuid } => assert_eq!(bonus_uuid, fallback_bonus_uuid),
        other => panic!("expected InBonus fallback, got {other:?}"),
    }
    assert_eq!(
        game.get_phase_events_count(),
        0,
        "이벤트 1개를 선택했으므로 CurrentPhaseEvents는 비워져야 한다"
    );
}
