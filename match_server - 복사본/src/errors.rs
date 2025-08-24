use thiserror::Error;

/// Unified error types for the matchmaking system
#[derive(Error, Debug)]
pub enum MatchmakerError {
    #[error("Redis operation failed: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("JSON serialization/deserialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Failed to acquire distributed lock for key: {key}")]
    LockAcquisition { key: String },
    
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Actor mailbox error: {0}")]
    Mailbox(#[from] actix::MailboxError),
    
    #[error("Invalid game mode: {mode}")]
    InvalidGameMode { mode: String },
    
    #[error("Player {player_id} already in queue for {game_mode}")]
    PlayerAlreadyInQueue { player_id: String, game_mode: String },
    
    #[error("Loading session {session_id} not found or expired")]
    LoadingSessionNotFound { session_id: String },
    
    #[error("Dedicated server allocation failed: {reason}")]
    DedicatedServerAllocation { reason: String },
    
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type alias for matchmaker operations
pub type MatchmakerResult<T> = Result<T, MatchmakerError>;

/// Extension trait for converting string errors to MatchmakerError
pub trait ToMatchmakerError<T> {
    fn to_matchmaker_error(self, message: &str) -> MatchmakerResult<T>;
}

impl<T, E: std::fmt::Display> ToMatchmakerError<T> for Result<T, E> {
    fn to_matchmaker_error(self, message: &str) -> MatchmakerResult<T> {
        self.map_err(|e| MatchmakerError::Internal {
            message: format!("{}: {}", message, e),
        })
    }
}

/// Helper functions for common error scenarios
impl MatchmakerError {
    pub fn lock_failed(key: impl Into<String>) -> Self {
        Self::LockAcquisition { key: key.into() }
    }
    
    pub fn invalid_game_mode(mode: impl Into<String>) -> Self {
        Self::InvalidGameMode { mode: mode.into() }
    }
    
    pub fn player_already_in_queue(player_id: impl Into<String>, game_mode: impl Into<String>) -> Self {
        Self::PlayerAlreadyInQueue {
            player_id: player_id.into(),
            game_mode: game_mode.into(),
        }
    }
    
    pub fn loading_session_not_found(session_id: impl Into<String>) -> Self {
        Self::LoadingSessionNotFound {
            session_id: session_id.into(),
        }
    }
    
    pub fn dedicated_allocation_failed(reason: impl Into<String>) -> Self {
        Self::DedicatedServerAllocation {
            reason: reason.into(),
        }
    }
    
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
    
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

/// Convert MatchmakerError to appropriate ServerMessage for client communication
impl From<MatchmakerError> for crate::protocol::ServerMessage {
    fn from(error: MatchmakerError) -> Self {
        use crate::protocol::{ErrorCode, ServerMessage};
        
        match error {
            MatchmakerError::InvalidGameMode { .. } => ServerMessage::Error {
                code: Some(ErrorCode::InvalidGameMode),
                message: error.to_string(),
            },
            MatchmakerError::PlayerAlreadyInQueue { .. } => ServerMessage::Error {
                code: Some(ErrorCode::AlreadyInQueue),
                message: error.to_string(),
            },
            MatchmakerError::LoadingSessionNotFound { .. } => ServerMessage::Error {
                code: Some(ErrorCode::WrongSessionId),
                message: error.to_string(),
            },
            _ => ServerMessage::Error {
                code: Some(ErrorCode::InternalError),
                message: "Internal server error".to_string(),
            },
        }
    }
}