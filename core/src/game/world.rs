use bevy_ecs::entity::Entity;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::ecs::components::Player;
use crate::ecs::resources::{
    ActionValidator, CurrentPhaseEvents, Enkephalin, GameProgression, GameState, Inventory,
    InventoryDiffDto, Qliphoth, SelectedEvent,
};
use crate::ecs::systems::spawn_player;
use crate::game::behavior::{BehaviorResult, GameError, PlayerBehavior};
use crate::game::data::{random_event_data::RandomEventTarget, GameDataBase};
use crate::game::enums::{BonusAction, GameOption, OrdealType, PhaseType, ShopAction};
use crate::game::events::event_selection::bonus::BonusExecutor;
use crate::game::events::event_selection::shop::ShopExecutor;
use crate::game::events::GeneratorContext;
use crate::game::managers::action_scheduler::ActionScheduler;
use crate::game::managers::event_manager::EventManager;

pub struct GameCore {
    world: bevy_ecs::world::World,
    game_data: Arc<GameDataBase>,
    _player_entity: Option<Entity>,
    run_seed: u64,
}

impl GameCore {
    /// GameCore 생성
    ///
    /// # Arguments
    /// * `game_data` - game_server에서 로드한 게임 데이터 (Arc로 공유)
    pub fn new(game_data: Arc<GameDataBase>, run_seed: u64) -> Self {
        info!("Initializing GameCore with run_seed={}", run_seed);

        let mut world = bevy_ecs::world::World::new();

        // Resources 등록
        world.insert_resource(Enkephalin::new(0));
        world.insert_resource(GameProgression::new());
        world.insert_resource(CurrentPhaseEvents::new());
        world.insert_resource(GameState::NotStarted);
        world.insert_resource(Inventory::new());
        world.insert_resource(Qliphoth::new());

        // CurrentGameContext 초기화 (NotStarted 상태의 allowed_actions 설정)
        let mut context = ActionValidator::new();
        let initial_actions = ActionScheduler::get_allowed_actions(&GameState::NotStarted);
        context.set_allowed_actions(initial_actions);
        world.insert_resource(context);

        debug!("GameCore world initialized with default resources");

        Self {
            world,
            game_data,
            _player_entity: None,
            run_seed,
        }
    }

    pub fn execute(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        debug!("Executing behavior {:?} for player {}", behavior, player_id);
        // 1. 행동 검증 (치팅 방지)
        // StartNewGame과 RequestPhaseData는 항상 허용
        let always_allowed = matches!(
            behavior,
            PlayerBehavior::StartNewGame | PlayerBehavior::RequestPhaseData
        );

        if !always_allowed {
            let context = self
                .world
                .get_resource::<ActionValidator>()
                .ok_or(GameError::MissingResource("ActionValidator"))?;
            if !context.is_action_allowed(&behavior) {
                warn!(
                    "Rejected behavior {:?} for player {} (not allowed in current context)",
                    behavior, player_id
                );
                return Err(GameError::InvalidAction);
            }
        }

        // 2. 행동 처리
        match behavior {
            // 복잡한 행동
            PlayerBehavior::StartNewGame => self.handle_start_new_game(player_id),
            PlayerBehavior::RequestPhaseData => self.handle_request_phase_data(),
            PlayerBehavior::SelectEvent { event_id } => self.handle_select_event(event_id),

            // 상점 관련 행동
            PlayerBehavior::PurchaseItem { item_uuid, .. } => {
                self.execute_shop_action(ShopAction::Purchase { item_uuid })
            }
            PlayerBehavior::SellItem { item_uuid } => {
                self.execute_shop_action(ShopAction::Sell { item_uuid })
            }
            PlayerBehavior::RerollShop => self.execute_shop_action(ShopAction::Reroll),
            PlayerBehavior::ExitShop => self.execute_shop_action(ShopAction::Exit),

            // 보너스 관련 행동
            PlayerBehavior::ClaimBonus => self.execute_bonus_action(BonusAction::Claim),
            PlayerBehavior::ExitBonus => self.execute_bonus_action(BonusAction::Exit),
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
    fn transition_to(&mut self, new_state: GameState) -> Result<(), GameError> {
        // 1. 상태 변경
        let mut state = self
            .world
            .get_resource_mut::<GameState>()
            .ok_or(GameError::MissingResource("GameState"))?;
        let old_state = state.clone();
        *state = new_state.clone();
        info!("Game state transition: {:?} -> {:?}", old_state, new_state);

        // 2. allowed_actions 자동 업데이트
        let allowed = ActionScheduler::get_allowed_actions(&new_state);
        let mut context = self
            .world
            .get_resource_mut::<ActionValidator>()
            .ok_or(GameError::MissingResource("ActionValidator"))?;
        context.set_allowed_actions(allowed);

        Ok(())
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
            info!("Spawning new player entity for {}", player_id);
            spawn_player(&mut self.world, player_id);
        } else {
            debug!("Player {} already exists in world", player_id);
        }
    }
}

impl GameCore {
    // 플레이어가 게임에 첫 진입을 하였을 때.
    // 각종 초기화만 수행
    fn handle_start_new_game(&mut self, player_id: Uuid) -> Result<BehaviorResult, GameError> {
        // 플레이어 생성
        self.initial_player(player_id);

        // 상태 전환: WaitingPhaseRequest (allowed_actions 자동 설정)
        self.transition_to(GameState::WaitingPhaseRequest)?;

        info!("New game started for player {}", player_id);

        Ok(BehaviorResult::StartNewGame)
    }

    // 상점 / 랜덤 이벤트 / 보너스 데이터 요청
    fn handle_request_phase_data(&mut self) -> Result<BehaviorResult, GameError> {
        // 1. 현재 Ordeal, Phase 가져오기
        let (ordeal, phase) = self.get_progression()?;

        // 2. Context 생성
        let ctx = GeneratorContext::new(&self.world, &self.game_data, self.run_seed);

        let qliphoth = self.get_qliphoth()?;

        // 3. EventManager에게 이벤트 생성 요청
        let phase_event = EventManager::generate_event(qliphoth, ordeal, phase, &ctx);

        info!(
            "Generated phase event for ordeal={:?}, phase={:?}, event_type={:?}",
            ordeal,
            phase,
            phase_event.event_type()
        );

        // 4. CurrentPhaseEvents에 각 옵션 추가
        let mut current_phase_event = self
            .world
            .get_resource_mut::<CurrentPhaseEvents>()
            .ok_or(GameError::MissingResource("CurrentPhaseEvents"))?;
        for option in phase_event.options() {
            current_phase_event.add_event(option);
        }

        // 5. 상태 전환: SelectingEvent (allowed_actions 자동 설정)
        self.transition_to(GameState::SelectingEvent)?;

        // 6. BehaviorResult로 반환
        Ok(BehaviorResult::RequestPhaseData(phase_event))
    }

    fn handle_select_event(
        &mut self,
        selected_event_id: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        // 1. CurrentPhaseEvents에서 선택된 이벤트 조회 및 제거
        let mut current_phase_events = self
            .world
            .get_resource_mut::<CurrentPhaseEvents>()
            .ok_or(GameError::MissingResource("CurrentPhaseEvents"))?;

        let event = current_phase_events
            .remove_event(selected_event_id)
            .ok_or_else(|| {
                warn!("Selected event not found: {}", selected_event_id);
                GameError::EventNotFound
            })?;

        self.world
            .insert_resource(SelectedEvent::new(event.clone()));

        // 2. 이벤트 타입에 따라 처리 및 상태 전환
        match &event {
            GameOption::Shop { shop } => {
                // 상태 전환: InShop (allowed_actions 자동 설정)
                self.transition_to(GameState::InShop {
                    shop_uuid: shop.uuid,
                })?;

                info!("Entered shop: id={}, uuid={}", shop.id, shop.uuid);

                Ok(BehaviorResult::EventSelected)
            }

            GameOption::Bonus { bonus } => {
                // 상태 전환: InBonus (allowed_actions 자동 설정)
                self.transition_to(GameState::InBonus {
                    bonus_uuid: bonus.uuid,
                })?;

                info!("Entered bonus event: id={}, uuid={}", bonus.id, bonus.uuid);

                // 클라이언트는 PhaseEvent 쪽 메타데이터를 이미 알고 있으므로
                // 여기서는 "보너스 화면으로 진입했다"는 신호만 보낸다.
                Ok(BehaviorResult::EventSelected)
            }

            GameOption::Random { event } => {
                // Random 이벤트는 inner_metadata 를 통해 실제 대상(Shop/Bonus/Suppress)로 라우팅
                let target = event.inner_metadata.resolve(&self.game_data)?;

                match target {
                    RandomEventTarget::Shop(shop_meta) => {
                        let shop = shop_meta.clone();

                        // SelectedEvent 를 Shop 기반으로 교체
                        self.world
                            .insert_resource(SelectedEvent::new(GameOption::Shop {
                                shop: shop.clone(),
                            }));

                        // 상태 전환: InShop
                        self.transition_to(GameState::InShop {
                            shop_uuid: shop.uuid,
                        })?;

                        info!(
                            "Random event '{}' routed to shop: id={}, uuid={}",
                            event.id, shop.id, shop.uuid
                        );

                        Ok(BehaviorResult::EventSelected)
                    }
                    RandomEventTarget::Bonus(bonus_meta) => {
                        let bonus = bonus_meta.clone();

                        self.world
                            .insert_resource(SelectedEvent::new(GameOption::Bonus {
                                bonus: bonus.clone(),
                            }));

                        self.transition_to(GameState::InBonus {
                            bonus_uuid: bonus.uuid,
                        })?;

                        info!(
                            "Random event '{}' routed to bonus: id={}, uuid={}",
                            event.id, bonus.id, bonus.uuid
                        );

                        Ok(BehaviorResult::EventSelected)
                    }
                    RandomEventTarget::Suppress(abno_meta) => {
                        let abnormality_id = abno_meta.id.clone();
                        let risk_level = abno_meta.risk_level;
                        let uuid = abno_meta.uuid;

                        self.world.insert_resource(SelectedEvent::new(
                            GameOption::SuppressAbnormality {
                                abnormality_id: abnormality_id.clone(),
                                risk_level,
                                uuid,
                            },
                        ));

                        self.transition_to(GameState::InSuppression {
                            abnormality_uuid: uuid,
                        })?;

                        info!(
                            "Random event '{}' routed to suppression: abnormality_id={}, uuid={}",
                            event.id, abnormality_id, uuid
                        );

                        Ok(BehaviorResult::SuppressAbnormality {
                            suppress_result: format!(
                                "기물 '{}' 진압 작업 시작 (위험도: {:?})",
                                abnormality_id, risk_level
                            ),
                        })
                    }
                }
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
                })?;

                // TODO: 진압 작업 로직 구현
                //   - 작업 타입 선택 (본능/통찰/애착/억압)
                //   - 성공/실패 판정
                //   - 보상 지급 또는 페널티
                Ok(BehaviorResult::SuppressAbnormality {
                    suppress_result: format!(
                        "기물 '{}' 진압 작업 시작 (위험도: {:?})",
                        abnormality_id, risk_level
                    ),
                })
            }

            // Ordeal: 시련 전투
            GameOption::OrdealBattle {
                ordeal_type,
                difficulty,
                uuid,
            } => {
                // 상태 전환: InBattle (allowed_actions 자동 설정)
                self.transition_to(GameState::InBattle { battle_uuid: *uuid })?;

                // TODO: 전투 시스템 구현
                //   - 전투 초기화
                //   - 전투 진행
                //   - 승패 판정 및 보상
                Ok(BehaviorResult::Ordeal {
                    battle_result: format!(
                        "시련 전투 시작: {:?} (난이도: {})",
                        ordeal_type, difficulty
                    ),
                })
            }
        }
    }

    // ============================================================
    // 상점 관련 통합 핸들러
    // ============================================================

    fn execute_shop_action(&mut self, action: ShopAction) -> Result<BehaviorResult, GameError> {
        match action {
            ShopAction::Purchase { item_uuid } => {
                ShopExecutor::purchase_item(&mut self.world, &self.game_data, item_uuid)
            }

            ShopAction::Sell { item_uuid } => ShopExecutor::sell_tiem(&mut self.world, item_uuid),

            // TODO: Reroll 시 자원 소모 ( 엔케팔린 혹은 특정 자원 )
            ShopAction::Reroll => ShopExecutor::reroll(&mut self.world),

            ShopAction::Exit => {
                // 상태 전환: SelectingEvent로 복귀 (allowed_actions 자동 설정)
                // TODO: 다음 Phase로 진행해야 하는지, SelectingEvent로 복귀해야 하는지 결정 필요
                self.transition_to(GameState::SelectingEvent)?;

                Ok(BehaviorResult::Ok)
            }
        }
    }

    // ============================================================
    // 보너스 관련 통합 핸들러
    // ============================================================

    fn execute_bonus_action(&mut self, action: BonusAction) -> Result<BehaviorResult, GameError> {
        match action {
            BonusAction::Claim => {
                // 1. 현재 선택된 보너스 메타데이터 조회 (소유권으로 복사하여 borrow 충돌 방지)
                let bonus = {
                    let selected = self
                        .world
                        .get_resource::<SelectedEvent>()
                        .ok_or(GameError::NotInBonusState)?;
                    selected.as_bonus()?.clone()
                };

                // 2. BonusExecutor를 통해 실제 보상 지급
                info!(
                    "Applying bonus '{}' (uuid={}) with amount={}",
                    bonus.id, bonus.uuid, bonus.amount
                );
                BonusExecutor::grant_bonus(&mut self.world, &bonus)?;

                // 4. 현재 Enkephalin 및 인벤토리 변경 사항을 BehaviorResult로 반환
                let enkephalin = self
                    .world
                    .get_resource::<Enkephalin>()
                    .map(|e| e.amount)
                    .unwrap_or(0);

                // TODO: BonusType::Item / Abnormality 지원 시 실제 diff 구성
                let inventory_diff = InventoryDiffDto {
                    added: Vec::new(),
                    updated: Vec::new(),
                    removed: Vec::new(),
                };

                // 5. 상태 전환: SelectingEvent로 복귀
                // TODO: 보너스 후 자동으로 다음 Phase로 진행할지 결정
                self.transition_to(GameState::SelectingEvent)?;

                Ok(BehaviorResult::BonusReward {
                    enkephalin,
                    inventory_diff,
                })
            }

            BonusAction::Exit => {
                // 상태 전환: SelectingEvent로 복귀
                self.transition_to(GameState::SelectingEvent)?;
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

    /// Enkephalin 양 설정 (테스트 헬퍼)
    pub fn set_enkephalin(&mut self, amount: u32) {
        if let Some(mut enkephalin) = self.world.get_resource_mut::<Enkephalin>() {
            enkephalin.amount = amount;
        }
    }

    /// 현재 게임 진행 상황 조회
    ///
    /// # Returns
    /// (현재 Ordeal, 현재 Phase) 튜플
    pub fn get_progression(&self) -> Result<(OrdealType, PhaseType), GameError> {
        Ok(self
            .world
            .get_resource::<GameProgression>()
            .map(|p| (p.current_ordeal, p.current_phase))
            .ok_or(GameError::MissingResource(""))?)
    }

    pub fn get_qliphoth(&self) -> Result<Qliphoth, GameError> {
        self.world
            .get_resource::<Qliphoth>()
            .ok_or(GameError::MissingResource(""))
            .cloned()
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
            .get_resource::<ActionValidator>()
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
            .get_resource::<ActionValidator>()
            .map(|ctx| ctx.is_action_allowed(action))
            .unwrap_or(false)
    }
}
