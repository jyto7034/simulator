use anyhow::Result;
use std::time::Duration;
use test_client::swarm::config::SwarmConfig;
use tracing::{info, warn};

fn setup_logger() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,test_client=debug")
        .try_init();
}

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

#[actix_web::test]
async fn test_swarm_with_template() -> Result<()> {
    // Initialize environment and logging
    env::init()?;
    setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    info!("🧪 Starting template-based swarm test");

    // Create config that uses template
    let config = SwarmConfig {
        duration_secs: 45,
        shards: 1,
        players_per_shard: 15,
        game_mode: Some("Normal_1v1".to_string()),
        match_server_base: Some("ws://127.0.0.1:8080".to_string()),
        seed: Some(99999),
        behavior_mix: None, // Will use template instead
        template_path: Some("test_client/configs/swarm_template.toml".to_string()),
        result_path: Some("logs/swarm_template_test_results.json".to_string()),
    };

    info!("Using template-based config: {:?}", config);

    // Run the swarm test
    let result = tokio::time::timeout(
        Duration::from_secs(config.duration_secs as u64 + 30),
        test_client::swarm::run_swarm(config),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Template-based swarm test completed successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("❌ Template-based swarm test failed: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("❌ Template-based swarm test timed out");
            Err(anyhow::anyhow!("Template-based swarm test timed out"))
        }
    }
}
