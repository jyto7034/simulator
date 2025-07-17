use anyhow::Result;

/// 플레이어 행동을 따라하는 객체를 만들어야함.
/// 해당 객체는
/// 1. 연결
/// 2. 부여받은 행동을 수행.
/// 3. 행동이 완료되면 종료.
/// 위와 같은 행동을 가짐.
///
/// 이 때 부여 받은 행동은 다음과 같음.
/// 0. 매칭 실패 ( on_error )
/// 1. 매칭 중 ( during_match )
/// 2. 매칭 성공 ( on_match_found )
/// 3. 로딩 중 ( on_start_loading )
/// 4. 로딩 완료 ( on_loading_complete )
///
/// 매칭 서버와 상호작용 시 가능한 플레이어의 행동

#[tokio::main]
async fn main() -> Result<()> {
    // Run example test
    // scenario::run_example_test().await?;

    Ok(())
}
