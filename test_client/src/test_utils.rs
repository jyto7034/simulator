use anyhow::Result;

pub async fn flush_redis_default() -> Result<()> {
    // Read redis config from env (host/port/db)
    let cfg = env::SimulatorConfig::global();
    let r = &cfg.database.redis;
    let url = if let Some(pw) = &r.password {
        format!("redis://:{}@{}:{}/{}", pw, r.host, r.port, r.db)
    } else {
        format!("redis://{}:{}/{}", r.host, r.port, r.db)
    };

    let client = redis::Client::open(url)?;
    let mut conn = client.get_async_connection().await?;

    // FLUSHDB for the selected DB only (safer than FLUSHALL)
    redis::cmd("FLUSHDB")
        .query_async::<_, ()>(&mut conn)
        .await?;
    Ok(())
}
