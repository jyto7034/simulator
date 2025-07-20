// use anyhow::Result;
// use test_client::scenario::TestScenario;
// use tracing::{error, info};
// use simulator_env;

// #[actix_web::test]
// pub async fn run_example_test() -> Result<()> {
//     // 환경 설정 초기화
//     simulator_env::init()?;

//     // 디버깅을 위해 사용되는 URL 출력
//     let match_url = simulator_env::env::match_server_url();
//     let ws_url = simulator_env::env::match_server_ws_url();

//     info!("Match server URL: {}", match_url);
//     info!("Match server WebSocket URL: {}", ws_url);

//     let mut scenario = TestScenario::setup_normal_match_test();

//     let result = scenario.run().await?;
//     info!("Test completed: {}", result.get_summary());

//     if result.is_success() {
//         info!("✓ Test passed!");
//     } else {
//         error!("✗ Test failed!");
//     }

//     Ok(())
// }
