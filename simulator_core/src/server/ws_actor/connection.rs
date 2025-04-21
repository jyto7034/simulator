use std::time::{Duration, Instant};

use actix::{
    fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Context, Handler, Running,
    StreamHandler,
};
use actix_ws::{Message, ProtocolError, Session};
use messages::{GameEvent, HandleUserAction, RegisterConnection, UserAction};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    card::types::PlayerType,
    enums::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
    game::GameActor,
    server::session::PlayerSessionManager,
};

/// WebSocket 연결을 관리하는 Actor
pub struct ConnectionActor {
    session: Session,           // 웹소켓 세션 제어
    game_addr: Addr<GameActor>, // 연결된 GameActor 주소
    player_type: PlayerType,

    last_pong: Instant,
    session_id: Uuid, // 이 연결의 플레이어 ID
    session_manager: PlayerSessionManager,
    ws_session: Session,
    cleanup_started: bool, // 중복 정리를 방지하기 위한 플래그
}

impl ConnectionActor {
    // HeartbeatActor와 동일한 Session을 공유하거나,
    // 핸들러에서 Session 클론을 받을 수 있음.
    pub fn new(session: Session, game_addr: Addr<GameActor>, session_id: Uuid) -> Self {
        Self {
            session,
            game_addr,
            player_type: todo!(),
            last_pong: todo!(),
            session_id: todo!(),
            session_manager: todo!(),
            ws_session: todo!(),
            cleanup_started: todo!(),
        }
    }

    fn start_heartbeat_check(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::from_secs(HEARTBEAT_INTERVAL), |act, ctx_inner| {
            // 타임아웃 확인
            if Instant::now().duration_since(act.last_pong) > Duration::from_secs(CLIENT_TIMEOUT) {
                warn!(
                    "Heartbeat timeout for player {:?} (session_id: {}). Closing connection.",
                    act.player_type, act.session_id
                );
                ctx_inner.stop();
                return;
            }

            // 1. Ping 작업을 Context::spawn을 사용하여 비동기로 실행
            debug!(
                "Spawning heartbeat ping task for player {:?} (session_id: {})",
                act.player_type, act.session_id
            );

            let mut session_clone = act.ws_session.clone();
            let player_type_log = act.player_type;
            let session_id_log = act.session_id;

            // 2. 비동기 블록을 직접 spawn
            ctx_inner.spawn(wrap_future::<_, Self>(async move {
                if let Err(e) = session_clone.ping(b"heartbeat").await {
                    error!(
                        "Failed to send ping to player {:?} (session_id: {}): {:?}",
                        player_type_log, session_id_log, e
                    );
                } else {
                    debug!(
                        "Ping sent successfully to player {:?} (session_id: {})",
                        player_type_log, session_id_log
                    );
                }
            }));
        });
    }

    fn start_cleanup_task(&mut self) {
        if self.cleanup_started {
            return;
        }
        self.cleanup_started = true;

        let manager_clone = self.session_manager.clone();
        let player_clone = self.player_type;
        let sid_clone = self.session_id;

        tokio::spawn(async move {
            manager_clone.end_session(player_clone, sid_clone).await;
            info!(
                "Session cleanup task completed: player={:?}, session_id={}",
                player_clone, sid_clone
            );
        });
    }
}

impl Actor for ConnectionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("ConnectionActor started for player {}", self.session_id);

        self.start_heartbeat_check(ctx);

        // 1. 메시지 전송 Future 스폰
        let player_type_log = self.player_type;
        let session_id_log = self.session_id;
        let mut session_clone = self.ws_session.clone();
        let message_to_send = serde_json::json!({
            "type": "heartbeat_connected",
            "player": self.player_type.to_string(),
            "session_id": self.session_id.to_string(),
        })
        .to_string();

        // 2. 비동기 작업을 정의하는 Future 생성
        let send_future = async move {
            if let Err(e) = session_clone.text(message_to_send).await {
                error!(
                    "Failed to send initial heartbeat_connected message to player {:?} (session_id: {}): {:?}",
                    player_type_log, session_id_log, e
                );
            } else {
                debug!(
                    "Sent initial heartbeat_connected message to player {:?} (session_id: {})",
                    player_type_log, session_id_log
                );
            }
        };

        // 3. 표준 Future를 ActorFuture로 감싸서 액터 컨텍스트에서 실행
        ctx.spawn(wrap_future::<_, Self>(send_future));

        // GameActor에게 자신을 등록 (Context<Self>의 address() 사용)
        self.game_addr.do_send(RegisterConnection {
            session_id: self.session_id,
            addr: ctx.address(),
        });
    }

    fn stopping(&mut self, _ctx: &mut Context<Self>) -> Running {
        // TODO: 필요시 GameActor에게 연결 종료 알림
        // self.game_addr.do_send(ClientDisconnected { session_id: self.session_id });
        info!(
            "ConnectionActor stopping for player {:?} (session_id: {})",
            self.player_type, self.session_id
        );
        self.start_cleanup_task();

        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "ConnectionActor stopped for player {:?} (session_id: {})",
            self.player_type, self.session_id
        );
    }
}

// 웹소켓 메시지 처리 (StreamHandler 구현)
impl StreamHandler<Result<Message, ProtocolError>> for ConnectionActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Context<Self>) {
        match msg {
            // Ping/Pong/Close는 HeartbeatActor가 주도적으로 처리하므로 여기서는 무시하거나 로깅만 수행
            Ok(Message::Ping(ping_msg)) => {
                debug!(
                    "ConnectionActor received Ping (handled by HeartbeatActor?) for {}",
                    self.session_id
                );
                self.last_pong = Instant::now();

                let player_type_log = self.player_type;
                let session_id_log = self.session_id;
                let mut session_clone = self.ws_session.clone();

                let send_future = async move {
                    if let Err(e) = session_clone.pong(&ping_msg).await {
                        error!(
                            "Failed to send initial heartbeat_connected message to player {:?} (session_id: {}): {:?}",
                            player_type_log, session_id_log, e
                        );
                    } else {
                        debug!(
                            "Sent initial heartbeat_connected message to player {:?} (session_id: {})",
                            player_type_log, session_id_log
                        );
                    }
                };

                ctx.spawn(wrap_future::<_, Self>(send_future));

                debug!(
                    "Received Ping from player {:?} (session_id: {})",
                    self.player_type, self.session_id
                );
            }
            Ok(Message::Pong(_)) => {
                self.last_pong = Instant::now();
                debug!(
                    "Received Pong from player {:?} (session_id: {})",
                    self.player_type, self.session_id
                );
            }
            Ok(Message::Close(reason)) => {
                info!(
                    "ConnectionActor received Close (handled by HeartbeatActor?) for {}: {:?}",
                    self.session_id, reason
                );
                ctx.stop(); // 액터 중지 (HeartbeatActor도 같이 중지될 것임)
            }

            // Text 메시지만 처리
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<UserAction>(&text.to_string()) {
                    Ok(user_action) => {
                        debug!(
                            "ConnectionActor forwarding action from {}: {:?}",
                            self.session_id, user_action
                        );
                        // GameActor에게 메시지 전달
                        self.game_addr.do_send(HandleUserAction {
                            session_id: self.session_id,
                            action: user_action,
                        });
                    }
                    Err(e) => {
                        error!(
                            "ConnectionActor failed to parse UserAction from {}: {}, error: {}",
                            self.session_id, text, e
                        );
                        let error_msg = format!("{{\"error\": \"Invalid message format: {}\"}}", e);
                        // 에러 메시지 전송 시도 (비동기 처리 필요)
                        let mut session_clone = self.session.clone();
                        let session_id_log = self.session_id;
                        ctx.spawn(wrap_future::<_, Self>(async move {
                            if let Err(e) = session_clone.text(error_msg).await {
                                error!(
                                    "ConnectionActor failed to send error text to {}: {:?}",
                                    session_id_log, e
                                );
                            }
                        }));
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                warn!(
                    "ConnectionActor received unexpected Binary message from {}",
                    self.session_id
                );
                // 필요한 경우 처리
            }
            Err(e) => {
                error!(
                    "ConnectionActor websocket error for player {}: {}",
                    self.session_id, e
                );
                ctx.stop(); // 에러 발생 시 액터 중지
            }
            _ => (),
        }
    }

    // 스트림 종료 시 호출됨
    fn finished(&mut self, ctx: &mut Context<Self>) {
        info!(
            "ConnectionActor websocket Stream finished for player {}, stopping actor.",
            self.session_id
        );
        ctx.stop();
    }
}

// GameActor로부터 오는 GameEvent 처리
impl Handler<GameEvent> for ConnectionActor {
    type Result = ();

    fn handle(&mut self, msg: GameEvent, ctx: &mut Context<Self>) {
        match serde_json::to_string(&msg) {
            Ok(json_string) => {
                debug!(
                    "ConnectionActor sending event to client {}: {}",
                    self.session_id, json_string
                );
                // 세션을 통해 text 메시지 전송 (비동기 처리)
                let mut session_clone = self.session.clone();
                let session_id_log = self.session_id;
                ctx.spawn(wrap_future::<_, Self>(async move {
                    if let Err(e) = session_clone.text(json_string).await {
                        error!(
                            "ConnectionActor failed to send text event to {}: {:?}",
                            session_id_log, e
                        );
                        // 실패 시 액터 중지 등의 추가 처리 고려
                    }
                }));
            }
            Err(e) => {
                error!(
                    "ConnectionActor failed to serialize GameEvent for player {}: {}",
                    self.session_id, e
                );
            }
        }
    }
}

mod messages {
    use crate::game::GameActor;

    use super::ConnectionActor;
    use actix::prelude::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Message, Serialize, Deserialize, Debug, Clone)]
    #[rtype(result = "()")]
    pub enum GameEvent {
        GameStateUpdate(GameStateSnapshot),
        RequestPlayerInput(PlayerInputRequest),
        GameOver { winner: Uuid },
    }

    #[derive(Message, Deserialize, Debug, Clone)]
    #[rtype(result = "()")]
    pub struct HandleUserAction {
        pub session_id: Uuid,
        pub action: UserAction,
    }

    impl Handler<HandleUserAction> for GameActor {
        type Result = ();

        fn handle(&mut self, msg: HandleUserAction, ctx: &mut Self::Context) -> Self::Result {
            todo!()
        }
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(tag = "action")]
    pub enum UserAction {
        #[serde(rename = "playCard")]
        PlayCard {
            card_id: Uuid,
            target_id: Option<Uuid>,
        },
        #[serde(rename = "attack")]
        Attack {
            attacker_id: Uuid,
            defender_id: Uuid,
        },
        #[serde(rename = "endTurn")]
        EndTurn,
        #[serde(rename = "submitInput")]
        SubmitInput {
            request_id: Uuid,
            #[serde(flatten)]
            response_data: PlayerInputResponseData,
        },
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct RegisterConnection {
        pub session_id: Uuid,
        pub addr: Addr<ConnectionActor>,
    }

    impl Handler<RegisterConnection> for GameActor {
        type Result = ();

        fn handle(&mut self, msg: RegisterConnection, _: &mut Self::Context) {
            todo!()
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GameStateSnapshot {
        pub current_phase: String, /* ... */
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct PlayerInputRequest {
        pub request_id: Uuid,
        pub input_type: PlayerInputType,
        pub options: Vec<String>,
        pub message: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum PlayerInputType {
        SelectCardFromHand,
        SelectTargetOnField,
        ChooseEffect,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum PlayerInputResponseData {
        CardSelection(Vec<Uuid>),
        TargetSelection(Uuid),
        EffectChoice(String),
    }
}
