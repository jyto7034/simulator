use std::{collections::HashMap, sync::Arc, time::Duration};

use actix::{Actor, Addr, Context, Handler, Message, Recipient, ResponseFuture};
use message::GameEvent;
use phase::{Phase, PhaseState};
use state::GameStateManager;
use tokio::sync::Mutex;
use tracing::{error, info};
use turn::Turn;
use uuid::Uuid;

use crate::{
    card::{
        types::{PlayerIdentity, PlayerKind},
        Card,
    },
    exception::GameError,
    player::{message::SetOpponent, PlayerActor},
};

pub mod choice;
pub mod error_message;
pub mod getter;
pub mod helper;
pub mod message;
pub mod phase;
pub mod state;
pub mod turn;

pub struct GameConfig {}

pub struct GameActor {
    // 플레이어 액터들의 주소 저장 (PlayerActor 정의 필요)
    pub players: HashMap<PlayerIdentity, Addr<PlayerActor>>,
    pub connections: Arc<Mutex<HashMap<Uuid, Recipient<GameEvent>>>>, // 플레이어의 ConnectionActor 주소 저장
    pub player_connection_ready: HashMap<PlayerKind, bool>, // 각 플레이어 초기화 완료 여부
    pub phase_state: PhaseState,
    pub all_cards: HashMap<PlayerKind, Vec<Card>>,
    pub game_state: Arc<Mutex<GameStateManager>>,
    pub turn: Turn,
    pub is_game_over: bool,
    pub game_id: Uuid,
}

impl Actor for GameActor {
    type Context = Context<Self>;
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
    pub fn new(
        game_id: Uuid,
        player1_id: Uuid,
        player2_id: Uuid,
        player1_deck_code: String,
        player2_deck_code: String,
        attacker_player_type: PlayerKind,
    ) -> Self {
        let p1_identity = PlayerIdentity {
            id: player1_id,
            kind: PlayerKind::Player1,
        };
        let p2_identity = PlayerIdentity {
            id: player2_id,
            kind: PlayerKind::Player2,
        };

        let mut player_actors_map = HashMap::new();

        // PlayerActor 생성 및 맵에 추가
        let p1_addr =
            PlayerActor::create(|_ctx| PlayerActor::new(p1_identity.kind, player1_deck_code));
        let p2_addr =
            PlayerActor::create(|_ctx| PlayerActor::new(p2_identity.kind, player2_deck_code));
        player_actors_map.insert(p1_identity, p1_addr.clone());
        player_actors_map.insert(p2_identity, p2_addr.clone());

        actix::spawn(async move {
            while !p1_addr.connected() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            info!(
                "PlayerActor 1 connected ( not session connection! ), P1 can now receive messages."
            );

            while !p2_addr.connected() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            info!(
                "PlayerActor 2 connected ( not session connection! ), P2 can now receive messages."
            );

            info!("Both players actor connected. ( not session connection! ) Sending SetOpponent messages.");
            p1_addr.do_send(SetOpponent {
                opponent: p2_addr.clone(),
            });
            p2_addr.do_send(SetOpponent {
                opponent: p1_addr.clone(),
            });
        });

        GameActor {
            players: player_actors_map,
            connections: Arc::new(Mutex::new(HashMap::new())),
            player_connection_ready: HashMap::new(),
            all_cards: HashMap::new(),
            phase_state: PhaseState::new(Phase::Mulligan),
            turn: Turn::new(),
            is_game_over: false,
            game_id,
            game_state: Arc::new(Mutex::new(GameStateManager::new())),
        }
    }

    pub fn all_players_ready(&self) -> bool {
        self.player_connection_ready.len() == 2
            && self
                .player_connection_ready
                .get(&PlayerKind::Player1)
                .is_some()
            && self
                .player_connection_ready
                .get(&PlayerKind::Player2)
                .is_some()
    }

    fn get_player_info_by_kind(&self, target_kind: PlayerKind) -> Option<(Uuid, &PlayerIdentity)> {
        for (identity, addr) in &self.players {
            if identity.kind == target_kind {
                return Some((identity.id, identity));
            }
        }
        None
    }

    /// PlayerKind를 기반으로 PlayerActor의 주소(Addr)를 가져옵니다.
    pub fn get_player_addr_by_kind(&self, target_kind: PlayerKind) -> Addr<PlayerActor> {
        for (identity, addr) in &self.players {
            if identity.kind == target_kind {
                return addr.clone();
            }
        }
        // TODO : 나중에 수정해야함.
        panic!("Player with kind {:?} not found", target_kind)
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
    pub fn get_player_uuid_by_kind(&self, target_kind: PlayerKind) -> Uuid {
        for (identity, _) in &self.players {
            if identity.kind == target_kind {
                return identity.id;
            }
        }
        // TODO : 나중에 수정해야함.
        panic!("Player with kind {:?} not found", target_kind)
    }

    /// PlayerKind를 기반으로 ConnectionActor의 주소(Addr)를 가져옵니다.
    pub fn get_connection_addr_by_kind(
        &self,
        target_kind: PlayerKind,
    ) -> Option<Recipient<GameEvent>> {
        todo!()
    }

    /// PlayerKind를 기반으로 해당 PlayerActor에게 메시지를 보내고 결과를 기다립니다. (send 버전)
    ///
    /// # Arguments
    /// * `target_kind` - 메시지를 보낼 대상 플레이어의 PlayerKind.
    /// * `msg` - 보낼 메시지.
    ///
    /// # Returns
    /// * `ResponseFuture<Result<M::Result, GameActorError>>` -
    ///   비동기적으로 PlayerActor 핸들러의 결과 또는 에러를 반환합니다.
    ///   `GameActorError`는 플레이어를 찾지 못했거나 Mailbox 에러를 포함할 수 있습니다.
    pub fn send_to_player_actor<M>(
        &self,
        target_kind: PlayerKind,
        msg: M,
    ) -> ResponseFuture<Result<M::Result, GameError>>
    where
        M: Message + Send + 'static, // 메시지 제약 조건
        M::Result: Send,             // 결과 제약 조건
        PlayerActor: Handler<M>,     // PlayerActor가 이 메시지를 처리할 수 있어야 함
    {
        // 1. target_kind에 해당하는 PlayerActor의 주소(Addr)를 찾습니다.
        let addr = self.get_player_addr_by_kind(target_kind);
        // 2. 주소를 찾았으면, send 메서드를 호출하고 결과를 await합니다.
        //    send의 결과는 Result<M::Result, MailboxError> 입니다.
        //    이를 GameError로 매핑하여 반환합니다.

        let game_id = self.game_id;
        Box::pin(async move {
            info!(
                "GAME ACTOR [{}]: Sending message to PlayerActor ({:?}) and awaiting response.",
                // self.game_id, // self 직접 접근 불가, 필요시 game_id를 클론해서 전달
                game_id,
                target_kind
            );
            match addr.send(msg).await {
                Ok(handler_result) => {
                    // PlayerActor 핸들러가 반환한 M::Result
                    // 이 M::Result 자체가 Result<T, E>일 수 있음 (핸들러가 오류를 반환하는 경우)
                    // 여기서는 M::Result를 그대로 반환 (필요시 내부 Result 처리)
                    Ok(handler_result)
                }
                Err(mailbox_error) => {
                    error!(
                        "GAME ACTOR: Mailbox error sending message to PlayerActor ({:?}): {:?}",
                        target_kind, mailbox_error
                    );
                    Err(GameError::MailboxError)
                }
            }
        })
    }

    /// PlayerKind를 기반으로 해당 PlayerActor에게 메시지를 보냅니다. (do_send 버전)
    pub fn do_send_to_player_actor<M>(&self, target_kind: PlayerKind, msg: M)
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
