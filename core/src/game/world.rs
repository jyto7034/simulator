use crate::game::resources::Enkephalin;

pub struct GameWorld {
    world: bevy_ecs::world::World,
}

impl GameWorld {
    pub fn new() -> Self {
        let mut world = bevy_ecs::world::World::new();

        world.insert_resource(Enkephalin::new(0));
        Self { world }
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






이 구조에 대해서 어떻게 생각해?
더 나은 대안이 있는지, 현재 설계의 한계점이라든지 너의 생각을 말해줘.


*/
