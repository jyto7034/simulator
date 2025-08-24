use actix::prelude::*;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

use crate::loading_session::events::*;
use crate::loading_session::session_actor::LoadingSessionActor;
use redis::aio::ConnectionManager;
use crate::Matchmaker;

/// LoadingSession들을 관리하는 중앙 매니저
pub struct LoadingSessionManager {
    /// 활성화된 세션들
    active_sessions: HashMap<Uuid, Addr<LoadingSessionActor>>,
    
    // Dependencies
    blacklist_manager_addr: Addr<crate::BlacklistManager>,
    matchmaker_addr: Addr<Matchmaker>,
    redis_conn: ConnectionManager,
}

impl LoadingSessionManager {
    pub fn new(
        blacklist_manager_addr: Addr<crate::BlacklistManager>,
        matchmaker_addr: Addr<Matchmaker>,
        redis_conn: ConnectionManager,
    ) -> Self {
        Self {
            active_sessions: HashMap::new(),
            blacklist_manager_addr,
            matchmaker_addr,
            redis_conn,
        }
    }

    fn create_session_actor(&self, msg: &CreateLoadingSession) -> Addr<LoadingSessionActor> {
        LoadingSessionActor::create(|_ctx| {
            LoadingSessionActor::new(
                msg.session_id,
                msg.players.clone(),
                msg.game_mode.clone(),
                msg.timeout_seconds,
                self.blacklist_manager_addr.clone(),
                self.matchmaker_addr.clone(),
                self.redis_conn.clone(),
            )
        })
    }
}

impl Actor for LoadingSessionManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("LoadingSessionManager started");
    }
}

impl Handler<CreateLoadingSession> for LoadingSessionManager {
    type Result = Result<(), anyhow::Error>;

    fn handle(&mut self, msg: CreateLoadingSession, _ctx: &mut Self::Context) -> Self::Result {
        let session_id = msg.session_id;
        
        // Check if session already exists
        if self.active_sessions.contains_key(&session_id) {
            warn!("Loading session {} already exists", session_id);
            return Ok(());
        }

        info!(
            "Creating loading session {} with {} players for game mode '{}'",
            session_id,
            msg.players.len(),
            msg.game_mode
        );

        // Create new session actor
        let session_addr = self.create_session_actor(&msg);
        
        // Store reference
        self.active_sessions.insert(session_id, session_addr);

        Ok(())
    }
}

impl Handler<PlayerLoadingComplete> for LoadingSessionManager {
    type Result = Result<(), anyhow::Error>;

    fn handle(&mut self, msg: PlayerLoadingComplete, _ctx: &mut Self::Context) -> Self::Result {
        let session_id = msg.session_id;
        
        if let Some(session_addr) = self.active_sessions.get(&session_id) {
            // Forward to specific session
            session_addr.do_send(msg);
        } else {
            warn!(
                "Received loading complete for unknown session: {}",
                session_id
            );
        }

        Ok(())
    }
}

impl Handler<CleanupSession> for LoadingSessionManager {
    type Result = ();

    fn handle(&mut self, msg: CleanupSession, _ctx: &mut Self::Context) {
        let session_id = msg.session_id;
        
        if let Some(session_addr) = self.active_sessions.remove(&session_id) {
            info!("Removing session {} from manager", session_id);
            session_addr.do_send(msg);
        }
    }
}