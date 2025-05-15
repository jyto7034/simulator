use std::time::{Duration, Instant};

use actix::{
    fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Context, Handler, Running,
    StreamHandler,
};
use actix_ws::{CloseCode, CloseReason, Message, ProtocolError, Session};
use messages::GameEvent;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    card::types::PlayerKind,
    enums::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
    game::{
        message::{HandleUserAction, RegisterConnection},
        GameActor,
    },
    server::actor::ServerMessage,
};

use super::{messages, UserAction};

/// WebSocket 연결을 관리하는 Actor
pub struct ConnectionActor {
    pub ws_session: Session,        // 웹소켓 세션 제어
    pub game_addr: Addr<GameActor>, // 연결된 GameActor 주소
    pub player_type: PlayerKind,

    pub last_pong: Instant,
    pub player_id: Uuid,       // 이 연결의 플레이어 ID
    pub cleanup_started: bool, // 중복 정리를 방지하기 위한 플래그
    pub initial_pong_received: bool,
}

impl ConnectionActor {
    /// ConnectionActor의 새 인스턴스를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `session` - 이 액터가 관리할 웹소켓 세션 객체.
    /// * `game_addr` - 이 플레이어가 참여하는 게임을 관리하는 GameActor의 주소.
    /// * `session_id` - 이 연결에 해당하는 플레이어의 고유 식별자.
    /// * `session_manager` - 세션 타임아웃 및 정리를 관리하는 PlayerSessionManager.
    ///
    /// # Returns
    ///
    /// 새로운 ConnectionActor 인스턴스를 반환합니다.
    pub fn new(
        session: Session,
        game_addr: Addr<GameActor>,
        player_id: Uuid,
        player_type: PlayerKind,
    ) -> Self {
        Self {
            ws_session: session,
            game_addr,
            player_id,
            last_pong: Instant::now(),
            player_type,
            cleanup_started: false,
            initial_pong_received: false,
        }
    }

    fn start_heartbeat_check(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::from_secs(HEARTBEAT_INTERVAL), |act, ctx_inner| {
            if Instant::now().duration_since(act.last_pong) > Duration::from_secs(CLIENT_TIMEOUT) {
                warn!(
                    "Heartbeat timeout for player {:?} (session_id: {}). Closing connection.",
                    act.player_type, act.player_id
                );
                let session_to_close = act.ws_session.clone();
                ctx_inner.spawn(wrap_future::<_, Self>(async move {
                    let _ = session_to_close
                        .close(Some(CloseReason::from(CloseCode::Policy)))
                        .await;
                }));
                ctx_inner.stop();
                return;
            }

            // 1. Ping 작업을 Context::spawn을 사용하여 비동기로 실행
            info!(
                "Spawning heartbeat ping task for player {:?} (session_id: {})",
                act.player_type, act.player_id
            );

            let mut session_clone = act.ws_session.clone();
            let player_type_log = act.player_type;
            let session_id_log = act.player_id;
            let last_pong = act.last_pong;

            // 2. 비동기 블록을 직접 spawn
            ctx_inner.spawn(wrap_future::<_, Self>(async move {
                if let Err(e) = session_clone.ping(b"heartbeat").await {
                    error!(
                        "Failed to send ping to player {:?} (session_id: {}): {:?}",
                        player_type_log, session_id_log, e
                    );
                } else {
                    info!(
                        "Ping sent successfully to player {:?} (session_id: {}) last_pong {:?}",
                        player_type_log, session_id_log, last_pong
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

        let player_clone = self.player_type;
        let sid_clone = self.player_id;
        let ws_session_clone = self.ws_session.clone();

        tokio::spawn(async move {
            ws_session_clone.close(None).await.unwrap_or_else(|e| {
                error!(
                    "Failed to close WebSocket session for player {:?} (session_id: {}): {:?}",
                    player_clone, sid_clone, e
                );
            });
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
        info!("ConnectionActor started for player {}", self.player_id);

        self.start_heartbeat_check(ctx);

        let player_type_log = self.player_type;
        let session_id_log = self.player_id;
        let mut session_clone = self.ws_session.clone();
        let init_msg = ServerMessage::HeartbeatConnected {
            player: self.player_type.to_string(),
            session_id: self.player_id,
        }
        .to_json();

        // 비동기 작업을 정의하는 Future 생성
        let send_future = async move {
            if let Err(e) = session_clone.text(init_msg).await {
                error!(
                    "Failed to send initial heartbeat_connected message to player {:?} (session_id: {}): {:?}",
                    player_type_log, session_id_log, e
                );
            } else {
                info!(
                    "Sent initial heartbeat_connected message to player {:?} (session_id: {})",
                    player_type_log, session_id_log
                );
            }
        };

        // 표준 Future를 ActorFuture로 감싸서 액터 컨텍스트에서 실행
        ctx.spawn(wrap_future::<_, Self>(send_future));
    }

    fn stopping(&mut self, _ctx: &mut Context<Self>) -> Running {
        // TODO: 필요시 GameActor에게 연결 종료 알림
        // self.game_addr.do_send(ClientDisconnected { session_id: self.session_id });
        info!(
            "ConnectionActor stopping for player {:?} (session_id: {})",
            self.player_type, self.player_id
        );
        self.start_cleanup_task();

        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "ConnectionActor stopped for player {:?} (session_id: {})",
            self.player_type, self.player_id
        );
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for ConnectionActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Context<Self>) {
        debug!(
            "ConnectionActor received message from player {:?} (session_id: {}): {:?}",
            self.player_type, self.player_id, msg
        );
        match msg {
            Ok(Message::Ping(ping_msg)) => {
                info!(
                    "ConnectionActor for player {:?} (session_id: {}) received Ping from client.",
                    self.player_type, self.player_id
                );

                let player_type_log = self.player_type;
                let session_id_log = self.player_id;
                let mut session_clone = self.ws_session.clone();

                let send_future = async move {
                    if let Err(e) = session_clone.pong(&ping_msg).await {
                        error!(
                            "ConnectionActor for player {:?} (session_id: {}): Failed to send Pong to client: {:?}",
                            player_type_log, session_id_log, e
                        );
                    } else {
                        info!(
                            "ConnectionActor for player {:?} (session_id: {}): Sent Pong to client.",
                            player_type_log, session_id_log
                        );
                    }
                };
                ctx.spawn(wrap_future::<_, Self>(send_future));
            }
            Ok(Message::Pong(_)) => {
                let prev_pong = self.last_pong;
                self.last_pong = Instant::now(); // 클라이언트 활성 시간 갱신
                info!(
                    "ConnectionActor for player {:?} (session_id: {}): Received Pong from client. prev_pong {:?}, last_pong {:?}",
                    self.player_type, self.player_id, prev_pong, self.last_pong
                );

                if !self.initial_pong_received {
                    self.initial_pong_received = true;
                    info!(
                        "ConnectionActor for player {:?} (session_id: {}): Initial Pong received. Registering with GameActor.",
                        self.player_type, self.player_id
                    );

                    // GameActor에게 자신을 등록
                    self.game_addr.do_send(RegisterConnection {
                        player_id: self.player_id,
                        addr: ctx.address(), // 현재 ConnectionActor의 주소
                    });
                }
            }
            Ok(Message::Close(reason)) => {
                info!(
                    "ConnectionActor for player {:?} (session_id: {}): Received Close from client. Reason: {:?}",
                    self.player_type, self.player_id, reason
                );
                ctx.stop();
            }
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<UserAction>(&text.to_string()) {
                    Ok(user_action) => {
                        info!(
                            "ConnectionActor for player {:?} (session_id: {}): Forwarding action to GameActor: {:?}",
                            self.player_type, self.player_id, user_action
                        );
                        self.game_addr.do_send(HandleUserAction {
                            player_id: self.player_id,
                            action: user_action,
                        });
                    }
                    Err(e) => {
                        error!(
                            "ConnectionActor for player {:?} (session_id: {}): Failed to parse UserAction from text '{}'. Error: {}",
                            self.player_type, self.player_id, text, e
                        );
                        let error_msg = format!("{{\"error\": \"Invalid message format: {}\"}}", e);
                        let mut session_clone = self.ws_session.clone();
                        let player_id_log = self.player_id; // 로그용 ID 클론
                        ctx.spawn(wrap_future::<_, Self>(async move {
                            if let Err(send_err) = session_clone.text(error_msg).await {
                                error!(
                                    "ConnectionActor for player_id {}: Failed to send error text to client: {:?}",
                                    player_id_log, send_err
                                );
                            }
                        }));
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                warn!(
                    "ConnectionActor for player {:?} (session_id: {}): Received unexpected Binary message.",
                    self.player_type, self.player_id
                );
            }
            Err(e) => {
                error!(
                    "ConnectionActor for player {:?} (session_id: {}): WebSocket error: {}",
                    self.player_type, self.player_id, e
                );
                ctx.stop();
            }
            _ => {
                // 예를 들어 Ok(Message::Continuation(_)) 등 명시적으로 처리하지 않은 메시지 타입
                warn!(
                    "ConnectionActor for player {:?} (session_id: {}): Received unhandled message type.",
                    self.player_type, self.player_id
                );
            }
        }
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        info!(
            "ConnectionActor for player {:?} (session_id: {}): WebSocket stream finished. Stopping actor.",
            self.player_type, self.player_id
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
                info!(
                    "ConnectionActor sending event to client {}: {}",
                    self.player_id, json_string
                );
                // 세션을 통해 text 메시지 전송 (비동기 처리)
                let mut session_clone = self.ws_session.clone();
                let session_id_log = self.player_id;
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
                    self.player_id, e
                );
            }
        }
    }
}
