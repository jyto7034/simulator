pub mod messages;
pub mod snapshots;
pub mod types;

use std::collections::HashMap;

use actix::{dev::SendError, Actor, Addr, Context, Handler, Recipient};
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

/// `SyncActor`는 게임 상태 동기화를 담당합니다.
// TODO: 상태 동기화 로직 개선
// TODO: 최적화
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
    fn broadcast(&mut self, payload: StateUpdatePayload) {
        let event = GameEvent::StateUpdate(payload);

        let mut dead_recipients = Vec::new();

        for (player, recipient) in &self.connections {
            match recipient.try_send(event.clone()) {
                Ok(_) => {}
                Err(SendError::Full(_)) => {
                    warn!("Mailbox for player {:?} is full. Update might be delayed or dropped if it happens frequently.", player);
                }
                Err(SendError::Closed(_)) => {
                    warn!(
                        "Mailbox for player {:?} is closed. Marking for removal.",
                        player
                    );
                    dead_recipients.push(*player);
                }
            }
        }

        // 루프가 끝난 후 죽은 수신자들을 정리
        for player in dead_recipients {
            self.connections.remove(&player);
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
