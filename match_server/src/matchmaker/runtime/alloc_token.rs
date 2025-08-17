use actix::Addr;
use redis::aio::ConnectionManager;
use tracing::info;
use uuid::Uuid;

use crate::matchmaker::messages::HandleLoadingComplete;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenOutcome {
    Won,
    Busy,
}

pub async fn try_acquire(
    redis: &mut ConnectionManager,
    session_id: Uuid,
    ttl_seconds: u64,
) -> Result<TokenOutcome, redis::RedisError> {
    let alloc_key = format!("alloc:{}", session_id);
    let token_res: Option<String> = redis::cmd("SET")
        .arg(&alloc_key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(ttl_seconds)
        .query_async(redis)
        .await?;
    Ok(if token_res.is_some() {
        TokenOutcome::Won
    } else {
        TokenOutcome::Busy
    })
}

pub fn schedule_watchdog(
    addr: Addr<crate::matchmaker::actor::Matchmaker>,
    player_id: Uuid,
    session_id: Uuid,
    delay_secs: u64,
) {
    info!(
        "Allocation already being handled for session {}, scheduling watchdog.",
        session_id
    );
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
        addr.do_send(HandleLoadingComplete {
            player_id,
            loading_session_id: session_id,
        });
    });
}
