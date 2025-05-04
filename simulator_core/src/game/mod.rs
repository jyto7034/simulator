use std::collections::HashMap;

use actix::{Actor, Addr, Context, Message};
use phase::{Phase, PhaseState};
use turn::Turn;
use uuid::Uuid;

use crate::{
    card::{
        types::{PlayerIdentity, PlayerKind},
        Card,
    },
    player::{message::SetOpponent, PlayerActor},
    server::actor::connection::ConnectionActor,
};

pub mod choice;
pub mod getter;
pub mod helper;
pub mod message;
pub mod phase;
pub mod turn;

pub struct GameConfig {}

pub struct GameActor {
    // 플레이어 액터들의 주소 저장 (PlayerActor 정의 필요)
    pub players: HashMap<PlayerIdentity, Addr<PlayerActor>>,
    player_states_ready: HashMap<PlayerKind, bool>, // 각 플레이어 초기화 완료 여부
    phase_state: PhaseState,
    all_cards: HashMap<PlayerKind, Vec<Card>>,
    turn: Turn,
    is_game_over: bool,
    game_id: Uuid,
}
impl Actor for GameActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}

    fn stopped(&mut self, ctx: &mut Self::Context) {}
}

impl GameActor {
    /// 새로운 게임 세션을 위한 GameActor를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `game_id` - 이 게임 세션의 고유 ID.
    /// * `attacker_player_type` - 선공 플레이어의 타입.
    ///
    /// # Returns
    ///
    /// 새로운 GameActor 인스턴스.
    pub fn new(game_id: Uuid, attacker_player_type: PlayerKind) -> Self {
        let initial_phase = Phase::Mulligan;

        let players_map = HashMap::new();

        let player_1_actor_addr = PlayerActor::create(|ctx| {
            let player_actor = PlayerActor::new(PlayerKind::Player1);

            player_actor
        });
        let player_2_actor_addr = PlayerActor::create(|ctx| {
            let player_actor = PlayerActor::new(PlayerKind::Player2);

            player_actor
        });

        let ready_map = HashMap::new();

        player_1_actor_addr.do_send(SetOpponent {
            opponent: player_2_actor_addr.clone(),
        });

        player_2_actor_addr.do_send(SetOpponent {
            opponent: player_1_actor_addr.clone(),
        });

        GameActor {
            players: players_map,
            player_states_ready: ready_map,
            phase_state: PhaseState::new(initial_phase),
            turn: Turn::new(),
            is_game_over: false,
            game_id,
            all_cards: HashMap::new(),
        }
    }

    pub fn all_players_ready(&self) -> bool {
        self.players.len() == 2 && self.player_states_ready.values().all(|&ready| ready)
    }

    fn get_player_info_by_kind(&self, target_kind: PlayerKind) -> Option<(Uuid, &PlayerIdentity)> {
        todo!()
    }

    /// PlayerKind를 기반으로 PlayerIdentity의 가변 참조를 가져옵니다.
    fn get_player_info_mut_by_kind(
        &mut self,
        target_kind: PlayerKind,
    ) -> Option<(Uuid, &mut PlayerIdentity)> {
        todo!()
    }

    /// PlayerKind를 기반으로 PlayerActor의 주소(Addr)를 가져옵니다.
    pub fn get_player_addr_by_kind(&self, target_kind: PlayerKind) -> Addr<PlayerActor> {
        todo!()
    }

    pub fn get_player_type_by_uuid(&self, player_id: Uuid) -> PlayerKind {
        for (identity, _) in &self.players {
            if identity.id == player_id {
                return identity.kind;
            }
        }
        // TODO : 나중에 수정해야함.
        panic!("Player with ID {} not found", player_id)
    }

    /// PlayerKind를 기반으로 ConnectionActor의 주소(Addr)를 가져옵니다.
    pub fn get_connection_addr_by_kind(
        &self,
        target_kind: PlayerKind,
    ) -> Option<Addr<ConnectionActor>> {
        todo!()
    }

    /// PlayerKind를 기반으로 해당 PlayerActor에게 메시지를 보냅니다. (do_send 버전)
    pub fn send_to_player_actor<M>(&self, target_kind: PlayerKind, msg: M)
    where
        M: Message + Send + 'static,
        M::Result: Send,
        // PlayerActor: Handler<M>, // Handler 제약은 받는 쪽에서 필요, 보내는 쪽에서는 불필요
    {
        todo!()
    }

    /// PlayerKind를 기반으로 해당 ConnectionActor에게 메시지를 보냅니다. (do_send 버전)
    /// (GameEvent 등을 보낼 때 사용)
    pub fn send_to_connection<M>(&self, target_kind: PlayerKind, msg: M)
    where
        M: Message + Send + 'static,
        M::Result: Send,
        // ConnectionActor: Handler<M>, // 받는 쪽에서 필요
    {
        todo!()
    }

    /// 게임 내 모든 플레이어의 ConnectionActor에게 메시지를 브로드캐스트합니다.
    pub fn broadcast_to_connections<M>(&self, msg: M)
    where
        M: Message + Send + Clone + 'static, // Clone 필요
        M::Result: Send,
        // ConnectionActor: Handler<M>, // 받는 쪽에서 필요
    {
        todo!()
    }
}
