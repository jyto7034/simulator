use std::time::{Duration, Instant};

use actix::{fut::wrap_future, Actor, ActorContext, AsyncContext, Context, Running, StreamHandler};
use actix_ws::{Message, ProtocolError, Session};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    card::types::PlayerType,
    enums::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
    server::session::PlayerSessionManager,
};

pub struct HeartbeatActor {
    last_pong: Instant,
    player_type: PlayerType,
    session_id: Uuid,
    session_manager: PlayerSessionManager,
    ws_session: Session,
    cleanup_started: bool, // 중복 정리를 방지하기 위한 플래그
}

impl HeartbeatActor {
    pub fn new(
        player_type: PlayerType,
        session_id: Uuid,
        session_manager: PlayerSessionManager,
        ws_session: Session,
    ) -> Self {
        Self {
            last_pong: Instant::now(),
            player_type,
            session_id,
            session_manager,
            ws_session,
            cleanup_started: false, // 초기값은 false
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

impl Actor for HeartbeatActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "HeartbeatActor started for player {:?} (session_id: {})",
            self.player_type, self.session_id
        );
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
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!(
            "HeartbeatActor stopping for player {:?} (session_id: {})",
            self.player_type, self.session_id
        );
        self.start_cleanup_task();
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "HeartbeatActor stopped for player {:?} (session_id: {})",
            self.player_type, self.session_id
        );
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for HeartbeatActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Pong(_)) => {
                self.last_pong = Instant::now();
                debug!(
                    "Received Pong from player {:?} (session_id: {})",
                    self.player_type, self.session_id
                );
            }
            Ok(Message::Ping(ping_msg)) => {
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
            Ok(Message::Text(text)) => {
                warn!(
                   "Received unexpected Text message on heartbeat from player {:?} (session_id: {}): {}",
                   self.player_type, self.session_id, text
               );
            }
            Ok(Message::Binary(_)) => {
                warn!(
                    "Received unexpected Binary message on heartbeat from player {:?} (session_id: {})",
                    self.player_type, self.session_id
                );
            }
            Ok(Message::Close(reason)) => {
                info!(
                    "Received Close message from player {:?} (session_id: {}). Reason: {:?}",
                    self.player_type, self.session_id, reason
                );
                ctx.stop(); // stopping에서 정리
            }
            Err(e) => {
                error!(
                    "WebSocket error for player {:?} (session_id: {}): {:?}",
                    self.player_type, self.session_id, e
                );
                ctx.stop(); // stopping에서 정리
            }
            _ => (),
        }
    }
}
