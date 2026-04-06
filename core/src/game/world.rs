use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::ecs::components::Player;
use crate::ecs::resources::item_slot::EquippedRef;
use crate::ecs::resources::{
    ActionValidator, CurrentPhaseEvents, Enkephalin, Field, GameProgression, GameState, Inventory,
    Position, Qliphoth, RewardSessionState, SelectedEvent, SelectedEventState, ShopSessionState,
    SuppressionBattleState,
};
use crate::ecs::systems::{progression, spawn_player};
use crate::game::behavior::{ActionKind, BehaviorResult, GameError, PlayerBehavior};
use crate::game::data::{random_event_data::RandomEventTarget, GameDataBase};
use crate::game::enums::{
    BonusAction, BonusEventOption, GameOption, OrdealType, PhaseEvent, PhaseType, RewardMode,
    ShopAction, ShopEventOption, SuppressionOption, ZoneType,
};
use crate::game::events::event_selection::bonus::BonusExecutor;
use crate::game::events::event_selection::shop::ShopExecutor;
use crate::game::events::suppression::SuppressionExecutor;
use crate::game::events::{suppression::SuppressionGenerator, EventGenerator, GeneratorContext};
use crate::game::managers::action_scheduler::ActionScheduler;
use crate::game::managers::event_manager::EventManager;
use crate::game::managers::uuid_manager::UuidManager;
// use crate::game::{battle::BattleWinner, determinism};
use crate::game::determinism;

pub struct GameCore {
    world: bevy_ecs::world::World,
    game_data: Arc<GameDataBase>,
    run_seed: u64,
}

#[derive(Debug, Clone)]
struct ResolvedSuppressionRequest {
    selected_option: SuppressionOption,
    abnormality_id: String,
    encounter_id: String,
    selected_uuid: Uuid,
    transition_from_selection: bool,
}

impl GameCore {
    /// GameCore 생성
    ///
    /// # Arguments
    /// * `game_data` - game_server에서 로드한 게임 데이터 (Arc로 공유)
    pub fn new(game_data: Arc<GameDataBase>, run_seed: u64) -> Self {
        info!("Initializing GameCore with run_seed={}", run_seed);

        let mut world = bevy_ecs::world::World::new();

        world.insert_resource(UuidManager::new(run_seed));

        // Resources 등록
        world.insert_resource(Enkephalin::new(0));
        world.insert_resource(GameProgression::new());
        world.insert_resource(CurrentPhaseEvents::new());
        world.insert_resource(GameState::NotStarted);
        world.insert_resource(Inventory::new());
        world.insert_resource(Qliphoth::new());
        world.insert_resource(Field::new(4, 4));

        // CurrentGameContext 초기화 (NotStarted 상태의 allowed_actions 설정)
        let mut context = ActionValidator::new();
        let initial_actions = ActionScheduler::get_allowed_actions(&GameState::NotStarted);
        context.set_allowed_actions(initial_actions);
        world.insert_resource(context);

        debug!("GameCore world initialized with default resources");

        Self {
            world,
            game_data,
            run_seed,
        }
    }

    pub fn execute(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        debug!("Executing behavior {:?} for player {}", behavior, player_id);
        let action_kind = behavior.kind();

        // 1. 상태 기반 액션 게이트
        let context = self
            .world
            .get_resource::<ActionValidator>()
            .ok_or(GameError::MissingResource("ActionValidator"))?;
        if !context.is_kind_allowed(action_kind) {
            warn!(
                "Rejected behavior {:?} for player {} (action kind {:?} not allowed in current state)",
                behavior, player_id, action_kind
            );
            return Err(GameError::InvalidAction);
        }

        // 2. payload 검증
        self.validate_behavior_payload(&behavior)?;

        // 3. 행동 처리
        match behavior {
            // 복잡한 행동
            PlayerBehavior::StartNewGame => self.handle_start_new_game(player_id),
            PlayerBehavior::RequestPhaseData => self.handle_request_phase_data(),
            PlayerBehavior::SelectEvent { event_id } => self.handle_select_event(event_id),
            PlayerBehavior::EquipItem {
                item_uuid,
                target_unit,
            } => self.handle_equip_item(item_uuid, target_unit),
            PlayerBehavior::UnEquipItem {
                item_uuid,
                target_unit,
            } => self.handle_unequip_item(item_uuid, target_unit),
            PlayerBehavior::MoveUnit {
                target_unit_uuid,
                dest_pos: dest_type,
            } => self.handle_move_unit(target_unit_uuid, dest_type),
            PlayerBehavior::TransferUnit {
                target_unit_uuid,
                dest_zone,
            } => self.handle_tranfer_unit(target_unit_uuid, dest_zone),

            // 상점 관련 행동
            PlayerBehavior::PurchaseItem { item_uuid } => {
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

            // 진압 관련 행동
            PlayerBehavior::StartSuppression { abnormality_id } => {
                self.handle_start_suppression(&abnormality_id)
            }
            PlayerBehavior::FinishSuppressionReplay => self.handle_finish_suppression_replay(),
        }
    }
}

impl GameCore {
    fn build_shop_session(&self, shop: &ShopEventOption) -> ShopSessionState {
        self.game_data
            .shop_data
            .get_by_uuid(&shop.uuid)
            .map(ShopSessionState::from)
            .unwrap_or_else(|| {
                panic!(
                    "Shop option '{}' ({}) is missing from GameData; static event data is invalid",
                    shop.id, shop.uuid
                )
            })
    }

    fn resolve_bonus_selection(&self, bonus: &BonusEventOption) -> BonusEventOption {
        self.game_data
            .bonus_data
            .get_by_uuid(&bonus.uuid)
            .map(BonusEventOption::from)
            .unwrap_or_else(|| {
                panic!(
                    "Bonus option '{}' ({}) is missing from GameData; static event data is invalid",
                    bonus.id, bonus.uuid
                )
            })
    }

    fn build_reward_session(
        &self,
        stage_uuid: Uuid,
        mode: RewardMode,
        rewards: Vec<BonusEventOption>,
    ) -> RewardSessionState {
        RewardSessionState {
            stage_uuid,
            mode,
            rewards,
            selected_reward_uuid: None,
        }
    }

    fn reward_state_result(&self, reward: &RewardSessionState) -> BehaviorResult {
        BehaviorResult::RewardState {
            mode: reward.mode,
            rewards: reward.rewards.clone(),
            selected_reward_uuid: reward.selected_reward_uuid,
        }
    }

    fn merge_inventory_diff(
        left: &mut crate::ecs::resources::InventoryDiffDto,
        right: crate::ecs::resources::InventoryDiffDto,
    ) {
        left.added.extend(right.added);
        left.updated.extend(right.updated);
        left.removed.extend(right.removed);
    }

    fn validate_behavior_payload(&self, behavior: &PlayerBehavior) -> Result<(), GameError> {
        match behavior {
            PlayerBehavior::StartNewGame
            | PlayerBehavior::RequestPhaseData
            | PlayerBehavior::RerollShop
            | PlayerBehavior::ExitShop
            | PlayerBehavior::ClaimBonus
            | PlayerBehavior::ExitBonus
            | PlayerBehavior::FinishSuppressionReplay
            | PlayerBehavior::UnEquipItem { .. } => Ok(()),
            PlayerBehavior::SelectEvent { event_id } => {
                self.validate_select_event_payload(*event_id)
            }
            PlayerBehavior::EquipItem {
                item_uuid,
                target_unit,
            } => self.validate_equip_item_payload(*item_uuid, *target_unit),
            PlayerBehavior::MoveUnit {
                target_unit_uuid, ..
            } => self.validate_unit_exists(*target_unit_uuid),
            PlayerBehavior::TransferUnit {
                target_unit_uuid, ..
            } => self.validate_unit_exists(*target_unit_uuid),
            PlayerBehavior::PurchaseItem { .. } | PlayerBehavior::SellItem { .. } => Ok(()),
            PlayerBehavior::StartSuppression { abnormality_id } => {
                self.resolve_suppression_request(abnormality_id).map(|_| ())
            }
        }
    }

    fn validate_select_event_payload(&self, event_id: Uuid) -> Result<(), GameError> {
        let current_phase_events = self
            .world
            .get_resource::<CurrentPhaseEvents>()
            .ok_or(GameError::MissingResource("CurrentPhaseEvents"))?;

        current_phase_events
            .get_event(event_id)
            .ok_or(GameError::EventNotFound)
            .map(|_| ())
    }

    fn validate_equip_item_payload(
        &self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<(), GameError> {
        let inventory = self
            .world
            .get_resource::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        let owned_equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;

        if owned_equipment.equipped_to.is_some() {
            return Err(GameError::InvalidAction);
        }

        inventory
            .abnormalities
            .get_owned(&target_unit)
            .ok_or(GameError::UnitNotFound)?;

        Ok(())
    }

    fn validate_unit_exists(&self, target_unit_uuid: Uuid) -> Result<(), GameError> {
        let field = self
            .world
            .get_resource::<Field>()
            .ok_or(GameError::MissingResource("Field"))?;

        field
            .get_position(target_unit_uuid)
            .ok_or(GameError::UnitNotFound)?;

        Ok(())
    }

    fn resolve_suppression_request(
        &self,
        abnormality_id: &str,
    ) -> Result<ResolvedSuppressionRequest, GameError> {
        match self.get_state() {
            GameState::InSuppression { abnormality_uuid } => {
                let selected = self
                    .world
                    .get_resource::<SelectedEvent>()
                    .ok_or(GameError::InvalidAction)?;
                let (expected_id, encounter_id, expected_uuid) = selected.as_suppression()?;
                let risk_level = match &selected.event {
                    SelectedEventState::Suppression(option) => option.risk_level,
                    _ => return Err(GameError::EventTypeMismatch),
                };
                if expected_uuid != abnormality_uuid || expected_id != abnormality_id {
                    warn!(
                        "Suppression mismatch: expected (id={}, uuid={}), got (id={}, uuid={})",
                        expected_id, expected_uuid, abnormality_id, abnormality_uuid
                    );
                    return Err(GameError::InvalidAction);
                }

                Ok(ResolvedSuppressionRequest {
                    selected_option: SuppressionOption {
                        abnormality_id: expected_id.to_string(),
                        encounter_id: encounter_id.to_string(),
                        risk_level,
                        uuid: expected_uuid,
                    },
                    abnormality_id: expected_id.to_string(),
                    encounter_id: encounter_id.to_string(),
                    selected_uuid: expected_uuid,
                    transition_from_selection: false,
                })
            }
            GameState::SelectingEvent => {
                let current_phase_events = self
                    .world
                    .get_resource::<CurrentPhaseEvents>()
                    .ok_or(GameError::MissingResource("CurrentPhaseEvents"))?;

                let matches = current_phase_events
                    .events
                    .values()
                    .filter_map(|option| match option {
                        GameOption::SuppressAbnormality {
                            abnormality_id: id, ..
                        } if id == abnormality_id => Some(option.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                let selected_option = match matches.as_slice() {
                    [] => {
                        warn!(
                            "Rejected StartSuppression for abnormality_id={} (not in current candidates)",
                            abnormality_id
                        );
                        return Err(GameError::InvalidAction);
                    }
                    [only] => only.clone(),
                    _ => {
                        warn!(
                            "Rejected StartSuppression for abnormality_id={} (ambiguous candidates)",
                            abnormality_id
                        );
                        return Err(GameError::InvalidAction);
                    }
                };

                let (selected_id, encounter_id, selected_uuid, risk_level) = match &selected_option
                {
                    GameOption::SuppressAbnormality {
                        abnormality_id,
                        encounter_id,
                        risk_level,
                        uuid,
                        ..
                    } => (
                        abnormality_id.clone(),
                        encounter_id.clone(),
                        *uuid,
                        *risk_level,
                    ),
                    _ => unreachable!("filtered suppression candidate must be suppression"),
                };

                Ok(ResolvedSuppressionRequest {
                    selected_option: SuppressionOption {
                        abnormality_id: selected_id.clone(),
                        encounter_id: encounter_id.clone(),
                        risk_level,
                        uuid: selected_uuid,
                    },
                    abnormality_id: selected_id,
                    encounter_id,
                    selected_uuid,
                    transition_from_selection: true,
                })
            }
            _ => Err(GameError::InvalidAction),
        }
    }

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
        // 이전 Phase의 잔여 선택지/선택 이벤트는 모두 폐기
        if let Some(mut current_phase_events) = self.world.get_resource_mut::<CurrentPhaseEvents>()
        {
            current_phase_events.clear();
        }
        let _ = self.world.remove_resource::<SelectedEvent>();

        // 1. 현재 Ordeal, Phase 가져오기
        let (ordeal, phase) = self.get_progression()?;

        // 2. Context 생성 (phase 기반 seed로 결정성 보장)
        let phase_seed = determinism::seed_for_phase(self.run_seed, ordeal, phase);
        let ctx = GeneratorContext::new(&self.world, &self.game_data, phase_seed);

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
        Ok(BehaviorResult::RequestPhaseData(Box::new(phase_event)))
    }

    fn handle_select_event(
        &mut self,
        selected_event_id: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        if matches!(self.get_state(), GameState::InBonus { .. }) {
            return self.handle_select_reward(selected_event_id);
        }

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
        // 한 Phase에서 이벤트는 1개만 선택되므로 나머지 옵션은 폐기
        current_phase_events.clear();

        // 2. 이벤트 타입에 따라 처리 및 상태 전환
        match event {
            GameOption::Shop { shop } => {
                let shop_session = self.build_shop_session(&shop);
                self.world
                    .insert_resource(SelectedEvent::new(SelectedEventState::Shop(shop_session)));
                // 상태 전환: InShop (allowed_actions 자동 설정)
                self.transition_to(GameState::InShop {
                    shop_uuid: shop.uuid,
                })?;

                info!("Entered shop: id={}, uuid={}", shop.id, shop.uuid);

                Ok(BehaviorResult::EventSelected)
            }

            GameOption::Bonus { bonus } => {
                let selected_bonus = self.resolve_bonus_selection(&bonus);
                let reward_session = self.build_reward_session(
                    selected_bonus.uuid,
                    RewardMode::ClaimAll,
                    vec![selected_bonus.clone()],
                );
                self.world
                    .insert_resource(SelectedEvent::new(SelectedEventState::Reward(
                        reward_session,
                    )));
                // 상태 전환: InBonus (allowed_actions 자동 설정)
                self.transition_to(GameState::InBonus {
                    bonus_uuid: selected_bonus.uuid,
                })?;

                info!(
                    "Entered bonus event: id={}, uuid={}",
                    selected_bonus.id, selected_bonus.uuid
                );

                // 클라이언트는 PhaseEvent 쪽 메타데이터를 이미 알고 있으므로
                // 여기서는 "보너스 화면으로 진입했다"는 신호만 보낸다.
                Ok(BehaviorResult::EventSelected)
            }

            GameOption::Random { event } => {
                // Random 이벤트는 inner_metadata 를 통해 실제 대상(Shop/Bonus/Suppress)로 라우팅
                // - Shop/Bonus: 해당 스테이지로 즉시 진입
                // - Suppress(PvE): 3개 후보를 생성해 "진압 선택 스테이지"로 즉시 진입
                let resolved = event
                    .inner_metadata
                    .resolve(&self.game_data)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Random event '{}' references missing metadata: {:?}",
                            event.id, event.inner_metadata
                        )
                    });

                match resolved {
                    RandomEventTarget::Shop(shop_meta) => {
                        let shop = ShopSessionState::from(shop_meta);
                        self.world
                            .insert_resource(SelectedEvent::new(SelectedEventState::Shop(
                                shop.clone(),
                            )));
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
                        let bonus = BonusEventOption::from(bonus_meta);
                        let reward_session = self.build_reward_session(
                            bonus.uuid,
                            RewardMode::ClaimAll,
                            vec![bonus.clone()],
                        );
                        self.world
                            .insert_resource(SelectedEvent::new(SelectedEventState::Reward(
                                reward_session,
                            )));
                        self.transition_to(GameState::InBonus {
                            bonus_uuid: bonus.uuid,
                        })?;

                        info!(
                            "Random event '{}' routed to bonus: id={}, uuid={}",
                            event.id, bonus.id, bonus.uuid
                        );
                        Ok(BehaviorResult::EventSelected)
                    }
                    RandomEventTarget::Suppress(_abno_meta) => {
                        // PvE 스테이지: 3개 진압 후보 생성 후 SelectingEvent로 진입
                        let (ordeal, phase) = self.get_progression()?;
                        let seed = determinism::seed_for_phase(self.run_seed, ordeal, phase)
                            ^ u64::from_be_bytes(event.uuid.as_bytes()[..8].try_into().unwrap());
                        let ctx = GeneratorContext::new(&self.world, &self.game_data, seed);

                        let generator = SuppressionGenerator;
                        let options = generator.generate(&ctx);
                        let candidates = options.map(|opt| match opt {
                            GameOption::SuppressAbnormality {
                                abnormality_id,
                                encounter_id,
                                risk_level,
                                uuid,
                            } => crate::game::enums::SuppressionOption {
                                abnormality_id,
                                encounter_id,
                                risk_level,
                                uuid,
                            },
                            _ => {
                                unreachable!("SuppressionGenerator must return suppression options")
                            }
                        });

                        // CurrentPhaseEvents에 후보를 저장(이후 StartSuppression 검증에 사용)
                        {
                            let mut current_phase_events = self
                                .world
                                .get_resource_mut::<CurrentPhaseEvents>()
                                .ok_or(GameError::MissingResource("CurrentPhaseEvents"))?;
                            current_phase_events.clear();
                            for option in candidates.clone() {
                                current_phase_events.add_event(option.into());
                            }
                        }
                        let _ = self.world.remove_resource::<SelectedEvent>();
                        self.transition_to(GameState::SelectingEvent)?;

                        info!(
                            "Random event '{}' routed to suppression selection (3 candidates)",
                            event.id
                        );

                        // NOTE: 기존 프로토콜을 최대한 재사용하기 위해 PhaseEvent를 반환한다.
                        Ok(BehaviorResult::RequestPhaseData(Box::new(
                            PhaseEvent::Suppression { candidates },
                        )))
                    }
                }
            }

            // Suppression: 진압 작업
            GameOption::SuppressAbnormality {
                abnormality_id,
                encounter_id,
                risk_level,
                uuid,
            } => {
                self.world
                    .insert_resource(SelectedEvent::new(SelectedEventState::Suppression(
                        SuppressionOption {
                            abnormality_id: abnormality_id.clone(),
                            encounter_id: encounter_id.clone(),
                            risk_level,
                            uuid,
                        },
                    )));

                // 상태 전환: InSuppression (allowed_actions 자동 설정)
                self.transition_to(GameState::InSuppression {
                    abnormality_uuid: uuid,
                })?;

                Ok(BehaviorResult::EventSelected)
            }

            // Ordeal: 시련 전투
            GameOption::OrdealBattle {
                ordeal_type,
                difficulty,
                uuid,
            } => {
                // TODO: 전투 시스템 구현 전까지는 softlock 방지를 위해 즉시 Phase를 진행
                warn!(
                    "Ordeal battle not implemented yet (ordeal_type={:?}, difficulty={}, uuid={}); advancing phase",
                    ordeal_type, difficulty, uuid
                );
                self.advance_to_next_phase()
            }
        }
    }

    fn handle_equip_item(
        &mut self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let mut inventory = self
            .world
            .get_resource_mut::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        // 1. 인벤토리에서 아이템 정보 불러오기
        let (base_uuid, equipment_type, allow_duplicate, equipped_to) = {
            let owned_equipment = inventory
                .equipments
                .get_item(&item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            (
                owned_equipment.meta.uuid,
                owned_equipment.meta.equipment_type,
                owned_equipment.meta.allow_duplicate_equip,
                owned_equipment.equipped_to,
            )
        };

        if equipped_to.is_some() {
            return Err(GameError::InvalidAction);
        }

        // 2. 유닛의 장착 슬롯 로직에 위임 (귀속/중복 룰 포함)
        {
            let owned_abnormality = inventory
                .abnormalities
                .get_owned_mut(&target_unit)
                .ok_or(GameError::UnitNotFound)?;
            owned_abnormality
                .item_slot
                .equip(
                    EquippedRef {
                        instance_uuid: item_uuid,
                        base_uuid,
                        equipment_type,
                    },
                    allow_duplicate,
                )
                .map_err(|_| GameError::InvalidAction)?;
        }

        // 3. 유효한 아이템인지 확인 후, 장착
        let owned_equipment = inventory
            .equipments
            .get_item_mut(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        owned_equipment.equipped_to = Some(target_unit);

        Ok(BehaviorResult::EquipItem)
    }

    fn handle_unequip_item(
        &mut self,
        _item_uuid: Uuid,
        _target_unit: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        // 기본 룰: 아이템은 귀속이며 일반 해제는 불가 (해제 아이템으로만 가능)
        Err(GameError::InvalidAction)
    }
    fn handle_tranfer_unit(
        &mut self,
        target_unit_uuid: Uuid,
        _dest_zone: ZoneType,
    ) -> Result<BehaviorResult, GameError> {
        let field = self
            .world
            .get_resource::<Field>()
            .ok_or(GameError::MissingResource("Field"))?;

        let _from = field
            .get_position(target_unit_uuid)
            .ok_or(GameError::UnitNotFound)?;

        Ok(BehaviorResult::TransferUnit)
    }

    fn handle_move_unit(
        &mut self,
        target_unit_uuid: Uuid,
        dest_type: Position,
    ) -> Result<BehaviorResult, GameError> {
        let mut field = self
            .world
            .get_resource_mut::<Field>()
            .ok_or(GameError::MissingResource("Field"))?;

        let _from = field
            .get_position(target_unit_uuid)
            .ok_or(GameError::UnitNotFound)?;

        field.move_unit(target_unit_uuid, dest_type)?;

        Ok(BehaviorResult::MoveUnit)
    }

    // ============================================================
    // 상점 관련 통합 핸들러
    // ============================================================

    fn execute_shop_action(&mut self, action: ShopAction) -> Result<BehaviorResult, GameError> {
        match action {
            ShopAction::Purchase { item_uuid } => {
                ShopExecutor::purchase_item(&mut self.world, &self.game_data, item_uuid)
            }

            // 환상체 판매 시, 장착 중인 아이템 해제됨.
            ShopAction::Sell { item_uuid } => ShopExecutor::sell_item(&mut self.world, item_uuid),

            // TODO: Reroll 시 자원 소모 ( 엔케팔린 혹은 특정 자원 )
            ShopAction::Reroll => ShopExecutor::reroll(&mut self.world),

            ShopAction::Exit => self.advance_to_next_phase(),
        }
    }

    // ============================================================
    // 보너스 관련 통합 핸들러
    // ============================================================

    fn execute_bonus_action(&mut self, action: BonusAction) -> Result<BehaviorResult, GameError> {
        match action {
            BonusAction::Claim => {
                // 1. 현재 reward 세션을 조회한다.
                let reward = {
                    let selected = self
                        .world
                        .get_resource::<SelectedEvent>()
                        .ok_or(GameError::NotInBonusState)?;
                    selected.as_reward()?.clone()
                };

                let rewards_to_apply: Vec<BonusEventOption> = match reward.mode {
                    RewardMode::ClaimAll => reward.rewards.clone(),
                    RewardMode::ChooseOne => vec![reward
                        .get_selected_reward()
                        .cloned()
                        .ok_or(GameError::InvalidAction)?],
                };

                let mut inventory_diff = crate::ecs::resources::InventoryDiffDto::default();

                for (index, bonus) in rewards_to_apply.iter().enumerate() {
                    info!(
                        "Applying bonus '{}' (uuid={}) with amount={}",
                        bonus.id, bonus.uuid, bonus.amount
                    );
                    let seed = {
                        let (ordeal, phase) = self.get_progression()?;
                        determinism::seed_for_phase(self.run_seed, ordeal, phase)
                            ^ u64::from_be_bytes(bonus.uuid.as_bytes()[..8].try_into().unwrap())
                            ^ index as u64
                    };

                    let granted =
                        BonusExecutor::grant_bonus(&mut self.world, &self.game_data, bonus, seed)?;
                    Self::merge_inventory_diff(&mut inventory_diff, granted);
                }

                // 4. 현재 Enkephalin 및 인벤토리 변경 사항을 BehaviorResult로 반환
                let enkephalin = self
                    .world
                    .get_resource::<Enkephalin>()
                    .map(|e| e.amount)
                    .unwrap_or(0);

                // 5. 보너스 수령 완료 상태로 전환 (Exit에서만 Phase 진행)
                self.transition_to(GameState::InBonusClaimed {
                    bonus_uuid: reward.stage_uuid,
                })?;

                if let Some(mut current_phase_events) =
                    self.world.get_resource_mut::<CurrentPhaseEvents>()
                {
                    current_phase_events.clear();
                }

                Ok(BehaviorResult::BonusReward {
                    enkephalin,
                    inventory_diff,
                })
            }

            BonusAction::Exit => self.advance_to_next_phase(),
        }
    }

    fn handle_select_reward(
        &mut self,
        selected_reward_id: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let selected_bonus = {
            let current_phase_events = self
                .world
                .get_resource::<CurrentPhaseEvents>()
                .ok_or(GameError::MissingResource("CurrentPhaseEvents"))?;
            let event = current_phase_events
                .get_event(selected_reward_id)
                .ok_or(GameError::EventNotFound)?;
            match event {
                GameOption::Bonus { bonus } => bonus.clone(),
                _ => return Err(GameError::EventTypeMismatch),
            }
        };

        let mut selected = self
            .world
            .get_resource_mut::<SelectedEvent>()
            .ok_or(GameError::NotInBonusState)?;
        let reward = selected.as_reward_mut()?;

        if reward.mode != RewardMode::ChooseOne {
            return Err(GameError::InvalidAction);
        }

        if !reward
            .rewards
            .iter()
            .any(|bonus| bonus.uuid == selected_bonus.uuid)
        {
            return Err(GameError::EventNotFound);
        }

        reward.selected_reward_uuid = Some(selected_bonus.uuid);
        let result = BehaviorResult::RewardState {
            mode: reward.mode,
            rewards: reward.rewards.clone(),
            selected_reward_uuid: reward.selected_reward_uuid,
        };

        Ok(result)
    }

    /// Phase 완료 후 다음 Phase로 진행
    ///
    /// 이벤트(상점/보너스/랜덤) 완료 후 호출되어 다음 Phase로 전환합니다.
    fn advance_to_next_phase(&mut self) -> Result<BehaviorResult, GameError> {
        // Phase 종료 시 클리포트 자동 회복
        {
            // let mut qliphoth = self
            //     .world
            //     .get_resource_mut::<Qliphoth>()
            //     .ok_or(GameError::MissingResource("Qliphoth"))?;
            // QliphothManager::apply_phase_recovery(&mut qliphoth);
        }

        // Phase가 끝났으므로 선택지/선택 이벤트 리소스는 폐기
        if let Some(mut current_phase_events) = self.world.get_resource_mut::<CurrentPhaseEvents>()
        {
            current_phase_events.clear();
        }
        let _ = self.world.remove_resource::<SelectedEvent>();

        let next_result = {
            let mut game_progression = self
                .world
                .get_resource_mut::<GameProgression>()
                .ok_or(GameError::MissingResource("GameProgression"))?;

            match progression::advance(&mut game_progression) {
                progression::ProgressionResult::NextPhase(next_phase) => {
                    info!("Advanced to next phase: {:?}", next_phase);
                    BehaviorResult::AdvancePhase {
                        next_phase_event: format!("{:?}", next_phase),
                    }
                }
                progression::ProgressionResult::NextOrdeal(next_ordeal) => {
                    info!("Advanced to next ordeal: {:?}", next_ordeal);
                    BehaviorResult::AdvancePhase {
                        next_phase_event: format!("{:?}", next_ordeal),
                    }
                }
                progression::ProgressionResult::GameComplete => {
                    info!("Game completed!");
                    BehaviorResult::Ok
                }
            }
        };
        let next_state = if matches!(next_result, BehaviorResult::Ok) {
            GameState::GameOver
        } else {
            GameState::WaitingPhaseRequest
        };
        self.transition_to(next_state)?;
        Ok(next_result)
    }

    // ============================================================
    // 진압 관련 통합 핸들러
    // ============================================================

    fn handle_start_suppression(
        &mut self,
        abnormality_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        let resolved = self.resolve_suppression_request(abnormality_id)?;

        if resolved.transition_from_selection {
            if let Some(mut current_phase_events) =
                self.world.get_resource_mut::<CurrentPhaseEvents>()
            {
                current_phase_events.clear();
            }
            self.world
                .insert_resource(SelectedEvent::new(SelectedEventState::Suppression(
                    resolved.selected_option.clone(),
                )));
            self.transition_to(GameState::InSuppression {
                abnormality_uuid: resolved.selected_uuid,
            })?;
        }

        info!(
            "Starting suppression for abnormality={} encounter={} uuid={}",
            resolved.abnormality_id, resolved.encounter_id, resolved.selected_uuid
        );

        let battle_result = SuppressionExecutor::start_battle(
            &mut self.world,
            self.game_data.clone(),
            &resolved.abnormality_id,
            &resolved.encounter_id,
            self.run_seed,
        )?;

        // 진압 작업 성공 시, 해당 몬스터의 드랍템 확률 발생
        info!(
            "Suppression battle completed - Winner: {:?}",
            battle_result.winner
        );

        let (reward_mode, rewards) =
            SuppressionExecutor::resolve_rewards(self.game_data.as_ref(), &resolved.encounter_id)
                .unwrap_or_else(|_| {
                    panic!(
                        "Suppression encounter '{}' has invalid reward configuration",
                        resolved.encounter_id
                    )
                });
        let winner = battle_result.winner;
        let timeline = battle_result.timeline;

        self.world
            .insert_resource(SelectedEvent::new(SelectedEventState::SuppressionBattle(
                SuppressionBattleState {
                    abnormality_id: resolved.abnormality_id,
                    encounter_id: resolved.encounter_id,
                    abnormality_uuid: resolved.selected_uuid,
                    winner,
                    timeline: timeline.clone(),
                    reward_mode,
                    rewards,
                },
            )));
        self.transition_to(GameState::InSuppressionReplay {
            abnormality_uuid: resolved.selected_uuid,
        })?;

        Ok(BehaviorResult::SuppressAbnormality { winner, timeline })
    }

    fn handle_finish_suppression_replay(&mut self) -> Result<BehaviorResult, GameError> {
        let battle = {
            let selected = self
                .world
                .get_resource::<SelectedEvent>()
                .ok_or(GameError::InvalidAction)?;
            selected.as_suppression_battle()?.clone()
        };

        let reward_session = self.build_reward_session(
            battle.abnormality_uuid,
            battle.reward_mode,
            battle.rewards.clone(),
        );

        if let Some(mut current_phase_events) = self.world.get_resource_mut::<CurrentPhaseEvents>()
        {
            current_phase_events.clear();
            if reward_session.mode == RewardMode::ChooseOne {
                for reward in reward_session.rewards.iter().cloned() {
                    current_phase_events.add_event(GameOption::Bonus { bonus: reward });
                }
            }
        }

        self.world
            .insert_resource(SelectedEvent::new(SelectedEventState::Reward(
                reward_session.clone(),
            )));
        self.transition_to(GameState::InBonus {
            bonus_uuid: reward_session.stage_uuid,
        })?;

        Ok(self.reward_state_result(&reward_session))
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
        self.world
            .get_resource::<GameProgression>()
            .map(|p| (p.current_ordeal, p.current_phase))
            .ok_or(GameError::MissingResource("GameProgression"))
    }

    pub fn get_qliphoth(&self) -> Result<Qliphoth, GameError> {
        self.world
            .get_resource::<Qliphoth>()
            .ok_or(GameError::MissingResource("Qliphoth"))
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

    /// 현재 허용된 액션 capability 목록 조회
    ///
    /// # Returns
    /// 현재 허용된 ActionKind 목록
    pub fn get_allowed_actions(&self) -> Vec<ActionKind> {
        self.world
            .get_resource::<ActionValidator>()
            .map(|ctx| ctx.allowed_actions())
            .unwrap_or_default()
    }

    /// 특정 행동 종류가 허용되는지 확인
    ///
    /// # Arguments
    /// * `action` - 확인할 행동
    ///
    /// # Returns
    /// payload와 무관한 capability가 허용되면 true, 아니면 false
    pub fn is_action_allowed(&self, action: &PlayerBehavior) -> bool {
        self.world
            .get_resource::<ActionValidator>()
            .map(|ctx| ctx.is_action_allowed(action))
            .unwrap_or(false)
    }

    pub fn game_state_name(&self) -> &'static str {
        match self.get_state() {
            GameState::NotStarted => "not_started",
            GameState::WaitingPhaseRequest => "waiting_phase_request",
            GameState::SelectingEvent => "selecting_event",
            GameState::InShop { .. } => "in_shop",
            GameState::InBonus { .. } => "in_bonus",
            GameState::InBonusClaimed { .. } => "in_bonus_claimed",
            GameState::InSuppression { .. } => "in_suppression",
            GameState::InSuppressionReplay { .. } => "in_suppression_replay",
            GameState::InBattle { .. } => "in_battle",
            GameState::GameOver => "game_over",
        }
    }

    pub fn qliphoth_level_name(&self, level: crate::ecs::resources::QliphothLevel) -> &'static str {
        match level {
            crate::ecs::resources::QliphothLevel::Stable => "stable",
            crate::ecs::resources::QliphothLevel::Caution => "caution",
            crate::ecs::resources::QliphothLevel::Critical => "critical",
            crate::ecs::resources::QliphothLevel::Meltdown => "meltdown",
        }
    }

    pub fn get_current_phase_events(&self) -> Vec<crate::game::enums::GameOption> {
        let mut events = self
            .world
            .get_resource::<CurrentPhaseEvents>()
            .map(|current| current.events.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        events.sort_by_key(|event| event.uuid());
        events
    }

    pub fn get_inventory_snapshot_json(&self) -> Result<Value, GameError> {
        let inventory = self
            .world
            .get_resource::<Inventory>()
            .ok_or(GameError::MissingResource("Inventory"))?;

        let mut abnormalities = inventory
            .abnormalities
            .iter_owned()
            .map(|owned| {
                let mut equipped_items = owned
                    .item_slot
                    .iter()
                    .map(|equipped| {
                        json!({
                            "instance_uuid": equipped.instance_uuid,
                            "base_uuid": equipped.base_uuid,
                            "equipment_type": equipped.equipment_type,
                        })
                    })
                    .collect::<Vec<_>>();
                equipped_items.sort_by(|left, right| {
                    left["instance_uuid"]
                        .to_string()
                        .cmp(&right["instance_uuid"].to_string())
                });

                json!({
                    "instance_uuid": owned.instance_uuid,
                    "item": crate::ecs::resources::InventoryItemDto::Abnormality(
                        crate::ecs::resources::AbnormalityItemDto::from_owned(
                            owned.instance_uuid,
                            owned.meta.as_ref(),
                        ),
                    ),
                    "growth_stacks": owned.growth_stacks,
                    "equipped_items": equipped_items,
                })
            })
            .collect::<Vec<_>>();
        abnormalities.sort_by(|left, right| {
            left["instance_uuid"]
                .to_string()
                .cmp(&right["instance_uuid"].to_string())
        });

        let mut equipments = inventory
            .equipments
            .iter()
            .map(|owned| {
                json!({
                    "instance_uuid": owned.instance_uuid,
                    "item": crate::ecs::resources::EquipmentItemDto::from_owned(
                        owned.instance_uuid,
                        owned.meta.as_ref(),
                    ),
                    "equipped_to": owned.equipped_to,
                })
            })
            .collect::<Vec<_>>();
        equipments.sort_by(|left, right| {
            left["instance_uuid"]
                .to_string()
                .cmp(&right["instance_uuid"].to_string())
        });

        let mut artifacts = inventory
            .artifacts
            .iter()
            .map(|artifact| {
                json!(crate::ecs::resources::ArtifactItemDto::from_metadata(
                    artifact.as_ref()
                ))
            })
            .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| left["uuid"].to_string().cmp(&right["uuid"].to_string()));

        Ok(json!({
            "abnormalities": abnormalities,
            "equipments": equipments,
            "artifacts": artifacts,
        }))
    }

    pub fn get_field_snapshot_json(&self) -> Result<Value, GameError> {
        let field = self
            .world
            .get_resource::<Field>()
            .ok_or(GameError::MissingResource("Field"))?;

        let mut placements = field
            .placements
            .iter()
            .map(|(position, placement)| {
                json!({
                    "position": position,
                    "unit_uuid": placement.uuid,
                    "side": placement.side,
                })
            })
            .collect::<Vec<_>>();
        placements.sort_by(|left, right| {
            let ly = left["position"]["y"].as_i64().unwrap_or_default();
            let ry = right["position"]["y"].as_i64().unwrap_or_default();
            let lx = left["position"]["x"].as_i64().unwrap_or_default();
            let rx = right["position"]["x"].as_i64().unwrap_or_default();
            ly.cmp(&ry).then_with(|| lx.cmp(&rx))
        });

        Ok(json!({
            "width": field.width,
            "height": field.height,
            "placements": placements,
        }))
    }

    pub fn get_selected_event_snapshot_json(&self) -> Result<Option<Value>, GameError> {
        let Some(selected) = self.world.get_resource::<SelectedEvent>() else {
            return Ok(None);
        };

        let value = match &selected.event {
            SelectedEventState::Shop(shop) => json!({
                "type": "shop",
                "id": shop.id,
                "name": shop.name,
                "uuid": shop.uuid,
                "shop_type": shop.shop_type,
                "can_reroll": shop.can_reroll,
                "visible_items": shop.visible_items,
                "hidden_items": shop.hidden_items,
            }),
            SelectedEventState::Reward(reward) => json!({
                "type": "reward",
                "stage_uuid": reward.stage_uuid,
                "mode": reward.mode,
                "rewards": reward.rewards,
                "selected_reward_uuid": reward.selected_reward_uuid,
            }),
            SelectedEventState::Suppression(option) => json!({
                "type": "suppression",
                "abnormality_id": option.abnormality_id,
                "encounter_id": option.encounter_id,
                "risk_level": option.risk_level,
                "uuid": option.uuid,
            }),
            SelectedEventState::SuppressionBattle(battle) => json!({
                "type": "suppression_battle",
                "abnormality_id": battle.abnormality_id,
                "encounter_id": battle.encounter_id,
                "abnormality_uuid": battle.abnormality_uuid,
                "winner": battle.winner,
                "reward_mode": battle.reward_mode,
                "rewards": battle.rewards,
                "has_timeline": true,
            }),
        };

        Ok(Some(value))
    }

    pub fn get_active_suppression_replay(
        &self,
    ) -> Option<(
        crate::game::battle::types::BattleWinner,
        crate::game::battle::timeline::Timeline,
    )> {
        self.world
            .get_resource::<SelectedEvent>()
            .and_then(|selected| match &selected.event {
                SelectedEventState::SuppressionBattle(battle) => {
                    Some((battle.winner, battle.timeline.clone()))
                }
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
        abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase, equipment_data::EquipmentDatabase, event_pools::EventPhasePool,
        event_pools::EventPoolConfig, pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase, shop_data::ShopDatabase, skill_data::SkillDatabase,
        GameDataBase,
    };
    use std::sync::Arc;

    fn empty_game_data() -> Arc<GameDataBase> {
        let pool = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        let event_pools = EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        };

        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools,
        }))
    }

    #[test]
    fn game_core_rejects_disallowed_actions_and_updates_allowed_actions_on_state_transition() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);

        assert!(matches!(core.get_state(), GameState::NotStarted));
        assert!(core.is_action_allowed(&PlayerBehavior::StartNewGame));
        assert!(!core.is_action_allowed(&PlayerBehavior::RequestPhaseData));

        let err = core
            .execute(player_id, PlayerBehavior::RequestPhaseData)
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let res = core
            .execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();
        assert!(res.is_start_new_game());
        assert!(matches!(core.get_state(), GameState::WaitingPhaseRequest));

        let allowed = core.get_allowed_actions();
        assert!(allowed.contains(&ActionKind::RequestPhaseData));
        assert!(allowed.contains(&ActionKind::EquipItem));
    }
}
