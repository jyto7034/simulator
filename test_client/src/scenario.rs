use anyhow::Result;
use simulator_env::env;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use uuid::Uuid;

use crate::behavior::{BehaviorType, NormalPlayer};
use crate::observer::{EventObserver, ObservationResult};
use crate::setup_logger;

pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub players: Vec<(Uuid, BehaviorType)>,
    pub observer: EventObserver,
}

impl TestScenario {
    pub fn new(name: String, description: String, match_server_url: String) -> Self {
        let _guard = setup_logger("test");
        Self {
            name: name.clone(),
            description,
            players: Vec::new(),
            observer: EventObserver::new(match_server_url, name),
        }
    }

    pub fn add_player(&mut self, player_id: Uuid, behavior: BehaviorType) {
        self.players.push((player_id, behavior));
    }

    pub fn setup_normal_match_test() -> Self {
        let mut scenario = Self::new(
            "normal_match_test".to_string(),
            "Test normal 2-player matching flow".to_string(),
            env::match_server_ws_url(), // WebSocket URL 직접 사용
        );

        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();

        scenario.add_player(player1, BehaviorType::Normal(NormalPlayer));
        scenario.add_player(player2, BehaviorType::Normal(NormalPlayer));

        scenario.observer.expect_queued(player1);
        scenario.observer.expect_queued(player2);
        scenario.observer.expect_start_loading(player1);
        scenario.observer.expect_start_loading(player2);
        scenario.observer.expect_match_found(player1);
        scenario.observer.expect_match_found(player2);

        scenario
    }

    pub async fn run(&mut self) -> Result<TestResult> {
        info!("Starting test scenario: {}", self.name);
        info!("Description: {}", self.description);

        let mut tasks = Vec::new();

        // Start observer
        let observer_task = {
            let mut observer = self.observer.clone();
            tokio::spawn(async move { observer.start_observation(None).await })
        };

        // Start players
        for (player_id, behavior) in &self.players {
            let player_id = *player_id;
            let behavior = behavior.clone();

            let task = tokio::spawn(async move { Self::run_player(player_id, behavior).await });

            tasks.push(task);
        }

        let mut player_results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => player_results.push(result),
                Err(e) => {
                    error!("Player task failed: {}", e);
                    player_results.push(Err(anyhow::anyhow!("Task join error: {}", e)));
                }
            }
        }

        let observer_result = observer_task
            .await
            .map_err(|e| anyhow::anyhow!("Observer task failed: {}", e))?;

        Ok(TestResult {
            scenario_name: self.name.clone(),
            player_results,
            observer_result: observer_result?,
        })
    }

    async fn run_player(player_id: Uuid, behavior: BehaviorType) -> Result<PlayerResult> {
        info!("Starting player {}", player_id);

        match behavior {
            BehaviorType::Normal(_player) => {
                info!("Player {} executing normal behavior", player_id);

                Ok(PlayerResult::Success { player_id })
            }
            BehaviorType::QuitDuringMatch(_) => {
                info!("Player {} executing quit during match", player_id);
                sleep(Duration::from_secs(1)).await; // Simulate connection
                sleep(Duration::from_millis(500)).await; // Quit early
                Ok(PlayerResult::Quit {
                    player_id,
                    reason: "Quit during match".to_string(),
                })
            }
            BehaviorType::QuitDuringLoading(_) => {
                info!("Player {} executing quit during loading", player_id);
                sleep(Duration::from_secs(1)).await; // Simulate connection
                sleep(Duration::from_secs(2)).await; // Simulate matching
                sleep(Duration::from_millis(100)).await; // Quit during loading
                Ok(PlayerResult::Quit {
                    player_id,
                    reason: "Quit during loading".to_string(),
                })
            }
            BehaviorType::SlowLoader(slow_loader) => {
                info!("Player {} executing slow loader behavior", player_id);
                sleep(Duration::from_secs(1)).await; // Simulate connection
                sleep(Duration::from_secs(2)).await; // Simulate matching
                sleep(Duration::from_secs(slow_loader.delay_seconds)).await; // Slow loading
                Ok(PlayerResult::Success { player_id })
            }
            BehaviorType::IgnoreMatchFound(_) => {
                info!("Player {} executing ignore match found", player_id);
                sleep(Duration::from_secs(1)).await; // Simulate connection
                sleep(Duration::from_secs(2)).await; // Simulate matching
                                                     // Ignore match found - don't proceed to loading
                Ok(PlayerResult::Failed {
                    player_id,
                    reason: "Ignored match found".to_string(),
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub scenario_name: String,
    pub player_results: Vec<Result<PlayerResult>>,
    pub observer_result: ObservationResult,
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        self.observer_result.is_success()
            && self.player_results.iter().all(|r| {
                match r {
                    Ok(PlayerResult::Success { .. }) => true,
                    Ok(PlayerResult::Quit { .. }) => true, // Quit might be expected behavior
                    _ => false,
                }
            })
    }

    pub fn get_summary(&self) -> String {
        let observer_summary = self.observer_result.get_summary();
        let player_count = self.player_results.len();
        let success_count = self
            .player_results
            .iter()
            .filter(|r| matches!(r, Ok(PlayerResult::Success { .. })))
            .count();

        format!(
            "Test: {} | Players: {}/{} succeeded | Observer: {}",
            self.scenario_name, success_count, player_count, observer_summary
        )
    }
}

#[derive(Debug)]
pub enum PlayerResult {
    Success { player_id: Uuid },
    Failed { player_id: Uuid, reason: String },
    Quit { player_id: Uuid, reason: String },
}

// Example usage function
pub async fn run_example_test() -> Result<()> {
    // 환경 설정 초기화
    simulator_env::init()?;

    let mut scenario = TestScenario::setup_normal_match_test();

    let result = scenario.run().await?;
    info!("Test completed: {}", result.get_summary());

    if result.is_success() {
        info!("✓ Test passed!");
    } else {
        error!("✗ Test failed!");
    }

    Ok(())
}
