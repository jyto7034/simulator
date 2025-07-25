use actix::{Actor, Addr, AsyncContext, Context};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

use crate::{
    behaviors::PlayerBehavior,
    observer_actor::ObserverActor,
    player_actor::message::{ConnectionEstablished, SetState},
    WsSink, WsStream, CONNECTION_TIMEOUT, DEFAULT_SERVER_URL,
};

pub mod handler;
pub mod message;

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
    pub observer: Addr<ObserverActor>,
    pub state: PlayerState,
    pub behavior: Box<dyn PlayerBehavior>,
    pub player_id: Uuid,
    pub stream: Option<WsStream>,
    pub sink: Option<WsSink>,
}

impl PlayerActor {
    pub fn new(
        observer: Addr<ObserverActor>,
        behavior: Box<dyn PlayerBehavior>,
        player_id: Uuid,
    ) -> Self {
        Self {
            observer,
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

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // TODO: 결과를 집계한 다음, 결과와 함께 종료 사실을 SingleScenarioActor 에게 전송.
        // PlayerCompleted Msg 사용하면 됨.
        // 근데 옵저버와 약간 기능이 겹치는 것 같은데 기능 관계를 한 번 정리해야할 듯.
    }
}

impl PlayerActor {
    async fn establish_connection() -> anyhow::Result<(WsSink, WsStream)> {
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
