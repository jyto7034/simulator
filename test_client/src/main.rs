use anyhow::Result;
use async_trait::async_trait;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, instrument, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use url::Url;
use uuid::Uuid;

// --- 로거 설정 ---
pub fn setup_logger(player_id: &str) -> WorkerGuard {
    let log_filename = format!("client_{}.log", player_id);
    let file_appender = RollingFileAppender::new(Rotation::NEVER, "logs", log_filename.clone());
    let (non_blocking_file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 3. 콘솔 출력 레이어 설정
    let console_layer = fmt::layer()
        .with_writer(io::stdout) // 표준 출력으로 설정
        .with_ansi(true) // ANSI 색상 코드 사용 (터미널 지원 시)
        .with_thread_ids(true) // 스레드 ID 포함
        .with_thread_names(true) // 스레드 이름 포함
        .with_file(true) // 파일 경로 포함
        .with_line_number(true) // 라인 번호 포함
        .with_target(false) // target 정보 제외 (선택 사항)
        .pretty(); // 사람이 읽기 좋은 포맷

    // 4. 파일 출력 레이어 설정
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file_writer) // Non-blocking 파일 로거 사용
        .with_ansi(false) // 파일에는 ANSI 코드 제외
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .pretty();

    // 5. 레지스트리(Registry)에 필터와 레이어 결합
    tracing_subscriber::registry()
        .with(filter) // 필터를 먼저 적용
        .with(console_layer) // 콘솔 레이어 추가
        .with(file_layer) // 파일 레이어 추가
        .init(); // 전역 Subscriber로 설정

    info!("Logger initialized. Log file: {}", log_filename);
    guard
}

// --- 메시지 정의 ---
#[derive(Serialize)]
struct ClientMessage<'a> {
    #[serde(rename = "type")]
    msg_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    player_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    game_mode: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    loading_session_id: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
struct ServerMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    loading_session_id: Option<Uuid>,
    #[serde(default)]
    server_address: String,
    #[serde(default)]
    session_id: Option<Uuid>,
}

// --- Type alias for the WebSocket stream sink part ---
type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

// --- 플레이어 행동 정의 ---
#[async_trait]
trait PlayerBehavior {
    async fn on_start_loading(
        &self,
        player_id: Uuid,
        loading_id: Uuid,
        ws_sink: &mut WsSink,
    ) -> Result<bool>;

    fn on_error(&self, player_id: Uuid, error_msg: &str) -> bool;

    fn on_match_found(&self, player_id: Uuid) -> bool {
        info!("[{}] >>> SUCCESS: MatchFound received!", player_id);
        false
    }
}

// --- 시나리오별 행동 구현 ---

struct Disconnector;
#[async_trait]
impl PlayerBehavior for Disconnector {
    async fn on_start_loading(
        &self,
        player_id: Uuid,
        _loading_id: Uuid,
        ws_sink: &mut WsSink,
    ) -> Result<bool> {
        warn!("[{}] Received StartLoading. Disconnecting now!", player_id);
        ws_sink.close().await?;
        Ok(false)
    }
    fn on_error(&self, player_id: Uuid, error_msg: &str) -> bool {
        error!("[{}] Received unexpected error: {}", player_id, error_msg);
        false
    }
}

struct Victim;
#[async_trait]
impl PlayerBehavior for Victim {
    async fn on_start_loading(
        &self,
        player_id: Uuid,
        loading_id: Uuid,
        ws_sink: &mut WsSink,
    ) -> Result<bool> {
        info!(
            "[{}] Received StartLoading. Sending LoadingComplete.",
            player_id
        );
        let msg = ClientMessage {
            msg_type: "loading_complete",
            loading_session_id: Some(loading_id),
            player_id: None,
            game_mode: None,
        };
        ws_sink
            .send(Message::Text(serde_json::to_string(&msg)?))
            .await?;
        Ok(true)
    }
    fn on_error(&self, player_id: Uuid, error_msg: &str) -> bool {
        if error_msg.contains("disconnected") || error_msg.contains("timed out") {
            info!(
                "[{}] >>> SUCCESS: Received expected cancellation error: {}",
                player_id, error_msg
            );
        } else {
            error!("[{}] Received unexpected error: {}", player_id, error_msg);
        }
        false
    }
}

struct TimeoutPlayer;
#[async_trait]
impl PlayerBehavior for TimeoutPlayer {
    async fn on_start_loading(
        &self,
        player_id: Uuid,
        _loading_id: Uuid,
        _ws_sink: &mut WsSink,
    ) -> Result<bool> {
        warn!(
            "[{}] Received StartLoading. Waiting for 65 seconds to cause a timeout...",
            player_id
        );
        sleep(Duration::from_secs(65)).await;
        info!("[{}] Timeout period passed.", player_id);
        Ok(false)
    }
    fn on_error(&self, player_id: Uuid, error_msg: &str) -> bool {
        info!(
            "[{}] Received error (likely after timeout): {}",
            player_id, error_msg
        );
        false
    }
}

struct GhostDisconnector;
#[async_trait]
impl PlayerBehavior for GhostDisconnector {
    async fn on_start_loading(
        &self,
        player_id: Uuid,
        _loading_id: Uuid,
        ws_sink: &mut WsSink,
    ) -> Result<bool> {
        warn!(
            "[{}] Received StartLoading. Disconnecting IMMEDIATELY to test race condition!",
            player_id
        );
        ws_sink.close().await?;
        Ok(false)
    }
    fn on_error(&self, player_id: Uuid, error_msg: &str) -> bool {
        error!("[{}] Received unexpected error: {}", player_id, error_msg);
        false
    }
}

// --- Enum Wrapper for Behaviors ---
enum Behavior {
    Disconnect(Disconnector),
    Victim(Victim),
    Timeout(TimeoutPlayer),
    Ghost(GhostDisconnector),
}

#[async_trait]
impl PlayerBehavior for Behavior {
    async fn on_start_loading(
        &self,
        player_id: Uuid,
        loading_id: Uuid,
        ws_sink: &mut WsSink,
    ) -> Result<bool> {
        match self {
            Behavior::Disconnect(b) => b.on_start_loading(player_id, loading_id, ws_sink).await,
            Behavior::Victim(b) => b.on_start_loading(player_id, loading_id, ws_sink).await,
            Behavior::Timeout(b) => b.on_start_loading(player_id, loading_id, ws_sink).await,
            Behavior::Ghost(b) => b.on_start_loading(player_id, loading_id, ws_sink).await,
        }
    }

    fn on_error(&self, player_id: Uuid, error_msg: &str) -> bool {
        match self {
            Behavior::Disconnect(b) => b.on_error(player_id, error_msg),
            Behavior::Victim(b) => b.on_error(player_id, error_msg),
            Behavior::Timeout(b) => b.on_error(player_id, error_msg),
            Behavior::Ghost(b) => b.on_error(player_id, error_msg),
        }
    }

    fn on_match_found(&self, player_id: Uuid) -> bool {
        match self {
            Behavior::Disconnect(b) => b.on_match_found(player_id),
            Behavior::Victim(b) => b.on_match_found(player_id),
            Behavior::Timeout(b) => b.on_match_found(player_id),
            Behavior::Ghost(b) => b.on_match_found(player_id),
        }
    }
}

// --- 플레이어 실행 로직 ---
#[instrument(skip_all, fields(player_id = %player_id))]
async fn run_player(player_id: Uuid, behavior: Behavior) -> Result<()> {
    info!("Starting player with specific behavior");

    let url = Url::parse("ws://127.0.0.1:8080/ws/")?;
    let (ws_stream, _) = connect_async(url.as_str()).await?;
    info!("Connected to server.");

    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    let enqueue_msg = ClientMessage {
        msg_type: "enqueue",
        player_id: Some(player_id),
        game_mode: Some("Normal_1v1"),
        loading_session_id: None,
    };
    ws_sink
        .send(Message::Text(serde_json::to_string(&enqueue_msg)?))
        .await?;
    info!("Sent enqueue request.");

    while let Some(msg) = ws_stream.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(_) => continue,
            Err(e) => {
                warn!("WebSocket stream error: {}", e);
                break;
            }
        };

        let server_msg: ServerMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to parse server message: {}. Raw: {}", e, msg);
                continue;
            }
        };

        info!("Received: {:?}", server_msg);

        let continue_loop = match server_msg.msg_type.as_str() {
            "start_loading" => {
                behavior
                    .on_start_loading(
                        player_id,
                        server_msg.loading_session_id.unwrap(),
                        &mut ws_sink,
                    )
                    .await?
            }
            "error" => behavior.on_error(player_id, &server_msg.message),
            "match_found" => behavior.on_match_found(player_id),
            "queued" => true,
            _ => true,
        };

        if !continue_loop {
            break;
        }
    }

    info!("Test finished for this player.");
    Ok(())
}

// --- 시나리오 정의 및 메인 함수 ---

#[derive(Debug, PartialEq, Clone, Copy)]
enum Scenario {
    Disconnect,
    Timeout,
    Ghost,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let scenario_str = args.get(1).map(|s| s.as_str()).unwrap_or("disconnect");
    let role_str = args.get(2).map(|s| s.as_str()).unwrap_or("victim");

    let player_id = Uuid::new_v4();
    let _guard = setup_logger(&player_id.to_string()[..8]);

    let scenario = match scenario_str {
        "disconnect" => Scenario::Disconnect,
        "timeout" => Scenario::Timeout,
        "ghost" => Scenario::Ghost,
        _ => {
            anyhow::bail!(
                "Unknown scenario: {}. Use: disconnect, timeout, ghost",
                scenario_str
            );
        }
    };

    info!("Selected Scenario: {:?}, Role: {}", scenario, role_str);

    let behavior = match (scenario, role_str) {
        (Scenario::Disconnect, "disconnector") => Behavior::Disconnect(Disconnector),
        (Scenario::Disconnect, "victim") => Behavior::Victim(Victim),

        (Scenario::Timeout, "timeout_player") => Behavior::Timeout(TimeoutPlayer),
        (Scenario::Timeout, "victim") => Behavior::Victim(Victim),

        (Scenario::Ghost, "disconnector") => Behavior::Ghost(GhostDisconnector),
        (Scenario::Ghost, "victim") => Behavior::Victim(Victim),

        _ => {
            anyhow::bail!("Invalid role '{}' for scenario '{:?}'", role_str, scenario);
        }
    };

    if let Err(e) = run_player(player_id, behavior).await {
        error!("Player execution failed: {:?}", e);
    }

    sleep(Duration::from_millis(200)).await;
    Ok(())
}
