use std::{net::IpAddr, time::Duration};

use crate::{
    matchmaker::Matchmaker, protocol::ClientMessage, session::SessionState,
    subscript::SubScriptionManager, AppState, GameMode,
};
use actix::ActorContext;
use actix::{Actor, Addr, StreamHandler};
use actix_web::web;
use actix_web_actors::ws::{self, Message, ProtocolError};
use uuid::Uuid;

type Ctx = ws::WebsocketContext<Session>;

pub struct Session {
    state: SessionState,
    matchmaker_addr: Addr<Matchmaker>,
    subscript_addr: Addr<SubScriptionManager>,
    app_state: web::Data<AppState>,
}

impl Session {
    pub fn new(
        matchmaker_addr: Addr<Matchmaker>,
        subscript_addr: Addr<SubScriptionManager>,
        heartbeat_interval_seconds: Duration,
        client_timeout_seconds: Duration,
        app_state: web::Data<AppState>,
        client_ip: IpAddr,
    ) -> Self {
        todo!()
    }
}

impl Actor for Session {
    type Context = Ctx;
}

impl Session {
    fn handle_enqueue(&self, ctx: &mut Ctx, player_id: Uuid, game_mode: GameMode) {}

    fn handle_loading_complete(&self, ctx: &mut Ctx, loading_session_id: Uuid) {}
}

impl StreamHandler<Result<Message, ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Enqueue {
                    player_id,
                    game_mode,
                }) => {
                    self.handle_enqueue(ctx, player_id, game_mode);
                }
                Ok(ClientMessage::LoadingComplete { loading_session_id }) => {
                    self.handle_loading_complete(ctx, loading_session_id);
                }
                Err(e) => {}
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
