use actix::{ActorContext, Handler};
use tracing::info;

use super::message::{PlayerCompleted, ScenarioCompleted};
use super::ScenarioResult;
use super::{ScenarioRunnerActor, SingleScenarioActor};

impl Handler<ScenarioCompleted> for ScenarioRunnerActor {
    type Result = ();

    fn handle(&mut self, msg: ScenarioCompleted, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "Scenario {} completed with result: {:?}",
            msg.scenario_id, msg.result
        );

        self.results.push(msg.result);
        self.completed_count += 1;

        if self.completed_count >= self.total_count {
            info!("All {} scenarios completed!", self.total_count);

            let success_count = self
                .results
                .iter()
                .filter(|r| matches!(r, ScenarioResult::Success))
                .count();

            info!(
                "Final results: {}/{} scenarios succeeded",
                success_count, self.total_count
            );

            ctx.stop();
            actix::System::current().stop();
        }
    }
}

impl Handler<PlayerCompleted> for SingleScenarioActor {
    type Result = ();

    fn handle(&mut self, msg: PlayerCompleted, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "Player {} completed with result: {:?}",
            msg.player_id, msg.result
        );

        self.player_results.push(msg.result);

        if self.player_results.len() >= 2 {
            let scenario_result = self.determine_scenario_result();

            self.runner_addr.do_send(ScenarioCompleted {
                scenario_id: self.scenario.id,
                result: scenario_result,
            });

            ctx.stop();
        }
    }
}

impl SingleScenarioActor {
    fn determine_scenario_result(&self) -> ScenarioResult {
        let all_success = self
            .player_results
            .iter()
            .all(|result| matches!(result, Ok(crate::BehaviorOutcome::Stop)));

        if all_success {
            ScenarioResult::Success
        } else {
            let failed_players: Vec<String> = self
                .player_results
                .iter()
                .enumerate()
                .filter_map(|(i, result)| {
                    if !matches!(result, Ok(crate::BehaviorOutcome::Stop)) {
                        Some(format!("Player {}: {:?}", i, result))
                    } else {
                        None
                    }
                })
                .collect();

            ScenarioResult::Failure(format!("Failed players: {}", failed_players.join(", ")))
        }
    }
}
