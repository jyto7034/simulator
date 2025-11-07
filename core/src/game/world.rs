use std::sync::Arc;

use bevy_ecs::entity::Entity;
use uuid::Uuid;

use crate::ecs::components::Player;
use crate::ecs::entities::spawn_player;
use crate::ecs::resources::{Enkephalin, GameProgression};
use crate::game::behavior::{BehaviorResult, GameError, PlayerBehavior};
use crate::game::data::GameData;
use crate::game::enums::PhaseEvent;
use crate::game::events::GeneratorContext;
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
        let ans = match behavior {
            PlayerBehavior::StartNewGame => self.handle_start_new_game(player_id),
            PlayerBehavior::SelectEvent { event_id } => self.handle_select_event(event_id),
            PlayerBehavior::SuppressAbnormality { abnormality_id } => {
                self.handle_suppress_abnormality(abnormality_id)
            }
            PlayerBehavior::Ordeal { opponent_data } => self.handle_ordeal(opponent_data),
            PlayerBehavior::AdvancePhase => self.handle_advance_phase(),
            PlayerBehavior::PurchaseItem { shop_id, item_id } => {
                self.handle_purchase_item(shop_id, item_id)
            }
            PlayerBehavior::SelectBonus { bonus_type } => self.handle_select_bonus(bonus_type),
        };

        Ok(ans)
    }
}

impl GameCore {
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
    fn handle_start_new_game(&mut self, player_id: Uuid) -> BehaviorResult {
        // 플레이어 생성
        self.initial_player(player_id);

        // 현재 Ordeal, Phase 기반으로 이벤트 생성
        let progression = self.world.get_resource::<GameProgression>().unwrap();
        let ordeal = progression.current_ordeal;
        let phase = progression.current_phase;

        let ctx = GeneratorContext::new(&self.world, &self.game_data, self.run_seed);

        let events = EventManager::generate_event(ordeal, phase, &ctx);

        if let PhaseEvent::EventSelection(result) = events {
            BehaviorResult::StartNewGame { result }
        } else {
            unreachable!("OrdealScheduler guarantees EventSelection in Phase I")
        }
    }

    fn handle_ordeal(&mut self, opponent_data: Player) -> BehaviorResult {
        // 현재 Ordeal, Phase 가져오기
        let progression = self.world.get_resource::<GameProgression>().unwrap();
        let ordeal = progression.current_ordeal;
        let phase = progression.current_phase;

        // opponent_data를 포함한 Context 생성
        let ctx = GeneratorContext::with_opponent(
            &self.world,
            &self.game_data,
            self.run_seed,
            opponent_data,
        );

        // Ordeal 이벤트 생성 (opponent_data 포함)
        let _event = EventManager::generate_event(ordeal, phase, &ctx);

        // TODO: 전투 실행 및 결과 처리
        todo!("Implement Ordeal battle execution")
    }

    fn handle_select_event(&mut self, event_id: String) -> BehaviorResult {
        // TODO: event_id로 이벤트 실행 (상점/보너스/랜덤)
        todo!("Implement event selection: {}", event_id)
    }

    fn handle_suppress_abnormality(&mut self, abnormality_id: String) -> BehaviorResult {
        // TODO: abnormality_id로 진압 작업 실행
        todo!("Implement suppression: {}", abnormality_id)
    }

    fn handle_advance_phase(&mut self) -> BehaviorResult {
        // TODO: Phase 진행 로직
        // 1. 현재 Phase 확인
        // 2. 다음 Phase로 이동
        // 3. 다음 Phase 이벤트 생성 및 반환
        todo!("Implement phase advancement")
    }

    fn handle_purchase_item(&mut self, shop_id: String, item_id: String) -> BehaviorResult {
        // TODO: shop_id, item_id로 아이템 구매
        // 1. 골드 확인
        // 2. 아이템 구매
        // 3. 가방에 추가 (또는 티어 업)
        todo!(
            "Implement item purchase: shop={}, item={}",
            shop_id,
            item_id
        )
    }

    fn handle_select_bonus(
        &mut self,
        bonus_type: crate::game::events::event_selection::bonus::BonusType,
    ) -> BehaviorResult {
        // TODO: bonus_type 적용
        // 1. 보너스 타입에 따라 처리 (Gold, Experience, Item, Abnormality)
        // 2. 플레이어 상태 업데이트
        todo!("Implement bonus selection: {:?}", bonus_type)
    }
}

/*
일단 설계구조를 먼저 잡고 싶어.
Game Server 에서 API 방식으로 사용하는거라 호출하는 입장에서 복잡하면 안돼.
그러니 내가 제안한 enum 방식이 적합하다고 생각해.
물론 이보다 더 좋은 방법이 있으면 내게 말해주고.

Game Core 에선 외부에서 사용하기 편하게
행동을 정의한 enum PlayerBehavior 과
fn execute() 함수를 제공해.

    pub enum PlayerBehavior {
        StartNewGame,
        SelectEvent(event_id: String),
        SuppressAbnormality(abnormality_id: String),
        ...
    }

    pub struct GameCore{
        world: becvy_ecs::world::World,
    }

    impl GameCore{
        pub fn execute(&mut self, behavior: PlayerBehavior) {
            match behavior {
                PlayerBehavior::StartNewGame => { ... }
                PlayerBehavior::SelectEvent(event_id) => { ... }
                PlayerBehavior::SuppressAbnormality(abnormality_id) => { ... }
                ...
            }
        }
    }

대충 이런 느낌이야.
Game Server 에선 GameCore 인스턴스를 생성하고 execute() 함수만 호출하면 돼.
이걸 싱글턴 같은 패턴으로 할까 고민 중이기도 하고. GameServer 특성 상 GameCore 인스턴스가 하나만 있진 않을거라
관리 구조체를 따로 만들어야하나 싶기도 하고 무튼.

게임 상태에 대해서는 스냅샷 방식을 채택 할 계획이야.
먼저 첫 접속 시, 게임의 모든 정보가 담긴 스냅샷을 찍어서 json 으로 유저에게 보내.
이후 플레이어의 행동을 성공적으로 execute 하면 이전 스냅샷에서 변화된 값만 추려서 json 으로 보내는거지.


GameCore 말고도 여러 헬퍼 구조체들이 존재해.
먼저 게임은 5단계로 나뉘어져 있어. 각 단계는 Ordeal (시련) 라고 불려.
각 시련은 Dawn, Noon, White, Dusk, Midnight 로 구분돼.

각 시련에는 5~6개의 Phase (단계) 가 존재해.
각 단계에는 EventSelection (이벤트 선택), Suppression (이상징후 억제), OrdealBattle (시련 전투) 같은 여러 이벤트가 있어.
#[derive(Clone, Copy, Debug)]
pub enum PhaseEventType {
    EventSelection,
    Suppression,
    Ordeal,
}
각 단계는 PhaseSchedule 라는 구조체로 정의할 수 있어.
pub struct PhaseSchedule {
    pub phase_number: u8,
    pub event_type: PhaseEventType,
}
OrdealScheduler 구조체는 시련에 따라 적절한 Phase 스케줄을 반환해 ( PhaseSchedule )
PhaseResolver 구조체는 OrdealScheduler 가 반환한 스케줄을 기반으로 실제 이벤트 발생 기능을 수행해.

발생할 수 있는 이벤트는 다음과 같아.
 - 상점
 - 무료 보너스 (골드/경험치/아이템/기물 등)
 - 랜덤 이벤트 (저점~고점 랜덤 이벤트)

pve 이벤트의 경우 3개의 랜덤 몬스터가 생기고 플레이어가 그 중 하나를 선택하는 방식이야

pvp 이벤트의 경우 match server/redis 에 저장된 ghost 와 전투를 벌이는 방식이야.


OrdealScheduler 가 작성해준 스케줄을 따라 PhaseResolver 가 이벤트를 발생시키는 구조이니까.
지금은 위 3가지 이벤트를 system 으로 구현하면 되겠다.
근데 3가지 이벤트를 발생시키는 코드를 모두 PhaseResolver 에 작성하면 너무 코드가 비대해지니까
각 이벤트 별로 모듈을 만들어서 PhaseResolver 가 호출하는 방식으로 하는것도 좋을 것 같아.



이 구조에 대해서 어떻게 생각해?
더 나은 대안이 있는지, 현재 설계의 한계점이라든지 너의 생각을 말해줘.


*/
