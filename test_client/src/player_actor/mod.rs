use actix::{Actor, Addr, AsyncContext, Context};
use futures_util::StreamExt;
use std::{sync::Arc, time::Duration};
use tokio_tungstenite::connect_async;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

use crate::{
    behavior::PlayerBehavior,
    player_actor::message::{ConnectionEstablished, SetState},
    WsSink, WsStream,
};

pub mod handler;
pub mod message;

const DEFAULT_SERVER_URL: &str = "ws://127.0.0.1:8080/ws/";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PlayerState {
    Idle,
    Enqueued,
    Loading,
    Disconnected,
}

#[derive(Clone)]
pub struct PlayerContext {
    pub player_id: Uuid,
    pub addr: Addr<PlayerActor>,
}

// 플레이어
pub struct PlayerActor {
    pub state: PlayerState,
    pub behavior: Arc<dyn PlayerBehavior>,
    pub player_id: Uuid,
    pub stream: Option<WsStream>,
    pub sink: Option<WsSink>,
}

impl PlayerActor {
    pub fn new(behavior: Arc<dyn PlayerBehavior>, player_id: Uuid) -> Self {
        Self {
            state: PlayerState::Idle,
            behavior,
            player_id,
            stream: None,
            sink: None,
        }
    }
}

impl Actor for PlayerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("PlayerActor started with state");
        let addr = ctx.address();
        let player_id = self.player_id;

        actix::spawn(async move {
            match Self::establish_connection().await {
                Ok((sink, stream)) => {
                    addr.do_send(ConnectionEstablished { sink, stream });
                }
                Err(e) => {
                    error!("Player [{}] failed to connect: {}", player_id, e);
                    addr.do_send(SetState(PlayerState::Disconnected));
                }
            }
        });
    }
}

impl PlayerActor {
    pub async fn establish_connection() -> anyhow::Result<(WsSink, WsStream)> {
        let url =
            Url::parse(DEFAULT_SERVER_URL).map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;

        let (ws_stream, _) = tokio::time::timeout(CONNECTION_TIMEOUT, connect_async(url.as_str()))
            .await
            .map_err(|_| anyhow::anyhow!("Connection timeout"))?
            .map_err(|e| anyhow::anyhow!("Connection failed: {}", e))?;

        let (sink, stream) = ws_stream.split();
        Ok((sink, stream))
    }
}
