use std::time::{Duration, Instant};

use actix::{
    fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Context, Running, SpawnHandle,
    StreamHandler,
};
use actix_ws::{Message, ProtocolError, Session};
use simulator_core::{
    card::types::PlayerKind, exception::{ConnectionError, GameError, SystemError}, game::{
        msg::connection::{RegisterConnection, UnRegisterConnection},
        GameActor,
    }, retry_with_condition, Condition, RetryConfig
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    connection::{
        messages::{PostRegistrationSetup, StopActorOnError},
        ServerMessage,
    },
    enums::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
};

use super::UserAction;

/// WebSocket 연결을 관리하는 Actor
pub struct ConnectionActor {
    pub ws_session: Session,        // 웹소켓 세션 제어
    pub game_addr: Addr<GameActor>, // 연결된 GameActor 주소
    pub player_type: PlayerKind,

    pub last_pong: Instant,
    pub player_id: Uuid,       // 이 연결의 플레이어 ID
    pub cleanup_started: bool, // 중복 정리를 방지하기 위한 플래그
    pub initial_pong_received: bool,
    pub heartbeat_handle: Option<SpawnHandle>,
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
            heartbeat_handle: None,
        }
    }

    pub fn send_ping(&self, ctx: &mut Context<Self>) {
        info!(
            "Spawning heartbeat ping task for player {:?} (session_id: {})",
            self.player_type, self.player_id
        );

        let mut session_clone = self.ws_session.clone();
        let player_type_log = self.player_type;
        let session_id_log = self.player_id;
        let connection_addr = ctx.address().clone(); // Corrected: use ctx.address()

        ctx.spawn(wrap_future::<_, Self>(async move {
            if let Err(e) = session_clone.ping(b"heartbeat").await {
                error!(
                    "Failed to send ping to player {:?} (session_id: {}): {:?}",
                    player_type_log, session_id_log, e
                );
                connection_addr.do_send(StopActorOnError { 
                    error: GameError::System(SystemError::Io(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e.to_string()))) 
                });
            } else {
                info!(
                    "Ping sent successfully to player {:?} (session_id: {})",
                    player_type_log, session_id_log
                );
            }
        }));
    }

    pub fn start_heartbeat_interval(&mut self, ctx: &mut Context<Self>) {
        if self.heartbeat_handle.is_some() {
            return;
        }

        let handle = ctx.run_interval(Duration::from_secs(HEARTBEAT_INTERVAL), |act, ctx_inner| {
            if Instant::now().duration_since(act.last_pong) > Duration::from_secs(CLIENT_TIMEOUT) {
                warn!(
                    "Heartbeat timeout for player {:?} (session_id: {}). Closing connection.",
                    act.player_type, act.player_id
                );
                ctx_inner.stop();
                return;
            }
            
            act.send_ping(ctx_inner);
        });

        self.heartbeat_handle = Some(handle);
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

    fn started(&mut self, _ctx: &mut Context<Self>) {
        let player_type_log = self.player_type;
        info!(
            "ConnectionActor started for player {} {}",
            player_type_log, self.player_id
        );
    }

    fn stopping(&mut self, ctx: &mut Context<Self>) -> Running {
        info!(
            "ConnectionActor stopping for player {:?} (session_id: {})",
            self.player_type, self.player_id
        );

        if let Some(handle) = self.heartbeat_handle.take() {
            ctx.cancel_future(handle);
            info!(
                "Heartbeat task cancelled for player {:?} (session_id: {})",
                self.player_type, self.player_id
            );
        } else {
            warn!(
                "No heartbeat task to cancel for player {:?} (session_id: {})",
                self.player_type, self.player_id
            );
        }

        self.game_addr.do_send(UnRegisterConnection {
            player_id: self.player_id,
        });

        self.start_cleanup_task();

        Running::Continue
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
                let player_type = self.player_type;
                let player_id = self.player_id;
                let game_addr = self.game_addr.clone();
                let addr = ctx.address().clone();

                // 1. 활성 시간 갱신
                self.last_pong = Instant::now();
                info!(
                    "ConnectionActor for player {:?} (session_id: {}): Received Pong from client",
                    self.player_type, self.player_id
                );

                // 2. 초기 Pong 수신 여부 확인
                if !self.initial_pong_received {
                    self.initial_pong_received = true;
                    info!(
                        "ConnectionActor for player {:?} (session_id: {}): Initial Pong received. Registering with GameActor.",
                        self.player_type, self.player_id
                    );

                    let connection_addr = ctx.address().clone();
                    let ws_session = self.ws_session.clone(); // ws_session도 클론

                    ctx.spawn(wrap_future::<_, Self>(async move {
                        // 3. GameActor에 연결 등록 메시지를 전송하는 비동기 함수 생성
                        let operation = || {
                            let game_addr_clone = game_addr.clone();
                            let connection_addr_clone = connection_addr.clone();
                            async move {
                                let register_connection_future = game_addr_clone
                                    .send(RegisterConnection {
                                        player_id,
                                        recipient: connection_addr_clone.recipient(),
                                    }).await;

                                match register_connection_future {
                                    Ok(handler_result) => {
                                        match handler_result {
                                            Ok(_) => {
                                                return Ok(());
                                            }
                                            Err(game_error) => {
                                                error!(
                                                    "ConnectionActor for player {:?} (session_id: {}): Message sent to GameActor, but registration failed with GameError: {:?}",
                                                    player_type, player_id, game_error
                                                );
                                                return Err(game_error);
                                            }
                                        }
                                    }
                                    Err(mailbox_error) => {
                                        error!(
                                        "ConnectionActor for player {:?} (session_id: {}): Failed to send RegisterConnection message to GameActor (MailboxError): {:?}",
                                        player_type, player_id, mailbox_error
                                    );
                                        return Err(GameError::System(SystemError::Mailbox(mailbox_error)));
                                    }
                                }
                            }
                        };

                        // 4. 생성된 비동기 함수를 retry 함수에 넘겨서 재시도 로직 적용
                        let condition = |e: &GameError| {
                            if let GameError::Connection(ConnectionError::SessionExists(_)) = e {
                                return Condition::Stop;
                            }
                            Condition::Continue
                        };
                        
                        if let Err(e) = retry_with_condition(operation, RetryConfig::default(), condition,"RegisterConnection").await{
                            error!(
                                "ConnectionActor for player {:?} (session_id: {}): Failed to register with GameActor after retries: {:?}",
                                player_type, player_id, e
                            );

                            // 클라이언트에게 에러 메시지 전송
                            let server_error_message = ServerMessage::from(e);
                            
                            // 2. ServerMessage를 JSON 문자열로 변환합니다.
                            //    (to_json 헬퍼 함수가 이 새로운 variant를 처리하도록 수정 필요)
                            let error_payload = server_error_message.to_json();
                            
                            let mut session_clone = ws_session.clone();
                            if let Err(send_err) = session_clone.text(error_payload).await {
                                warn!("Failed to send error message to client: {:?}", send_err);
                            }
                            addr.do_send(StopActorOnError {
                                error: GameError::System(SystemError::Internal(
                                    "Failed to register with GameActor after retries".to_string(),
                                )),
                            });
                        } else {
                            info!(
                                "ConnectionActor for player {:?} (session_id: {}): Successfully registered with GameActor.",
                                player_type, player_id
                            );

                            connection_addr.do_send(PostRegistrationSetup);
                        }
                    }));
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
                        // self.game_addr.do_send(HandleUserAction {
                        //     player_id: self.player_id,
                        //     action: user_action,
                        // });
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
