use actix::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

use crate::blacklist::messages::RecordViolation;
use crate::blacklist::ViolationType;
use crate::loading_session::events::*;
use crate::state_events::StateEventEmitter;
use redis::aio::ConnectionManager;
use crate::Matchmaker;

#[derive(Debug, Clone)]
pub enum PlayerLoadingState {
    Loading { started_at: Instant },
    Ready { completed_at: Instant },
    TimedOut { timeout_at: Instant },
}

/// 개별 Loading Session을 관리하는 Actor
pub struct LoadingSessionActor {
    pub session_id: Uuid,
    pub game_mode: String,
    pub players: HashMap<Uuid, PlayerLoadingState>,
    pub timeout_duration: Duration,
    pub created_at: Instant,
    
    // Dependencies
    pub blacklist_manager_addr: Addr<crate::BlacklistManager>,
    pub matchmaker_addr: Addr<Matchmaker>,
    pub redis_conn: ConnectionManager,
}

impl LoadingSessionActor {
    pub fn new(
        session_id: Uuid,
        player_ids: Vec<Uuid>,
        game_mode: String,
        timeout_seconds: u64,
        blacklist_manager_addr: Addr<crate::BlacklistManager>,
        matchmaker_addr: Addr<Matchmaker>,
        redis_conn: ConnectionManager,
    ) -> Self {
        let players = player_ids
            .into_iter()
            .map(|id| (id, PlayerLoadingState::Loading { 
                started_at: Instant::now() 
            }))
            .collect();

        Self {
            session_id,
            game_mode,
            players,
            timeout_duration: Duration::from_secs(timeout_seconds),
            created_at: Instant::now(),
            blacklist_manager_addr,
            matchmaker_addr,
            redis_conn,
        }
    }

    fn check_session_completion(&mut self, ctx: &mut Context<Self>) {
        let all_ready = self.players.values().all(|state| {
            matches!(state, PlayerLoadingState::Ready { .. })
        });

        if all_ready {
            info!("Loading session {} completed - all players ready", self.session_id);
            
            // Emit session completed event
            let player_ids: Vec<Uuid> = self.players.keys().cloned().collect();
            let _completed_at = Instant::now();
            
            // Publish state event
            actix::spawn({
                let mut redis_conn = self.redis_conn.clone();
                let session_id = self.session_id;
                async move {
                    let mut emitter = StateEventEmitter::new(&mut redis_conn);
                    if let Err(e) = emitter
                        .loading_session_completed(session_id.to_string(), player_ids.iter().map(|id| id.to_string()).collect())
                        .await
                    {
                        warn!("Failed to publish loading_session_completed event: {}", e);
                    }
                }
            });

            // Stop this actor - session is complete
            ctx.stop();
        }
    }

    fn schedule_player_timeout(&self, player_id: Uuid, ctx: &mut Context<Self>) {
        let timeout_delay = self.timeout_duration;
        
        ctx.run_later(timeout_delay, move |act, ctx| {
            // Check if player is still in loading state
            if let Some(PlayerLoadingState::Loading { .. }) = act.players.get(&player_id) {
                act.handle_player_timeout(player_id, ctx);
            }
        });
    }

    fn handle_player_timeout(&mut self, player_id: Uuid, ctx: &mut Context<Self>) {
        warn!("Player {} timed out in session {}", player_id, self.session_id);
        
        // Update player state
        if let Some(state) = self.players.get_mut(&player_id) {
            *state = PlayerLoadingState::TimedOut { timeout_at: Instant::now() };
        }

        // Record violation for blacklist
        self.blacklist_manager_addr.do_send(RecordViolation {
            player_id,
            violation_type: ViolationType::Timeout,
            ip_addr: None,
        });

        // Emit timeout event  
        actix::spawn({
            let mut redis_conn = self.redis_conn.clone();
            let session_id = self.session_id;
            async move {
                let mut emitter = StateEventEmitter::new(&mut redis_conn);
                if let Err(e) = emitter
                    .loading_session_timeout(session_id.to_string(), vec![player_id.to_string()])
                    .await
                {
                    warn!("Failed to publish loading_session_timeout event: {}", e);
                }
            }
        });

        // Check if we need to requeue players
        self.check_requeue_needed(ctx);
    }

    fn check_requeue_needed(&mut self, ctx: &mut Context<Self>) {
        let has_ready_players = self.players.values().any(|state| {
            matches!(state, PlayerLoadingState::Ready { .. })
        });
        
        let has_timeout_players = self.players.values().any(|state| {
            matches!(state, PlayerLoadingState::TimedOut { .. })
        });

        // If we have mix of ready and timed-out players, requeue all
        if has_ready_players && has_timeout_players {
            info!("Session {} has mixed states, requeuing all players", self.session_id);
            self.requeue_all_players(ctx);
        }
        
        // If all remaining players timed out, also requeue
        let all_finished = self.players.values().all(|state| {
            !matches!(state, PlayerLoadingState::Loading { .. })
        });
        
        if all_finished && has_timeout_players {
            info!("All players in session {} finished/timed out, requeuing", self.session_id);
            self.requeue_all_players(ctx);
        }
    }

    fn requeue_all_players(&mut self, ctx: &mut Context<Self>) {
        // Requeue ONLY players who were ready
        let players_to_requeue: Vec<Uuid> = self.players.iter()
            .filter_map(|(id, state)| {
                if matches!(state, PlayerLoadingState::Ready { .. }) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        if players_to_requeue.is_empty() {
            info!("No players were ready in session {}, no one to requeue.", self.session_id);
        } else {
            info!("Requeuing ready players {:?} from session {}", players_to_requeue, self.session_id);
            // Send requeue message to matchmaker
            self.matchmaker_addr.do_send(crate::matchmaker::messages::DelayedRequeuePlayers {
                player_ids: players_to_requeue.iter().map(|id| id.to_string()).collect(),
                game_mode: self.game_mode.clone(),
                delay: Duration::from_secs(5),
            });
        }

        // Emit session canceled event
        actix::spawn({
            let mut redis_conn = self.redis_conn.clone();
            let session_id = self.session_id;
            let players_to_requeue_clone = players_to_requeue.clone();
            async move {
                let mut emitter = StateEventEmitter::new(&mut redis_conn);
                if let Err(e) = emitter
                    .loading_session_canceled(
                        session_id.to_string(), 
                        "Mixed timeout/ready states".to_string(),
                        players_to_requeue_clone.iter().map(|id| id.to_string()).collect()
                    )
                    .await
                {
                    warn!("Failed to publish loading_session_canceled event: {}", e);
                }
            }
        });

        // Stop this actor
        ctx.stop();
    }
}

impl Actor for LoadingSessionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "Loading session {} started with {} players, timeout: {}s",
            self.session_id,
            self.players.len(),
            self.timeout_duration.as_secs()
        );

        // Schedule timeout check for each player
        for player_id in self.players.keys().cloned() {
            self.schedule_player_timeout(player_id, ctx);
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Loading session {} stopped", self.session_id);
    }
}

impl Handler<PlayerLoadingComplete> for LoadingSessionActor {
    type Result = Result<(), anyhow::Error>;

    fn handle(&mut self, msg: PlayerLoadingComplete, ctx: &mut Self::Context) -> Self::Result {
        let PlayerLoadingComplete { player_id, session_id } = msg;

        // Verify this is the correct session
        if session_id != self.session_id {
            warn!(
                "Received loading complete for wrong session: expected {}, got {}",
                self.session_id, session_id
            );
            return Ok(());
        }

        // Update player state
        if let Some(state) = self.players.get_mut(&player_id) {
            match state {
                PlayerLoadingState::Loading { .. } => {
                    info!("Player {} completed loading in session {}", player_id, session_id);
                    *state = PlayerLoadingState::Ready { completed_at: Instant::now() };
                    
                    // Check if all players are now ready
                    self.check_session_completion(ctx);
                }
                PlayerLoadingState::Ready { .. } => {
                    warn!("Player {} already completed loading", player_id);
                }
                PlayerLoadingState::TimedOut { .. } => {
                    warn!("Player {} completed loading after timeout", player_id);
                }
            }
        } else {
            warn!("Received loading complete for unknown player: {}", player_id);
        }

        Ok(())
    }
}

impl Handler<CleanupSession> for LoadingSessionActor {
    type Result = ();

    fn handle(&mut self, msg: CleanupSession, ctx: &mut Self::Context) {
        if msg.session_id == self.session_id {
            info!("Cleaning up loading session {}", self.session_id);
            ctx.stop();
        }
    }
}