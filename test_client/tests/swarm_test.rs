use anyhow::Result;
use std::time::Duration;
use test_client::{setup_logger, swarm::config::SwarmConfig};
use tracing::{info, warn};

use test_client::test_utils::flush_redis_default;

#[actix_web::test]
async fn test_swarm_minimal() -> Result<()> {
    // Initialize environment and logging
    env::init()?;
    setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    info!("🧪 Starting basic swarm test");

    // Load basic swarm configuration
    let config_content = include_str!("../configs/swarm_minimal.toml");
    let config = SwarmConfig::from_toml_str(config_content)?;

    info!("Loaded swarm config: {:?}", config);

    // Run the swarm test with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(config.duration_secs as u64 + 30), // Add 30s buffer
        test_client::swarm::run_swarm(config),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Basic swarm test completed successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("❌ Swarm test failed: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("❌ Swarm test timed out");
            Err(anyhow::anyhow!("Swarm test timed out"))
        }
    }
}

#[actix_web::test]
async fn test_swarm_medium() -> Result<()> {
    // Initialize environment and logging
    env::init()?;
    setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    info!("🧪 Starting medium swarm test");

    // Load basic swarm configuration
    let config_content = include_str!("../configs/swarm_medium.toml");
    let config = SwarmConfig::from_toml_str(config_content)?;

    info!("Loaded swarm config: {:?}", config);

    // Run the swarm test with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(config.duration_secs as u64 + 30), // Add 30s buffer
        test_client::swarm::run_swarm(config),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Basic swarm test completed successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("❌ Swarm test failed: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("❌ Swarm test timed out");
            Err(anyhow::anyhow!("Swarm test timed out"))
        }
    }
}

#[actix_web::test]
async fn test_swarm_max() -> Result<()> {
    // Initialize environment and logging
    env::init()?;
    setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    info!("🧪 Starting basic swarm test");

    // Load basic swarm configuration
    let config_content = include_str!("../configs/swarm_max.toml");
    let config = SwarmConfig::from_toml_str(config_content)?;

    info!("Loaded swarm config: {:?}", config);

    // Run the swarm test with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(config.duration_secs as u64 + 30), // Add 30s buffer
        test_client::swarm::run_swarm(config),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Load swarm test completed successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("❌ Swarm test failed: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("❌ Swarm test timed out");
            Err(anyhow::anyhow!("Swarm test timed out"))
        }
    }
}

#[actix_web::test]
async fn test_swarm_ultra() -> Result<()> {
    // Initialize environment and logging
    env::init()?;
    setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    info!("🧪 Starting basic swarm test");

    // Load basic swarm configuration
    let config_content = include_str!("../configs/swarm_ultra.toml");
    let config = SwarmConfig::from_toml_str(config_content)?;

    info!("Loaded swarm config: {:?}", config);

    // Run the swarm test with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(config.duration_secs as u64 + 30), // Add 30s buffer
        test_client::swarm::run_swarm(config),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Load swarm test completed successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("❌ Swarm test failed: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("❌ Swarm test timed out");
            Err(anyhow::anyhow!("Swarm test timed out"))
        }
    }
}

#[actix_web::test]
async fn test_swarm_load() -> Result<()> {
    // Initialize environment and logging
    env::init()?;
    setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    info!("🧪 Starting load swarm test");

    // Load load test configuration
    let config_content = include_str!("../configs/swarm_load_test.toml");
    let config = SwarmConfig::from_toml_str(config_content)?;

    info!("Loaded load test config: {:?}", config);

    // Run the swarm test with longer timeout for load test
    let result = tokio::time::timeout(
        Duration::from_secs(config.duration_secs as u64 + 60), // Add 60s buffer for load test
        test_client::swarm::run_swarm(config),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Load swarm test completed successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("❌ Load swarm test failed: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("❌ Load swarm test timed out");
            Err(anyhow::anyhow!("Load swarm test timed out"))
        }
    }
}
