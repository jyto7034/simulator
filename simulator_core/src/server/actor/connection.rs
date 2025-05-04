use std::time::{Duration, Instant};

use actix::{
    fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Context, Handler, Running,
    StreamHandler,
};
use actix_ws::{CloseCode, CloseReason, Message, ProtocolError, Session};
use messages::{GameEvent, HandleUserAction, RegisterConnection};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    card::types::PlayerKind,
    enums::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
    game::GameActor,
    server::actor::ServerMessage,
};

use super::{messages, UserAction};

/// WebSocket 연결을 관리하는 Actor
pub struct ConnectionActor {
    ws_session: Session,        // 웹소켓 세션 제어
    game_addr: Addr<GameActor>, // 연결된 GameActor 주소
    player_type: PlayerKind,

    last_pong: Instant,
    player_id: Uuid,       // 이 연결의 플레이어 ID
    cleanup_started: bool, // 중복 정리를 방지하기 위한 플래그
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
        }
    }

    fn start_heartbeat_check(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::from_secs(HEARTBEAT_INTERVAL), |act, ctx_inner| {
            if Instant::now().duration_since(act.last_pong) > Duration::from_secs(CLIENT_TIMEOUT) {
                warn!(
                    "Heartbeat timeout for player {:?} (session_id: {}). Closing connection.",
                    act.player_type, act.player_id
                );
                ctx_inner.stop();
                return;
            }

            // 1. Ping 작업을 Context::spawn을 사용하여 비동기로 실행
            debug!(
                "Spawning heartbeat ping task for player {:?} (session_id: {})",
                act.player_type, act.player_id
            );

            let mut session_clone = act.ws_session.clone();
            let player_type_log = act.player_type;
            let session_id_log = act.player_id;

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
                debug!(
                    "Sent initial heartbeat_connected message to player {:?} (session_id: {})",
                    player_type_log, session_id_log
                );
            }
        };

        // 표준 Future를 ActorFuture로 감싸서 액터 컨텍스트에서 실행
        ctx.spawn(wrap_future::<_, Self>(send_future));

        // GameActor에게 자신을 등록 (Context<Self>의 address() 사용)
        self.game_addr.do_send(RegisterConnection {
            player_id: self.player_id,
            addr: ctx.address(),
        });
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

// 웹소켓 메시지 처리 (StreamHandler 구현)
impl StreamHandler<Result<Message, ProtocolError>> for ConnectionActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Context<Self>) {
        match msg {
            // Ping/Pong/Close는 HeartbeatActor가 주도적으로 처리하므로 여기서는 무시하거나 로깅만 수행
            Ok(Message::Ping(ping_msg)) => {
                debug!(
                    "ConnectionActor received Ping (handled by HeartbeatActor?) for {}",
                    self.player_id
                );
                self.last_pong = Instant::now();

                let player_type_log = self.player_type;
                let session_id_log = self.player_id;
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
                    self.player_type, self.player_id
                );
            }
            Ok(Message::Pong(_)) => {
                self.last_pong = Instant::now();
                debug!(
                    "Received Pong from player {:?} (session_id: {})",
                    self.player_type, self.player_id
                );
            }
            Ok(Message::Close(reason)) => {
                info!(
                    "ConnectionActor received Close (handled by HeartbeatActor?) for {}: {:?}",
                    self.player_id, reason
                );
                ctx.stop(); // 액터 중지 (HeartbeatActor도 같이 중지될 것임)
            }

            // Text 메시지만 처리
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<UserAction>(&text.to_string()) {
                    Ok(user_action) => {
                        debug!(
                            "ConnectionActor forwarding action from {}: {:?}",
                            self.player_id, user_action
                        );
                        // GameActor에게 메시지 전달
                        self.game_addr.do_send(HandleUserAction {
                            player_id: self.player_id,
                            action: user_action,
                        });
                    }
                    Err(e) => {
                        error!(
                            "ConnectionActor failed to parse UserAction from {}: {}, error: {}",
                            self.player_id, text, e
                        );
                        let error_msg = format!("{{\"error\": \"Invalid message format: {}\"}}", e);
                        // 에러 메시지 전송 시도 (비동기 처리 필요)
                        let mut session_clone = self.ws_session.clone();
                        let session_id_log = self.player_id;
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
                    self.player_id
                );
                // 필요한 경우 처리
            }
            Err(e) => {
                error!(
                    "ConnectionActor websocket error for player {}: {}",
                    self.player_id, e
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
            self.player_id
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
