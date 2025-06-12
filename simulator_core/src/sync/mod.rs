pub mod messages;
pub mod snapshots;
pub mod types;

use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, Recipient};
use tracing::{info, warn};

use crate::{
    card::types::PlayerKind,
    game::{msg::GameEvent, GameActor},
    sync::{
        messages::{
            NotifyChanges, RegisterConnectionToSync, RequestStateHashSync,
            UnregisterConnectionFromSync,
        },
        types::StateUpdatePayload,
    },
};

pub struct SyncActor {
    game_addr: Addr<GameActor>,
    connections: HashMap<PlayerKind, Recipient<GameEvent>>,
    event_sequence: u64,
}

impl SyncActor {
    /// SyncActor의 새 인스턴스를 생성합니다.
    pub fn new(game_addr: Addr<GameActor>) -> Self {
        info!("SyncActor created.");
        Self {
            game_addr,
            connections: HashMap::new(),
            event_sequence: 0,
        }
    }

    /// 모든 연결된 클라이언트에게 페이로드를 브로드캐스트합니다.
    async fn broadcast(&self, payload: StateUpdatePayload) {
        let event = GameEvent::StateUpdate(payload); // GameEvent enum 래핑
        for (player, recipient) in &self.connections {
            // TODO: Retry 로직 적용해야함.
            if let Err(e) = recipient.send(event.clone()).await {
                // Clone을 통해 각 클라이언트에 메시지 전송
                warn!("Failed to send state update to player {:?}: {}", player, e);
            }
        }
    }
}

impl Actor for SyncActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("SyncActor started and is ready to handle sync events.");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("SyncActor stopped.");
    }
}

/// GameActor로부터 상태 변경 알림을 처리하는 핸들러
impl Handler<NotifyChanges> for SyncActor {
    type Result = ();

    fn handle(&mut self, msg: NotifyChanges, _ctx: &mut Context<Self>) {
        if msg.0.is_empty() {
            return;
        }

        self.event_sequence += 1;
        info!(
            "Handling NotifyChanges with seq: {}. Delta count: {}",
            self.event_sequence,
            msg.0.len()
        );

        let payload = StateUpdatePayload {
            seq: self.event_sequence,
            changes: msg.0,
            state_hash: None, // 일반 업데이트에는 해시를 포함하지 않음
        };

        self.broadcast(payload);
    }
}

/// 주기적인 해시 동기화 요청을 처리하는 핸들러
impl Handler<RequestStateHashSync> for SyncActor {
    type Result = ();

    fn handle(&mut self, _msg: RequestStateHashSync, _ctx: &mut Context<Self>) {
        info!("Handling RequestStateHashSync. Fetching hash from GameActor.");
        // GameActor에게 현재 상태의 해시를 요청합니다.
        // GameActor는 GetStateHash 메시지에 대한 핸들러가 필요합니다.
        // let future = self.game_addr.send(GetStateHash).into_actor(self).then(|res, act, _| {
        //     match res {
        //         Ok(hash) => {
        //             act.event_sequence += 1;
        //             let payload = StateUpdatePayload {
        //                 seq: act.event_sequence,
        //                 changes: vec![], // 해시 동기화에는 델타가 없을 수 있음
        //                 state_hash: Some(hash),
        //             };
        //             act.broadcast(payload);
        //         }
        //         Err(e) => {
        //             error!("Failed to get state hash from GameActor: {}", e);
        //         }
        //     }
        //     fut::ready(())
        // });
        // ctx.spawn(future);

        todo!()
    }
}

/// 새로운 클라이언트 연결을 등록하는 핸들러
impl Handler<RegisterConnectionToSync> for SyncActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterConnectionToSync, _ctx: &mut Context<Self>) {
        info!("Registering connection for player {:?}.", msg.player);
        self.connections.insert(msg.player, msg.recipient);
    }
}

/// 클라이언트 연결 해제를 처리하는 핸들러
impl Handler<UnregisterConnectionFromSync> for SyncActor {
    type Result = ();

    fn handle(&mut self, msg: UnregisterConnectionFromSync, _ctx: &mut Context<Self>) {
        info!("Unregistering connection for player {:?}.", msg.player);
        self.connections.remove(&msg.player);
    }
}
