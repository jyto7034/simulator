use std::{collections::HashMap, sync::Arc, time::Duration};

use actix::{
    fut, Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, Recipient, ResponseFuture, Running, SpawnHandle
};
use futures::{future::join_all, FutureExt};
use msg::GameEvent;
use state::GameStateManager;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use turn::TurnState;
use uuid::Uuid;

use crate::{card::{types::{PlayerIdentity, PlayerKind}, Card}, exception::{GameError, SystemError}, game::{msg::{system::GetPlayerStateSnapshot}, state::GamePhase}, player::{message::{GameOver, Terminate}, PlayerActor}, sync::{snapshots::{GameStateSnapshot, OpponentStateSnapshot}, SyncActor}};

pub mod choice;
pub mod phase;
pub mod state;
pub mod turn;
pub mod msg;

/// 게임 설정을 위한 빈 구조체입니다. 현재는 필드를 포함하지 않습니다.
pub struct GameConfig {}

/// 게임 액터는 게임 로직의 핵심을 담당합니다.
/// 플레이어 연결 관리, 게임 상태 관리, 턴 진행 등을 처리합니다.
// TODO: 상태 관리 개선
// TODO: 락 사용 최소화
pub struct GameActor {
    /// 플레이어 액터들의 주소를 저장합니다.
    pub players: HashMap<PlayerIdentity, Addr<PlayerActor>>,
    /// 플레이어의 ConnectionActor 주소를 저장합니다.
    pub connections: Arc<Mutex<HashMap<Uuid, Recipient<GameEvent>>>>,
    /// 각 플레이어 초기화 완료 여부를 나타냅니다.
    pub player_connection_ready: HashMap<PlayerKind, bool>,

    /// 상대방 플레이어의 연결 대기 타이머 핸들입니다.
    pub opponent_wait_timer_handle: Option<SpawnHandle>,
    /// 연결을 기다리는 상대방 플레이어의 종류입니다.
    pub opponent_player_kind: Option<PlayerKind>,

    /// 재접속 타이머 핸들입니다.
    pub reconnection_timer: HashMap<PlayerKind, SpawnHandle>,

    /// 게임의 현재 턴 상태를 관리합니다.
    pub turn: TurnState,

    /// 모든 카드 정보를 저장합니다. 플레이어 종류별로 카드를 관리합니다.
    pub all_cards: HashMap<PlayerKind, Vec<Card>>,
    /// 게임 상태를 관리하는 객체입니다.
    pub game_state: Arc<Mutex<GameStateManager>>,
    /// 게임의 고유 ID입니다.
    pub game_id: Uuid,

    /// gsm lock 에 실패한 경우 등에 사용되는 플래그입니다.
    pub unexpected_stop: bool,

    /// 동기화 액터의 주소를 저장합니다.
    pub sync_actor: Option<Addr<SyncActor>>,
}

impl Actor for GameActor {
    type Context = Context<Self>;

    /// 액터가 시작될 때 호출되는 메서드입니다.
    ///
    /// # Arguments
    ///
    /// * `ctx` - 액터 컨텍스트.
    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("GameActor [{}] has started. Initializing SyncActor.", self.game_id);
        
        let game_addr = ctx.address();

        let sync_actor_addr = SyncActor::create(|_| {
            SyncActor::new(game_addr)
        });

        // 3. 생성된 SyncActor의 주소를 GameActor의 필드에 저장합니다.
        self.sync_actor = Some(sync_actor_addr);

        let mut gsm = self.game_state.lock().now_or_never().unwrap(); // started는 동기적으로 실행되므로 now_or_never 가능
        gsm.initialize_players();
    }

    /// 액터가 정지될 때 호출되는 메서드입니다.
    ///
    /// # Arguments
    ///
    /// * `ctx` - 액터 컨텍스트.
    fn stopping(&mut self, ctx: &mut Context<Self>) -> Running {
        {
            // Start gsm Lock
            let mut gsm = self.game_state.try_lock().unwrap();
            let current_phase = gsm.current_phase();
            let connection_len = self.connections.try_lock().unwrap().len();
            if current_phase == GamePhase::Stopping && connection_len == 0 {
                info!(
                    "GameActor [{}]: Entering Stopped.",
                    self.game_id
                );
                gsm.transition_to_phase(GamePhase::Stopped);
                return Running::Stop;
            }else if current_phase == GamePhase::Stopping{
                info!(
                    "GameActor [{}]: stopping() called again, but cleanup is already in progress. Ignoring.",
                    self.game_id
                );
                // 이미 정리 작업이 진행 중이므로, 해당 작업이 끝날 때까지 액터를 살려둬야 합니다.
                return Running::Continue; 
            }
            
            gsm.transition_to_phase(GamePhase::Stopping);
        } // Drop gsm
            
        info!(
            "GameActor [{}] is stopping. Initiating comprehensive cleanup.",
            self.game_id
        );

        // 1. GameStateManager 정리
        let game_state = self.game_state.clone();
        let connections = self.connections.clone();
        let player_addrs: Vec<Addr<PlayerActor>> = self.players.values().cloned().collect();
        let game_id_clone = self.game_id;

        let cleanup_future = async move {
            info!(
                "GameActor [{}]: Starting comprehensive cleanup task.",
                game_id_clone
            );

            // 1. PlayerActor들에게 GameOver 전송
            let mut send_futures = Vec::new();
            for player_addr in player_addrs {
                info!(
                    "GameActor [{}]: Preparing to send GameOver to PlayerActor ({:?}).",
                    game_id_clone, player_addr
                );
                let fut = player_addr.send(GameOver);
                send_futures.push(async move { 
                    match fut.await {
                        Ok(_) => {
                            info!("GameActor [{}]: Successfully sent GameOver to PlayerActor ({:?})", game_id_clone, player_addr);
                        },
                        Err(e) => {
                            warn!(
                                "GameActor [{}]: Failed to send GameOver to PlayerActor ({:?}): {:?}. Attempting Terminate.",
                                game_id_clone, player_addr, e
                            );
                            player_addr.do_send(Terminate);
                        }
                    }
                });
            }

            // 2. GameStateManager에서 모든 연결된 플레이어 제거
            {
                let mut gsm = game_state.lock().await;
                let connected_players: Vec<_> = gsm.player_states.keys().cloned().collect();
                for player_kind in connected_players {
                    gsm.update_player_connection_status(player_kind, false);
                }
                info!(
                    "GameActor [{}]: All players removed from GameStateManager.",
                    game_id_clone
                );
            }

            // 3. 모든 연결 정리
            {
                let mut connections_guard = connections.lock().await;
                connections_guard.clear();
                info!(
                    "GameActor [{}]: All connections cleared.",
                    game_id_clone
                );
            }

            join_all(send_futures).await;

            info!(
                "GameActor [{}]: Comprehensive cleanup task completed.",
                game_id_clone
            );
        };

       let stop_self_after_cleanup = fut::wrap_future(cleanup_future).then(
            move |_, _act: &mut GameActor, ctx_then: &mut Context<GameActor>| {
                info!(
                    "GameActor [{}]: All cleanup completed. Now stopping context.",
                    _act.game_id
                );
                ctx_then.address().do_send(msg::system::Terminate);
                
                fut::ready(())
            },
        );

        ctx.spawn(stop_self_after_cleanup);

        info!("GameActor [{}]: stopping() method finished, comprehensive cleanup scheduled.", self.game_id);
        Running::Continue
    }

    /// 액터가 완전히 정지된 후 호출되는 메서드입니다.
    ///
    /// # Arguments
    ///
    /// * `ctx` - 액터 컨텍스트.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("GameActor [{}] has stopped.", self.game_id);
    }
}

impl GameActor {
    /// 새로운 게임 세션을 위한 GameActor를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `game_id` - 이 게임 세션의 고유 ID.
    /// * `player1_id` - 플레이어 1의 ID.
    /// * `player2_id` - 플레이어 2의 ID.
    /// * `player1_deck_code` - 플레이어 1의 덱 코드.
    /// * `player2_deck_code` - 플레이어 2의 덱 코드.
    /// * `attacker_player_type` - 선공 플레이어의 타입.
    ///
    /// # Returns
    ///
    /// 새로운 GameActor 인스턴스.
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use crate::game::GameActor;
    /// use crate::card::types::PlayerKind;
    ///
    /// let game_id = Uuid::new_v4();
    /// let player1_id = Uuid::new_v4();
    /// let player2_id = Uuid::new_v4();
    /// let player1_deck_code = "test_deck_1".to_string();
    /// let player2_deck_code = "test_deck_2".to_string();
    /// let attacker_player_type = PlayerKind::Player1;
    ///
    /// let game_actor = GameActor::new(
    ///     game_id,
    ///     player1_id,
    ///     player2_id,
    ///     player1_deck_code,
    ///     player2_deck_code,
    ///     attacker_player_type,
    /// );
    ///
    /// assert_eq!(game_actor.game_id, game_id);
    /// ```
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

            info!("Both players actor connected. ( not session connection! ) Sending SetOpponent messages");
        });

        let game_state_manager = GameStateManager::new();

        Self {
            players: player_actors_map,
            connections: Arc::new(Mutex::new(HashMap::new())),
            player_connection_ready: HashMap::from([
                (PlayerKind::Player1, false),
                (PlayerKind::Player2, false),
            ]),
            opponent_wait_timer_handle: None,
            opponent_player_kind: None,
            reconnection_timer: HashMap::new(),
            turn: TurnState::new(attacker_player_type),
            all_cards: HashMap::new(),
            game_state: Arc::new(Mutex::new(game_state_manager)),
            game_id,
            unexpected_stop: false,
            sync_actor: None,
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

    pub async fn create_snapshot_for(
        &self,
        perspective_of: PlayerKind,
    ) -> Result<GameStateSnapshot, GameError> {
        let my_player_addr = self.get_player_addr_by_kind(perspective_of);
        let opponent_player_addr = self.get_player_addr_by_kind(perspective_of.reverse());

        // 두 플레이어의 상태를 병렬로 가져옵니다.
        let (my_state_res, opponent_state_res) = tokio::join!(
            my_player_addr.send(GetPlayerStateSnapshot),
            opponent_player_addr.send(GetPlayerStateSnapshot)
        );

        let my_state = my_state_res??;
        let opponent_state = opponent_state_res??;

        // 상대방 정보는 공개된 정보로 변환합니다.
        let opponent_info_for_me = OpponentStateSnapshot {
            player_kind: opponent_state.player_kind,
            health: opponent_state.health,
            mana: opponent_state.mana,
            mana_max: opponent_state.mana_max,
            deck_count: opponent_state.deck_count,
            hand_count: opponent_state.hand.len(), // 손패는 개수만
            field: opponent_state.field,
            graveyard: opponent_state.graveyard,
        };

        // TODO: 현재 시퀀스 번호와 해시를 SyncActor로부터 가져오거나 GameActor가 직접 관리
        let current_seq = 0; // self.sync_actor.send(GetCurrentSeq).await?
        let current_hash = None; // self.calculate_hash()

        let snapshot = GameStateSnapshot {
            seq: current_seq,
            state_hash: current_hash,
            current_phase: self.turn.current_phase.to_string(), // Phase를 문자열로
            turn_player: self.turn.current_turn_plyaer,
            turn_count: self.turn.turn_count,
            my_info: my_state,
            opponent_info: opponent_info_for_me,
        };
        
        Ok(snapshot)
    }

    fn get_player_info_by_kind(&self, target_kind: PlayerKind) -> Option<(Uuid, &PlayerIdentity)> {
        for (identity, _) in &self.players {
            if identity.kind == target_kind {
                return Some((identity.id, identity));
            }
        }
        None
    }

    fn get_player_identity_by_kind(&self, target_kind: PlayerKind) -> Option<&PlayerIdentity> {
        for (identity, _) in &self.players {
            if identity.kind == target_kind {
                return Some(identity);
            }
        }
        None
    }

    fn get_player_identity_by_uuid(&self, player_id: Uuid) -> Option<&PlayerIdentity> {
        for (identity, _) in &self.players {
            if identity.id == player_id {
                return Some(identity);
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
                    Err(GameError::System(SystemError::Mailbox(mailbox_error)))
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