use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    StreamHandler, WrapFuture,
};
use actix_web::web;
use actix_web_actors::ws::{self, Message, ProtocolError};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    game::{
        load_balance_actor::messages::GetOrCreatePlayerActor,
        player_game_actor::{
            messages::{
                AttachSession, DetachSession, ExecutePlayerBehavior, ForceDisconnect,
                PlayerGameClientMessage, PlayerGameServerMessage, QuitPlayerActor,
            },
            state::PlayerGameActorError,
            PlayerGameActor,
        },
    },
    AppState,
};

type Ctx = ws::WebsocketContext<PlayerGameSession>;

pub struct PlayerGameSession {
    app_state: web::Data<AppState>,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
    last_heartbeat: Instant,
    session_id: Uuid,
    player_id: Option<Uuid>,
    player_actor: Option<Addr<PlayerGameActor>>,
}

impl PlayerGameSession {
    pub fn new(
        app_state: web::Data<AppState>,
        heartbeat_interval: Duration,
        heartbeat_timeout: Duration,
    ) -> Self {
        Self {
            app_state,
            heartbeat_interval,
            heartbeat_timeout,
            last_heartbeat: Instant::now(),
            session_id: Uuid::new_v4(),
            player_id: None,
            player_actor: None,
        }
    }

    fn hb(&self, ctx: &mut Ctx) {
        ctx.run_interval(self.heartbeat_interval, |act, ctx| {
            if act.last_heartbeat.elapsed() > act.heartbeat_timeout {
                warn!(
                    "PlayerGameSession heartbeat timed out for session {}",
                    act.session_id
                );
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    fn send_json(ctx: &mut Ctx, message: &PlayerGameServerMessage) {
        match serde_json::to_string(message) {
            Ok(json) => ctx.text(json),
            Err(error) => {
                warn!("Failed to serialize PlayerGameServerMessage: {}", error);
                ctx.stop();
            }
        }
    }

    fn send_error(
        &self,
        ctx: &mut Ctx,
        request_id: Option<String>,
        code: &'static str,
        message: impl Into<String>,
    ) {
        Self::send_json(
            ctx,
            &PlayerGameServerMessage::Error {
                request_id,
                code: code.to_string(),
                message: message.into(),
            },
        );
    }

    fn handle_auth(&mut self, ctx: &mut Ctx, player_id: Uuid, token: Option<String>) {
        if self.player_actor.is_some() {
            self.send_error(
                ctx,
                None,
                "already_authed",
                "Session is already authenticated",
            );
            return;
        }

        if let Err(error) = mock_authenticate(player_id, token.as_deref()) {
            Self::send_json(ctx, &error.to_server_message(None));
            ctx.close(Some(ws::CloseCode::Policy.into()));
            ctx.stop();
            return;
        }

        let app_state = self.app_state.clone();
        let session_id = self.session_id;
        let socket = ctx.address().recipient::<PlayerGameServerMessage>();
        let control = ctx.address().recipient::<ForceDisconnect>();

        async move {
            let actor = app_state
                .load_balance_addr
                .send(GetOrCreatePlayerActor {
                    player_id,
                    game_data: app_state.game_data.clone(),
                    run_seed: derive_run_seed(player_id, app_state.current_run_id),
                })
                .await
                .map_err(mailbox_error)?;

            let snapshot = actor
                .send(AttachSession {
                    session_id,
                    socket,
                    control,
                })
                .await
                .map_err(mailbox_error)??;

            Ok::<_, PlayerGameActorError>((actor, snapshot))
        }
        .into_actor(self)
        .map(move |result, act, ctx| match result {
            Ok((actor, snapshot)) => {
                act.player_id = Some(player_id);
                act.player_actor = Some(actor);
                Self::send_json(ctx, &PlayerGameServerMessage::Authed { player_id });
                Self::send_json(
                    ctx,
                    &PlayerGameServerMessage::StateSnapshot { state: snapshot },
                );
            }
            Err(error) => {
                Self::send_json(ctx, &error.to_server_message(None));
                ctx.close(Some(ws::CloseCode::Error.into()));
                ctx.stop();
            }
        })
        .spawn(ctx);
    }

    fn handle_command(
        &mut self,
        ctx: &mut Ctx,
        request_id: String,
        behavior: game_core::game::behavior::PlayerBehavior,
    ) {
        let Some(actor) = self.player_actor.clone() else {
            self.send_error(
                ctx,
                Some(request_id),
                "not_authenticated",
                "Authenticate before sending commands",
            );
            return;
        };

        let session_id = self.session_id;

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: request_id.clone(),
                behavior,
            })
            .into_actor(self)
            .map(move |result, _act, ctx| match result {
                Ok(Ok(result)) => {
                    Self::send_json(ctx, &result.response);
                    Self::send_json(
                        ctx,
                        &PlayerGameServerMessage::StateSnapshot {
                            state: result.state_snapshot,
                        },
                    );
                }
                Ok(Err(error)) => {
                    Self::send_json(ctx, &error.to_server_message(Some(request_id.clone())))
                }
                Err(error) => Self::send_json(
                    ctx,
                    &PlayerGameActorError::new("internal_error", error.to_string())
                        .to_server_message(Some(request_id.clone())),
                ),
            })
            .spawn(ctx);
    }
}

impl Actor for PlayerGameSession {
    type Context = Ctx;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("PlayerGameSession started: {}", self.session_id);
        self.hb(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(actor) = &self.player_actor {
            actor.do_send(DetachSession {
                session_id: self.session_id,
            });
        }
    }
}

impl Handler<PlayerGameServerMessage> for PlayerGameSession {
    type Result = ();

    fn handle(&mut self, msg: PlayerGameServerMessage, ctx: &mut Self::Context) -> Self::Result {
        Self::send_json(ctx, &msg);
    }
}

impl Handler<ForceDisconnect> for PlayerGameSession {
    type Result = ();

    fn handle(&mut self, msg: ForceDisconnect, ctx: &mut Self::Context) -> Self::Result {
        Self::send_json(
            ctx,
            &PlayerGameServerMessage::Error {
                request_id: None,
                code: msg.code,
                message: msg.message,
            },
        );
        ctx.close(Some(ws::CloseCode::Normal.into()));
        ctx.stop();
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for PlayerGameSession {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(bytes)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&bytes);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<PlayerGameClientMessage>(&text) {
                    Ok(PlayerGameClientMessage::Auth { player_id, token }) => {
                        self.handle_auth(ctx, player_id, token);
                    }
                    Ok(PlayerGameClientMessage::Command {
                        request_id,
                        behavior,
                    }) => {
                        self.handle_command(ctx, request_id, behavior.into());
                    }
                    Ok(PlayerGameClientMessage::Ping) => {
                        Self::send_json(ctx, &PlayerGameServerMessage::Pong);
                    }
                    Ok(PlayerGameClientMessage::Quit) => {
                        if let Some(actor) = self.player_actor.take() {
                            actor.do_send(QuitPlayerActor);
                        }
                        self.player_id = None;
                        ctx.close(Some(ws::CloseCode::Normal.into()));
                        ctx.stop();
                    }
                    Err(error) => {
                        self.send_error(ctx, None, "invalid_message_format", error.to_string());
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Binary(_)) => {
                self.send_error(
                    ctx,
                    None,
                    "unsupported_message",
                    "Binary frames are not supported",
                );
            }
            Ok(ws::Message::Continuation(_)) => {
                self.send_error(
                    ctx,
                    None,
                    "unsupported_message",
                    "Continuation frames are not supported",
                );
            }
            Ok(ws::Message::Nop) => {}
            Err(error) => {
                warn!("WebSocket protocol error: {}", error);
                ctx.stop();
            }
        }
    }
}

fn mock_authenticate(player_id: Uuid, _token: Option<&str>) -> Result<(), PlayerGameActorError> {
    if player_id.is_nil() {
        return Err(PlayerGameActorError::new(
            "auth_failed",
            "player_id must not be nil",
        ));
    }
    Ok(())
}

fn derive_run_seed(player_id: Uuid, current_run_id: Uuid) -> u64 {
    let mut hasher = DefaultHasher::new();
    player_id.hash(&mut hasher);
    current_run_id.hash(&mut hasher);
    hasher.finish()
}

fn mailbox_error(error: actix::MailboxError) -> PlayerGameActorError {
    PlayerGameActorError::new("internal_error", error.to_string())
}
