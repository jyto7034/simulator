use std::sync::Arc;

use bevy_ecs::entity::Entity;
use uuid::Uuid;

use crate::ecs::components::Player;
use crate::ecs::resources::{
    CurrentGameContext, CurrentPhaseEvents, Enkephalin, GameProgression, GameState, Inventory,
    SelectedEvent,
};
use crate::ecs::systems::spawn_player;
use crate::game::behavior::{BehaviorResult, GameError, PlayerBehavior};
use crate::game::data::GameData;
use crate::game::enums::{GameOption, OrdealType, PhaseType, RandomEventAction, ShopAction};
use crate::game::events::event_selection::bonus::BonusExecutor;
use crate::game::events::event_selection::random::RandomEventExecutor;
use crate::game::events::event_selection::shop::ShopExecutor;
use crate::game::events::GeneratorContext;
use crate::game::managers::action_scheduler::ActionScheduler;
use crate::game::managers::event_manager::EventManager;

pub struct GameCore {
    world: bevy_ecs::world::World,
    game_data: Arc<GameData>,
    player_entity: Option<Entity>,
    run_seed: u64,
}

impl GameCore {
    /// GameCore 생성
    ///
    /// # Arguments
    /// * `game_data` - game_server에서 로드한 게임 데이터 (Arc로 공유)
    pub fn new(game_data: Arc<GameData>, run_seed: u64) -> Self {
        let mut world = bevy_ecs::world::World::new();

        // Resources 등록
        world.insert_resource(Enkephalin::new(0));
        world.insert_resource(GameProgression::new());
        world.insert_resource(CurrentPhaseEvents::new());
        world.insert_resource(GameState::NotStarted);
        world.insert_resource(Inventory::new());

        // CurrentGameContext 초기화 (NotStarted 상태의 allowed_actions 설정)
        let mut context = CurrentGameContext::new();
        let initial_actions = ActionScheduler::get_allowed_actions(&GameState::NotStarted);
        context.set_allowed_actions(initial_actions);
        world.insert_resource(context);

        Self {
            world,
            game_data,
            player_entity: None,
            run_seed,
        }
    }

    /// 게임 데이터 참조 반환
    pub fn game_data(&self) -> &GameData {
        &self.game_data
    }
    pub fn execute(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        // 1. 행동 검증 (치팅 방지)
        // StartNewGame과 RequestPhaseData는 항상 허용
        let always_allowed = matches!(
            behavior,
            PlayerBehavior::StartNewGame | PlayerBehavior::RequestPhaseData
        );

        if !always_allowed {
            let context = self.world.get_resource::<CurrentGameContext>().unwrap();
            if !context.is_action_allowed(&behavior) {
                return Err(GameError::InvalidAction);
            }
        }

        // 2. 행동 처리
        match behavior {
            // 복잡한 행동 → 개별 핸들러
            PlayerBehavior::StartNewGame => Ok(self.handle_start_new_game(player_id)),
            PlayerBehavior::RequestPhaseData => Ok(self.handle_request_phase_data()),
            PlayerBehavior::SelectEvent { event_id } => Ok(self.handle_select_event(event_id)),

            // 상점 관련 행동 → 통합 핸들러
            PlayerBehavior::PurchaseItem {
                item_uuid,
                ..
            } => self.execute_shop_action(ShopAction::Purchase { item_uuid }),
            PlayerBehavior::RerollShop => self.execute_shop_action(ShopAction::Reroll),
            PlayerBehavior::ExitShop => self.execute_shop_action(ShopAction::Exit),

            // 랜덤 이벤트 관련 행동 → 통합 핸들러
            PlayerBehavior::SelectEventChoice { choice_id } => {
                self.execute_random_event_action(RandomEventAction::SelectChoice { choice_id })
            }
            PlayerBehavior::ExitRandomEvent => {
                self.execute_random_event_action(RandomEventAction::Exit)
            }
        }
    }
}

impl GameCore {
    /// 게임 상태 전환
    ///
    /// # Arguments
    /// * `new_state` - 전환할 새로운 상태
    ///
    /// # Effects
    /// 1. GameState Resource 업데이트
    /// 2. ActionScheduler를 통해 allowed_actions 자동 업데이트
    fn transition_to(&mut self, new_state: GameState) {
        // 1. 상태 변경
        let mut state = self.world.get_resource_mut::<GameState>().unwrap();
        *state = new_state.clone();

        // 2. allowed_actions 자동 업데이트
        let allowed = ActionScheduler::get_allowed_actions(&new_state);
        let mut context = self.world.get_resource_mut::<CurrentGameContext>().unwrap();
        context.set_allowed_actions(allowed);
    }

    fn initial_player(&mut self, player_id: Uuid) {
        // 기존 플레이어가 있는지 확인
        let player_exists = self
            .world
            .query::<&Player>()
            .iter(&self.world)
            .any(|player| player.id == player_id);

        // 없으면 생성
        if !player_exists {
            spawn_player(&mut self.world, player_id);
        }
    }
}

impl GameCore {
    // 플레이어가 게임에 첫 진입을 하였을 때.
    // 각종 초기화만 수행
    fn handle_start_new_game(&mut self, player_id: Uuid) -> BehaviorResult {
        // 플레이어 생성
        self.initial_player(player_id);

        // 상태 전환: WaitingPhaseRequest (allowed_actions 자동 설정)
        self.transition_to(GameState::WaitingPhaseRequest);

        BehaviorResult::StartNewGame
    }

    // 상점 / 랜덤 이벤트 / 보너스 데이터 요청
    fn handle_request_phase_data(&mut self) -> BehaviorResult {
        // 1. 현재 Ordeal, Phase 가져오기
        let (ordeal, phase) = self.get_progression();

        // 2. Context 생성
        let ctx = GeneratorContext::new(&self.world, &self.game_data, self.run_seed);

        // 3. EventManager에게 이벤트 생성 요청
        let phase_event = EventManager::generate_event(ordeal, phase, &ctx);

        // 4. CurrentPhaseEvents에 각 옵션 추가
        let mut current_phase_event = self.world.get_resource_mut::<CurrentPhaseEvents>().unwrap();
        for option in phase_event.options() {
            current_phase_event.add_event(option);
        }

        // 5. 상태 전환: SelectingEvent (allowed_actions 자동 설정)
        self.transition_to(GameState::SelectingEvent);

        // 6. BehaviorResult로 반환
        BehaviorResult::RequestPhaseData(phase_event)
    }

    fn handle_select_event(&mut self, selected_event_id: Uuid) -> BehaviorResult {
        // 1. CurrentPhaseEvents에서 선택된 이벤트 조회 및 제거
        let mut current_phase_events = self.world.get_resource_mut::<CurrentPhaseEvents>().unwrap();

        let event = match current_phase_events.remove_event(selected_event_id) {
            Some(event) => event,
            None => {
                // 이벤트를 찾지 못한 경우 에러 반환
                panic!("Selected event not found: {}", selected_event_id)
            }
        };

        // 2. 이벤트 타입에 따라 처리 및 상태 전환
        match &event {
            GameOption::Shop { shop } => {
                // 상점을 SelectedEvent 에 저장 (리롤용)
                self.world
                    .insert_resource(SelectedEvent::new(event.clone()));

                // 상태 전환: InShop (allowed_actions 자동 설정)
                self.transition_to(GameState::InShop {
                    shop_uuid: shop.uuid,
                });

                BehaviorResult::EventSelected
            }

            GameOption::Bonus { bonus } => {
                // 보너스는 즉시 처리
                // 1. 지급할 양 결정 (min_amount ~ max_amount 범위)
                // TODO: seed 기반으로 변경
                let amount = rand::random::<u32>() % (bonus.max_amount - bonus.min_amount + 1)
                    + bonus.min_amount;

                // 2. BonusExecutor 헬퍼 함수 호출
                let _ = BonusExecutor::grant_bonus(&mut self.world, bonus, amount);

                // 3. 상태는 SelectingEvent 유지 (또는 다음 Phase로)
                // TODO: 보너스 후 자동으로 다음 Phase로 진행?
                BehaviorResult::EventSelected
            }

            GameOption::Random { event: event_meta } => {
                // 상태 전환: InRandomEvent (allowed_actions 자동 설정)
                self.transition_to(GameState::InRandomEvent {
                    event_uuid: event_meta.uuid,
                });

                BehaviorResult::EventSelected
            }

            // Suppression: 진압 작업
            GameOption::SuppressAbnormality {
                abnormality_id,
                risk_level,
                uuid,
            } => {
                // 상태 전환: InSuppression (allowed_actions 자동 설정)
                self.transition_to(GameState::InSuppression {
                    abnormality_uuid: *uuid,
                });

                // TODO: 진압 작업 로직 구현
                //   - 작업 타입 선택 (본능/통찰/애착/억압)
                //   - 성공/실패 판정
                //   - 보상 지급 또는 페널티
                BehaviorResult::SuppressAbnormality {
                    suppress_result: format!(
                        "기물 '{}' 진압 작업 시작 (위험도: {:?})",
                        abnormality_id, risk_level
                    ),
                }
            }

            // Ordeal: 시련 전투
            GameOption::OrdealBattle {
                ordeal_type,
                difficulty,
                uuid,
            } => {
                // 상태 전환: InBattle (allowed_actions 자동 설정)
                self.transition_to(GameState::InBattle { battle_uuid: *uuid });

                // TODO: 전투 시스템 구현
                //   - 전투 초기화
                //   - 전투 진행
                //   - 승패 판정 및 보상
                BehaviorResult::Ordeal {
                    battle_result: format!(
                        "시련 전투 시작: {:?} (난이도: {})",
                        ordeal_type, difficulty
                    ),
                }
            }
        }
    }

    // ============================================================
    // 상점 관련 통합 핸들러
    // ============================================================

    fn execute_shop_action(&mut self, action: ShopAction) -> Result<BehaviorResult, GameError> {
        match action {
            ShopAction::Purchase { item_uuid } => {
                // ShopExecutor 헬퍼 함수 호출
                ShopExecutor::purchase_item(&mut self.world, &self.game_data, item_uuid)
            }

            ShopAction::Sell { item_uuid } => {
                ShopExecutor::sell_tiem(&mut self.world, &self.game_data, item_uuid)
            }

            ShopAction::Reroll => ShopExecutor::reroll(&mut self.world),

            ShopAction::Exit => {
                // 상태 전환: SelectingEvent로 복귀 (allowed_actions 자동 설정)
                // TODO: 다음 Phase로 진행해야 하는지, SelectingEvent로 복귀해야 하는지 결정 필요
                self.transition_to(GameState::SelectingEvent);

                Ok(BehaviorResult::Ok)
            }
        }
    }

    // ============================================================
    // 랜덤 이벤트 관련 통합 핸들러
    // ============================================================

    fn execute_random_event_action(
        &mut self,
        action: RandomEventAction,
    ) -> Result<BehaviorResult, GameError> {
        match action {
            RandomEventAction::SelectChoice { choice_id } => {
                // TODO: 현재 이벤트 메타데이터를 어딘가에서 가져와야 함
                // 임시로 더미 데이터 생성
                let event = crate::game::data::random_event_data::RandomEventMetadata {
                    id: choice_id.clone(),
                    uuid: Uuid::new_v4(),
                    event_type:
                        crate::game::events::event_selection::random::RandomEventType::SuspiciousBox,
                    name: format!("선택지 {} 결과", choice_id),
                    description: "결과 설명".to_string(),
                    image: "result.png".to_string(),
                    risk_level: crate::game::data::random_event_data::EventRiskLevel::Low,
                };

                // RandomEventExecutor 헬퍼 함수 호출
                RandomEventExecutor::process_choice(&mut self.world, &event, &choice_id)?;

                Ok(BehaviorResult::RandomEventState { event })
            }

            RandomEventAction::Exit => {
                // 상태 전환: SelectingEvent로 복귀 (allowed_actions 자동 설정)
                // TODO: 다음 Phase로 진행해야 하는지, SelectingEvent로 복귀해야 하는지 결정 필요
                self.transition_to(GameState::SelectingEvent);

                Ok(BehaviorResult::Ok)
            }
        }
    }
}

// ============================================================
// 테스트 헬퍼 메서드들
// ============================================================

impl GameCore {
    /// 현재 게임 상태 조회
    ///
    /// # Returns
    /// 현재 GameState의 복사본
    pub fn get_state(&self) -> GameState {
        self.world
            .get_resource::<GameState>()
            .cloned()
            .unwrap_or(GameState::NotStarted)
    }

    /// 현재 Enkephalin 양 조회
    ///
    /// # Returns
    /// 현재 Enkephalin 양. Resource가 없으면 0 반환
    pub fn get_enkephalin(&self) -> u32 {
        self.world
            .get_resource::<Enkephalin>()
            .map(|e| e.amount)
            .unwrap_or(0)
    }

    /// 현재 게임 진행 상황 조회
    ///
    /// # Returns
    /// (현재 Ordeal, 현재 Phase) 튜플
    pub fn get_progression(&self) -> (OrdealType, PhaseType) {
        self.world
            .get_resource::<GameProgression>()
            .map(|p| (p.current_ordeal, p.current_phase))
            .unwrap_or((OrdealType::Dawn, PhaseType::I))
    }

    /// 현재 Level 조회
    ///
    /// # Returns
    /// 현재 Level. Resource가 없으면 1 반환
    pub fn get_level(&self) -> u32 {
        use crate::ecs::resources::Level;

        self.world
            .get_resource::<Level>()
            .map(|l| l.level)
            .unwrap_or(1)
    }

    /// 현재 승리 횟수 조회
    ///
    /// # Returns
    /// 현재 WinCount. Resource가 없으면 0 반환
    pub fn get_win_count(&self) -> u32 {
        use crate::ecs::resources::WinCount;

        self.world
            .get_resource::<WinCount>()
            .map(|w| w.count)
            .unwrap_or(0)
    }

    /// 현재 Phase의 이벤트 개수 조회
    ///
    /// # Returns
    /// CurrentPhaseEvents에 저장된 이벤트 개수
    pub fn get_phase_events_count(&self) -> usize {
        self.world
            .get_resource::<CurrentPhaseEvents>()
            .map(|events| events.len())
            .unwrap_or(0)
    }

    /// 현재 허용된 행동 목록 조회
    ///
    /// # Returns
    /// 현재 허용된 PlayerBehavior 목록
    pub fn get_allowed_actions(&self) -> Vec<PlayerBehavior> {
        self.world
            .get_resource::<CurrentGameContext>()
            .map(|ctx| ctx.allowed_actions.clone())
            .unwrap_or_default()
    }

    /// 특정 행동이 허용되는지 확인
    ///
    /// # Arguments
    /// * `action` - 확인할 행동
    ///
    /// # Returns
    /// 허용되면 true, 아니면 false
    pub fn is_action_allowed(&self, action: &PlayerBehavior) -> bool {
        self.world
            .get_resource::<CurrentGameContext>()
            .map(|ctx| ctx.is_action_allowed(action))
            .unwrap_or(false)
    }
}
