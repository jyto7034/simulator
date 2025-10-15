pub mod dequeue;
pub mod enqueue;
pub mod notify;
pub mod try_match;

use std::future::Future;
use tokio::time::{timeout, Duration};
use tracing::error;

pub async fn with_redis_timeout<F, T>(
    operation_name: &str,
    timeout_secs: u64,
    future: F,
) -> Result<T, String>
where
    F: Future<Output = Result<T, redis::RedisError>>,
{
    match timeout(Duration::from_secs(timeout_secs), future).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(format!("{} failed: {}", operation_name, err)),
        Err(_) => {
            error!(
                "{} timeout after {}s - Redis may be unresponsive",
                operation_name, timeout_secs
            );
            Err(format!(
                "{} timeout after {}s",
                operation_name, timeout_secs
            ))
        }
    }
}
