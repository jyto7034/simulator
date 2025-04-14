use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{card::types::PlayerType, exception::GameError, game::phase::Phase};

#[derive(Clone, Debug)]
struct SessionInfo {
    endpoint: Phase,
    session_id: Uuid,
}
struct PlayerSessionManagerInner {
    sessions: RwLock<HashMap<PlayerType, SessionInfo>>,
}

#[derive(Clone)]
pub struct PlayerSessionManager {
    inner: Arc<PlayerSessionManagerInner>,
}

impl PlayerSessionManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(PlayerSessionManagerInner {
                sessions: RwLock::new(HashMap::new()),
                // heartbeat_timeout 제거
            }),
        }
    }

    /// 새 세션을 등록하거나, 이미 존재하면 오류를 반환합니다.
    ///
    /// # Returns
    /// * `Ok(Uuid)`: 새로 생성된 세션 ID
    /// * `Err(GameError::ActiveSessionExists)`: 해당 플레이어가 이미 활성 세션을 가지고 있음
    pub async fn register_session(
        &self,
        player: PlayerType,
        endpoint: Phase,
    ) -> Result<Uuid, GameError> {
        let mut sessions = self.inner.sessions.write().await;

        // 플레이어가 이미 세션을 가지고 있는지 확인
        if sessions.contains_key(&player) {
            // 이미 세션이 존재하면 등록 거부 (더 명확한 오류 처리)
            warn!(
                "Session registration failed: Player {:?} already has an active session.",
                player
            );
            // 기존 세션 정보를 반환할 수도 있지만, 여기서는 오류로 처리
            // if let Some(existing_session) = sessions.get(&player) {
            //     // 기존 세션 정보 로깅 등
            // }
            return Err(GameError::ActiveSessionExists);
        }

        // 새 세션 생성 (last_heartbeat 설정 불필요)
        let session_id = Uuid::new_v4();
        sessions.insert(
            player,
            SessionInfo {
                // last_heartbeat 제거
                endpoint,
                session_id,
            },
        );
        info!(
            "New session registered: player={:?}, endpoint={:?}, session_id={}",
            player, endpoint, session_id
        );
        Ok(session_id)
    }

    // update_heartbeat 메소드 완전 삭제
    // pub async fn update_heartbeat(&self, player: PlayerType, session_id: Uuid) -> bool { ... }

    /// 주어진 플레이어, 세션 ID, 엔드포인트 조합이 유효하게 등록된 세션인지 확인합니다.
    pub async fn is_valid_session(
        &self,
        player: PlayerType,
        session_id: Uuid,
        endpoint: Phase,
    ) -> bool {
        let sessions = self.inner.sessions.read().await; // 읽기 락 사용

        sessions.get(&player).map_or(false, |session| {
            session.session_id == session_id && session.endpoint == endpoint
        })
        // map_or 사용으로 더 간결하게 표현
        // if let Some(session) = sessions.get(&player) {
        //     session.session_id == session_id && session.endpoint == endpoint
        // } else {
        //     false
        // }
    }

    /// 특정 플레이어와 세션 ID에 해당하는 세션 정보를 제거합니다.
    /// 해당 세션이 존재하지 않거나 ID가 일치하지 않으면 아무 작업도 하지 않습니다.
    pub async fn end_session<T: Into<PlayerType> + Copy>(&self, player: T, session_id: Uuid) {
        let player_type = player.into();
        info!(
            "Attempting to end session: player={:?}, session_id={}",
            player_type, session_id
        );

        // 쓰기 락을 한 번만 획득하도록 수정
        let mut sessions = self.inner.sessions.write().await;
        let mut should_remove = false;

        // 세션 존재 여부 및 ID 일치 확인
        if let Some(session) = sessions.get(&player_type) {
            if session.session_id == session_id {
                debug!(
                    "Session found and matches ID for removal: player={:?}, session_id={}",
                    player_type, session.session_id
                );
                should_remove = true;
            } else {
                warn!(
                    "Session found for player {:?}, but session ID mismatch (expected {}, found {}). Session not removed.",
                    player_type, session_id, session.session_id
                );
            }
        } else {
            debug!(
                "No active session found for player {:?}. Nothing to remove.",
                player_type
            );
        }

        // 조건 충족 시 제거
        if should_remove {
            if sessions.remove(&player_type).is_some() {
                info!(
                    "Session ended successfully: player={:?}, session_id={}",
                    player_type, session_id
                );
            } else {
                // 이론적으로 should_remove가 true면 이 경우는 발생하지 않음
                error!(
                    "Failed to remove session for player {:?} despite check.",
                    player_type
                );
            }
        }
    }
}
