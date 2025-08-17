pub mod connection;
pub mod enums;
pub mod server;
pub mod test;
pub mod types;

use std::{io, sync::Once};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
static INIT: Once = Once::new();
static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;
pub fn setup_logger() {
    INIT.call_once(|| {
        // 1. 파일 로거 설정
        let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "app.log");
        let (non_blocking_file_writer, _guard) = tracing_appender::non_blocking(file_appender);

        // 2. 로그 레벨 필터 설정 (환경 변수 또는 기본값 INFO)
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // 기본 INFO 레벨

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

        unsafe {
            GUARD = Some(_guard);
        }

        tracing::info!("로거 초기화 완료: 콘솔 및 파일(logs/app.log) 출력 활성화.");
    });
}
