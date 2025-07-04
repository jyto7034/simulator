use std::env;

pub async fn connect_to_redis_with_auth() -> redis::RedisResult<()> {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set, e.g., redis://:password@host:port");

    let client = redis::Client::open(redis_url)?;
    let _ = client.get_async_connection().await?;

    Ok(())
}
