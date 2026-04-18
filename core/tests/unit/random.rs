use game_core::ecs::resources::GameState;
use game_core::game::behavior::{ActionKind, BehaviorResult, GameError, PlayerBehavior};
use game_core::game::data::bonus_data::{BonusDatabase, BonusMetadata, BonusType};
use game_core::game::data::random_event_data::{RandomEventInnerMetadata, RandomEventMetadata};
use game_core::game::data::{GameDataBase, GameDataBaseParts};
use game_core::game::enums::{GameOption, PhaseEvent, RewardMode};
use game_core::game::events::event_selection::random::RandomEventType;
use game_core::game::world::GameCore;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::{create_test_game_data, create_test_game_data_with_random_event};

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

fn select_suppression_candidate(
    game: &mut GameCore,
    player_id: Uuid,
    phase_event: &PhaseEvent,
) -> Result<BehaviorResult, GameError> {
    let candidate_uuid = phase_event
        .options()
        .iter()
        .find_map(|opt| match opt {
            GameOption::SuppressAbnormality { uuid, .. } => Some(*uuid),
            _ => None,
        })
        .expect("Suppression option should exist");

    game.execute(
        player_id,
        PlayerBehavior::SelectEvent {
            event_id: candidate_uuid,
        },
    )
}

fn create_test_game_data_with_suppression_rewards(
    random_event: RandomEventMetadata,
    reward_mode: RewardMode,
    rewards: Vec<BonusMetadata>,
) -> Arc<GameDataBase> {
    let base = create_test_game_data();
    let base = base.as_ref();

    let random_events_db =
        game_core::game::data::random_event_data::RandomEventDatabase::new(vec![
            random_event.clone()
        ]);
    let bonus_db = BonusDatabase::new(rewards.clone());

    let mut pve_db = base.pve_data.as_ref().clone();
    let reward_bonus_uuids = rewards.iter().map(|bonus| bonus.uuid).collect::<Vec<_>>();
    for encounter in &mut pve_db.encounters {
        encounter.reward_mode = reward_mode;
        encounter.reward_bonus_uuids = reward_bonus_uuids.clone();
    }

    let mut event_pools = base.event_pools.clone();
    event_pools.dawn.random_events = vec![game_core::game::data::event_pools::WeightedEvent {
        weight: 1,
        uuid: random_event.uuid,
    }];
    event_pools.dawn.bonuses = rewards
        .iter()
        .map(|bonus| game_core::game::data::event_pools::WeightedEvent {
            weight: 1,
            uuid: bonus.uuid,
        })
        .collect();

    Arc::new(GameDataBase::new(GameDataBaseParts {
        abnormality_data: Arc::clone(&base.abnormality_data),
        artifact_data: Arc::clone(&base.artifact_data),
        equipment_data: Arc::clone(&base.equipment_data),
        shop_data: Arc::clone(&base.shop_data),
        bonus_data: Arc::new(bonus_db),
        random_event_data: Arc::new(random_events_db),
        pve_data: Arc::new(pve_db),
        skill_data: Arc::clone(&base.skill_data),
        event_pools,
    }))
}

fn enkephalin_bonus(uuid: Uuid, id: &str, amount: u32) -> BonusMetadata {
    BonusMetadata {
        id: id.to_string(),
        bonus_type: BonusType::Enkephalin,
        uuid,
        name: id.to_string(),
        description: format!("grants {amount} enkephalin"),
        icon: format!("{id}.png"),
        amount,
    }
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
    assert!(contains_action_kind(&allowed, ActionKind::PurchaseItem));
    assert!(contains_action_kind(&allowed, ActionKind::ExitShop));
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
    assert!(contains_action_kind(&allowed, ActionKind::ClaimBonus));
    assert!(contains_action_kind(&allowed, ActionKind::ExitBonus));
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
    let BehaviorResult::RequestPhaseData(event) = result else {
        panic!("expected RequestPhaseData(Suppression)");
    };
    let PhaseEvent::Suppression { candidates } = *event else {
        panic!("expected RequestPhaseData(Suppression)");
    };

    // Then: 후보는 3개이며, 게임 상태는 SelectingEvent(선택 스테이지)이다.
    assert_eq!(candidates.len(), 3);
    assert!(matches!(game.get_state(), GameState::SelectingEvent));
    assert_eq!(game.get_phase_events_count(), 3);

    // Then: PvE 시작 행동(StartSuppression)이 허용된다.
    let allowed = game.get_allowed_actions();
    assert!(contains_action_kind(&allowed, ActionKind::StartSuppression));

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

    // When: 실제 후보의 abnormality_id로 StartSuppression을 시도한다.
    let real_candidate = candidates
        .first()
        .expect("at least one suppression candidate should exist");
    let result = game.execute(
        player_id,
        PlayerBehavior::StartSuppression {
            abnormality_id: real_candidate.abnormality_id.clone(),
        },
    );

    // Then: PvE 전투 결과가 반환되고 replay 단계로 진입한다.
    let suppression_result = result.unwrap();
    let (winner, timeline) = suppression_result
        .as_suppress_abnormality()
        .expect("suppression battle result should be returned");
    assert!(!timeline.entries.is_empty());
    assert!(matches!(
        winner,
        game_core::game::battle::types::BattleWinner::Player
            | game_core::game::battle::types::BattleWinner::Opponent
            | game_core::game::battle::types::BattleWinner::Draw
    ));
    assert!(matches!(
        game.get_state(),
        GameState::InSuppressionReplay { .. }
    ));
}

#[test]
fn selected_suppression_candidate_starts_battle_from_in_suppression_state() {
    let base = create_test_game_data();
    let abno_uuid = base.abnormality_data.items[0].uuid;

    let random_event = RandomEventMetadata {
        id: "random_pve_select".to_string(),
        name: "Random PvE Select".to_string(),
        uuid: Uuid::from_u128(0xD00D_0000_0000_0100),
        event_type: RandomEventType::Suppress,
        risk_level: game_core::game::enums::RiskLevel::HE,
        description: "routes to suppression selection".to_string(),
        image: "pve.png".to_string(),
        inner_metadata: RandomEventInnerMetadata::Suppress(abno_uuid),
    };
    let game_data = create_test_game_data_with_random_event(random_event);
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let result = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap();
    let BehaviorResult::RequestPhaseData(phase_event) = result else {
        panic!("expected RequestPhaseData(Suppression)");
    };
    let phase_event = *phase_event;

    let selected_result = select_suppression_candidate(&mut game, player_id, &phase_event).unwrap();
    assert!(matches!(selected_result, BehaviorResult::EventSelected));
    assert!(matches!(game.get_state(), GameState::InSuppression { .. }));

    let candidate = phase_event
        .as_suppression()
        .expect("suppression candidates should exist")
        .first()
        .expect("at least one suppression candidate should exist");

    let battle_result = game
        .execute(
            player_id,
            PlayerBehavior::StartSuppression {
                abnormality_id: candidate.abnormality_id.clone(),
            },
        )
        .unwrap();

    let (winner, timeline) = battle_result
        .as_suppress_abnormality()
        .expect("suppression battle result should be returned");
    assert!(!timeline.entries.is_empty());
    assert!(matches!(
        winner,
        game_core::game::battle::types::BattleWinner::Player
            | game_core::game::battle::types::BattleWinner::Opponent
            | game_core::game::battle::types::BattleWinner::Draw
    ));
    assert!(matches!(
        game.get_state(),
        GameState::InSuppressionReplay { .. }
    ));

    let replay_result = game
        .execute(player_id, PlayerBehavior::FinishSuppressionReplay)
        .unwrap();
    if winner == game_core::game::battle::types::BattleWinner::Player {
        let (mode, rewards, selected_reward_uuid) = replay_result
            .as_reward_state()
            .expect("reward state should be returned after a winning replay");
        assert_eq!(mode, RewardMode::ChooseOne);
        assert_eq!(rewards.len(), 1);
        assert_eq!(selected_reward_uuid, None);
        assert!(matches!(game.get_state(), GameState::InBonus { .. }));

        let selected_reward = game
            .execute(
                player_id,
                PlayerBehavior::SelectEvent {
                    event_id: rewards[0].uuid,
                },
            )
            .unwrap();
        let (_, _, selected_reward_uuid) = selected_reward
            .as_reward_state()
            .expect("reward selection should echo reward state");
        assert_eq!(selected_reward_uuid, Some(rewards[0].uuid));
    } else {
        assert!(matches!(replay_result, BehaviorResult::AdvancePhase { .. }));
        assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));
    }
}

#[test]
fn suppression_reward_choose_one_requires_selection_then_claim_and_exit() {
    let base = create_test_game_data();
    let abno_uuid = base.abnormality_data.items[0].uuid;
    let reward_a = enkephalin_bonus(Uuid::from_u128(0xD00D_1000_0000_0001), "reward_a", 40);
    let reward_b = enkephalin_bonus(Uuid::from_u128(0xD00D_1000_0000_0002), "reward_b", 70);

    let random_event = RandomEventMetadata {
        id: "random_pve_reward_choose_one".to_string(),
        name: "Random PvE Reward Choose One".to_string(),
        uuid: Uuid::from_u128(0xD00D_1000_0000_0010),
        event_type: RandomEventType::Suppress,
        risk_level: game_core::game::enums::RiskLevel::HE,
        description: "routes to suppression selection".to_string(),
        image: "pve.png".to_string(),
        inner_metadata: RandomEventInnerMetadata::Suppress(abno_uuid),
    };
    let game_data = create_test_game_data_with_suppression_rewards(
        random_event,
        RewardMode::ChooseOne,
        vec![reward_a.clone(), reward_b.clone()],
    );
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();
    game.set_enkephalin(0);

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let result = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap();
    let suppression_event = result
        .as_request_phase_data()
        .expect("random suppression should return suppression candidates");
    let candidates = suppression_event
        .as_suppression()
        .expect("suppression candidates should exist");
    let candidate = candidates
        .first()
        .expect("at least one suppression candidate should exist");

    let selected_result =
        select_suppression_candidate(&mut game, player_id, suppression_event).unwrap();
    assert!(matches!(selected_result, BehaviorResult::EventSelected));
    assert!(matches!(game.get_state(), GameState::InSuppression { .. }));
    assert_eq!(game.get_phase_events_count(), 0);

    let battle_result = game
        .execute(
            player_id,
            PlayerBehavior::StartSuppression {
                abnormality_id: candidate.abnormality_id.clone(),
            },
        )
        .unwrap();
    let (winner, timeline) = battle_result
        .as_suppress_abnormality()
        .expect("suppression battle result should be returned");
    assert!(!timeline.entries.is_empty());
    assert!(matches!(
        game.get_state(),
        GameState::InSuppressionReplay { .. }
    ));
    assert_eq!(game.get_phase_events_count(), 0);

    let replay_allowed = game.get_allowed_actions();
    assert_eq!(replay_allowed, vec![ActionKind::FinishSuppressionReplay]);
    let err = game
        .execute(player_id, PlayerBehavior::ClaimBonus)
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let replay_result = game
        .execute(player_id, PlayerBehavior::FinishSuppressionReplay)
        .unwrap();
    if winner == game_core::game::battle::types::BattleWinner::Player {
        let (mode, rewards, selected_reward_uuid) = replay_result
            .as_reward_state()
            .expect("reward state should be returned after a winning replay");
        assert_eq!(mode, RewardMode::ChooseOne);
        assert_eq!(rewards.len(), 2);
        assert_eq!(selected_reward_uuid, None);
        assert!(matches!(game.get_state(), GameState::InBonus { .. }));
        assert_eq!(game.get_phase_events_count(), 2);

        let bonus_allowed = game.get_allowed_actions();
        assert!(contains_action_kind(
            &bonus_allowed,
            ActionKind::SelectEvent
        ));
        assert!(contains_action_kind(&bonus_allowed, ActionKind::ClaimBonus));
        assert!(contains_action_kind(&bonus_allowed, ActionKind::ExitBonus));

        let err = game
            .execute(player_id, PlayerBehavior::ClaimBonus)
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let select_result = game
            .execute(
                player_id,
                PlayerBehavior::SelectEvent {
                    event_id: reward_b.uuid,
                },
            )
            .unwrap();
        let (_, _, selected_reward_uuid) = select_result
            .as_reward_state()
            .expect("reward selection should return updated reward state");
        assert_eq!(selected_reward_uuid, Some(reward_b.uuid));

        let claim_result = game.execute(player_id, PlayerBehavior::ClaimBonus).unwrap();
        let (enkephalin, inventory_diff) = claim_result
            .as_bonus_reward()
            .expect("claim should grant reward");
        assert_eq!(enkephalin, reward_b.amount);
        assert!(inventory_diff.added.is_empty());
        assert!(inventory_diff.updated.is_empty());
        assert!(inventory_diff.removed.is_empty());
        assert!(matches!(game.get_state(), GameState::InBonusClaimed { .. }));
        assert_eq!(game.get_phase_events_count(), 0);

        let claimed_allowed = game.get_allowed_actions();
        assert_eq!(claimed_allowed, vec![ActionKind::ExitBonus]);

        let exit_result = game.execute(player_id, PlayerBehavior::ExitBonus).unwrap();
        assert!(matches!(exit_result, BehaviorResult::AdvancePhase { .. }));
        assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));
    } else {
        assert!(matches!(replay_result, BehaviorResult::AdvancePhase { .. }));
        assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));
        assert_eq!(game.get_phase_events_count(), 0);
    }
}

#[test]
fn suppression_reward_claim_all_claims_everything_then_exits() {
    let base = create_test_game_data();
    let abno_uuid = base.abnormality_data.items[0].uuid;
    let reward_a = enkephalin_bonus(Uuid::from_u128(0xD00D_2000_0000_0001), "reward_all_a", 15);
    let reward_b = enkephalin_bonus(Uuid::from_u128(0xD00D_2000_0000_0002), "reward_all_b", 25);

    let random_event = RandomEventMetadata {
        id: "random_pve_reward_claim_all".to_string(),
        name: "Random PvE Reward Claim All".to_string(),
        uuid: Uuid::from_u128(0xD00D_2000_0000_0010),
        event_type: RandomEventType::Suppress,
        risk_level: game_core::game::enums::RiskLevel::HE,
        description: "routes to suppression selection".to_string(),
        image: "pve.png".to_string(),
        inner_metadata: RandomEventInnerMetadata::Suppress(abno_uuid),
    };
    let game_data = create_test_game_data_with_suppression_rewards(
        random_event,
        RewardMode::ClaimAll,
        vec![reward_a.clone(), reward_b.clone()],
    );
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();
    game.set_enkephalin(0);

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let result = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap();
    let suppression_event = result
        .as_request_phase_data()
        .expect("random suppression should return suppression candidates");
    let candidate = suppression_event
        .as_suppression()
        .expect("suppression candidates should exist")
        .first()
        .expect("at least one suppression candidate should exist")
        .clone();

    select_suppression_candidate(&mut game, player_id, suppression_event).unwrap();

    let battle_result = game
        .execute(
            player_id,
            PlayerBehavior::StartSuppression {
                abnormality_id: candidate.abnormality_id,
            },
        )
        .unwrap();
    let (winner, _timeline) = battle_result
        .as_suppress_abnormality()
        .expect("suppression battle result should be returned");
    assert!(matches!(
        game.get_state(),
        GameState::InSuppressionReplay { .. }
    ));

    let replay_result = game
        .execute(player_id, PlayerBehavior::FinishSuppressionReplay)
        .unwrap();
    if winner == game_core::game::battle::types::BattleWinner::Player {
        let (mode, rewards, selected_reward_uuid) = replay_result
            .as_reward_state()
            .expect("reward state should be returned after a winning replay");
        assert_eq!(mode, RewardMode::ClaimAll);
        assert_eq!(rewards.len(), 2);
        assert_eq!(selected_reward_uuid, None);
        assert!(matches!(game.get_state(), GameState::InBonus { .. }));
        assert_eq!(game.get_phase_events_count(), 0);

        let claim_result = game.execute(player_id, PlayerBehavior::ClaimBonus).unwrap();
        let (enkephalin, inventory_diff) = claim_result
            .as_bonus_reward()
            .expect("claim should grant all rewards");
        assert_eq!(enkephalin, reward_a.amount + reward_b.amount);
        assert!(inventory_diff.added.is_empty());
        assert!(inventory_diff.updated.is_empty());
        assert!(inventory_diff.removed.is_empty());
        assert!(matches!(game.get_state(), GameState::InBonusClaimed { .. }));

        let exit_result = game.execute(player_id, PlayerBehavior::ExitBonus).unwrap();
        assert!(matches!(exit_result, BehaviorResult::AdvancePhase { .. }));
        assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));
    } else {
        assert!(matches!(replay_result, BehaviorResult::AdvancePhase { .. }));
        assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));
    }
}

#[test]
fn random_resolution_failure_returns_invalid_static_data() {
    // Given: Random이 존재하지 않는 Bonus UUID를 가리키도록 만든다.
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
    let err = select_random_from_phase_event(&mut game, player_id, &phase_event).unwrap_err();
    assert!(
        matches!(err, GameError::InvalidStaticData(message) if message.contains("random_broken"))
    );
}
