use std::{collections::HashMap, sync::Arc, time::Instant};
use tracing::{debug, info};
use uuid::Uuid;

use crate::{card::types::PlayerType, game::phase::Phase};

#[derive(Clone)]
struct SessionInfo {
    endpoint: Phase,
    last_heartbeat: Instant,
    session_id: Uuid, // 세션 식별자
}

// 내부 상태를 Arc로 래핑하여 공유
struct PlayerSessionManagerInner {
    sessions: tokio::sync::RwLock<HashMap<PlayerType, SessionInfo>>,
    heartbeat_timeout: u64,
}

#[derive(Clone)]
pub struct PlayerSessionManager {
    inner: Arc<PlayerSessionManagerInner>,
}

impl PlayerSessionManager {
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            inner: Arc::new(PlayerSessionManagerInner {
                sessions: tokio::sync::RwLock::new(HashMap::new()),
                heartbeat_timeout: timeout_seconds,
            }),
        }
    }

    // 세션 등록/갱신
    pub async fn register_session(&self, player: PlayerType, endpoint: Phase) -> Uuid {
        let mut sessions = self.inner.sessions.write().await;

        // 기존 세션 확인
        if let Some(session) = sessions.get_mut(&player) {
            if session.endpoint == endpoint {
                // 같은 엔드포인트면 하트비트 갱신
                session.last_heartbeat = Instant::now();
                return session.session_id;
            } else if session.last_heartbeat.elapsed().as_secs() < self.inner.heartbeat_timeout {
                // 다른 엔드포인트에 활성 세션 있음
                return session.session_id; // 기존 세션 ID 반환
            }
            // 타임아웃된 세션은 새로 덮어씀
        }

        // 새 세션 생성
        let session_id = Uuid::new_v4();
        sessions.insert(
            player,
            SessionInfo {
                endpoint: endpoint,
                last_heartbeat: Instant::now(),
                session_id,
            },
        );

        session_id
    }

    // 하트비트 업데이트
    pub async fn update_heartbeat(&self, player: PlayerType, session_id: Uuid) -> bool {
        let mut sessions = self.inner.sessions.write().await;

        if let Some(session) = sessions.get_mut(&player) {
            if session.session_id == session_id {
                session.last_heartbeat = Instant::now();
                return true;
            }
        }
        false
    }

    // 세션 확인 (유효한 세션이면 true)
    pub async fn is_valid_session(
        &self,
        player: PlayerType,
        session_id: Uuid,
        endpoint: Phase,
    ) -> bool {
        let sessions = self.inner.sessions.read().await;

        if let Some(session) = sessions.get(&player) {
            return session.session_id == session_id && session.endpoint == endpoint;
        }

        false
    }

    // 세션 종료
    pub async fn end_session<T: Into<PlayerType> + Copy>(&self, player: T, session_id: Uuid) {
        info!("세션 종료 시도: player={:?}", player.into());
        let player_type = player.into();

        let should_remove = {
            debug!("세션 읽기 락 획득 시도");
            let sessions = self.inner.sessions.write().await;
            if let Some(session) = sessions.get(&player_type) {
                debug!(
                    "세션 발견: player={:?}, session_id={}",
                    player_type, session.session_id
                );
                session.session_id == session_id
            } else {
                debug!("세션 없음: player={:?}", player_type);
                false
            }
        };

        if should_remove {
            let mut sessions = self.inner.sessions.write().await;
            debug!("세션 삭제 시도: player={:?}", player.into());
            sessions.remove(&player_type);
        }
        info!("세션 종료 완료: player={:?}", player.into());
    }
}
