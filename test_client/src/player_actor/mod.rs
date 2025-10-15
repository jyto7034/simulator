use actix::{Actor, Addr, AsyncContext, Context, ContextFutureSpawner, WrapFuture};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

use crate::{
    behaviors::PlayerBehavior,
    observer_actor::ObserverActor,
    player_actor::message::{ConnectionEstablished, SetState},
    BehaviorOutcome, WsSink, WsStream, CONNECTION_TIMEOUT,
};

pub mod handler;
pub mod message;

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    Idle,
    Enqueued,
    Matched,
    Disconnected,
    Error(String),
}

pub struct PlayerActor {
    pub observer: Addr<ObserverActor>,
    pub state: PlayerState,
    pub behaviors: Box<dyn PlayerBehavior>,
    pub player_id: Uuid,
    pub test_session_id: String,
    pub stream: Option<WsStream>,
    pub sink: Option<WsSink>,
    pub connection_closed: bool,
}
impl Actor for PlayerActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let behavior = self.behaviors.clone_trait();
        let addr = ctx.address();
        let player_id = self.player_id;

        let player_context = PlayerContext {
            player_id,
            pod_id: None,
            addr: addr.clone(),
            test_session_id: self.test_session_id.clone(),
        };

        async move {
            // behavior에게 연결 시도 여부를 물어봄
            match behavior.try_connect(&player_context).await {
                BehaviorOutcome::Continue => {
                    // 연결 진행
                    match Self::establish_connection().await {
                        Ok((sink, stream)) => {
                            addr.do_send(ConnectionEstablished { sink, stream });
                        }
                        Err(_) => {
                            error!("Player [{}] failed to connect", player_id);
                            addr.do_send(SetState(PlayerState::Disconnected));
                        }
                    }
                }
                BehaviorOutcome::Complete => {
                    // behavior가 연결하지 않고 종료
                    info!("Player [{}] completed without connecting", player_id);
                    addr.do_send(SetState(PlayerState::Disconnected));
                }
                BehaviorOutcome::Error(err) => {
                    error!("Player [{}] try_connect error: {}", player_id, err);
                    addr.do_send(SetState(PlayerState::Error(err)));
                }
                BehaviorOutcome::IntendError => {
                    info!("Player [{}] intentionally failed at try_connect", player_id);
                    addr.do_send(SetState(PlayerState::Disconnected));
                }
            }
        }
        .into_actor(self)
        .wait(ctx);
    }
}

pub struct PlayerContext {
    pub player_id: Uuid,
    pub pod_id: Option<String>,
    pub addr: Addr<PlayerActor>,
    pub test_session_id: String,
}

impl PlayerActor {
    pub fn new(
        observer: Addr<ObserverActor>,
        behavior: Box<dyn PlayerBehavior>,
        player_id: Uuid,
        test_session_id: String,
    ) -> Self {
        Self {
            observer,
            state: PlayerState::Idle,
            behaviors: behavior,
            player_id,
            test_session_id,
            stream: None,
            sink: None,
            connection_closed: false,
        }
    }

    // behavior/try_connect 에서 사용.
    async fn establish_connection() -> Result<(WsSink, WsStream), ()> {
        let url = Url::parse(&crate::server_ws_url()).map_err(|e| error!("Invalid URL: {}", e))?;

        let (ws_stream, _) = tokio::time::timeout(CONNECTION_TIMEOUT, connect_async(url.as_str()))
            .await
            .map_err(|_| error!("Connection timeout"))?
            .map_err(|e| error!("Connection failed: {}", e))?;

        let (sink, stream) = ws_stream.split();

        Ok((sink, stream))
    }
}
