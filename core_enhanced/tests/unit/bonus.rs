use game_core::ecs::resources::GameState;
use game_core::game::behavior::{ActionKind, BehaviorResult, GameError, PlayerBehavior};
use game_core::game::enums::{GameOption, PhaseEvent};
use game_core::game::world::GameCore;
use uuid::Uuid;

use crate::common::create_test_game_data;

fn contains_action_kind(haystack: &[ActionKind], needle: ActionKind) -> bool {
    haystack.contains(&needle)
}

fn start_game_and_request_phase(game: &mut GameCore, player_id: Uuid) -> PhaseEvent {
    game.execute(player_id, PlayerBehavior::StartNewGame)
        .unwrap();
    match game
        .execute(player_id, PlayerBehavior::RequestPhaseData)
        .unwrap()
    {
        BehaviorResult::RequestPhaseData(event) => *event,
        other => panic!("expected RequestPhaseData, got {other:?}"),
    }
}

fn select_bonus_from_phase_event(
    game: &mut GameCore,
    player_id: Uuid,
    phase_event: &PhaseEvent,
) -> Uuid {
    let bonus_uuid = phase_event
        .options()
        .iter()
        .find_map(|opt| match opt {
            GameOption::Bonus { bonus } => Some(bonus.uuid),
            _ => None,
        })
        .expect("Bonus option should exist");

    let result = game.execute(
        player_id,
        PlayerBehavior::SelectEvent {
            event_id: bonus_uuid,
        },
    );
    assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));
    bonus_uuid
}

#[test]
fn select_bonus_option_transitions_and_allows_bonus_actions() {
    // Given: 결정적인 테스트 데이터로 새 게임을 생성한다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    // When: 게임을 시작하고 Phase 데이터를 요청한다.
    let phase_event = start_game_and_request_phase(&mut game, player_id);
    // When: Phase 선택지에서 보너스를 선택한다.
    let bonus_uuid = select_bonus_from_phase_event(&mut game, player_id, &phase_event);

    // Then: 게임 상태는 선택한 보너스 UUID로 InBonus가 된다.
    match game.get_state() {
        GameState::InBonus { bonus_uuid: uuid } => assert_eq!(uuid, bonus_uuid),
        other => panic!("expected InBonus state, got {other:?}"),
    }

    // Then: 허용 행동 목록에 Claim/Exit이 포함된다(variant 기준).
    let allowed_actions = game.get_allowed_actions();
    assert!(contains_action_kind(
        &allowed_actions,
        ActionKind::ClaimBonus
    ));
    assert!(contains_action_kind(
        &allowed_actions,
        ActionKind::ExitBonus
    ));
    assert!(!contains_action_kind(
        &allowed_actions,
        ActionKind::ExitShop
    ));
}

#[test]
fn claim_bonus_grants_reward_and_requires_exit() {
    // Given: 보너스에 진입한 상태
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _bonus_uuid = select_bonus_from_phase_event(&mut game, player_id, &phase_event);

    // Given: 보너스 스냅샷에서 기대 amount를 캡처하고, Enkephalin을 0으로 만든다.
    let (_shop, bonus, _random) = phase_event
        .as_event_selection()
        .expect("EventSelection phase expected");
    let expected_amount = bonus.amount;
    game.set_enkephalin(0);

    // When: 보너스를 claim 한다.
    let result = game.execute(player_id, PlayerBehavior::ClaimBonus).unwrap();

    // Then: BonusReward 결과를 반환하고, Enkephalin이 증가한다.
    let (enkephalin, inventory_diff) = result.as_bonus_reward().unwrap();
    assert_eq!(enkephalin, expected_amount);
    assert!(inventory_diff.added.is_empty());
    assert!(inventory_diff.updated.is_empty());
    assert!(inventory_diff.removed.is_empty());

    // Then: 보너스 수령 완료 상태로 전환되며, 나가기만 가능해진다.
    assert!(matches!(game.get_state(), GameState::InBonusClaimed { .. }));

    let err = game
        .execute(player_id, PlayerBehavior::ClaimBonus)
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    // When: 보너스 화면에서 나간다.
    let exit = game.execute(player_id, PlayerBehavior::ExitBonus).unwrap();
    // Then: Phase가 진행된다.
    assert!(matches!(exit, BehaviorResult::AdvancePhase { .. }));
    assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));
}

#[test]
fn claim_bonus_is_rejected_outside_bonus_state() {
    // Given: 게임은 시작했지만 보너스에는 진입하지 않았다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    game.execute(player_id, PlayerBehavior::StartNewGame)
        .unwrap();

    // When: 보너스를 claim하려고 시도한다.
    let err = game
        .execute(player_id, PlayerBehavior::ClaimBonus)
        .unwrap_err();

    // Then: GameState 게이팅에 의해 거부된다.
    assert!(matches!(err, GameError::InvalidAction));
}
