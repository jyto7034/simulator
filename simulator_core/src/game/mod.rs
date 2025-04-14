use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, Message};
use phase::PhaseState;
use tracing::warn;
use turn::Turn;

use crate::{card::types::PlayerType, player::PlayerActor};

pub mod choice;
pub mod getter;
pub mod helper;
pub mod message;
pub mod phase;
pub mod turn;

pub struct GameConfig {}

pub struct GameActor {
    // 플레이어 액터들의 주소 저장 (PlayerActor 정의 필요)
    players: HashMap<PlayerType, Addr<PlayerActor>>,
    player_states_ready: HashMap<PlayerType, bool>, // 각 플레이어 초기화 완료 여부
    phase_state: PhaseState,
    turn: Turn,
    // 초기화에 필요한 정보 (GameConfig 등)
    // 굳이 저장 할 필요는 없는 듯?
    // config: Option<GameConfig>,
    // 게임 진행 관련 상태 추가 가능 (예: 게임 종료 여부)
    is_game_over: bool,
}
impl Actor for GameActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}

    fn stopped(&mut self, ctx: &mut Self::Context) {}
}

impl GameActor {
    pub fn new() -> Self {
        todo!()
    }

    pub fn all_players_ready(&self) -> bool {
        self.players.len() == 2 && self.player_states_ready.values().all(|&ready| ready)
    }

    pub fn send_to_player<M>(&self, player_type: PlayerType, msg: M)
    where
        M: Message + Send + 'static,
        M::Result: Send,
        PlayerActor: Handler<M>,
    {
        if let Some(addr) = self.players.get(&player_type) {
            addr.do_send(msg);
        } else {
            warn!(
                "GAME ACTOR: Error - Player {:?} not found for sending message.",
                player_type
            );
        }
    }

    pub fn broadcast<M>(&self, player_type: PlayerType, msg: M)
    where
        M: Message + Send + Clone + 'static,
        M::Result: Send,
        PlayerActor: Handler<M>,
    {
        for addr in self.players.values() {
            if let Some(addr) = self.players.get(&player_type) {
                addr.do_send(msg.clone());
            } else {
                warn!(
                    "GAME ACTOR: Error - Player {:?} not found for broadcasting message.",
                    player_type
                );
            }
        }
    }
}
